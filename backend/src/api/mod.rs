use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::db::Data;

mod error;
mod handlers;

#[derive(Clone)]
struct AppState {
    data: Data,
}

pub async fn start_api(data: Data) {
    let state = AppState { data };

    let v1_routes = Router::new()
        .route(
            "/feeds",
            post(handlers::feeds::new_feed).get(handlers::feeds::query_feeds),
        )
        .route("/feeds/{id}/icon", get(handlers::feeds::get_feed_icon))
        .route("/feeds/{id}", get(handlers::feeds::get_feed))
        .route(
            "/feeds/{id}/entries",
            get(handlers::feeds::get_feed_entries),
        )
        .layer(cors("http://localhost:3000"))
        .with_state(state);

    let api_routes = Router::new().nest(
        "/api",
        Router::new()
            .nest("/v1", v1_routes)
            .route("/health", get(health)),
    );

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, api_routes).await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}

fn cors(front_base_url: &str) -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::HEAD, Method::GET])
        .allow_headers([
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ACCEPT_ENCODING,
            header::ACCEPT_LANGUAGE,
        ])
        .allow_origin(
            front_base_url
                .parse::<HeaderValue>()
                .expect("allow origin value"),
        )
}
