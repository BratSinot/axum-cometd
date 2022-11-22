use crate::{context::InnerLongPoolingServiceContext, messages::SubscriptionMessage};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::mpsc;

pub(crate) fn spawn<Msg>(
    subscription: String,
    mut rx: mpsc::Receiver<Msg>,
    inner: Arc<InnerLongPoolingServiceContext<Msg>>,
) where
    Msg: Debug + Clone + Send + 'static,
{
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            tracing::debug!(
                subscription = subscription,
                "`{subscription}` channel got message: `{msg:?}`."
            );

            let client_id_channels = inner.client_id_channels.read().await;

            for (client_id, client_channel) in inner
                .client_ids_by_subscriptions
                .read()
                .await
                .get(&subscription)
                .into_iter()
                .flatten()
                .filter_map(|client_id| client_id_channels.get(client_id).map(|v| (client_id, v)))
            {
                tracing::trace!(
                    client_id = client_id,
                    subscription = subscription,
                    "Message `{msg:?}` from channel `{subscription}` was sent to client `{client_id}`."
                );

                let _todo = client_channel.send(SubscriptionMessage {
                    subscription: subscription.clone(),
                    msg: msg.clone(),
                });
            }
        }
    });
}
