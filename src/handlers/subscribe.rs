use crate::{messages::Message, LongPoolingServiceContext};
use axum::{extract::State, Json};
use std::{fmt::Debug, sync::Arc};

pub(crate) async fn subscribe<Msg>(
    State(context): State<Arc<LongPoolingServiceContext<Msg>>>,
    Json([message]): Json<[Message; 1]>,
) -> Result<Json<[Message; 1]>, Json<[Message; 1]>>
where
    Msg: Debug + Clone + Send + Sync + 'static,
{
    tracing::info!("Got subscribe request: `{message:?}`.");

    let Message {
        id,
        channel,
        subscription,
        client_id,
        ..
    } = message;

    if channel.as_deref() != Some("/meta/subscribe") {
        return Err(Json([Message::error(
            "no subscribe channel",
            Some("/meta/disconnect".to_owned()),
            None,
            None,
        )]));
    };

    let subscription = subscription.ok_or_else(|| {
        Message::error("empty subscription", channel.clone(), client_id, id.clone())
    })?;
    let client_id = client_id
        .ok_or_else(|| Message::error("empty clientId", channel.clone(), None, id.clone()))?;

    context
        .subscribe(client_id, &subscription)
        .await
        .map_err(|error| {
            Message::error(
                error.to_string(),
                channel.clone(),
                Some(client_id),
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
