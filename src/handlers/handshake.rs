use crate::{
    error::HandlerResult,
    messages::{Advice, Message},
    types::{CookieId, Event, BAYEUX_BROWSER},
    CheckExt, LongPollingServiceContext,
};
use axum::{extract::State, http::HeaderMap, Extension, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::sync::Arc;

pub(crate) async fn handshake<AdditionalData>(
    State(context): State<Arc<LongPollingServiceContext<AdditionalData>>>,
    Extension(data): Extension<AdditionalData>,
    headers: HeaderMap,
    mut jar: CookieJar,
    Json([message]): Json<[Message; 1]>,
) -> HandlerResult<(CookieJar, Json<[Message; 1]>)>
where
    AdditionalData: Send + Sync + 'static,
{
    tracing::info!(
        channel = "/meta/handshake",
        request_id = %message.id.as_deref().unwrap_or("empty"),
        "Got handshake request: `{message:?}`."
    );

    let Message {
        channel,
        id,
        minimum_version,
        ..
    } = message;

    channel.check_or("/meta/handshake", || {
        Message::session_unknown(id.clone(), channel.clone(), None)
    })?;
    minimum_version.check_or("1.0", || {
        Message::wrong_minimum_version(id.clone(), minimum_version.clone())
    })?;

    #[allow(clippy::option_if_let_else)]
    let cookie_id = if let Some(cookie_id) = jar
        .get(BAYEUX_BROWSER)
        .map(Cookie::value)
        .map(CookieId::parse)
        .and_then(Result::ok)
    {
        cookie_id
    } else {
        let cookie_id = CookieId::gen();
        jar = jar.add(Cookie::new(BAYEUX_BROWSER, cookie_id.to_string()));
        cookie_id
    };

    let client_id = context.register(cookie_id).await.ok_or_else(|| {
        Message::session_unknown(id.clone(), channel.clone(), Some(Advice::handshake()))
    })?;

    let _ = context
        .tx
        .broadcast(Arc::new(Event::SessionAddedArgs {
            client_id,
            headers,
            data,
        }))
        .await;

    tracing::debug!(
        channel = "/meta/handshake",
        request_id = id.as_deref().unwrap_or("empty"),
        client_id = %client_id,
        "Got client_id: `{client_id}`."
    );

    Ok((
        jar,
        Json([Message {
            client_id: Some(client_id),
            version: Some("1.0".into()),
            supported_connection_types: Some(vec!["long-polling".into()]),
            advice: Some(Advice::retry(
                context.consts.timeout_ms,
                context.consts.interval_ms,
            )),
            ..Message::ok(id, channel)
        }]),
    ))
}
