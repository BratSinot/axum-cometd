mod build_router;
mod builder;
mod subscription_task;

pub use build_router::*;
pub use builder::*;

use crate::{
    types::{ClientId, ClientIdGen, ClientReceiver, ClientSender, SubscriptionId},
    CometdError, CometdResult,
};
use ahash::{AHashMap, AHashSet};
use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};

#[derive(Debug)]
pub struct LongPoolingServiceContext<Msg> {
    pub(crate) consts: LongPoolingServiceContextConsts,
    client_ids_by_subscriptions: RwLock<AHashMap<SubscriptionId, AHashSet<ClientId>>>,
    subscription_channels: RwLock<AHashMap<SubscriptionId, mpsc::Sender<Msg>>>,
    client_id_channels: Arc<RwLock<AHashMap<ClientId, ClientSender<Msg>>>>,
}

impl<Msg> LongPoolingServiceContext<Msg>
where
    Msg: Debug + Clone + Send + 'static,
{
    #[inline]
    pub async fn send(&self, topic: &str, msg: Msg) -> Result<(), mpsc::error::SendError<Msg>> {
        if let Some(tx) = self.subscription_channels.read().await.get(topic) {
            tx.send(msg).await
        } else {
            tracing::trace!(
                topic = topic,
                "No `{topic}` channel was found for message: `{msg:?}`."
            );

            Ok(())
        }
    }

    pub(crate) async fn register(self: &Arc<Self>) -> ClientId {
        static CLIENT_ID_GEN: ClientIdGen = ClientIdGen::new();

        let client_id = CLIENT_ID_GEN.next();

        match self
            .client_id_channels
            .write()
            .await
            .entry(client_id.clone())
        {
            Entry::Occupied(_) => {
                unreachable!("impossible")
            }
            Entry::Vacant(v) => {
                let (tx, _rx) = broadcast::channel(self.consts.client_channel_capacity);
                v.insert(ClientSender::create(
                    self.clone(),
                    client_id.clone(),
                    Duration::from_millis(self.consts.max_interval_ms),
                    tx,
                ));
            }
        }

        tracing::info!(
            client_id = client_id,
            "New client was registered with clientId {client_id}."
        );

        client_id
    }

    pub(crate) async fn subscribe(
        self: &Arc<Self>,
        client_id: &ClientId,
        subscription: &str,
    ) -> CometdResult<()> {
        if !self.client_id_channels.read().await.contains_key(client_id) {
            tracing::error!(
                client_id = client_id,
                "Non-existing client with clientId {client_id}."
            );
            return Err(CometdError::ClientDoesntExist(client_id.clone()));
        }

        if let Entry::Vacant(v) = self
            .subscription_channels
            .write()
            .await
            .entry(subscription.to_string())
        {
            let (tx, rx) = mpsc::channel::<Msg>(self.consts.subscription_channel_capacity);

            subscription_task::spawn(subscription.to_string(), rx, self.clone());
            v.insert(tx);

            tracing::info!(
                subscription = subscription,
                "New subscription ({subscription}) channel was registered."
            );
        };

        self.client_ids_by_subscriptions
            .write()
            .await
            .entry(subscription.to_string())
            .or_default()
            .insert(client_id.clone());

        tracing::info!(
            client_id = client_id,
            subscription = subscription,
            "Client with clientId `{client_id}` subscribe on `{subscription}` channel."
        );

        Ok(())
    }

    // TODO: Spawn task and send unsubscribe command through channel.
    pub(crate) async fn unsubscribe(&self, client_id: &str) {
        let (mut client_ids_by_subscriptions, mut subscription_channels, mut client_id_channels) = tokio::join!(
            self.client_ids_by_subscriptions.write(),
            self.subscription_channels.write(),
            self.client_id_channels.write()
        );

        client_ids_by_subscriptions.retain(|subscription, client_ids| {
            if client_ids.remove(client_id) {
                tracing::info!(
                    client_id = client_id,
                    subscription = subscription,
                    "Client `{client_id}` was unsubscribed from channel `{subscription}."
                );
            }

            if client_ids.is_empty() {
                subscription_channels.remove(subscription);
                false
            } else {
                tracing::info!(
                    subscription = subscription,
                    "Channel `{subscription}` have no active subscriber. Eliminate channel."
                );
                true
            }
        });

        if client_id_channels.remove(client_id).is_some() {
            tracing::info!(
                client_id = client_id,
                "Client `{client_id}` was unsubscribed."
            );
        } else {
            tracing::warn!(
                client_id = client_id,
                "Can't find client `{client_id}`. Can't unsubscribed."
            );
        }
    }

    #[inline]
    pub(crate) async fn get_client_receiver(
        &self,
        client_id: &str,
    ) -> CometdResult<ClientReceiver<Msg>> {
        let rx = self
            .client_id_channels
            .read()
            .await
            .get(client_id)
            .ok_or_else(|| CometdError::ClientDoesntExist(client_id.into()))?
            .subscribe();

        Ok(rx)
    }
}
