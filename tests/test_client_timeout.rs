use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use core::time::Duration;
use serde_json::json;
use std::sync::Arc;
use test_common::ClientMock;

const TIMEOUT_MS: u64 = 1000;
const MAX_INTERVAL_MS: u64 = 2000;

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(TIMEOUT_MS)
        .max_interval_ms(MAX_INTERVAL_MS)
        .build();
    let router = RouterBuilder::new().build::<()>(Arc::clone(&context));

    ClientMock::create("", "/", "", "", router)
}

#[tokio::test]
async fn test_client_timeout() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;

    tokio::time::sleep(Duration::from_millis(MAX_INTERVAL_MS + 500)).await;

    let json_body = mock_client.subscribe(&["/topic"]).await.unwrap_err();
    assert_eq!(
        json_body,
        json!([{
            "id": mock_client.last_id(),
            "channel": "/meta/subscribe",
            "error": "402::session_unknown",
            "successful": false
        }])
    );
}
