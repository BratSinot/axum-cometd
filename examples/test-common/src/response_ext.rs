use axum::{async_trait, response::Response};
use serde_json::Value as JsonValue;

#[async_trait]
pub trait ResponseExt {
    async fn to_json(self) -> JsonValue;
}

#[async_trait]
impl ResponseExt for Response {
    async fn to_json(self) -> JsonValue {
        let body = hyper::body::to_bytes(self.into_body()).await.unwrap();
        serde_json::from_slice::<JsonValue>(&body).unwrap()
    }
}
