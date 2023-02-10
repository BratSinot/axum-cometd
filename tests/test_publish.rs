use axum::http::StatusCode;
use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use core::time::Duration;
use serde_json::{json, Value as JsonValue};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use test_common::{ClientMock, ResponseExt, TEST_CLIENT_ID};
use tokio::time::timeout;

const TIMEOUT: Duration = Duration::from_secs(1);

fn gen_message() -> JsonValue {
    static ID: AtomicU64 = AtomicU64::new(0);
    json!({
        "num": ID.fetch_add(1, Ordering::SeqCst),
        "msg": "Hello",
    })
}

fn build_mock_client() -> ClientMock {
    let context = LongPollingServiceContextBuilder::new()
        .timeout(TIMEOUT)
        .build();
    let router = RouterBuilder::new().build::<()>(Arc::clone(&context));

    ClientMock::create("", "/", "", "", router)
}

#[tokio::test]
async fn test_bad_request() {
    let mock_client = build_mock_client();

    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([
                { "channel": "/meta/random" },
                { "channel": "/topic0" },
            ]),
        )
        .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}

#[tokio::test]
async fn test_channel_missing() {
    let mock_client = build_mock_client();

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "id": id,
                "clientId": TEST_CLIENT_ID,
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "id": id,
            "error": "400::channel_missing",
            "successful": false,
        }])
    )
}

#[tokio::test]
async fn test_session_unknown() {
    let mock_client = build_mock_client();

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "id": id,
                "channel": "/topic",
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "id": id,
            "channel": "/topic",
            "successful": false,
            "error": "402::session_unknown",
            "advice": {
                "reconnect": "handshake",
                "interval": 0,
            }
        }])
    );

    let id = mock_client.next_id();
    let response = mock_client
        .send_request(
            mock_client.connect_endpoint(),
            json!([{
                "id": id,
                "channel": "/topic",
                "clientId": TEST_CLIENT_ID,
            }]),
        )
        .await
        .to_json()
        .await;

    assert_eq!(
        response,
        json!([{
            "id": id,
            "channel": "/topic",
            "successful": false,
            "error": "402::session_unknown",
        }])
    );
}

// sub: /*
// send: /topic*
#[tokio::test]
async fn test_publish_invalid_channel() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/*"]).await.unwrap();

    let msg = ("/topic*".to_owned(), gen_message());
    mock_client.publish([msg.clone()]).await;

    let resp = mock_client.connect().await;
    assert_eq!(resp, []);
}

// sub: /*
// send: /topic, /topic0, /topic1, /topic1/second
#[tokio::test]
async fn test_publish() {
    let mut mock_client = build_mock_client();
    mock_client.handshake().await;
    mock_client.subscribe(&["/*"]).await.unwrap();

    let (msg0, msg1, msg2, msg3) = (
        ("/topic".to_owned(), gen_message()),
        ("/topic0".to_owned(), gen_message()),
        ("/topic1".to_owned(), gen_message()),
        ("/topic1/second".to_owned(), gen_message()),
    );
    mock_client
        .publish([msg0.clone(), msg1.clone(), msg2.clone(), msg3.clone()])
        .await;

    let (resp0, resp1, resp2, resp3) = timeout(Duration::from_secs(2), async {
        (
            mock_client.connect().await,
            mock_client.connect().await,
            mock_client.connect().await,
            mock_client.connect().await,
        )
    })
    .await
    .unwrap();

    assert_eq!(resp0, [msg0]);
    assert_eq!(resp1, [msg1]);
    assert_eq!(resp2, [msg2]);
    assert_eq!(resp3, []);
}
