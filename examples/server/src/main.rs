use axum_cometd::LongPoolingServiceContext;
use std::{error::Error, fmt::Debug, time::Duration};

#[derive(Debug, Clone, serde::Serialize)]
struct Data {
    msg: Box<str>,
    r#bool: bool,
    num: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "[::0]:1025".parse()?;

    let context = LongPoolingServiceContext::new();
    let app = context.build_router("/notifications/");

    println!("Listen on: `{addr}`.");

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
