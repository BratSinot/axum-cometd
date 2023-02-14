use crate::{messages::Message, *};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Extension, Json,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub(crate) async fn subscribe<AdditionalData, CustomData>(
    State(context): State<Arc<LongPollingServiceContext<AdditionalData, CustomData>>>,
    Extension(data): Extension<AdditionalData>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(message): Json<Box<[Message; 1]>>,
) -> HandlerResult<Json<Box<[Message; 1]>>>
where
    AdditionalData: Send + Sync + 'static,
    CustomData: Send + Sync + 'static,
{
    let [message] = *message;

    tracing::info!(
        channel = "/meta/subscribe",
        request_id = message.id.as_deref().unwrap_or("empty"),
        client_id = %message.client_id.as_ref().unwrap_or(&ClientId::zero()),
        "Got subscribe request: `{message:?}`."
    );

    let Message {
        id,
        channel,
        subscription,
        client_id,
        ..
    } = message;

    let session_unknown = || Message::session_unknown(id.clone(), channel.clone(), None);

    channel.check_or("/meta/subscribe", session_unknown)?;

    let cookie_id = jar.get_cookie_id().ok_or_else(session_unknown)?;
    let client_id = client_id.ok_or_else(session_unknown)?;
    context
        .check_client(&cookie_id, &client_id)
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

    context.subscribe(&client_id, &subscription).await;

    let _ = context
        .tx
        .broadcast(Arc::new(Event::Subscribe {
            client_id,
            headers,
            channels: subscription.clone(),
            data,
        }))
        .await;

    Ok(Json(Box::from([Message {
        subscription: Some(subscription),
        ..Message::ok(id, channel)
    }])))
}
