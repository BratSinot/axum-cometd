mod publish;
mod wait_message;

use crate::{error::HandlerResult, messages::Message, LongPoolingServiceContext};
use axum::{extract::State, Json};
use publish::*;
use std::sync::Arc;
use wait_message::*;

pub(crate) async fn connect(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json(messages): Json<Vec<Message>>,
) -> HandlerResult<Json<Vec<Message>>> {
    tracing::info!("Got connect request: `{messages:?}`.");

    let ret = match <[_; 1]>::try_from(messages) {
        Ok([message]) => {
            if message.channel.as_deref() == Some("/meta/connect") {
                wait_client_message_handle(&context, message).await?
            } else {
                publish_handle(&context, vec![message]).await?
            }
        }
        Err(messages) => publish_handle(&context, messages).await?,
    };

    Ok(Json(ret))
}
