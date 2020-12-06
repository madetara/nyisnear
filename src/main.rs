mod core;
use crate::core::bot;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting bot...");

    bot::run().await.unwrap();
}
