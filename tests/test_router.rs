use axum::Router;
use axum_cometd::{LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use std::time::Duration;
use test_common::*;
use tokio::{sync::Mutex, try_join};

async fn receive_message_and_extract_data(app: &Router, client_id: &str) -> JsonValue {
    let response = receive_message(app, "/root/conn", client_id).await;
    let [mut resp, mut data] = serde_json::from_value::<[JsonValue; 2]>(response).unwrap();

    if resp.get("data").is_some() {
        std::mem::swap(&mut resp, &mut data);
    }

    assert!(resp["successful"].as_bool().unwrap());

    data["data"].take()
}

#[tokio::test]
async fn test_different_paths() {
    let builder = LongPollingServiceContextBuilder::new()
        .timeout_ms(1000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10);
    let _ = format!("{builder:?}");
    let context = builder.build();

    let builder = RouterBuilder::new()
        .subscribe_base_path("/sub")
        .handshake_base_path("/hand")
        .connect_base_path("/conn")
        .disconnect_base_path("/disconn");
    let _ = format!("{builder:?}");
    let app = Router::new().nest("/root", builder.build(&context));

    let client_id = get_client_id(&app, "/root/hand", 60_000).await;
    subscribe_to_channel(&app, "/root/sub", &client_id, "/SUPER_IMPORTANT_CHANNEL").await;

    let (data, ()) = try_join!(
        async { Ok(receive_message_and_extract_data(&app, &client_id).await) },
        context.send(
            "/SUPER_IMPORTANT_CHANNEL",
            json!({"msg": "integration_test"})
        )
    )
    .unwrap();

    assert_eq!(data, json!({"msg": "integration_test"}));

    let response = receive_message(&app, "/root/conn", &client_id).await;
    assert_eq!(
        response,
        json!([{
            "id": "4",
            "advice": {
                "interval":0,
                "reconnect":"retry",
                "timeout": 1000
            },
            "channel": "/meta/connect",
            "successful":true
        }])
    );
}

#[tokio::test]
async fn test_callbacks() {
    let client_id_check = Arc::new(Mutex::new(String::new()));
    let removed_client_id = Arc::new(Mutex::new(String::new()));

    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(5000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10)
        .async_session_added({
            let client_id_check = client_id_check.clone();
            move |(context, client_id, _)| {
                let client_id_check = client_id_check.clone();
                async move {
                    *client_id_check.lock().await = client_id.to_string();
                    tokio::spawn(async move {
                        context.unsubscribe(client_id).await;
                    });
                }
            }
        })
        .async_session_removed({
            let removed_client_id = removed_client_id.clone();
            move |(_, client_id)| {
                let removed_client_id = removed_client_id.clone();
                async move {
                    *removed_client_id.lock().await = client_id.to_string();
                }
            }
        })
        .build();

    let app = RouterBuilder::new().build(&context);

    let response_client_id = get_client_id(&app, "", 0).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    assert_eq!(*client_id_check.lock().await, response_client_id);
    assert_eq!(*removed_client_id.lock().await, response_client_id);
}
