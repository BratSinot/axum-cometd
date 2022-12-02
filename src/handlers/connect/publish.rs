use crate::{context::Subscription, messages::Message, LongPoolingServiceContext};
use axum::http::StatusCode;

#[inline]
pub(super) async fn publish_handle(
    context: &LongPoolingServiceContext,
    mut messages: Vec<Message>,
) -> Result<Vec<Message>, StatusCode> {
    if messages.iter().any(|message| {
        if let Some(channel) = &message.channel {
            channel.contains("/meta/")
        } else {
            true
        }
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

            *message = match (channel, data, client_id) {
                (None, _, _) => Message::error("400::channel_missing", None, None, id),
                (_, None, _) => Message::error("400::data_missing", None, None, id),
                (_, _, None) => Message::error("400::client_id_missing", None, None, id),
                (Some(subscription), Some(data), Some(client_id)) => {
                    if context.check_client_id(&client_id).await {
                        if let Some(tx) = subscriptions_data_read_guard
                            .get(&subscription)
                            .map(Subscription::tx)
                        {
                            if tx.send(data).await.is_err() {
                                tracing::error!(
                                    client_id = %client_id,
                                    subscription = subscription,
                                    "Channel was closed!"
                                );
                            }
                        } else {
                            tracing::trace!(
                                client_id = %client_id,
                                subscription = subscription,
                                "No `{subscription}` channel was found for message: `{data:?}`."
                            );
                        }

                        Message {
                            id,
                            channel: Some(subscription),
                            successful: Some(true),
                            ..Default::default()
                        }
                    } else {
                        Message::error("402::session_unknown", Some(subscription), None, id)
                    }
                }
            };
        }

        Ok(messages)
    }
}
