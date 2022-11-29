use axum::Router;
use axum_cometd::{LongPoolingServiceContextBuilder, RouterBuilder};
use serde_json::{json, Value as JsonValue};
use test_common::*;
use tokio::try_join;

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
    let builder = LongPoolingServiceContextBuilder::new()
        .timeout_ms(20_000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10);
    let _ = format!("{builder:?}");
    let context = builder.build::<serde_json::Value>();

    let builder = RouterBuilder::new()
        .base_path("/root")
        .subscribe_base_path("/sub")
        .handshake_base_path("/hand")
        .connect_base_path("/conn")
        .disconnect_base_path("/disconn");
    let _ = format!("{builder:?}");
    let app = builder.build(&context);

    let client_id = get_client_id(&app, "/root/hand", 60_000).await;
    subscribe_to_subscription(&app, "/root/sub", &client_id, "SUPER_IMPORTANT_CHANNEL").await;

    let (data, ()) = try_join!(
        async { Ok(receive_message_and_extract_data(&app, &client_id).await) },
        context.send(
            "SUPER_IMPORTANT_CHANNEL",
            json!({"msg": "integration_test"})
        )
    )
    .unwrap();

    assert_eq!(data, json!({"msg": "integration_test"}))
}
