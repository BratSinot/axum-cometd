use axum::{Router, Server};
use axum_cometd::{
    Event, LongPollingServiceContext, LongPollingServiceContextBuilder, RouterBuilder,
};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::{
    fmt::Debug,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, serde::Serialize)]
struct Data {
    channel: &'static str,
    msg: String,
    timestamp: u64,
}

#[inline]
fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("server=debug,axum_cometd=debug")
        .init();

    let context = LongPollingServiceContextBuilder::new()
        .timeout_ms(5000)
        .max_interval_ms(2000)
        .client_channel_capacity(500)
        .client_storage_capacity(10_000)
        .subscription_channel_capacity(500)
        .subscription_storage_capacity(10_000)
        .build();

    let mut rx = context.rx();
    tokio::task::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match *event {
                Event::SessionAdded {
                    client_id,
                    ref headers,
                    ..
                } => {
                    tracing::info!("Got new session {client_id}: `{headers:?}.");
                }
                Event::SessionRemoved { client_id, .. } => {
                    tracing::info!("Removed session {client_id}.");
                }
                _ => {}
            }
        }
    });

    let service = RouterBuilder::new()
        .subscribe_base_path("/notifications")
        .handshake_base_path("/notifications")
        .connect_base_path("/notifications")
        .disconnect_base_path("/notifications")
        .build(Arc::clone(&context))
        .into_make_service();
    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 1025);

    let handler = tokio::task::spawn(Server::bind(&addr).serve(service));

    tracing::info!("Listen on: `{addr}`.");

    spawn_topic(Arc::clone(&context), "/topic0");
    spawn_topic(context, "/topic1");

    handler.await.unwrap().unwrap();
}

fn spawn_topic(context: Arc<LongPollingServiceContext<(), ()>>, channel: &'static str) {
    tokio::task::spawn(async move {
        let mut rng: StdRng = SeedableRng::from_entropy();
        let distribution = Uniform::new(500, 1000);

        loop {
            context
                .send(
                    channel,
                    Data {
                        channel,
                        msg: format!("Hello from {channel}"),
                        timestamp: timestamp(),
                    },
                )
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(rng.sample(distribution))).await;
        }
    });
}
