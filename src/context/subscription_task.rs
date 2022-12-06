use crate::{
    context::{Channel, LongPollingServiceContext},
    messages::SubscriptionMessage,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::mpsc;

pub(crate) fn spawn(
    channel: String,
    mut rx: mpsc::Receiver<JsonValue>,
    inner: Arc<LongPollingServiceContext>,
) {
    tokio::task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            tracing::debug!(
                channel = channel,
                "`{channel}` channel got message: `{msg:?}`."
            );

            let client_id_channels = inner.client_id_senders.read().await;

            for (client_id, client_channel) in inner
                .channels_data
                .read()
                .await
                .get(&channel)
                .into_iter()
                .flat_map(Channel::client_ids)
                .filter_map(|client_id| client_id_channels.get(client_id).map(|v| (client_id, v)))
            {
                tracing::trace!(
                    client_id = %client_id,
                    channel = channel,
                    "Message `{msg:?}` from channel `{channel}` was sent to client `{client_id}`."
                );

                if client_channel
                    .send(SubscriptionMessage {
                        channel: channel.clone(),
                        msg: msg.clone(),
                    })
                    .await
                    .is_err()
                {
                    tracing::error!(
                        client_id = %client_id,
                        channel = channel,
                        "Channel was closed!"
                    );
                }
            }
        }
    });
}
