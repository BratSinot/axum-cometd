use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::json;
use std::sync::Arc;
use test_common::{ClientMock, ResponseExt};

const TIMEOUT_MS: u64 = 1000;

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(TIMEOUT_MS)
        .build();
    let router = RouterBuilder::new().build::<()>(Arc::clone(&context));

    ClientMock::create("", "/", "", "", router)
}

#[tokio::test]
async fn test_wrong_channel() {
    let mock_client = build_mock_client();

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.disconnect_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/non_connect"
            }]),
        )
        .await
        .to_json()
        .await;
    assert_eq!(
        response,
        json!([{
            "id": id,
            "successful": false,
            "channel": "/meta/non_connect",
            "error": "402::session_unknown"
        }])
    );

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.disconnect_endpoint(),
            json!([{
                "id": id,
            }]),
        )
        .await
        .to_json()
        .await;
    assert_eq!(
        response,
        json!([{
            "id": id,
            "successful": false,
            "error": "402::session_unknown"
        }])
    );
}

#[tokio::test]
async fn test_missing_client_id() {
    let mock_client = build_mock_client();

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.disconnect_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/disconnect"
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "id": id,
            "successful": false,
            "channel": "/meta/disconnect",
            "error": "402::session_unknown"
        }])
    );
}
