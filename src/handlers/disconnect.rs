use crate::{
    error::HandlerResult, messages::Message, CheckExt, CookieJarExt, LongPollingServiceContext,
    ZERO_CLIENT_ID,
};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub(crate) async fn disconnect<AdditionalData, CustomData>(
    State(context): State<Arc<LongPollingServiceContext<AdditionalData, CustomData>>>,
    jar: CookieJar,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<StatusCode> {
    tracing::info!(
        channel = "/meta/disconnect",
        request_id = message.id.as_deref().unwrap_or("empty"),
        client_id = %message.client_id.unwrap_or(ZERO_CLIENT_ID),
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
        .check_client(cookie_id, &client_id)
        .await
        .ok_or_else(session_unknown)?;

    context.unsubscribe(client_id).await;

    Ok(StatusCode::BAD_REQUEST)
}
