use axum_cometd::{LongPollingServiceContext, LongPollingServiceContextBuilder, RouterBuilder};
use serde_json::{json, Value as JsonValue};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use test_common::ClientMock;
use tokio::try_join;

fn build_context_and_mock_client() -> (Arc<LongPollingServiceContext>, ClientMock) {
    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(1000)
        .build();
    let router = RouterBuilder::new().build(Arc::clone(&context));
    let mock_client = ClientMock::create("", "/", "", "", router);

    (context, mock_client)
}

fn gen_message() -> JsonValue {
    static ID: AtomicU64 = AtomicU64::new(0);
    json!({
        "msg": "Hello",
        "id": ID.fetch_add(1, Ordering::SeqCst),
    })
}

// sub: /*
// send: /topic, /topic/second
#[tokio::test]
async fn test_single_wildcard() {
    let (context, mut mock_client) = build_context_and_mock_client();

    mock_client.handshake().await;
    mock_client.subscribe(&["/*"]).await.unwrap();

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![]);
}

// sub: /**
// send: /topic, /topic/second, /first/second/third
#[tokio::test]
async fn test_all_wildcard() {
    let (context, mut mock_client) = build_context_and_mock_client();

    mock_client.handshake().await;
    mock_client.subscribe(&["/**"]).await.unwrap();

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic/second".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/first/second/third", &test_message),
    )
    .unwrap();
    assert_eq!(
        responses,
        vec![("/first/second/third".to_owned(), test_message)]
    );
}

// sub: /topic/*
// send: /topic/, /topic/second, /topic/second/third
#[tokio::test]
async fn test_nested_single_wildcard() {
    let (context, mut mock_client) = build_context_and_mock_client();

    mock_client.handshake().await;
    mock_client.subscribe(&["/topic/*"]).await.unwrap();

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic/".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic/second".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second/third", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![]);
}

// sub: /topic/**
// send: /topic/, /topic/second, /topic/second/third, /qwe
#[tokio::test]
async fn test_nested_all_wildcard() {
    let (context, mut mock_client) = build_context_and_mock_client();

    mock_client.handshake().await;
    mock_client.subscribe(&["/topic/**"]).await.unwrap();

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic/".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![("/topic/second".to_owned(), test_message)]);

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/topic/second/third", &test_message),
    )
    .unwrap();
    assert_eq!(
        responses,
        vec![("/topic/second/third".to_owned(), test_message)]
    );

    let test_message = gen_message();
    let (responses, ()) = try_join!(
        async { Ok(mock_client.connect().await) },
        context.send("/qwe", &test_message),
    )
    .unwrap();
    assert_eq!(responses, vec![]);
}
