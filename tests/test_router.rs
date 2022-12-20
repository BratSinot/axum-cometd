use axum::Router;
use axum_cometd::{CallBackArguments, LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use test_common::*;
use tokio::{sync::Mutex, try_join};

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
    let router = Router::new().nest("/root", builder.build(&context));

    let mut mock_client = ClientMock::create(
        "/root/hand",
        "/root/sub",
        "/root/conn",
        "/root/disconn",
        router,
    );

    mock_client.handshake().await;
    mock_client
        .subscribe(&["/SUPER_IMPORTANT_CHANNEL"])
        .await
        .unwrap();
    let (response, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send(
            "/SUPER_IMPORTANT_CHANNEL",
            json!({"msg": "integration_test"})
        )
    )
    .unwrap();

    assert_eq!(
        &response,
        &[(
            "/SUPER_IMPORTANT_CHANNEL".to_owned(),
            json!({"msg": "integration_test"})
        )]
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
            move |CallBackArguments {
                      context, client_id, ..
                  }| {
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

    let router = RouterBuilder::new().build(&context);

    let mut mock_client = ClientMock::create("", "/", "", "", router);
    mock_client.handshake().await;
    let client_id = mock_client.client_id().unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    assert_eq!(*client_id_check.lock().await, client_id);
    assert_eq!(*removed_client_id.lock().await, client_id);
}
