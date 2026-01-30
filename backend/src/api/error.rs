use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ApiError::UnexpectedError(ref err) => {
                tracing::error!("unexpected error: {err:#}");

                #[cfg(debug_assertions)]
                let error_message = Some(format!("{err:#}"));
                #[cfg(not(debug_assertions))]
                let error_message = Some("unexpected error".to_string());

                (StatusCode::INTERNAL_SERVER_ERROR, error_message)
            }
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, None),
            ApiError::BadRequest(err) => (StatusCode::BAD_REQUEST, Some(err.to_string())),
        };

        return (status_code, Json(json!({ "error": error_message }))).into_response();
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        return ApiError::UnexpectedError(anyhow::anyhow!(err));
    }
}
