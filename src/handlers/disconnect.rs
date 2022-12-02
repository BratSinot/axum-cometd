use crate::{error::HandlerResult, messages::Message, LongPoolingServiceContext};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub(crate) async fn disconnect(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<Json<[Message; 1]>> {
    tracing::info!("Got disconnect request: `{message:?}`.");

    let Message {
        id,
        channel,
        client_id,
        ..
    } = message;

    let ret = if channel.as_deref() != Some("/meta/disconnect") {
        Ok(Message::error("no disconnect channel", None, None, None))
    } else if let Some(client_id) = client_id {
        context.unsubscribe(client_id).await;

        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(Message::error("empty clientId", channel, client_id, id))
    }?;

    Ok(Json([ret]))
}
