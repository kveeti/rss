use tokio::sync::watch;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

pub mod api;
pub mod config;
pub mod db;
pub mod feed_loader;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = Config::new().expect("valid config");

    let data = db::new_pg_data(&config.database_url)
        .await
        .expect("creating data");

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        shutdown_signal().await;
        tracing::info!("shutdown signal received, notifying tasks");
        let _ = shutdown_tx.send(true);
    });

    let _ = tokio::join!(
        feed_loader::feed_sync_loop(data.clone(), shutdown_rx.clone()),
        api::start_api(data, config.into(), shutdown_rx)
    );

    tracing::info!("shutdown complete");
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("error installing ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("error installing signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    #[cfg(not(unix))]
    ctrl_c.await;
}
