use axum::{Router, routing::get};
use tokio::net::TcpListener;

pub async fn start_api() {
    let routes = Router::new().route("/", get(hello));

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, routes).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, world!"
}
