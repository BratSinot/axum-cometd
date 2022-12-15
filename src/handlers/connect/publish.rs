use crate::{
    messages::{Advice, Message},
    LongPollingServiceContext, SendError,
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
                        match context.send(&channel, data.unwrap_or_default()).await {
                            Ok(()) => {}
                            Err(SendError::Closed) => {
                                tracing::error!(
                                    client_id = %client_id,
                                    channel = channel,
                                    "Channel was closed!"
                                );
                            }
                            Err(SendError::ClientWasntFound(_)) => unreachable!(
                                "LongPollingServiceContext::send shouldn't return ClientWasntFound"
                            ),
                            Err(SendError::InvalidChannel) => {
                                tracing::error!(
                                    client_id = %client_id,
                                    channel = channel,
                                    "Invalid channel: `{channel}`!"
                                );
                            }
                        }

                        Message::ok(id, Some(channel))
                    } else {
                        Message::session_unknown(id, Some(channel), None)
                    }
                }
            };
        }

        Ok(messages)
    }
}
