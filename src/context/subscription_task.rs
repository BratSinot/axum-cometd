use crate::{
    context::{LongPoolingServiceContext, Subscription},
    messages::SubscriptionMessage,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::mpsc;

pub(crate) fn spawn(
    subscription: String,
    mut rx: mpsc::Receiver<JsonValue>,
    inner: Arc<LongPoolingServiceContext>,
) {
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            tracing::debug!(
                subscription = subscription,
                "`{subscription}` channel got message: `{msg:?}`."
            );

            let client_id_channels = inner.client_id_channels.read().await;

            for (client_id, client_channel) in inner
                .subscriptions_data
                .read()
                .await
                .get(&subscription)
                .into_iter()
                .flat_map(Subscription::client_ids)
                .filter_map(|client_id| client_id_channels.get(client_id).map(|v| (client_id, v)))
            {
                tracing::trace!(
                    client_id = %client_id,
                    subscription = subscription,
                    "Message `{msg:?}` from channel `{subscription}` was sent to client `{client_id}`."
                );

                if client_channel
                    .send(SubscriptionMessage {
                        subscription: subscription.clone(),
                        msg: msg.clone(),
                    })
                    .await
                    .is_err()
                {
                    tracing::error!(
                        client_id = %client_id,
                        subscription = subscription,
                        "Channel was closed!"
                    );
                }
            }
        }
    });
}
