use crate::{consts::DEFAULT_TIMEOUT_MS, handlers::*, LongPoolingServiceContext};
use axum::{routing::post, Extension, Router};
use serde::Serialize;
use std::{fmt::Debug, time::Duration};
use tower_http::timeout::TimeoutLayer;

impl<Msg> LongPoolingServiceContext<Msg> {
    #[inline]
    pub fn build_router(&self, base_path: &str) -> Router
    where
        Msg: Debug + Clone + Send + Serialize + 'static,
    {
        Router::new()
            .nest(
                base_path,
                Router::new()
                    .route("/", post(subscribe::<Msg>))
                    .route("/handshake", post(handshake::<Msg>))
                    .route("/connect", post(connect::<Msg>))
                    .route("/disconnect", post(disconnect::<Msg>))
                    .layer(Extension(self.clone())),
            )
            .layer(TimeoutLayer::new(Duration::from_millis(DEFAULT_TIMEOUT_MS)))
    }
}
