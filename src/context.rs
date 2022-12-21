mod build_router;
mod builder;
mod subscription_task;

pub use {build_router::*, builder::*};

use crate::{
    messages::SubscriptionMessage,
    types::{Callback, ChannelId, ClientId, ClientIdGen, ClientReceiver, ClientSender},
    utils::{ChannelNameValidator, WildNamesCache},
    SessionAddedArgs, SendError, SessionRemovedArgs,
};
use ahash::{AHashMap, AHashSet};
use axum::http::HeaderMap;
use serde::Serialize;
use serde_json::json;
use std::{collections::hash_map::Entry, fmt::Debug, ops::Deref, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};

/// Context for sending messages to channels.
#[derive(Debug)]
pub struct LongPollingServiceContext {
    session_added: Callback<SessionAddedArgs>,
    session_removed: Callback<SessionRemovedArgs>,

    pub(crate) wildnames_cache: WildNamesCache,
    pub(crate) channel_name_validator: ChannelNameValidator,
    pub(crate) consts: LongPollingServiceContextConsts,
    pub(crate) channels_data: RwLock<AHashMap<ChannelId, Channel>>,
    client_id_senders: Arc<RwLock<(ClientIdGen, AHashMap<ClientId, ClientSender>)>>,
}

#[derive(Debug)]
pub(crate) struct Channel {
    client_ids: AHashSet<ClientId>,
    tx: mpsc::Sender<SubscriptionMessage>,
}

impl Channel {
    #[inline(always)]
    fn client_ids(&self) -> &AHashSet<ClientId> {
        &self.client_ids
    }

    #[inline(always)]
    pub(crate) fn tx(&self) -> &mpsc::Sender<SubscriptionMessage> {
        &self.tx
    }
}

impl LongPollingServiceContext {
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
    ///     let context = axum_cometd::LongPollingServiceContextBuilder::new()
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
    pub async fn send(
        &self,
        channel: &str,
        message: impl Debug + Serialize,
    ) -> Result<(), SendError> {
        self.channel_name_validator
            .validate_send_channel_name(channel, SendError::InvalidChannel)?;

        let subscription_message = SubscriptionMessage {
            channel: channel.to_string(),
            msg: json!(message),
        };
        let wildnames = self.wildnames_cache.fetch_wildnames(channel).await;
        let read_guard = self.channels_data.read().await;
        for channel in std::iter::once(channel).chain(wildnames.iter().map(String::deref)) {
            if let Some(tx) = read_guard.get(channel).map(Channel::tx) {
                tx.send(subscription_message.clone()).await?;
            } else {
                tracing::trace!(
                    channel = channel,
                    "No `{channel}` channel was found for message: `{message:?}`."
                );
            }
        }

        Ok(())
    }

    /// Send message direct to client.
    #[inline]
    pub async fn send_to_client(
        &self,
        channel: &str,
        client_id: &ClientId,
        msg: impl Debug + Serialize,
    ) -> Result<(), SendError> {
        self.channel_name_validator
            .validate_send_channel_name(channel, SendError::InvalidChannel)?;

        if let Some(tx) = self.client_id_senders.read().await.1.get(client_id) {
            tx.send(SubscriptionMessage {
                channel: channel.to_string(),
                msg: json!(msg),
            })
            .await?;

            Ok(())
        } else {
            tracing::trace!(
                client_id = %client_id,
                "No `{client_id}` client was found for message: `{msg:?}`."
            );

            Err(SendError::ClientWasntFound(*client_id))
        }
    }

    pub(crate) async fn register(self: &Arc<Self>, headers: HeaderMap) -> ClientId {
        #[allow(clippy::option_map_unit_fn)]
        let client_id = {
            let mut client_id_channels_write_guard = self.client_id_senders.write().await;

            let client_id = client_id_channels_write_guard.0.next();
            let (tx, rx) = mpsc::channel(self.consts.client_channel_capacity);

            client_id_channels_write_guard
                .1
                .insert(
                    client_id,
                    ClientSender::create(
                        self.clone(),
                        client_id,
                        Duration::from_millis(self.consts.max_interval_ms),
                        tx,
                        rx,
                    ),
                )
                .map(|_| panic!("ClientIdGen::next return already used ClientId!"));

            client_id
        };

        self.session_added
            .call(SessionAddedArgs {
                context: self.clone(),
                client_id,
                headers,
            })
            .await;

        tracing::info!(
            client_id = %client_id,
            "New client was registered with clientId `{client_id}`."
        );

        client_id
    }

    pub(crate) async fn subscribe(
        self: &Arc<Self>,
        client_id: ClientId,
        channels: &[String],
    ) -> Result<(), ClientId> {
        if !self.check_client_id(&client_id).await {
            tracing::error!(
                client_id = %client_id,
                "Non-existing client with clientId `{client_id}`."
            );
            return Err(client_id);
        }

        let mut channels_data_write_guard = self.channels_data.write().await;
        for channel in channels.iter() {
            match channels_data_write_guard.entry(channel.to_string()) {
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
        }

        tracing::info!(
            client_id = %client_id,
            channels = debug(channels),
            "Client with clientId `{client_id}` subscribe on `{channels:?}` channels."
        );

        Ok(())
    }

    // TODO: Spawn task and send unsubscribe command through channel?
    /// Remove client.
    #[inline]
    pub async fn unsubscribe(self: &Arc<Self>, client_id: ClientId) {
        tokio::join!(
            self.remove_client_id_from_subscriptions(&client_id),
            self.remove_client_tx(&client_id),
        );

        self.session_removed.call(SessionRemovedArgs{ context: self.clone(), client_id }).await;
    }

    #[inline]
    async fn remove_client_id_from_subscriptions(&self, client_id: &ClientId) {
        // TODO: drain_filter: https://github.com/rust-lang/rust/issues/59618
        // TODO: Replace on LinkedList?
        let mut removed_channels = AHashSet::new();

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
                    removed_channels.insert(channel.clone());
                    false
                } else {
                    true
                }
            });

        self.wildnames_cache
            .remove_wildnames(removed_channels)
            .await;
    }

    #[inline]
    async fn remove_client_tx(&self, client_id: &ClientId) {
        if self
            .client_id_senders
            .write()
            .await
            .1
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
        self.client_id_senders
            .read()
            .await
            .1
            .contains_key(client_id)
    }

    #[inline]
    pub(crate) async fn get_client_receiver(&self, client_id: &ClientId) -> Option<ClientReceiver> {
        self.client_id_senders
            .read()
            .await
            .1
            .get(client_id)
            .map(ClientSender::subscribe)
    }
}
