use axum_cometd::{LongPoolingServiceContextBuilder, RouterBuilder};
use std::{borrow::Cow, error::Error, fmt::Debug, time::Duration};

#[derive(Debug, Clone, serde::Serialize)]
struct Data<'a> {
    msg: Cow<'a, str>,
    r#bool: bool,
    num: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("axum_cometd=trace")
        .init();

    let addr = "[::0]:1025".parse()?;

    let context = LongPoolingServiceContextBuilder::new()
        .timeout_ms(1000)
        .max_interval_ms(2000)
        .client_channel_capacity(10_000)
        .subscription_channel_capacity(20_000)
        .build();
    let app = RouterBuilder::new()
        .base_path("/notifications/")
        .build(&context);

    tracing::info!("Listen on: `{addr}`.");

    tokio::task::spawn(axum::Server::bind(&addr).serve(app.into_make_service()));

    loop {
        context
            .send(
                "/topic",
                Data {
                    msg: "Hello World!!!".into(),
                    r#bool: true,
                    num: u64::MAX,
                },
            )
            .await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
