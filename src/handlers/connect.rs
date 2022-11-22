use crate::{
    messages::{Advice, Message, SubscriptionMessage},
    LongPoolingServiceContext,
};
use axum::{Extension, Json};
use serde::Serialize;
use serde_json::json;
use std::{fmt::Debug, sync::Arc, time::Duration};

pub(crate) async fn connect<Msg>(
    Extension(context): Extension<Arc<LongPoolingServiceContext<Msg>>>,
    Json(messages): Json<Vec<Message>>,
) -> Result<Json<[Message; 2]>, Json<[Message; 1]>>
where
    Msg: Debug + Clone + Serialize + Send + 'static,
{
    tracing::info!("Got connect request: `{messages:?}`.");

    let Message {
        id,
        channel,
        connection_type,
        advice,
        client_id,
        ..
    } = messages
        .into_iter()
        .find(|message| message.channel.as_deref() == Some("/meta/connect"))
        .ok_or_else(|| {
            Message::error(
                "no connect channel",
                Some("/meta/connect".to_owned()),
                None,
                None,
            )
        })?;

    check_supported_connect_type(&connection_type, &channel, &client_id, &id)?;

    let client_id = client_id
        .ok_or_else(|| Message::error("empty clientId", channel.clone(), None, id.clone()))?;
    let timeout = advice
        .and_then(|advice| advice.timeout)
        .unwrap_or(context.consts.timeout_ms);

    let mut rx = context
        .get_client_receiver(&client_id)
        .await
        .map_err(|error| {
            Message::error(
                error.to_string(),
                channel.clone(),
                Some(client_id.clone()),
                id.clone(),
            )
        })?;

    let SubscriptionMessage { subscription, msg } = rx
        .recv_timeout(Duration::from_millis(timeout))
        .await
        .map_err(|_| Message {
            id: id.clone(),
            channel: channel.clone(),
            successful: Some(true),
            advice: Some(Advice::retry(
                context.consts.timeout_ms,
                context.consts.interval_ms,
            )),
            ..Default::default()
        })?
        .ok_or_else(|| {
            Message::error(
                "channel was closed",
                channel.clone(),
                Some(client_id.clone()),
                id.clone(),
            )
        })?;

    Ok(Json([
        Message {
            channel: Some(subscription),
            data: Some(json!(msg)),
            ..Default::default()
        },
        Message {
            id,
            channel,
            successful: Some(true),
            ..Default::default()
        },
    ]))
}

#[inline(always)]
fn check_supported_connect_type(
    connection_type: &Option<String>,
    channel: &Option<String>,
    client_id: &Option<String>,
    id: &Option<String>,
) -> Result<(), Message> {
    if connection_type.as_deref() != Some("long-polling") {
        Err(Message::error(
            "unsupported connectionType",
            channel.clone(),
            client_id.clone(),
            id.clone(),
        ))
    } else {
        Ok(())
    }
}
