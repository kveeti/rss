use axum::{
    Router,
    routing::{get, post},
};
use db::Data;
use tokio::net::TcpListener;

pub mod error;
mod handlers;

#[derive(Clone)]
struct AppState {
    data: Data,
}

pub async fn start_api(data: Data) {
    let state = AppState { data };

    let routes = Router::new()
        .route("/", get(hello))
        .route(
            "/feeds",
            post(handlers::feeds::new_feed).get(handlers::feeds::query_feeds),
        )
        .route("/feeds/{id}/icon", get(handlers::feeds::get_feed_icon))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, routes).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, world!"
}
