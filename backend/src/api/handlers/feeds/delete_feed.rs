use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::api::{AppState, error::ApiError};

pub async fn delete_feed(
    State(state): State<AppState>,
    Path(feed_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted = state.data.delete_feed(&feed_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(ApiError::NotFound("feed not found".to_string()))
    }
}
