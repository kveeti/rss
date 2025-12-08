use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use feed_loader::NewFeedResult;
use serde_json::json;

use crate::{AppState, error::ApiError};

#[derive(Debug, serde::Deserialize)]
pub struct AddFeedQuery {
    url: String,
}

pub async fn new_feed(
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
