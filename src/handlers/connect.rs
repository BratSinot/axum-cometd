mod publish;
mod wait_message;

use crate::{error::HandlerResult, messages::Message, LongPollingServiceContext};
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use publish::*;
use std::sync::Arc;
use wait_message::*;

pub(crate) async fn connect<AdditionalData, CustomData>(
    State(context): State<Arc<LongPollingServiceContext<AdditionalData, CustomData>>>,
    jar: CookieJar,
    Json(messages): Json<Vec<Message>>,
) -> HandlerResult<Json<Vec<Message>>> {
    tracing::debug!("Got connect request: `{messages:?}`.");

    let ret = match <[_; 1]>::try_from(messages) {
        Ok([message]) => {
            if message.channel.as_deref() == Some("/meta/connect") {
                wait_client_message_handle(&context, jar, message).await?
            } else {
                publish_handle(&context, jar, vec![message]).await?
            }
        }
        Err(messages) => publish_handle(&context, jar, messages).await?,
    };

    Ok(Json(ret))
}
