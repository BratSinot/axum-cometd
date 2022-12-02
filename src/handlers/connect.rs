use crate::{
    error::HandlerResult,
    messages::{Advice, Message, SubscriptionMessage},
    types::ClientId,
    LongPoolingServiceContext,
};
use axum::{extract::State, Json};
use serde_json::json;
use std::{sync::Arc, time::Duration};

pub(crate) async fn connect(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<Json<[Message; 2]>> {
    tracing::info!("Got connect request: `{message:?}`.");

    let Message {
        id,
        channel,
        connection_type,
        advice,
        client_id,
        ..
    } = message;

    if channel.as_deref() != Some("/meta/connect") {
        Err(Message::error(
            "no connect channel",
            Some("/meta/connect".to_owned()),
            None,
            None,
        ))?;
    }

    check_supported_connect_type(&connection_type, &channel, &client_id, &id)?;

    let client_id = client_id
        .ok_or_else(|| Message::error("empty clientId", channel.clone(), None, id.clone()))?;
    let timeout = advice
        .and_then(|advice| advice.timeout)
        .unwrap_or(context.consts.timeout_ms);

    let mut rx = context
        .get_client_receiver(client_id)
        .await
        .map_err(|error| {
            Message::error(
                error.to_string(),
                channel.clone(),
                Some(client_id),
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
                Some(client_id),
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
    client_id: &Option<ClientId>,
    id: &Option<String>,
) -> Result<(), Message> {
    if connection_type.as_deref() != Some("long-polling") {
        Err(Message::error(
            "unsupported connectionType",
            channel.clone(),
            *client_id,
            id.clone(),
        ))
    } else {
        Ok(())
    }
}
