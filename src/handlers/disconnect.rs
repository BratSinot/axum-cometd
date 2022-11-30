use crate::{messages::Message, LongPoolingServiceContext};
use axum::{extract::State, http::StatusCode, Json};
use std::{fmt::Debug, sync::Arc};

pub(crate) async fn disconnect<Msg>(
    State(context): State<Arc<LongPoolingServiceContext<Msg>>>,
    Json([message]): Json<[Message; 1]>,
) -> Result<Json<[Message; 1]>, StatusCode>
where
    Msg: Debug,
{
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
