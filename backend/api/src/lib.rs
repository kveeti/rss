use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use db::Data;
use feed_loader::NewFeedResult;
use serde_json::json;
use tokio::net::TcpListener;

use crate::error::ApiError;

pub mod error;

#[derive(Clone)]
struct AppState {
    data: Data,
}

pub async fn start_api(data: Data) {
    let state = AppState { data };

    let routes = Router::new()
        .route("/", get(hello))
        .route("/load_feed", get(load_feed))
        .route("/feeds", post(add_feed))
        .with_state(state);

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

#[derive(Debug, serde::Deserialize)]
struct AddFeedQuery {
    url: String,
}

async fn add_feed(
    State(state): State<AppState>,
    Query(query): Query<AddFeedQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let existing_feed = state.data.get_feed_by_url(&query.url).await.unwrap();
    if existing_feed.is_some() {
        return Ok((
            StatusCode::CONFLICT,
            Json(json!({ "status": "feed_already_saved" })),
        )
            .into_response());
    }

    let res = feed_loader::new_feed(&query.url).await.unwrap();

    let response = match res {
        NewFeedResult::DiscoveredMultiple(feed_urls) => (
            StatusCode::OK,
            Json(json!({
                "status": "discovered_multiple",
                "feed_urls": feed_urls
            })),
        )
            .into_response(),

        NewFeedResult::Feed { feed, entries } => {
            state
                .data
                .add_feed_and_entries(feed, entries)
                .await
                .context("error adding feed and entries")?;

            (StatusCode::OK, Json(json!({ "status": "feed_added" }))).into_response()
        }

        NewFeedResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "not_found" })),
        )
            .into_response(),

        NewFeedResult::NotAllowed => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "not_allowed" })),
        )
            .into_response(),

        NewFeedResult::Unknown { status, body } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "unknown", "status": status, "body": body })),
        )
            .into_response(),
    };

    Ok(response)
}
