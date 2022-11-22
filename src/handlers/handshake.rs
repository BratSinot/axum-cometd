use crate::{
    consts::{DEFAULT_INTERVAL_MS, DEFAULT_TIMEOUT_MS},
    messages::{Advice, Message, Reconnect},
    LongPoolingServiceContext,
};
use axum::{Extension, Json};

pub(crate) async fn handshake<Msg>(
    Extension(context): Extension<LongPoolingServiceContext<Msg>>,
    Json(messages): Json<Vec<Message>>,
) -> Result<Json<[Message; 1]>, Json<[Message; 1]>>
where
    Msg: Clone + Send + 'static,
{
    println!("handshake: `{messages:?}`.");

    let Message {
        advice,
        channel,
        id,
        minimum_version,
        supported_connection_types,
        ..
    } = messages
        .into_iter()
        .find(|message| message.channel.as_deref() == Some("/meta/handshake"))
        .ok_or_else(|| {
            Message::error(
                "no handshake channel",
                Some("/meta/handshake".to_owned()),
                None,
                None,
            )
        })?;

    let ret = if minimum_version.as_deref() != Some("1.0") {
        Err(Message::error(
            "unsupported protocol version",
            channel,
            None,
            id,
        ))
    } else if supported_connection_types
        .iter()
        .flatten()
        .any(|connection_type| connection_type == "long-polling")
    {
        Err(Message::error(
            "unsupported connection-type",
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
            advice: Some(Advice {
                reconnect: Some(Reconnect::Retry),
                timeout: Some(DEFAULT_TIMEOUT_MS),
                interval: Some(DEFAULT_INTERVAL_MS),
                ..Default::default()
            }),
            ..Default::default()
        })
    }?;

    Ok(Json([ret]))
}
