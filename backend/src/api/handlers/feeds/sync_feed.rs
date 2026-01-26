use anyhow::Context;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{AppState, error::ApiError},
    feed_loader::{FeedResult, load_feed, sync_result_for_error, sync_result_for_feed_result},
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

    let feed_res = load_feed(&feed.feed_url).await;

    match feed_res {
        Ok(FeedResult::Loaded(loaded_feed)) => {
            state
                .data
                .upsert_feed_and_entries_and_icon(
                    &loaded_feed.feed,
                    loaded_feed.entries,
                    loaded_feed.icon,
                )
                .await?;

            let feed = state
                .data
                .get_feed_by_id_with_entry_counts(&feed_id)
                .await
                .context("error getting updated feed")?;

            Ok((StatusCode::OK, Json(feed)))
        }
        Ok(result) => {
            let sync_result = sync_result_for_feed_result(&result);
            state
                .data
                .set_feed_sync_result(&feed.feed_url, sync_result)
                .await?;

            let feed = state
                .data
                .get_feed_by_id_with_entry_counts(&feed_id)
                .await
                .context("error getting updated feed")?;

            Ok((StatusCode::OK, Json(feed)))
        }
        Err(err) => {
            let sync_result = sync_result_for_error(&err);
            state
                .data
                .set_feed_sync_result(&feed.feed_url, sync_result)
                .await?;

            let feed = state
                .data
                .get_feed_by_id_with_entry_counts(&feed_id)
                .await
                .context("error getting updated feed")?;

            Ok((StatusCode::OK, Json(feed)))
        }
    }
}
