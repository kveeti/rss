use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    routing::{get, post},
};
use std::time::Duration;
use tokio::{net::TcpListener, sync::watch};
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::db::Data;

mod error;
mod frontend;
mod handlers;

#[derive(Clone)]
struct AppState {
    data: Data,
}

pub struct ApiConfig {
    pub host: String,
    pub frontend_dir: Option<String>,
}

pub async fn start_api(data: Data, config: ApiConfig, mut shutdown_rx: watch::Receiver<bool>) {
    let state = AppState { data };

    let v1_routes = Router::new()
        .route(
            "/feeds",
            post(handlers::feeds::new_feed).get(handlers::feeds::query_feeds),
        )
        .route("/feeds/import", post(handlers::feeds::import_opml))
        .route("/feeds/export", get(handlers::feeds::export_opml))
        .route(
            "/feeds/import/{job_id}/events",
            get(handlers::feeds::import_opml_events),
        )
        .route("/feeds/{id}/icon", get(handlers::feeds::get_feed_icon))
        .route(
            "/feeds/{id}",
            get(handlers::feeds::get_feed)
                .put(handlers::feeds::update_feed)
                .delete(handlers::feeds::delete_feed),
        )
        .route(
            "/feeds/{id}/entries",
            get(handlers::feeds::get_feed_entries),
        )
        .route("/feeds/{id}/sync", post(handlers::feeds::sync_feed))
        .route("/entries", get(handlers::entries::query_entries))
        .route(
            "/entries/{id}/read",
            post(handlers::entries::update_entry_read),
        );

    let mut app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/v1", v1_routes)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(|request: &Request<Body>| {
                            tracing::info_span!(
                                "request",
                                method = %request.method(),
                                path = %request.uri().path(),
                            )
                        })
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .route("/health", get(health)),
        )
        .with_state(state);

    if let Some(dir) = &config.frontend_dir {
        tracing::info!("serving frontend from {dir}");
        app = app.merge(frontend::router(dir));
    }

    let listener = TcpListener::bind(&config.host)
        .await
        .expect("tcp listener successful bind");
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.wait_for(|&v| v).await;
            tracing::info!("api server shutting down");
        })
        .await
        .expect("axum successful serve");
}

async fn health(State(state): State<AppState>) -> StatusCode {
    match tokio::time::timeout(Duration::from_secs(2), state.data.ping()).await {
        Ok(Ok(())) => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}
