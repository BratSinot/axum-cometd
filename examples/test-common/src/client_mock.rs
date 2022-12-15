use crate::ResponseExt;
use ahash::AHashMap;
use axum::{
    http::{header::CONTENT_TYPE, Request, StatusCode},
    response::Response,
    Router,
};
use hyper::Body;
use serde_json::{json, Value as JsonValue};
use std::sync::atomic::{AtomicU64, Ordering};
use tower::ServiceExt;

#[derive(Debug)]
pub struct ClientMock {
    handshake_endpoint: String,
    subscribe_endpoint: String,
    connect_endpoint: String,
    disconnect_endpoint: String,

    id: AtomicU64,
    client_id: Option<String>,
    router: Router,
}

impl ClientMock {
    #[inline]
    pub fn create(
        handshake_base_path: &str,
        subscribe_base_path: &str,
        connect_base_path: &str,
        disconnect_base_path: &str,
        router: Router,
    ) -> Self {
        Self {
            handshake_endpoint: format!("{handshake_base_path}/handshake"),
            subscribe_endpoint: subscribe_base_path.to_owned(),
            connect_endpoint: format!("{connect_base_path}/connect"),
            disconnect_endpoint: format!("{disconnect_base_path}/disconnect"),
            id: AtomicU64::new(0),
            client_id: Default::default(),
            router,
        }
    }

    #[inline(always)]
    pub fn handshake_endpoint(&self) -> &str {
        &self.handshake_endpoint
    }

    #[inline(always)]
    pub fn subscribe_endpoint(&self) -> &str {
        &self.subscribe_endpoint
    }

    #[inline(always)]
    pub fn connect_endpoint(&self) -> &str {
        &self.connect_endpoint
    }

    #[inline(always)]
    pub fn disconnect_endpoint(&self) -> &str {
        &self.disconnect_endpoint
    }

    #[inline(always)]
    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    pub async fn handshake(&mut self) {
        let body = json!([{
          "id": self.next_id(),
          "version": "1.0",
          "minimumVersion": "1.0",
          "channel": "/meta/handshake",
          "supportedConnectionTypes": [ "long-polling" ],
        }]);

        let response = self.send_request(&self.handshake_endpoint, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        let mut json_body = response.to_json().await;
        let client_id = json_body[0]["clientId"].take().into_string();
        self.client_id = Some(client_id);
    }

    pub async fn subscribe(&self, subscriptions: &[&str]) {
        let body = json!([{
          "id": self.next_id(),
          "channel": "/meta/subscribe",
          "subscription": subscriptions,
          "clientId": self.client_id.as_deref().expect("Handshake first"),
        }]);

        let response = self.send_request(&self.subscribe_endpoint, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        let json_body = response.to_json().await;
        let successful = json_body[0]["successful"].into_bool();
        assert!(successful);
    }

    pub async fn connect(&self) -> Vec<(String, JsonValue)> {
        let id = self.next_id();
        let body = json!([{
          "id": id,
          "channel": "/meta/connect",
          "connectionType": "long-polling",
          "clientId": self.client_id.as_deref().expect("Handshake first"),
        }]);

        let response = self.send_request(&self.connect_endpoint, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        let mut messages = response.to_json().await.into_array();

        let position = messages
            .iter()
            .position(|message| message["id"].as_str() == Some(&id))
            .expect("The response corresponding request id cannot be found.");
        assert_eq!(messages.remove(position)["successful"], true);

        messages
            .into_iter()
            .map(|mut message| {
                let channel = message["channel"].take().into_string();
                let data = message["data"].take();

                (channel, data)
            })
            .collect()
    }

    pub async fn disconnect(&self) {
        let body = json!([{
          "id": self.next_id(),
          "channel": "/meta/disconnect",
          "clientId": self.client_id.as_deref().expect("Handshake first"),
        }]);

        let response = self.send_request(&self.disconnect_endpoint, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    pub async fn publish(&self, send: impl IntoIterator<Item = (String, JsonValue)>) {
        let client_id = self.client_id.as_deref().expect("Handshake first");

        let (mut ids, body) = send
            .into_iter()
            .map(|(channel, data)| {
                let id = self.next_id();
                let data = json!({
                  "id": id,
                  "channel": channel,
                  "data": data,
                  "clientId": client_id,
                });

                ((id, channel), data)
            })
            .unzip::<_, _, AHashMap<_, _>, Vec<_>>();

        let response = self
            .send_request(&self.connect_endpoint, JsonValue::from(body))
            .await;
        assert_eq!(response.status(), StatusCode::OK);

        let responses = response.to_json().await.into_array();
        assert_eq!(responses.len(), ids.len());

        for message in responses {
            let channel = ids.remove(message["id"].as_str().unwrap()).unwrap();
            assert_eq!(message["channel"], channel);
            assert_eq!(message["successful"], true);
        }
    }

    #[inline(always)]
    pub fn next_id(&self) -> String {
        self.id.fetch_add(1, Ordering::Relaxed).to_string()
    }

    #[inline]
    pub async fn send_request(&self, endpoint: &str, body: JsonValue) -> Response {
        self.router
            .clone()
            .oneshot(build_req(endpoint, body))
            .await
            .unwrap()
    }
}

#[inline]
fn build_req(uri: &str, body: JsonValue) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("POST")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

trait JsonValueExt {
    #[allow(clippy::wrong_self_convention)]
    fn into_bool(&self) -> bool;
    fn into_string(self) -> String;
    fn into_array(self) -> Vec<JsonValue>;
}

impl JsonValueExt for JsonValue {
    fn into_bool(&self) -> bool {
        match self {
            JsonValue::Bool(b) => *b,
            _ => panic!("JsonValue not Bool"),
        }
    }

    fn into_string(self) -> String {
        match self {
            JsonValue::String(str) => str,
            _ => panic!("JsonValue not String"),
        }
    }

    fn into_array(self) -> Vec<JsonValue> {
        match self {
            JsonValue::Array(arr) => arr,
            _ => panic!("JsonValue not Array"),
        }
    }
}
