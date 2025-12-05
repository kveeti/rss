use api::start_api;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from("aio=debug,api=debug".to_string()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    start_api().await;
}
