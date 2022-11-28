use axum_cometd::{LongPoolingServiceContext, LongPoolingServiceContextBuilder, RouterBuilder};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::{
    fmt::Debug,
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
        .with_env_filter("axum_cometd=debug")
        .init();

    let addr = "[::0]:1025".parse().unwrap();

    let context = LongPoolingServiceContextBuilder::new()
        .timeout_ms(5000)
        .max_interval_ms(2000)
        .client_channel_capacity(10_000)
        .subscription_channel_capacity(20_000)
        .build();
    let app = RouterBuilder::new()
        .base_path("/notifications/")
        .build(&context);

    tracing::info!("Listen on: `{addr}`.");

    tokio::task::spawn(axum::Server::bind(&addr).serve(app.into_make_service()));

    spawn_topic(context.clone(), "/topic0");
    spawn_topic(context, "/topic1");

    tokio::time::sleep(Duration::from_secs(366 * 24 * 60)).await;
}

fn spawn_topic(context: Arc<LongPoolingServiceContext<Data>>, channel: &'static str) {
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
