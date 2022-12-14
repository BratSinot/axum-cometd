use axum::http::StatusCode;
use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::json;
use std::time::Duration;
use test_common::{ClientMock, ResponseExt};
use tokio::join;

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

    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "channel": "/meta/non_connect"
            }]),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_empty_client_id() {
    let mock_client = build_mock_client();

    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "channel": "/meta/connect"
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "channel": "/meta/connect",
            "successful": false,
            "error": "402::session_unknown",
            "advice": {
                "interval": 0,
                "reconnect": "handshake"
            },
        }])
    );
}

#[tokio::test]
async fn test_client_doesnt_exist() {
    let mock_client = build_mock_client();

    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "channel": "/meta/connect",
                "clientId": "5804e4865f649fb91645030760db1f358c837af9",
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "channel": "/meta/connect",
            "successful": false,
            "error": "402::session_unknown",
            "advice": {
                "interval": 0,
                "reconnect": "handshake"
            },
        }])
    );
}

#[tokio::test]
async fn test_reconnect() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/FOO_BAR"]).await;

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
              "id": id,
              "channel": "/meta/connect",
              "connectionType": "long-polling",
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
            "channel": "/meta/connect",
            "successful": true,
            "advice": {
                "interval": 0,
                "reconnect": "retry",
                "timeout": TIMEOUT_MS,
            },
        }])
    );
}

#[tokio::test]
async fn test_channel_was_closed() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/FOO_BAR"]).await;

    let id = mock_client.next_id();
    let ((), response) = join!(
        async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            mock_client.disconnect().await
        },
        async {
            mock_client
                .send_request(
                    mock_client.connect_endpoint(),
                    json!([{
                      "id": id,
                      "channel": "/meta/connect",
                      "connectionType": "long-polling",
                      "clientId": mock_client.client_id(),
                    }]),
                )
                .await
        }
    );
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
