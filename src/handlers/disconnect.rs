use crate::{error::HandlerResult, messages::Message, *};
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub(crate) async fn disconnect<AdditionalData, CustomData>(
    State(context): State<Arc<LongPollingServiceContext<AdditionalData, CustomData>>>,
    jar: CookieJar,
    Json(message): Json<Box<[Message; 1]>>,
) -> HandlerResult<Json<Box<[Message; 1]>>> {
    let [message] = *message;

    tracing::info!(
        channel = "/meta/disconnect",
        request_id = message.id.as_deref().unwrap_or("empty"),
        client_id = %message.client_id.as_ref().unwrap_or(&ClientId::zero()),
        "Got disconnect request: `{message:?}`."
    );

    let Message {
        id,
        channel,
        client_id,
        ..
    } = message;

    let session_unknown = || Message::session_unknown(id.clone(), channel.clone(), None);

    channel.check_or("/meta/disconnect", session_unknown)?;

    let cookie_id = jar.get_cookie_id().ok_or_else(session_unknown)?;
    let client_id = client_id.ok_or_else(session_unknown)?;
    context
        .check_client(&cookie_id, &client_id)
        .await
        .ok_or_else(session_unknown)?;

    context.unsubscribe(client_id).await;

    Ok(Json(Box::from([Message::ok(id, channel)])))
}
