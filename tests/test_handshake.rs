use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::json;
use test_common::{ClientMock, ResponseExt};

const TIMEOUT_MS: u64 = 1000;

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(TIMEOUT_MS)
        .build();
    let router = RouterBuilder::new().build(&context);

    ClientMock::create("", "/", "", "", router)
}

#[tokio::test]
async fn test_wrong_channel() {
    let mock_client = build_mock_client();

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.handshake_endpoint(),
            json!([{
                "id": id,
                "channel": "/meta/non_handshake"
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
            "channel": "/meta/non_handshake",
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
