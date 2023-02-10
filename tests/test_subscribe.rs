use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use core::time::Duration;
use serde_json::json;
use std::sync::Arc;
use test_common::{ClientMock, ResponseExt};

const TIMEOUT: Duration = Duration::from_secs(1);

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout(TIMEOUT)
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
            mock_client.subscribe_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/non_subscribe"
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
            "channel": "/meta/non_subscribe",
            "error": "402::session_unknown"
        }])
    );

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.handshake_endpoint(),
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
async fn test_subscription_missing() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.subscribe_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/subscribe",
                "clientId": mock_client.client_id(),
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
            "channel": "/meta/subscribe",
            "error": "403::subscription_missing"
        }])
    );

    let response = mock_client
        .send_request(
            mock_client.subscribe_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/subscribe",
                "subscription": [],
                "clientId": mock_client.client_id(),
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
            "channel": "/meta/subscribe",
            "error": "403::subscription_missing"
        }])
    );
}
