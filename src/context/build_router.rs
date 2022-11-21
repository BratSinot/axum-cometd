use crate::{handlers::*, LongPoolingServiceContext};
use axum::{routing::post, Extension, Router};
use serde::Serialize;
use std::fmt::Debug;

impl<Msg> LongPoolingServiceContext<Msg> {
    pub fn build_router(&self, base_path: &str) -> Router
    where
        Msg: Debug + Clone + Send + Serialize + 'static,
    {
        Router::new().nest(
            base_path,
            Router::new()
                .route("/", post(subscribe::<Msg>))
                .route("/handshake", post(handshake::<Msg>))
                .route("/connect", post(connect::<Msg>))
                .route("/disconnect", post(disconnect::<Msg>))
                .layer(Extension(self.clone())),
        )
    }
}
