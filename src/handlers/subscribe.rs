use crate::{error::HandlerResult, messages::Message, CheckExt, LongPollingServiceContext};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub(crate) async fn subscribe(
    State(context): State<Arc<LongPollingServiceContext>>,
    headers: HeaderMap,
    jar: CookieJar,
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

    let session_unknown = || Message::session_unknown(id.clone(), channel.clone(), None);

    channel.check_or("/meta/subscribe", session_unknown)?;

    let client_id = client_id.ok_or_else(session_unknown)?;
    context
        .check_client(&jar, &client_id)
        .await
        .ok_or_else(session_unknown)?;

    let subscription = subscription.ok_or_else(|| Message::subscription_missing(id.clone()))?;
    subscription
        .is_empty()
        .check_or(&false, || Message::subscription_missing(id.clone()))?;

    subscription.iter().try_for_each(|name| {
        context
            .channel_name_validator
            .validate_subscribe_channel_name(name)
            .check(&true, StatusCode::BAD_REQUEST)
    })?;

    context
        .subscribe(client_id, headers, subscription.clone())
        .await;

    Ok(Json([Message {
        subscription: Some(subscription),
        ..Message::ok(id, channel)
    }]))
}
