mod build_router;
mod builder;
mod subscription_task;

pub use {build_router::*, builder::*};

use crate::{
    types::{ChannelId, ClientId, ClientIdGen, ClientReceiver, ClientSender},
    SendError,
};
use ahash::{AHashMap, AHashSet};
use serde::Serialize;
use serde_json::{json, Value as JsonValue};
use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};

/// Context for sending messages to channels.
#[derive(Debug)]
pub struct LongPoolingServiceContext {
    consts: LongPoolingServiceContextConsts,
    channels_data: RwLock<AHashMap<ChannelId, Channel>>,
    client_id_senders: Arc<RwLock<AHashMap<ClientId, ClientSender>>>,
}

#[derive(Debug)]
pub(crate) struct Channel {
    client_ids: AHashSet<ClientId>,
    tx: mpsc::Sender<JsonValue>,
}

impl Channel {
    #[inline(always)]
    fn client_ids(&self) -> &AHashSet<ClientId> {
        &self.client_ids
    }

    #[inline(always)]
    pub(crate) fn tx(&self) -> &mpsc::Sender<JsonValue> {
        &self.tx
    }

    #[inline(always)]
    fn tx_cloned(&self) -> mpsc::Sender<JsonValue> {
        self.tx.clone()
    }
}

impl LongPoolingServiceContext {
    /// Send message to channel.
    ///
    /// # Example
    /// ```rust    ///
    ///     #[derive(Debug, Clone, serde::Serialize)]
    ///     struct Data<'a> {
    ///         msg: std::borrow::Cow<'a, str>,
    ///         r#bool: bool,
    ///         num: u64,
    ///     }
    ///
    /// # async {
    ///     let context = axum_cometd::LongPoolingServiceContextBuilder::new()
    ///         .timeout_ms(1000)
    ///         .max_interval_ms(2000)
    ///         .client_channel_capacity(10_000)
    ///         .subscription_channel_capacity(20_000)
    ///         .build();
    ///
    ///     loop {
    ///         context
    ///             .send(
    ///                 "/topic",
    ///                 Data {
    ///                     msg: "Hello World!!!".into(),
    ///                     r#bool: true,
    ///                     num: u64::MAX,
    ///                 },
    ///             )
    ///             .await?;
    ///         tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    ///     }
    /// # Ok::<(), axum_cometd::SendError>(())
    /// # };
    /// ```
    #[inline]
    pub async fn send<Msg>(&self, channel: &str, msg: Msg) -> Result<(), SendError>
    where
        Msg: Debug + Serialize,
    {
        let tx = self
            .channels_data
            .read()
            .await
            .get(channel)
            .map(Channel::tx_cloned);
        if let Some(tx) = tx {
            tx.send(json!(msg)).await?;
        } else {
            tracing::trace!(
                channel = channel,
                "No `{channel}` channel was found for message: `{msg:?}`."
            );
        }

        Ok(())
    }

    pub(crate) async fn register(self: &Arc<Self>) -> ClientId {
        static CLIENT_ID_GEN: ClientIdGen = ClientIdGen::new();

        let client_id = {
            let mut client_id_channels_write_guard = self.client_id_senders.write().await;
            loop {
                let client_id = CLIENT_ID_GEN.next();

                match client_id_channels_write_guard.entry(client_id) {
                    Entry::Occupied(_) => continue,
                    Entry::Vacant(v) => {
                        let (tx, rx) =
                            async_broadcast::broadcast(self.consts.client_channel_capacity);
                        v.insert(ClientSender::create(
                            self.clone(),
                            client_id,
                            Duration::from_millis(self.consts.max_interval_ms),
                            tx,
                            rx.deactivate(),
                        ));
                        break client_id;
                    }
                }
            }
        };

        tracing::info!(
            client_id = %client_id,
            "New client was registered with clientId `{client_id}`."
        );

        client_id
    }

    pub(crate) async fn subscribe(
        self: &Arc<Self>,
        client_id: ClientId,
        channel: &str,
    ) -> Result<(), ClientId> {
        if !self.client_id_senders.read().await.contains_key(&client_id) {
            tracing::error!(
                client_id = %client_id,
                "Non-existing client with clientId `{client_id}`."
            );
            return Err(client_id);
        }

        match self.channels_data.write().await.entry(channel.to_string()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let (tx, rx) = mpsc::channel(self.consts.subscription_channel_capacity);

                subscription_task::spawn(channel.to_string(), rx, self.clone());
                tracing::info!(
                    channel = channel,
                    "New subscription ({channel}) channel was registered."
                );

                v.insert(Channel {
                    client_ids: Default::default(),
                    tx,
                })
            }
        }
        .client_ids
        .insert(client_id);

        tracing::info!(
            client_id = %client_id,
            channel = channel,
            "Client with clientId `{client_id}` subscribe on `{channel}` channel."
        );

        Ok(())
    }

    // TODO: Spawn task and send unsubscribe command through channel.
    #[inline]
    pub(crate) async fn unsubscribe(&self, client_id: ClientId) {
        tokio::join!(
            self.remove_client_id_from_subscriptions(&client_id),
            self.remove_client_channel(&client_id),
        );
    }

    #[inline]
    async fn remove_client_id_from_subscriptions(&self, client_id: &ClientId) {
        self.channels_data
            .write()
            .await
            .retain(|channel, Channel { client_ids, tx: _ }| {
                if client_ids.remove(client_id) {
                    tracing::info!(
                        client_id = %client_id,
                        channel = channel,
                        "Client `{client_id}` was unsubscribed from channel `{channel}."
                    );
                }

                if client_ids.is_empty() {
                    tracing::info!(
                        channel = channel,
                        "Channel `{channel}` have no active subscriber. Eliminate channel."
                    );
                    false
                } else {
                    true
                }
            });
    }

    #[inline]
    async fn remove_client_channel(&self, client_id: &ClientId) {
        if self
            .client_id_senders
            .write()
            .await
            .remove(client_id)
            .is_some()
        {
            tracing::info!(
                client_id = %client_id,
                "Client `{client_id}` was unsubscribed."
            );
        } else {
            tracing::warn!(
                client_id = %client_id,
                "Can't find client `{client_id}`. Can't unsubscribed."
            );
        }
    }

    #[inline]
    pub(crate) async fn check_client_id(&self, client_id: &ClientId) -> bool {
        self.client_id_senders.read().await.contains_key(client_id)
    }

    #[inline(always)]
    pub(crate) fn consts(&self) -> &LongPoolingServiceContextConsts {
        &self.consts
    }

    #[inline(always)]
    pub(crate) fn subscriptions_data(&self) -> &RwLock<AHashMap<ChannelId, Channel>> {
        &self.channels_data
    }

    #[inline]
    pub(crate) async fn get_client_receiver(&self, client_id: &ClientId) -> Option<ClientReceiver> {
        self.client_id_senders
            .read()
            .await
            .get(client_id)
            .map(ClientSender::subscribe)
    }
}
