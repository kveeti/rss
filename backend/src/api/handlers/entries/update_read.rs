use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;

use crate::api::{AppState, error::ApiError};

#[derive(serde::Deserialize)]
pub struct UpdateEntryReadBody {
    pub read: bool,
}

pub async fn update_entry_read(
    State(state): State<AppState>,
    Path(entry_id): Path<String>,
    Json(body): Json<UpdateEntryReadBody>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .data
        .update_entry_read_status(&entry_id, body.read)
        .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response())
}
