use crate::ext::ReceiverExt;
use crate::{
    consts::{DEFAULT_INTERVAL_MS, DEFAULT_TIMEOUT_MS},
    messages::{Advice, Message, Reconnect, SubscriptionMessage},
    LongPoolingServiceContext,
};
use axum::{Extension, Json};
use serde::Serialize;
use serde_json::json;
use std::{fmt::Debug, time::Duration};

pub(crate) async fn connect<Msg>(
    Extension(context): Extension<LongPoolingServiceContext<Msg>>,
    Json(messages): Json<Vec<Message>>,
) -> Result<Json<[Message; 2]>, Json<[Message; 1]>>
where
    Msg: Debug + Clone + Serialize + Send + 'static,
{
    println!("connect: `{messages:?}`.");

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
        .unwrap_or(DEFAULT_TIMEOUT_MS);

    let SubscriptionMessage { subscription, msg } = context
        .get_client_receiver(&client_id)
        .await
        .map_err(|error| {
            Message::error(
                error.to_string(),
                channel.clone(),
                Some(client_id.clone()),
                id.clone(),
            )
        })?
        .recv_ignore_lagged_timeout(Duration::from_millis(timeout))
        .await
        .map_err(|_| Message {
            id: id.clone(),
            channel: channel.clone(),
            successful: Some(true),
            advice: Some(Advice::retry()),
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
