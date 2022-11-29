use axum::{
    body::Body,
    http::Request,
    http::{header::CONTENT_TYPE, StatusCode},
    Router,
};
use serde_json::{json, Value as JsonValue};
use tower::ServiceExt;

pub fn build_req(uri: &str, body: JsonValue) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("POST")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub async fn handshake(app: &Router, base_url: &str, body: JsonValue) -> JsonValue {
    let response = app
        .clone()
        .oneshot(build_req(&format!("{base_url}/handshake"), body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<JsonValue>(&body).unwrap()
}

pub async fn get_client_id(app: &Router, base_url: &str, timeout_ms: u64) -> String {
    handshake(
        app,
        base_url,
        json!([{
            "id": "2",
            "version": "1.0",
            "minimumVersion": "1.0",
            "channel": "/meta/handshake",
            "supportedConnectionTypes": [ "long-polling" ],
            "advice": { "timeout": timeout_ms, "interval": 0 },
        }]),
    )
    .await[0]["clientId"]
        .as_str()
        .unwrap()
        .to_string()
}

pub async fn subscribe(app: &Router, base_url: &str, body: JsonValue) -> JsonValue {
    let response = app
        .clone()
        .oneshot(build_req(&format!("{base_url}"), body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<JsonValue>(&body).unwrap()
}

pub async fn subscribe_to_subscription(
    app: &Router,
    base_url: &str,
    client_id: &str,
    subscription: &str,
) {
    let successful = subscribe(
        app,
        base_url,
        json!([{
            "id": "3",
            "channel": "/meta/subscribe",
            "subscription": subscription,
            "clientId": client_id,
        }]),
    )
    .await[0]["successful"]
        .as_bool()
        .unwrap();
    assert!(successful);
}

pub async fn disconnect(app: &Router, base_url: &str, body: JsonValue) {
    let response = app
        .clone()
        .oneshot(build_req(&format!("{base_url}/disconnect"), body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

pub async fn disconnect_client_id(app: &Router, base_url: &str, client_id: &str) {
    disconnect(
        app,
        base_url,
        json!([{
          "id": "9",
          "channel": "/meta/disconnect",
          "clientId": client_id
        }]),
    )
    .await
}

pub async fn connect(app: &Router, base_url: &str, body: JsonValue) -> JsonValue {
    let response = app
        .clone()
        .oneshot(build_req(&format!("{base_url}/connect"), body))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<JsonValue>(&body).unwrap()
}

pub async fn receive_message(app: &Router, base_url: &str, client_id: &str) -> JsonValue {
    connect(
        app,
        base_url,
        json!([{
            "id": "4",
            "channel": "/meta/connect",
            "connectionType": "long-polling",
            "advice": { "timeout": 0 },
            "clientId": client_id,
        }]),
    )
    .await
}
