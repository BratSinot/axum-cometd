use crate::{
    handlers,
    messages::{Advice, Message, Reconnect},
    LongPoolingServiceContextBuilder,
};
use axum::{Extension, Json};
use serde_json::{json, Value as JsonValue};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_wrong_channel() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let Json([message]) = handlers::connect(
        Extension(context.clone()),
        Json([Message {
            channel: Some("/meta/non_connect".to_owned()),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err();

    assert_eq!(
        message,
        Message {
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel: Some("/meta/connect".into()),
            error: Some("no connect channel".into()),
            successful: Some(false),
            ..Default::default()
        }
    );
}

#[tokio::test]
async fn test_empty_client_id() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let Json([message]) = handlers::connect(
        Extension(context.clone()),
        Json([Message {
            channel: Some("/meta/connect".to_owned()),
            connection_type: Some("long-polling".into()),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err();

    assert_eq!(
        message,
        Message {
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel: Some("/meta/connect".into()),
            error: Some("empty clientId".into()),
            successful: Some(false),
            ..Default::default()
        }
    );
}

#[tokio::test]
async fn test_client_doesnt_exist() {
    let client_id =
        serde_json::from_value(json!("5804e4865f649fb91645030760db1f358c837af9")).unwrap();

    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let Json([message]) = handlers::connect(
        Extension(context.clone()),
        Json([Message {
            channel: Some("/meta/connect".to_owned()),
            connection_type: Some("long-polling".into()),
            client_id: Some(client_id),
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err();

    assert_eq!(
        message,
        Message {
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel: Some("/meta/connect".into()),
            client_id: Some(client_id),
            error: Some(format!("Client with id {client_id} doesn't exist.")),
            successful: Some(false),
            ..Default::default()
        }
    );
}

#[tokio::test]
async fn test_wrong_connect_type() {
    let client_id =
        Some(serde_json::from_value(json!("5804e4865f649fb91645030760db1f358c837af9")).unwrap());

    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let Json([message]) = handlers::connect(
        Extension(context.clone()),
        Json([Message {
            channel: Some("/meta/connect".to_owned()),
            connection_type: Some("non-long-polling".into()),
            client_id,
            ..Default::default()
        }]),
    )
    .await
    .unwrap_err();

    assert_eq!(
        message,
        Message {
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel: Some("/meta/connect".into()),
            client_id,
            error: Some("unsupported connectionType".into()),
            successful: Some(false),
            ..Default::default()
        }
    );
}

#[tokio::test]
async fn test_reconnect() {
    /*let client_id =
    serde_json::from_value(json!("5804e4865f649fb91645030760db1f358c837af9")).unwrap();*/

    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let client_id = context.register().await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let Json([message]) = timeout(
        Duration::from_millis(1000),
        handlers::connect(
            Extension(context.clone()),
            Json([Message {
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
    .unwrap_err();

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
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let client_id = context.register().await;
    context.subscribe(client_id, "FOO_BAR").await.unwrap();

    let ((), Json([message])) = tokio::join!(
        async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            context.unsubscribe(client_id).await;
        },
        async {
            handlers::connect(
                Extension(context.clone()),
                Json([Message {
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
        }
    );

    assert_eq!(
        message,
        Message {
            id: Some("4".into()),
            advice: Some(Advice {
                reconnect: Some(Reconnect::None),
                ..Default::default()
            }),
            channel: Some("/meta/connect".into()),
            client_id: Some(client_id),
            error: Some("channel was closed".into()),
            successful: Some(false),
            ..Default::default()
        }
    );
}
