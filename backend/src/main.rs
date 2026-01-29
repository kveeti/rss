use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

pub mod api;
pub mod config;
pub mod db;
pub mod feed_loader;

#[tokio::main]
pub async fn main() {
    let crate_name = env!("CARGO_CRATE_NAME");

    tracing_subscriber::registry()
        .with(EnvFilter::from(format!(
            "{crate_name}=debug,tower_http=info,sqlx=info,axum::rejection=trace"
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::new().expect("valid config");

    let data = db::new_pg_data(&config.database_url)
        .await
        .expect("creating data");

    let _ = tokio::join!(
        feed_loader::feed_sync_loop(data.clone()),
        api::start_api(data, config.into())
    );
}
