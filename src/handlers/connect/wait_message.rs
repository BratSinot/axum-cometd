use crate::{
    error::HandlerResult,
    messages::{Advice, Message, SubscriptionMessage},
    types::ClientReceiverError,
    LongPollingServiceContext,
};
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use serde_json::json;
use std::time::Duration;

#[inline]
pub(super) async fn wait_client_message_handle(
    context: &LongPollingServiceContext,
    jar: CookieJar,
    message: Message,
) -> HandlerResult<Vec<Message>> {
    let Message {
        id,
        channel,
        advice,
        client_id,
        ..
    } = message;
    let session_unknown =
        || Message::session_unknown(id.clone(), channel.clone(), Some(Advice::handshake()));

    let client_id = client_id.ok_or_else(session_unknown)?;
    context
        .check_client(&jar, &client_id)
        .await
        .ok_or_else(session_unknown)?;

    let timeout = advice
        .and_then(|advice| advice.timeout)
        .unwrap_or(context.consts.timeout_ms);

    let mut rx = context
        .get_client_receiver(&client_id)
        .await
        .ok_or_else(session_unknown)?;

    let SubscriptionMessage {
        channel: recv_channel,
        msg,
    } = rx
        .recv_timeout(Duration::from_millis(timeout))
        .await
        .map_err(|error| {
            client_receiver_error_to_message(error, id.clone(), channel.clone(), context)
        })?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(vec![
        Message {
            channel: Some(recv_channel),
            data: Some(json!(msg)),
            ..Default::default()
        },
        Message::ok(id, channel),
    ])
}

#[inline]
fn client_receiver_error_to_message(
    error: ClientReceiverError,
    id: Option<String>,
    channel: Option<String>,
    context: &LongPollingServiceContext,
) -> Message {
    match error {
        ClientReceiverError::Elapsed(_) => Message {
            advice: Some(Advice::retry(
                context.consts.timeout_ms,
                context.consts.interval_ms,
            )),
            ..Message::ok(id, channel)
        },
        ClientReceiverError::AlreadyLocked(_) => Message {
            id,
            channel,
            successful: Some(false),
            error: Some("Two connection with same client_id.".to_owned()),
            ..Default::default()
        },
    }
}
