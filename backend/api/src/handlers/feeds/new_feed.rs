use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use feed_loader::GetFeedResult;
use serde_json::json;

use crate::{AppState, error::ApiError};

#[derive(Debug, serde::Deserialize)]
pub struct AddFeedQuery {
    url: String,
    force_similar_feed: Option<bool>,
}

pub async fn new_feed(
    State(state): State<AppState>,
    Query(query): Query<AddFeedQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = feed_loader::get_feed(&query.url).await.unwrap();
    let force_similar = query.force_similar_feed.unwrap_or(false);

    let existing_feed = if !force_similar {
        state
            .data
            .get_similar_named_feed(&query.url)
            .await
            .context("error searching for similar named feed")?
    } else {
        None
    };

    let response = match res {
        GetFeedResult::DiscoveredMultiple(feed_urls) => (
            StatusCode::OK,
            Json(json!({
                "status": "discovered_multiple",
                "feed_urls": feed_urls,
                "similar_feed_url": existing_feed.map(|f| f.feed_url)
            })),
        )
            .into_response(),

        GetFeedResult::Feed {
            feed,
            entries,
            icon,
        } => {
            if let Some(existing_feed) = existing_feed
                && !force_similar
            {
                (
                    StatusCode::OK,
                    Json(json!({
                        "status": "similar_feed",
                        "similar_feed_url": existing_feed.feed_url
                    })),
                )
                    .into_response()
            } else {
                state
                    .data
                    .upsert_feed_and_entries_and_icon(&feed, entries, icon)
                    .await?;

                (StatusCode::OK, Json(json!({ "status": "feed_added" }))).into_response()
            }
        }

        GetFeedResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "not_found" })),
        )
            .into_response(),

        GetFeedResult::NotAllowed => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "not_allowed" })),
        )
            .into_response(),

        GetFeedResult::Unknown { status, body } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "unknown", "status": status, "body": body })),
        )
            .into_response(),
    };

    Ok(response)
}
