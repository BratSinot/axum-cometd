mod build_router;
mod builder;
mod subscription_task;

pub use {build_router::*, builder::*};

use crate::{
    messages::SubscriptionMessage,
    types::{ChannelId, ClientId, ClientReceiver, ClientSender, CookieId},
    utils::{ChannelNameValidator, WildNamesCache},
    Event, SendError,
};
use ahash::{AHashMap, AHashSet};
use async_broadcast::{InactiveReceiver, Receiver, Sender};
use core::{fmt::Debug, ops::Deref, time::Duration};
use serde::Serialize;
use serde_json::json;
use std::{collections::hash_map::Entry, sync::Arc};
use tokio::sync::{mpsc, RwLock};

/// Context for sending messages to channels.
#[derive(Debug)]
pub struct LongPollingServiceContext<AdditionalData, CustomData> {
    pub(crate) tx: Sender<Arc<Event<AdditionalData, CustomData>>>,
    pub(crate) inactive_rx: InactiveReceiver<Arc<Event<AdditionalData, CustomData>>>,

    pub(crate) wildnames_cache: WildNamesCache,
    pub(crate) channel_name_validator: ChannelNameValidator,
    pub(crate) consts: LongPollingServiceContextConsts,
    pub(crate) channels_data: RwLock<AHashMap<ChannelId, Channel>>,
    client_id_senders: Arc<RwLock<AHashMap<ClientId, ClientSender>>>,
}

#[derive(Debug)]
pub(crate) struct Channel {
    client_ids: AHashSet<ClientId>,
    tx: mpsc::Sender<SubscriptionMessage>,
}

impl Channel {
    #[inline(always)]
    const fn client_ids(&self) -> &AHashSet<ClientId> {
        &self.client_ids
    }

    #[inline(always)]
    pub(crate) const fn tx(&self) -> &mpsc::Sender<SubscriptionMessage> {
        &self.tx
    }
}

impl<AdditionalData, CustomData> LongPollingServiceContext<AdditionalData, CustomData> {
    /// Get new events receiver.
    ///
    /// # Example
    /// ```rust
    /// # async {
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build::<(), ()>();
    ///     let mut rx = context.rx();
    ///     
    ///     while let Ok(event) = rx.recv().await {
    ///         println!("Got event: `{event:?}`");
    ///     }
    /// # };
    /// ```
    pub fn rx(&self) -> Receiver<Arc<Event<AdditionalData, CustomData>>> {
        self.inactive_rx.activate_cloned()
    }

    /// Get new events sender.
    ///
    /// # Example
    /// ```rust
    /// # use std::sync::Arc;
    /// # use axum_cometd::Event;
    ///  async {
    /// # let context = axum_cometd::LongPollingServiceContextBuilder::new().build::<(), ()>();
    ///     let tx = context.tx();
    ///     
    ///     let  _ = tx.broadcast(Arc::new(Event::CustomData(()))).await;
    /// # };
    /// ```
    pub fn tx(&self) -> Sender<Arc<Event<AdditionalData, CustomData>>> {
        self.tx.clone()
    }

    /// Send message to channel.
    ///
    /// # Example
    /// ```rust
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
    ///         .build::<(), ()>();
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
            .validate_send_channel_name(channel)
            .then_some(())
            .ok_or(SendError::InvalidChannel)?;

        let subscription_message = SubscriptionMessage {
            channel: channel.to_owned(),
            msg: json!(message),
        };
        let wildnames = self.wildnames_cache.fetch_wildnames(channel).await;
        let read_guard = self.channels_data.read().await;
        for channel in core::iter::once(channel).chain(wildnames.iter().map(String::deref)) {
            if let Some(tx) = read_guard.get(channel).map(Channel::tx) {
                tx.send(subscription_message.clone()).await?;
            } else {
                tracing::warn!(
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
            .validate_send_channel_name(channel)
            .then_some(())
            .ok_or(SendError::InvalidChannel)?;

        if let Some(tx) = self.client_id_senders.read().await.get(client_id) {
            tx.send(SubscriptionMessage {
                channel: channel.to_owned(),
                msg: json!(msg),
            })
            .await?;

            Ok(())
        } else {
            tracing::warn!(
                client_id = %client_id,
                "No `{client_id}` client was found for message: `{msg:?}`."
            );

            Err(SendError::ClientWasntFound(*client_id))
        }
    }

    pub(crate) async fn register(self: &Arc<Self>, cookie_id: CookieId) -> Option<ClientId>
    where
        AdditionalData: Send + Sync + 'static,
        CustomData: Send + Sync + 'static,
    {
        let client_id = {
            let mut client_id_channels_write_guard = self.client_id_senders.write().await;

            let client_id = ClientId::gen();
            let (tx, rx) = mpsc::channel(self.consts.client_channel_capacity);

            match client_id_channels_write_guard.entry(client_id) {
                Entry::Occupied(_) => return None,
                Entry::Vacant(v) => {
                    v.insert(ClientSender::create(
                        Arc::clone(self),
                        cookie_id,
                        client_id,
                        Duration::from_millis(self.consts.max_interval_ms),
                        tx,
                        rx,
                    ));
                }
            }

            Some(client_id)
        }?;

        tracing::info!(
            client_id = %client_id,
            "New client was registered with clientId `{client_id}`."
        );

        Some(client_id)
    }

    pub(crate) async fn subscribe(self: &Arc<Self>, client_id: ClientId, channels: &[String])
    where
        AdditionalData: Send + Sync + 'static,
        CustomData: Send + Sync + 'static,
    {
        let mut channels_data_write_guard = self.channels_data.write().await;
        for channel in channels {
            match channels_data_write_guard.entry(channel.clone()) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => {
                    let (tx, rx) = mpsc::channel(self.consts.subscription_channel_capacity);

                    subscription_task::spawn(channel.clone(), rx, Arc::clone(self));
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
    }

    // TODO: Spawn task and send unsubscribe command through channel?
    /// Remove client.
    #[inline]
    pub async fn unsubscribe(self: &Arc<Self>, client_id: ClientId) {
        tokio::join!(
            self.remove_client_id_from_subscriptions(&client_id),
            self.remove_client_tx(&client_id),
        );

        let _ = self
            .tx
            .broadcast(Arc::new(Event::SessionRemoved { client_id }))
            .await;
    }

    #[inline]
    async fn remove_client_id_from_subscriptions(&self, client_id: &ClientId) {
        // TODO: drain_filter: https://github.com/rust-lang/rust/issues/59618
        // TODO: Replace on LinkedList?
        let mut removed_channels = AHashSet::new();

        self.channels_data.write().await.retain(
            |channel,
             &mut Channel {
                 ref mut client_ids,
                 tx: _,
             }| {
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
            },
        );

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
    pub(crate) async fn check_client(
        &self,
        cookie_id: CookieId,
        client_id: &ClientId,
    ) -> Option<()> {
        self.client_id_senders
            .read()
            .await
            .get(client_id)
            .map(ClientSender::cookie_id)
            .eq(&Some(cookie_id))
            .then_some(())
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
