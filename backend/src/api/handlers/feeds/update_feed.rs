use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::api::{AppState, error::ApiError};

#[derive(Debug, serde::Deserialize)]
pub struct UpdateFeedBody {
    title: String,
    feed_url: String,
    site_url: Option<String>,
}

pub async fn update_feed(
    State(state): State<AppState>,
    Path(feed_id): Path<String>,
    Json(payload): Json<UpdateFeedBody>,
) -> Result<impl IntoResponse, ApiError> {
    let title = payload.title.trim();
    let feed_url = payload.feed_url.trim();
    let site_url = normalize_optional(payload.site_url);

    if title.is_empty() {
        return Err(ApiError::BadRequest("title is required".to_string()));
    }
    if feed_url.is_empty() {
        return Err(ApiError::BadRequest("feed_url is required".to_string()));
    }

    state
        .data
        .update_feed(&feed_id, title, feed_url, site_url.as_deref())
        .await?;

    let updated_feed = state
        .data
        .get_feed_by_id_with_entry_counts(&feed_id)
        .await?;

    Ok((StatusCode::OK, Json(updated_feed)).into_response())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}
