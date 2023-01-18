use crate::ext::CheckExt;
use crate::{
    messages::{Advice, Message},
    LongPollingServiceContext, SendError,
};
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;

#[inline]
pub(super) async fn publish_handle(
    context: &LongPollingServiceContext,
    jar: CookieJar,
    mut messages: Vec<Message>,
) -> Result<Vec<Message>, StatusCode> {
    is_contains_meta_channel(&messages).check(&false, StatusCode::BAD_REQUEST)?;

    for message in &mut messages {
        let Message {
            id,
            channel,
            data,
            client_id,
            ..
        } = core::mem::take(message);

        *message = match (channel, client_id) {
            (None, _) => Message::channel_missing(id),
            (channel, None) => Message::session_unknown(id, channel, Some(Advice::handshake())),
            (Some(channel), Some(client_id)) => {
                if context.check_client(&jar, &client_id).await.is_some() {
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

#[inline]
fn is_contains_meta_channel(messages: &[Message]) -> bool {
    messages.iter().any(|message| {
        message
            .channel
            .as_ref()
            .map_or(false, |channel| channel.contains("/meta/"))
    })
}
