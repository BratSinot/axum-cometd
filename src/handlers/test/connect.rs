use crate::{
    handlers,
    messages::{Advice, Message, Reconnect},
    HandlerError, LongPoolingServiceContextBuilder,
};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

impl HandlerError {
    #[inline(always)]
    fn into_status_code(self) -> Option<StatusCode> {
        if let Self::StatusCode(code) = self {
            Some(code)
        } else {
            None
        }
    }

    #[inline(always)]
    fn into_message(self) -> Option<Message> {
        if let Self::Message(message) = self {
            Some(message)
        } else {
            None
        }
    }
}

#[tokio::test]
async fn test_wrong_channel() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let message = handlers::connect(
        State(context.clone()),
        Json(vec![Message {
            channel: Some("/meta/non_connect".to_owned()),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err()
    .into_status_code()
    .unwrap();

    assert_eq!(message, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_empty_client_id() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let message = handlers::connect(
        State(context.clone()),
        Json(vec![Message {
            channel: Some("/meta/connect".to_owned()),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err()
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message::session_unknown(
            None,
            Some("/meta/connect".into()),
            Some(Advice::handshake())
        )
    );
}

#[tokio::test]
async fn test_client_doesnt_exist() {
    let client_id =
        serde_json::from_value(json!("5804e4865f649fb91645030760db1f358c837af9")).unwrap();

    let context = LongPoolingServiceContextBuilder::new().build();
    let message = handlers::connect(
        State(context.clone()),
        Json(vec![Message {
            channel: Some("/meta/connect".to_owned()),
            client_id: Some(client_id),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err()
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message::session_unknown(
            None,
            Some("/meta/connect".into()),
            Some(Advice::handshake())
        )
    );
}

#[tokio::test]
async fn test_wrong_connect_type() {
    let client_id =
        Some(serde_json::from_value(json!("5804e4865f649fb91645030760db1f358c837af9")).unwrap());

    let context = LongPoolingServiceContextBuilder::new().build();
    let message = handlers::connect(
        State(context.clone()),
        Json(vec![Message {
            channel: Some("/meta/connect".to_owned()),
            client_id,
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err()
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message::session_unknown(
            None,
            Some("/meta/connect".into()),
            Some(Advice::handshake())
        )
    );
}

#[tokio::test]
async fn test_reconnect() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let message = timeout(
        Duration::from_millis(1000),
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some("4".into()),
                channel: Some("/meta/connect".to_owned()),
                advice: Some(Advice {
                    timeout: Some(0),
                    ..Default::default()
                }),
                client_id: Some(client_id),
                ..Default::default()
            }]),
        ),
    )
    .await
    .unwrap()
    .unwrap_err()
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message {
            id: Some("4".into()),
            channel: Some("/meta/connect".into()),
            successful: Some(true),
            advice: Some(Advice {
                interval: Some(0),
                reconnect: Some(Reconnect::Retry),
                timeout: Some(20000),
                ..Default::default()
            }),
            ..Default::default()
        }
    );
}

#[tokio::test]
async fn test_channel_was_closed() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let ((), status_code) = tokio::join!(
        async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            context.unsubscribe(client_id).await;
        },
        async {
            handlers::connect(
                State(context.clone()),
                Json(vec![Message {
                    id: Some("4".into()),
                    channel: Some("/meta/connect".to_owned()),
                    advice: Some(Advice {
                        timeout: Some(3000),
                        ..Default::default()
                    }),
                    client_id: Some(client_id),
                    ..Default::default()
                }]),
            )
            .await
            .unwrap_err()
            .into_status_code()
            .unwrap()
        }
    );

    assert_eq!(status_code, StatusCode::INTERNAL_SERVER_ERROR);
}
