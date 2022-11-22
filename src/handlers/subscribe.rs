use crate::{messages::Message, LongPoolingServiceContext};
use axum::{Extension, Json};
use std::fmt::Debug;

pub(crate) async fn subscribe<Msg>(
    Extension(context): Extension<LongPoolingServiceContext<Msg>>,
    Json(messages): Json<Vec<Message>>,
) -> Result<Json<[Message; 1]>, Json<[Message; 1]>>
where
    Msg: Debug + Clone + Send + 'static,
{
    tracing::info!("Got subscribe request: `{messages:?}`.");

    let Message {
        id,
        channel,
        subscription,
        client_id,
        ..
    } = messages
        .into_iter()
        .find(|message| message.channel.as_deref() == Some("/meta/subscribe"))
        .ok_or_else(|| {
            Message::error(
                "no subscribe channel",
                Some("/meta/disconnect".to_owned()),
                None,
                None,
            )
        })?;

    let subscription = subscription.ok_or_else(|| {
        Message::error(
            "empty subscription",
            channel.clone(),
            client_id.clone(),
            id.clone(),
        )
    })?;
    let client_id = client_id
        .ok_or_else(|| Message::error("empty clientId", channel.clone(), None, id.clone()))?;

    context
        .subscribe(&client_id, &subscription)
        .await
        .map_err(|error| {
            Message::error(
                error.to_string(),
                channel.clone(),
                Some(client_id.clone()),
                id.clone(),
            )
        })?;

    Ok(Json([Message {
        id,
        channel,
        subscription: Some(subscription),
        successful: Some(true),
        ..Default::default()
    }]))
}
