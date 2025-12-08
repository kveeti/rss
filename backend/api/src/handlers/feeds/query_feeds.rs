use anyhow::Context;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{AppState, error::ApiError};

pub async fn query_feeds(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let feeds = state
        .data
        .get_feeds_with_entry_counts()
        .await
        .context("error getting feeds")?;

    Ok((StatusCode::OK, Json(feeds)).into_response())
}
