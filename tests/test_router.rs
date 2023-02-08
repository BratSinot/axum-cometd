use async_broadcast::Receiver;
use axum_cometd::{Event, LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use test_common::*;
use tokio::time::timeout;
use tokio::try_join;

#[tokio::test]
async fn test_different_paths() {
    let builder = LongPollingServiceContextBuilder::new()
        .timeout_ms(1000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10);
    let _ = format!("{builder:?}");
    let context = builder.build::<(), ()>();

    let builder = RouterBuilder::new()
        .subscribe_base_path("/root/sub")
        .handshake_base_path("/root/hand")
        .connect_base_path("/root/conn")
        .disconnect_base_path("/root/disconn");
    let _ = format!("{builder:?}");
    let router = builder.build(Arc::clone(&context));

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
async fn test_event_channel() {
    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(5000)
        .max_interval_ms(60_000)
        .client_channel_capacity(10)
        .subscription_channel_capacity(10)
        .build();
    let mut rx = context.rx();

    let router = RouterBuilder::new().build::<()>(Arc::clone(&context));

    let mut mock_client = ClientMock::create("", "/", "", "", router);
    mock_client.handshake().await;
    let orig_client_id = mock_client.client_id().unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;

    async fn recv(rx: &mut Receiver<Arc<Event<(), ()>>>) -> Arc<Event<(), ()>> {
        timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap()
    }

    matches!(recv(&mut rx).await.as_ref(), Event::SessionAdded{ client_id, .. } if client_id.to_string() == orig_client_id && context.unsubscribe(*client_id).await == ());
    matches!(recv(&mut rx).await.as_ref(), Event::SessionRemoved{ client_id, .. } if client_id.to_string() == orig_client_id);
}
