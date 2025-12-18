use anyhow::{Context, anyhow};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{AppState, error::ApiError},
    feed_loader::{GetFeedResult, get_feed},
};

pub async fn sync_feed(
    State(state): State<AppState>,
    Path(feed_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let feed = state
        .data
        .get_one_feed_to_sync(&feed_id)
        .await
        .context("error getting feed to sync")?
        .ok_or(ApiError::NotFound("feed not found".to_string()))?;

    let feed_res = get_feed(&feed.feed_url)
        .await
        .context("error getting feed")?;

    match feed_res {
        GetFeedResult::Feed {
            feed,
            entries,
            icon,
        } => {
            state
                .data
                .upsert_feed_and_entries_and_icon(&feed, entries, icon)
                .await?;

            let feed = state
                .data
                .get_feed_by_id_with_entry_counts(&feed_id)
                .await
                .context("error getting updated feed")?;

            Ok((StatusCode::OK, Json(feed)))
        }
        _ => Err(ApiError::UnexpectedError(anyhow!(
            "unexpected feed response"
        ))),
    }
}
