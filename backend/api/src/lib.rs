use axum::{Json, Router, extract::Query, http::StatusCode, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

pub async fn start_api() {
    let routes = Router::new()
        .route("/", get(hello))
        .route("/load_feed", get(load_feed));

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, routes).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, serde::Deserialize)]
struct LoadFeedQuery {
    url: String,
}

async fn load_feed(Query(query): Query<LoadFeedQuery>) -> impl IntoResponse {
    let feed = feed_loader::new_feed(&query.url).await.unwrap();

    (StatusCode::OK, Json(feed)).into_response()
}
