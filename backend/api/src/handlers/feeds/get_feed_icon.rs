use anyhow::Context;
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

use crate::{AppState, error::ApiError};

pub async fn get_feed_icon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let icon = state
        .data
        .get_icon_by_feed_id(&id)
        .await
        .context("error getting feed")?;

    if let Some(icon) = icon {
        let content_type = icon.content_type.parse::<HeaderValue>().unwrap();
        let data = if content_type == "image/svg+xml" {
            Body::from(String::from_utf8_lossy(&icon.data).to_string())
        } else {
            Body::from(icon.data)
        };
        let mut headers = HeaderMap::new();
        headers.append(header::CONTENT_TYPE, content_type);
        return Ok((headers, data).into_response());
    }

    return Ok((
        StatusCode::NOT_FOUND,
        Json(json!({ "status": "not_found" })),
    )
        .into_response());
}
