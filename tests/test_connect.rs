use axum::http::StatusCode;
use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use core::time::Duration;
use serde_json::json;
use std::sync::Arc;
use test_common::{ClientMock, ResponseExt as _, TEST_CLIENT_ID};
use tokio::join;

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
                "clientId": TEST_CLIENT_ID,
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
    mock_client.subscribe(&["/FOO_BAR"]).await.unwrap();

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
                "timeout": TIMEOUT.as_millis() as u64,
            },
        }])
    );
}

#[tokio::test]
async fn test_channel_was_closed() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/FOO_BAR"]).await.unwrap();

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

#[tokio::test]
async fn test_double_connect_same_client_id() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/FOO_BAR"]).await.unwrap();

    let connect = || async {
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
        (id, response)
    };

    let ((id0, resp0), (id1, resp1)) = join!(connect(), connect());

    assert_eq!(
        resp0,
        json!([{
            "id": id0,
            "channel": "/meta/connect",
            "successful": true,
            "advice": {
                "interval": 0,
                "reconnect": "retry",
                "timeout": TIMEOUT.as_millis() as u64
            },
        }])
    );

    assert_eq!(
        resp1,
        json!([{
            "id": id1,
            "channel": "/meta/connect",
            "successful": false,
            "error": "Two connection with same client_id.",
        }])
    );
}
