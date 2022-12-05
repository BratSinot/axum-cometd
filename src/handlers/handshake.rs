use crate::{
    error::HandlerResult,
    messages::{Advice, Message},
    LongPoolingServiceContext,
};
use axum::{extract::State, http::HeaderMap, Json};
use std::sync::Arc;

pub(crate) async fn handshake(
    State(context): State<Arc<LongPoolingServiceContext>>,
    headers: HeaderMap,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<Json<[Message; 1]>> {
    tracing::info!("Got handshake request: `{message:?}`.");

    let Message {
        channel,
        id,
        minimum_version,
        ..
    } = message;

    if channel.as_deref() != Some("/meta/handshake") {
        Err(Message::session_unknown(id, channel, None).into())
    } else if minimum_version.as_deref() != Some("1.0") {
        Err(Message::wrong_minimum_version(id, minimum_version).into())
    } else {
        let client_id = context.register(headers).await;

        Ok(Json([Message {
            id,
            channel,
            successful: Some(true),
            client_id: Some(client_id),
            version: Some("1.0".into()),
            supported_connection_types: Some(vec!["long-polling".into()]),
            advice: Some(Advice::retry(
                context.consts().timeout_ms,
                context.consts().interval_ms,
            )),
            ..Default::default()
        }]))
    }
}
