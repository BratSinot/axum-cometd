use crate::{
    context::Channel,
    messages::{Advice, Message},
    LongPollingServiceContext,
};
use axum::http::StatusCode;

#[inline]
pub(super) async fn publish_handle(
    context: &LongPollingServiceContext,
    mut messages: Vec<Message>,
) -> Result<Vec<Message>, StatusCode> {
    if messages.iter().any(|message| {
        message
            .channel
            .as_ref()
            .map(|channel| channel.contains("/meta/"))
            .unwrap_or(false)
    }) {
        Err(StatusCode::BAD_REQUEST)
    } else {
        let subscriptions_data_read_guard = context.subscriptions_data().read().await;

        for message in messages.iter_mut() {
            let Message {
                id,
                channel,
                data,
                client_id,
                ..
            } = std::mem::take(message);

            *message = match (channel, client_id) {
                (None, _) => Message::channel_missing(id),
                (channel, None) => Message::session_unknown(id, channel, Some(Advice::handshake())),
                (Some(channel), Some(client_id)) => {
                    if context.check_client_id(&client_id).await {
                        if let Some(tx) =
                            subscriptions_data_read_guard.get(&channel).map(Channel::tx)
                        {
                            if tx.send(data.unwrap_or_default()).await.is_err() {
                                tracing::error!(
                                    client_id = %client_id,
                                    channel = channel,
                                    "Channel was closed!"
                                );
                            }
                        } else {
                            tracing::trace!(
                                client_id = %client_id,
                                channel = channel,
                                "No `{channel}` channel was found for message: `{data:?}`."
                            );
                        }

                        Message {
                            id,
                            channel: Some(channel),
                            successful: Some(true),
                            ..Default::default()
                        }
                    } else {
                        Message::session_unknown(id, Some(channel), None)
                    }
                }
            };
        }

        Ok(messages)
    }
}
