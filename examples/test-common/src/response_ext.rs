use axum::{async_trait, response::Response};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;

#[async_trait]
pub trait ResponseExt {
    async fn to<T: DeserializeOwned>(self) -> T;
    async fn to_json(self) -> JsonValue;
}

#[async_trait]
impl ResponseExt for Response {
    async fn to<T: DeserializeOwned>(self) -> T {
        let body = hyper::body::to_bytes(self.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn to_json(self) -> JsonValue {
        self.to().await
    }
}
