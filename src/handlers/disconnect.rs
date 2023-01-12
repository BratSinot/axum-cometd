use crate::{error::HandlerResult, messages::Message, LongPollingServiceContext};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub(crate) async fn disconnect(
    State(context): State<Arc<LongPollingServiceContext>>,
    jar: CookieJar,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<StatusCode> {
    tracing::info!("Got disconnect request: `{message:?}`.");

    let Message {
        id,
        channel,
        client_id,
        ..
    } = message;

    if channel.as_deref() != Some("/meta/disconnect") {
        Err(Message::session_unknown(id, channel, None).into())
    } else {
        let session_unknown = || Message::session_unknown(id.clone(), channel.clone(), None);

        let client_id = client_id.ok_or_else(session_unknown)?;
        context
            .check_client(&jar, &client_id)
            .await
            .ok_or_else(session_unknown)?;

        context.unsubscribe(client_id).await;

        Ok(StatusCode::BAD_REQUEST)
    }
}
