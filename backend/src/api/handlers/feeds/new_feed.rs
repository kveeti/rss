use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

use crate::{
    api::{AppState, error::ApiError},
    feed_loader::{self, FeedResult},
};

#[derive(Debug, serde::Deserialize)]
pub struct AddFeedQuery {
    url: String,
    force_similar_feed: Option<bool>,
}

pub async fn new_feed(
    State(state): State<AppState>,
    Query(query): Query<AddFeedQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let res = feed_loader::load_feed(&query.url).await.unwrap();
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
        FeedResult::NeedsChoice(feed_urls) => (
            StatusCode::OK,
            Json(json!({
                "status": "discovered_multiple",
                "feed_urls": feed_urls,
                "similar_feed_url": existing_feed.map(|f| f.feed_url)
            })),
        )
            .into_response(),

        FeedResult::Loaded(loaded_feed) => {
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
                    .upsert_feed_and_entries_and_icon(
                        &loaded_feed.feed,
                        loaded_feed.entries,
                        loaded_feed.icon,
                    )
                    .await?;

                (StatusCode::OK, Json(json!({ "status": "feed_added" }))).into_response()
            }
        }

        FeedResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "not_found" })),
        )
            .into_response(),

        FeedResult::Disallowed => (
            StatusCode::FORBIDDEN,
            Json(json!({ "status": "not_allowed" })),
        )
            .into_response(),
    };

    Ok(response)
}
