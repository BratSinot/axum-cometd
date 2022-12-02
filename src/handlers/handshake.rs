use crate::{
    messages::{Advice, Message},
    LongPoolingServiceContext,
};
use axum::{extract::State, Json};
use std::sync::Arc;

pub(crate) async fn handshake(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json([message]): Json<[Message; 1]>,
) -> Result<Json<[Message; 1]>, Json<[Message; 1]>> {
    tracing::info!("Got handshake request: `{message:?}`.");

    let Message {
        advice,
        channel,
        id,
        minimum_version,
        supported_connection_types,
        ..
    } = message;

    let ret = if channel.as_deref() != Some("/meta/handshake") {
        Err(Message::error(
            "no handshake channel",
            Some("/meta/handshake".to_owned()),
            None,
            None,
        ))
    } else if minimum_version.as_deref() != Some("1.0") {
        Err(Message::error(
            "unsupported protocol version",
            channel,
            None,
            id,
        ))
    } else if !supported_connection_types
        .iter()
        .flatten()
        .any(|connection_type| connection_type == "long-polling")
    {
        Err(Message::error(
            "unsupported connectionType",
            channel,
            None,
            id,
        ))
    } else if advice.as_ref().and_then(Advice::interval) != Some(0) {
        Err(Message::error("unsupported interval", channel, None, id))
    } else {
        let client_id = context.register().await;

        Ok(Message {
            id,
            channel,
            successful: Some(true),
            client_id: Some(client_id),
            version: Some("1.0".into()),
            supported_connection_types: Some(vec!["long-polling".into()]),
            advice: Some(Advice::retry(
                context.consts.timeout_ms,
                context.consts.interval_ms,
            )),
            ..Default::default()
        })
    }?;

    Ok(Json([ret]))
}
