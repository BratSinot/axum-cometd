use crate::{
    handlers,
    messages::{Advice, Message, Reconnect},
    LongPoolingServiceContextBuilder,
};
use axum::{extract::State, Json};
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;

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
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message::error("400::channel_missing", None, None, None)
    );
}

#[tokio::test]
async fn test_empty_client_id() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let message = handlers::connect(
        State(context.clone()),
        Json(vec![Message {
            channel: Some("/meta/connect".to_owned()),
            connection_type: Some("long-polling".into()),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err()
    .into_message()
    .unwrap();

    assert_eq!(
        message,
        Message::error("empty clientId", Some("/meta/connect".into()), None, None)
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
            connection_type: Some("long-polling".into()),
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
        Message::error(
            format!("Client with id {client_id} doesn't exist."),
            Some("/meta/connect".into()),
            Some(client_id),
            None
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
            connection_type: Some("non-long-polling".into()),
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
        Message::error(
            "unsupported connectionType",
            Some("/meta/connect".into()),
            client_id,
            None
        )
    );
}

#[tokio::test]
async fn test_reconnect() {
    let context = LongPoolingServiceContextBuilder::new().build();
    let client_id = context.register().await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let message = timeout(
        Duration::from_millis(1000),
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some("4".into()),
                channel: Some("/meta/connect".to_owned()),
                connection_type: Some("long-polling".into()),
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
    let client_id = context.register().await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let ((), message) = tokio::join!(
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
                    connection_type: Some("long-polling".into()),
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
            .into_message()
            .unwrap()
        }
    );

    assert_eq!(
        message,
        Message::error(
            "channel was closed",
            Some("/meta/connect".into()),
            Some(client_id),
            Some("4".into())
        )
    );
}
