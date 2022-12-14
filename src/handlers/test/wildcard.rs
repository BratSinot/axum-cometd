use crate::{
    handlers,
    messages::{Advice, Message},
    LongPollingServiceContextBuilder,
};
use axum::{extract::State, Json};
use rand::Rng;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;

fn gen_message() -> Value {
    json!({
        "msg": "Hello",
        "id": rand::thread_rng().gen::<u64>(),
    })
}

// sub: /**
// send: /topic, /topic/second
#[tokio::test]
async fn test_all_wildcard() {
    let context = LongPollingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    let wait_message = |id: usize| {
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some(id.to_string()),
                channel: Some("/meta/connect".to_owned()),
                advice: Some(Advice {
                    timeout: Some(10000),
                    ..Default::default()
                }),
                client_id: Some(client_id),
                ..Default::default()
            }]),
        )
    };

    context
        .subscribe(client_id, &["/**".to_string()])
        .await
        .unwrap();

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic/second", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);
}

// sub: /topic/**
// send: /topic, /topic/second, /topic/second/third, /qwe
#[tokio::test]
async fn test_nested_all_wildcard() {
    let context = LongPollingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    let wait_message = |id: usize| {
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some(id.to_string()),
                channel: Some("/meta/connect".to_owned()),
                advice: Some(Advice {
                    timeout: Some(1000),
                    ..Default::default()
                }),
                client_id: Some(client_id),
                ..Default::default()
            }]),
        )
    };

    context
        .subscribe(client_id, &["/topic/**".to_string()])
        .await
        .unwrap();

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic/", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic/second", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context
            .send("/topic/second/third", &test_message)
            .await
            .unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (timeout, ()) = tokio::join!(timeout(Duration::from_millis(1), wait_message(0)), async {
        context.send("/qwe", &test_message).await.unwrap()
    });
    assert!(timeout.is_err());
}

// sub: /*
// send: /topic, /topic/second
#[tokio::test]
async fn test_single_wildcard() {
    let context = LongPollingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    let wait_message = |id: usize| {
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some(id.to_string()),
                channel: Some("/meta/connect".to_owned()),
                advice: Some(Advice {
                    timeout: Some(10000),
                    ..Default::default()
                }),
                client_id: Some(client_id),
                ..Default::default()
            }]),
        )
    };

    context
        .subscribe(client_id, &["/*".to_string()])
        .await
        .unwrap();

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (timeout, ()) = tokio::join!(timeout(Duration::from_secs(1), wait_message(0)), async {
        context.send("/topic/second", &test_message).await.unwrap()
    });
    assert!(timeout.is_err());
}

// sub: /topic/*
// send: /topic/, /topic/second, /topic/second/third
#[tokio::test]
async fn test_nested_single_wildcard() {
    let context = LongPollingServiceContextBuilder::new().build();
    let client_id = context.register(Default::default()).await;
    let wait_message = |id: usize| {
        handlers::connect(
            State(context.clone()),
            Json(vec![Message {
                id: Some(id.to_string()),
                channel: Some("/meta/connect".to_owned()),
                advice: Some(Advice {
                    timeout: Some(10000),
                    ..Default::default()
                }),
                client_id: Some(client_id),
                ..Default::default()
            }]),
        )
    };

    context
        .subscribe(client_id, &["/topic/*".to_string()])
        .await
        .unwrap();

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic/", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (Json(msgs), ()) = tokio::join!(async { wait_message(0).await.unwrap() }, async {
        context.send("/topic/second", &test_message).await.unwrap()
    });
    assert_eq!(msgs.len(), 2);
    let data = msgs.into_iter().find_map(|msg| msg.data).unwrap();
    assert_eq!(data, test_message);

    let test_message = gen_message();
    let (timeout, ()) = tokio::join!(timeout(Duration::from_secs(1), wait_message(0)), async {
        context
            .send("/topic/second/third", &test_message)
            .await
            .unwrap()
    });
    assert!(timeout.is_err());
}
