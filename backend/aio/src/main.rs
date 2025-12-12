use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from(
            "aio=debug,api=debug,feed_loader=debug,db=debug".to_string(),
        ))
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
