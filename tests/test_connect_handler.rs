use axum_cometd::{LongPoolingServiceContextBuilder, RouterBuilder};
use serde_json::{json, Value as JsonValue};
use std::time::Duration;
use test_common::*;
use tokio::time::timeout;

#[tokio::test]
async fn test_wrong_channel() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let response = connect(
        &app,
        "",
        json!([{
            "channel": "/meta/non_connect",
        }]),
    )
    .await;

    assert_eq!(
        json!([
          {
            "advice": {
              "reconnect": "none"
            },
            "channel": "/meta/connect",
            "error": "no connect channel",
            "successful": false
          }
        ]),
        response
    );
}

#[tokio::test]
async fn test_empty_client_id() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let response = test_common::connect(
        &app,
        "",
        json!([{
            "channel": "/meta/connect",
            "connectionType": "long-polling",
        }]),
    )
    .await;

    assert_eq!(
        json!([
          {
            "advice": {
              "reconnect": "none"
            },
            "channel": "/meta/connect",
            "error": "empty clientId",
            "successful": false
          }
        ]),
        response
    );
}

#[tokio::test]
async fn test_client_doesnt_exist() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let response = connect(
        &app,
        "",
        json!([{
            "channel": "/meta/connect",
            "connectionType": "long-polling",
            "clientId": "5804e4865f649fb91645030760db1f358c837af9",
        }]),
    )
    .await;

    assert_eq!(
        json!([{
            "advice": { "reconnect": "none" },
            "channel": "/meta/connect",
            "clientId": "5804e4865f649fb91645030760db1f358c837af9",
            "error": format!("Client with id 5804e4865f649fb91645030760db1f358c837af9 doesn't exist."),
            "successful": false
        }]),
        response
    );
}

#[tokio::test]
async fn test_wrong_connect_type() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let response = connect(
        &app,
        "",
        json!([{
            "channel": "/meta/connect",
            "connectionType": "non-long-polling",
            "clientId": "5804e4865f649fb91645030760db1f358c837af9",
        }]),
    )
    .await;

    assert_eq!(
        json!([{
            "advice": { "reconnect": "none" },
            "channel": "/meta/connect",
            "clientId": "5804e4865f649fb91645030760db1f358c837af9",
            "error": "unsupported connectionType",
            "successful": false
        }]),
        response
    );
}

#[tokio::test]
async fn test_reconnect() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let client_id = get_client_id(&app, "", 500).await;
    subscribe_to_subscription(&app, "", &client_id, "FOO_BAR").await;
    let body = timeout(
        Duration::from_millis(1000),
        receive_message(&app, "", &client_id),
    )
    .await
    .unwrap();

    assert_eq!(
        body,
        json!([{
            "id": "4",
            "channel": "/meta/connect",
            "successful": true,
            "advice": {
                "interval": 0,
                "reconnect": "retry",
                "timeout": 20000,
            },
        }])
    );
}

/*#[tokio::test]
async fn test_channel_was_closed() {
    let context = LongPoolingServiceContextBuilder::new().build::<JsonValue>();
    let app = RouterBuilder::new().build(&context);

    let client_id = get_client_id(&app, "", 1000).await;
    subscribe_to_subscription(&app, "", &client_id, "FOO_BAR").await;

    let (_, body) = tokio::join!(
        async {
            tokio::time::sleep(Duration::from_secs(100)).await;
            disconnect_client_id(&app, "", &client_id).await
        },
        receive_message(&app, "", &client_id)
    );

    println!("body: {body:?}");
}*/
