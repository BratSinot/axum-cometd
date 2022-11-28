mod build_router;
mod builder;
mod subscription_task;

pub use {build_router::*, builder::*};

use crate::{
    types::{ClientId, ClientIdGen, ClientReceiver, ClientSender, SubscriptionId},
    CometdError, CometdResult, SendError,
};
use ahash::{AHashMap, AHashSet};
use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};

/// Context for sending messages to channels.
#[derive(Debug)]
pub struct LongPoolingServiceContext<Msg> {
    pub(crate) consts: LongPoolingServiceContextConsts,
    subscriptions_data: RwLock<AHashMap<SubscriptionId, Subscription<Msg>>>,
    client_id_channels: Arc<RwLock<AHashMap<ClientId, ClientSender<Msg>>>>,
}

#[derive(Debug)]
struct Subscription<Msg> {
    client_ids: AHashSet<ClientId>,
    tx: mpsc::Sender<Msg>,
}

impl<Msg> Subscription<Msg> {
    #[inline(always)]
    fn client_ids(&self) -> &AHashSet<ClientId> {
        &self.client_ids
    }

    #[inline(always)]
    fn tx_cloned(&self) -> mpsc::Sender<Msg> {
        self.tx.clone()
    }
}

impl<Msg> LongPoolingServiceContext<Msg>
where
    Msg: Debug + Clone + Send + 'static,
{
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
    /// # Ok::<(), axum_cometd::SendError<Data>>(())
    /// # };
    /// ```
    #[inline]
    pub async fn send(&self, topic: &str, msg: Msg) -> Result<(), SendError<Msg>> {
        let tx = self
            .subscriptions_data
            .read()
            .await
            .get(topic)
            .map(Subscription::tx_cloned);
        if let Some(tx) = tx {
            tx.send(msg).await?;
        } else {
            tracing::trace!(
                topic = topic,
                "No `{topic}` channel was found for message: `{msg:?}`."
            );
        }

        Ok(())
    }

    pub(crate) async fn register(self: &Arc<Self>) -> ClientId {
        static CLIENT_ID_GEN: ClientIdGen = ClientIdGen::new();

        let client_id = {
            let mut client_id_channels_write_guard = self.client_id_channels.write().await;
            loop {
                let client_id = CLIENT_ID_GEN.next();

                match client_id_channels_write_guard.entry(client_id) {
                    Entry::Occupied(_) => continue,
                    Entry::Vacant(v) => {
                        let (tx, _rx) = broadcast::channel(self.consts.client_channel_capacity);
                        v.insert(ClientSender::create(
                            self.clone(),
                            client_id,
                            Duration::from_millis(self.consts.max_interval_ms),
                            tx,
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
        subscription: &str,
    ) -> Result<(), ClientId> {
        if !self
            .client_id_channels
            .read()
            .await
            .contains_key(&client_id)
        {
            tracing::error!(
                client_id = %client_id,
                "Non-existing client with clientId `{client_id}`."
            );
            return Err(client_id);
        }

        match self
            .subscriptions_data
            .write()
            .await
            .entry(subscription.to_string())
        {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let (tx, rx) = mpsc::channel::<Msg>(self.consts.subscription_channel_capacity);

                subscription_task::spawn(subscription.to_string(), rx, self.clone());
                tracing::info!(
                    subscription = subscription,
                    "New subscription ({subscription}) channel was registered."
                );

                v.insert(Subscription {
                    client_ids: Default::default(),
                    tx,
                })
            }
        }
        .client_ids
        .insert(client_id);

        tracing::info!(
            client_id = %client_id,
            subscription = subscription,
            "Client with clientId `{client_id}` subscribe on `{subscription}` channel."
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
        self.subscriptions_data.write().await.retain(
            |subscription, Subscription { client_ids, tx: _ }| {
                if client_ids.remove(client_id) {
                    tracing::info!(
                        client_id = %client_id,
                        subscription = subscription,
                        "Client `{client_id}` was unsubscribed from channel `{subscription}."
                    );
                }

                if client_ids.is_empty() {
                    tracing::info!(
                        subscription = subscription,
                        "Channel `{subscription}` have no active subscriber. Eliminate channel."
                    );
                    false
                } else {
                    true
                }
            },
        );
    }

    #[inline]
    async fn remove_client_channel(&self, client_id: &ClientId) {
        if self
            .client_id_channels
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
    pub(crate) async fn get_client_receiver(
        &self,
        client_id: ClientId,
    ) -> CometdResult<ClientReceiver<Msg>> {
        let rx = self
            .client_id_channels
            .read()
            .await
            .get(&client_id)
            .ok_or(CometdError::ClientDoesntExist(client_id))?
            .subscribe();

        Ok(rx)
    }
}
