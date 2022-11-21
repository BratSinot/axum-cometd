mod build_router;

use crate::{messages::SubscriptionMessage, CometdError, CometdResult};
use ahash::{AHashMap, AHashSet};
use std::{
    collections::hash_map::Entry,
    fmt::Debug,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, mpsc, RwLock};

#[derive(Debug)]
pub struct ClientIdGen(AtomicU64);

impl ClientIdGen {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn next(&self) -> ClientId {
        self.0.fetch_add(1, Ordering::Relaxed).to_string()
    }
}

// TODO: Replace on Arc<str>?
pub type ClientId = String;
pub type SubscriptionId = String;

#[derive(Debug)]
pub struct LongPoolingServiceContext<Msg>(Arc<InnerLongPoolingServiceContext<Msg>>);

impl<Msg> Clone for LongPoolingServiceContext<Msg> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Msg> Default for LongPoolingServiceContext<Msg> {
    fn default() -> Self {
        Self(Arc::new(InnerLongPoolingServiceContext {
            client_ids_by_subscriptions: Default::default(),
            subscription_channels: Default::default(),
            client_id_channels: Default::default(),
        }))
    }
}

#[derive(Debug)]
struct InnerLongPoolingServiceContext<Msg> {
    client_ids_by_subscriptions: RwLock<AHashMap<SubscriptionId, AHashSet<ClientId>>>,
    subscription_channels: RwLock<AHashMap<SubscriptionId, mpsc::Sender<Msg>>>,
    client_id_channels:
        Arc<RwLock<AHashMap<ClientId, broadcast::Sender<SubscriptionMessage<Msg>>>>>,
}

impl<Msg> LongPoolingServiceContext<Msg>
where
    Msg: Clone + Send + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn send(&self, topic: &str, msg: Msg) -> Result<(), mpsc::error::SendError<Msg>> {
        if let Some(tx) = self.0.subscription_channels.read().await.get(topic) {
            tx.send(msg).await
        } else {
            println!("No {topic} channel was found.");
            Ok(())
        }
    }

    pub async fn register(&self) -> ClientId {
        static CLIENT_ID_GEN: ClientIdGen = ClientIdGen::new();

        let client_id = CLIENT_ID_GEN.next();

        match self
            .0
            .client_id_channels
            .write()
            .await
            .entry(client_id.clone())
        {
            Entry::Occupied(_) => {
                unreachable!("impossible")
            }
            Entry::Vacant(v) => {
                let (tx, _rx) = broadcast::channel(1_000_000);
                v.insert(tx);
            }
        }

        client_id
    }

    pub async fn subscribe(&self, client_id: &ClientId, subscription: String) -> CometdResult<()> {
        if !self
            .0
            .client_id_channels
            .read()
            .await
            .contains_key(client_id)
        {
            return Err(CometdError::ClientDoesntExist(client_id.clone()));
        }

        if let Entry::Vacant(v) = self
            .0
            .subscription_channels
            .write()
            .await
            .entry(subscription.clone())
        {
            let (tx, mut rx) = mpsc::channel::<Msg>(1_000_000);
            let inner = self.0.clone();

            let subscription = subscription.clone();
            tokio::task::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    let client_id_channels = inner.client_id_channels.read().await;

                    for client_channel in inner
                        .client_ids_by_subscriptions
                        .read()
                        .await
                        .get(&subscription)
                        .into_iter()
                        .flatten()
                        .filter_map(|client_id| client_id_channels.get(client_id))
                    {
                        let _todo = client_channel.send(SubscriptionMessage {
                            subscription: subscription.clone(),
                            msg: msg.clone(),
                        });
                    }
                }
            });
            v.insert(tx);
        };

        self.0
            .client_ids_by_subscriptions
            .write()
            .await
            .entry(subscription)
            .or_default()
            .insert(client_id.clone());

        Ok(())
    }

    // TODO: Spawn task and send unsubscribe command through channel.
    pub async fn unsubscribe(&self, client_id: &str) {
        let (mut client_ids_by_subscriptions, mut subscription_channels, mut client_id_channels) = tokio::join!(
            self.0.client_ids_by_subscriptions.write(),
            self.0.subscription_channels.write(),
            self.0.client_id_channels.write()
        );

        client_ids_by_subscriptions.retain(|subscription, client_ids| {
            client_ids.remove(client_id);
            if client_ids.is_empty() {
                subscription_channels.remove(subscription);
                false
            } else {
                true
            }
        });

        client_id_channels.remove(client_id);
    }

    pub(crate) async fn get_client_receiver(
        &self,
        client_id: &str,
    ) -> CometdResult<broadcast::Receiver<SubscriptionMessage<Msg>>> {
        let rx = self
            .0
            .client_id_channels
            .read()
            .await
            .get(client_id)
            .ok_or_else(|| CometdError::ClientDoesntExist(client_id.into()))?
            .subscribe();

        Ok(rx)
    }
}
