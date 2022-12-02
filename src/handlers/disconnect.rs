use crate::{error::HandlerResult, messages::Message, LongPoolingServiceContext};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub(crate) async fn disconnect(
    State(context): State<Arc<LongPoolingServiceContext>>,
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
        let client_id =
            client_id.ok_or_else(|| Message::session_unknown(id.clone(), channel.clone(), None))?;

        context.unsubscribe(client_id).await;

        Ok(StatusCode::BAD_REQUEST)
    }
}
