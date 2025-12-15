use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod api;
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

    let data = db::Data::new("postgres://pg:pg@localhost:5432/db")
        .await
        .expect("creating Data");

    let _ = tokio::join!(
        feed_loader::feed_sync_loop(data.clone()),
        api::start_api(data)
    );
}
