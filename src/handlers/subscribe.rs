use crate::{error::HandlerResult, messages::Message, LongPoolingServiceContext};
use axum::{extract::State, Json};
use std::sync::Arc;

pub(crate) async fn subscribe(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<Json<[Message; 1]>> {
    tracing::info!("Got subscribe request: `{message:?}`.");

    let Message {
        id,
        channel,
        subscription,
        client_id,
        ..
    } = message;

    if channel.as_deref() != Some("/meta/subscribe") {
        Err(Message::session_unknown(id, channel, None).into())
    } else {
        let subscription = subscription.ok_or_else(|| Message::subscription_missing(id.clone()))?;
        let client_id =
            client_id.ok_or_else(|| Message::session_unknown(id.clone(), channel.clone(), None))?;

        context
            .subscribe(client_id, &subscription)
            .await
            .map_err(|_| Message::session_unknown(id.clone(), channel.clone(), None))?;

        Ok(Json([Message {
            id,
            channel,
            subscription: Some(subscription),
            successful: Some(true),
            ..Default::default()
        }]))
    }
}
