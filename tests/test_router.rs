use axum::{
    body::Body,
    http::StatusCode,
    http::{header::CONTENT_TYPE, Request},
    Router,
};
use axum_cometd::{LongPoolingServiceContextBuilder, RouterBuilder};
use futures_util::future::FutureExt;
use serde_json::{json, Value as JsonValue};
use tokio::try_join;
use tower::ServiceExt;

fn build_req(uri: &str, body: JsonValue) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("POST")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn handshake(app: &Router) -> String {
    let response = app
        .clone()
        .oneshot(build_req(
            "/root/hand/handshake",
            json!([{
                "id": "2",
                "version": "1.0",
                "minimumVersion": "1.0",
                "channel": "/meta/handshake",
                "supportedConnectionTypes": [ "long-polling" ],
                "advice": {
                    "timeout": 60000,
                    "interval": 0,
                },
            }]),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<JsonValue>(&body).unwrap()[0]["clientId"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn subscribe(app: &Router, client_id: &str) -> bool {
    let response = app
        .clone()
        .oneshot(build_req(
            "/root/sub/",
            json!([{
                "id": "3",
                "channel": "/meta/subscribe",
                "subscription": "SUPER_IMPORTANT_CHANNEL",
                "clientId": client_id,
            }]),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice::<JsonValue>(&body).unwrap()[0]["successful"]
        .as_bool()
        .unwrap()
}

async fn connect(app: &Router, client_id: &str) -> JsonValue {
    let response = app
        .clone()
        .oneshot(build_req(
            "/root/conn/connect",
            json!([{
                "id": "4",
                "channel": "/meta/connect",
                "connectionType": "long-polling",
                "advice": {
                    "timeout": 0
                },
                "clientId": client_id,
            }]),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let [mut resp, mut data] = serde_json::from_slice::<[JsonValue; 2]>(&body).unwrap();

    if resp.get("data").is_some() {
        std::mem::swap(&mut resp, &mut data);
    }

    assert!(resp["successful"].as_bool().unwrap());

    data["data"].take()
}

#[tokio::test]
async fn test_different_paths() {
    let context = LongPoolingServiceContextBuilder::new()
        .timeout_ms(20_000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10)
        .build::<serde_json::Value>();

    let app = RouterBuilder::new()
        .base_path("/root/")
        .subscribe_base_path("/sub/")
        .handshake_base_path("/hand/")
        .connect_base_path("/conn/")
        .disconnect_base_path("/disconn/")
        .build(&context);

    let client_id = handshake(&app).await;
    let successful = subscribe(&app, &client_id).await;
    assert!(successful);

    let (data, ()) = try_join!(
        connect(&app, &client_id).map(Ok),
        context.send(
            "SUPER_IMPORTANT_CHANNEL",
            json!({"msg": "integration_test"})
        )
    )
    .unwrap();

    assert_eq!(data, json!({"msg": "integration_test"}))
}
