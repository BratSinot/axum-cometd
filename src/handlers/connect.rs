mod meta_connect;
mod publish;

use crate::{error::HandlerResult, messages::Message, LongPoolingServiceContext};
use axum::{extract::State, Json};
use meta_connect::*;
use publish::*;
use std::sync::Arc;

pub(crate) async fn connect(
    State(context): State<Arc<LongPoolingServiceContext>>,
    Json(messages): Json<Vec<Message>>,
) -> HandlerResult<Json<Vec<Message>>> {
    tracing::info!("Got connect request: `{messages:?}`.");

    let ret = match <[_; 1]>::try_from(messages) {
        Ok([message]) => meta_connect_handle(&context, message).await?,
        Err(messages) => publish_handle(&context, messages).await?,
    };

    Ok(Json(ret))
}
