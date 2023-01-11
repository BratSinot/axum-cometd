use crate::{error::HandlerResult, messages::Message, LongPollingServiceContext};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use std::sync::Arc;

pub(crate) async fn subscribe(
    State(context): State<Arc<LongPollingServiceContext>>,
    headers: HeaderMap,
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

        if subscription.is_empty() {
            return Err(Message::subscription_missing(id).into());
        }

        let client_id =
            client_id.ok_or_else(|| Message::session_unknown(id.clone(), channel.clone(), None))?;

        subscription.iter().try_for_each(|name| {
            context
                .channel_name_validator
                .validate_subscribe_channel_name(name)
                .then_some(())
                .ok_or(StatusCode::BAD_REQUEST)
        })?;

        context
            .subscribe(client_id, headers, subscription.clone())
            .await
            .map_err(|_| Message::session_unknown(id.clone(), channel.clone(), None))?;

        Ok(Json([Message {
            subscription: Some(subscription),
            ..Message::ok(id, channel)
        }]))
    }
}
