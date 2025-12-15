use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{AppState, error::ApiError},
    db::Cursor,
};

#[derive(Debug, serde::Deserialize)]
pub struct GetFeedEntriesQuery {
    left: Option<String>,
    right: Option<String>,
    limit: Option<i64>,
}

pub async fn get_feed_entries(
    State(state): State<AppState>,
    Path(feed_id): Path<String>,
    Query(input): Query<GetFeedEntriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let cursor = if let Some(left) = input.left {
        Some(Cursor::Left(left))
    } else if let Some(right) = input.right {
        Some(Cursor::Right(right))
    } else {
        None
    };

    let limit = input.limit;

    let entries = state.data.get_feed_entries(&feed_id, cursor, limit).await?;

    Ok((StatusCode::OK, Json(entries)).into_response())
}
