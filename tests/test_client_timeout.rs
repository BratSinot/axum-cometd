use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use core::time::Duration;
use serde_json::json;
use std::sync::Arc;
use test_common::ClientMock;

const TIMEOUT: Duration = Duration::from_secs(1);
const MAX_INTERVAL: Duration = Duration::from_secs(2);

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout(TIMEOUT)
        .max_interval(MAX_INTERVAL)
        .build();
    let router = RouterBuilder::new().build::<()>(Arc::clone(&context));

    ClientMock::create("", "/", "", "", router)
}

#[tokio::test]
async fn test_client_timeout() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;

    tokio::time::sleep(MAX_INTERVAL + Duration::from_millis(500)).await;

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
