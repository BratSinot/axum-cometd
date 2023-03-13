use crate::{
    error::HandlerResult,
    messages::{Advice, Message, SubscriptionMessage},
    types::ClientReceiverError,
    CookieJarExt as _, LongPollingServiceContext,
};
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use core::time::Duration;
use serde_json::json;

#[inline]
pub(super) async fn wait_client_message_handle<AdditionalData, CustomData>(
    context: &LongPollingServiceContext<AdditionalData, CustomData>,
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

    let cookie_id = jar.get_cookie_id().ok_or_else(session_unknown)?;
    let client_id = client_id.ok_or_else(session_unknown)?;
    context
        .check_client(cookie_id, &client_id)
        .await
        .ok_or_else(session_unknown)?;

    let timeout = advice
        .and_then(|advice| advice.timeout)
        .map_or(context.consts.timeout, Duration::from_millis);

    let mut rx = context
        .get_client_receiver(&client_id)
        .await
        .ok_or_else(session_unknown)?;

    let SubscriptionMessage {
        channel: recv_channel,
        msg,
    } = rx
        .recv_timeout(timeout)
        .await
        .map_err(|error| {
            client_receiver_error_to_message(&error, id.clone(), channel.clone(), context)
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
fn client_receiver_error_to_message<AdditionalData, CustomData>(
    error: &ClientReceiverError,
    id: Option<String>,
    channel: Option<String>,
    context: &LongPollingServiceContext<AdditionalData, CustomData>,
) -> Message {
    match *error {
        ClientReceiverError::Elapsed(ref _err) => Message {
            advice: Some(Advice::retry(
                context.consts.timeout,
                context.consts.interval,
            )),
            ..Message::ok(id, channel)
        },
        ClientReceiverError::AlreadyLocked(ref _err) => Message {
            id,
            channel,
            successful: Some(false),
            error: Some("Two connection with same client_id.".to_owned()),
            ..Default::default()
        },
    }
}
