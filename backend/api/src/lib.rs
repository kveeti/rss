use anyhow::Context;
use axum::{
    Json, Router,
    body::Body,
    debug_handler,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
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
        .route("/feeds", post(add_feed).get(get_feeds))
        .route("/feeds/{id}/icon", get(get_feed_icon))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("listening at {}", listener.local_addr().unwrap());

    axum::serve(listener, routes).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, world!"
}

#[derive(Debug, serde::Deserialize)]
struct AddFeedQuery {
    url: String,
}

#[debug_handler]
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

        NewFeedResult::Feed {
            feed,
            entries,
            icon,
        } => {
            state
                .data
                .add_feed_and_entries_and_icon(feed, entries, icon)
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

async fn get_feeds(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let feeds = state
        .data
        .get_feeds_with_entry_counts()
        .await
        .context("error getting feeds")?;

    Ok((StatusCode::OK, Json(feeds)).into_response())
}

async fn get_feed_icon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let icon = state
        .data
        .get_icon_by_feed_id(&id)
        .await
        .context("error getting feed")?;

    if let Some(icon) = icon {
        let content_type = icon.content_type.parse::<HeaderValue>().unwrap();
        let data = if content_type == "image/svg+xml" {
            Body::from(String::from_utf8_lossy(&icon.data).to_string())
        } else {
            Body::from(icon.data)
        };
        let mut headers = HeaderMap::new();
        headers.append(header::CONTENT_TYPE, content_type);
        return Ok((headers, data).into_response());
    }

    return Ok((
        StatusCode::NOT_FOUND,
        Json(json!({ "status": "not_found" })),
    )
        .into_response());
}
