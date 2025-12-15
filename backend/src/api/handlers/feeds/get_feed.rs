use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::api::{AppState, error::ApiError};

pub async fn get_feed(
    State(state): State<AppState>,
    Path(feed_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let feed = state
        .data
        .get_feed_by_id_with_entry_counts(&feed_id)
        .await?;

    Ok((StatusCode::OK, Json(feed)).into_response())
}
