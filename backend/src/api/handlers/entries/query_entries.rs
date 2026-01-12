use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use reqwest::StatusCode;

use crate::{
    api::{AppState, error::ApiError},
    db::{Cursor, QueryFeedsFilters, SortOrder},
};

#[derive(serde::Deserialize)]
pub struct QueryEntriesQuery {
    left: Option<String>,
    right: Option<String>,
    limit: Option<u64>,
    query: Option<String>,
    feed_id: Option<String>,
    unread: Option<bool>,
    starred: Option<bool>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    sort: Option<SortOrder>,
}

pub async fn query_entries(
    State(state): State<AppState>,
    Query(query): Query<QueryEntriesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let cursor = if let Some(left) = query.left {
        Some(Cursor::Left(left))
    } else if let Some(right) = query.right {
        Some(Cursor::Right(right))
    } else {
        None
    };

    let has_filters = query.limit.is_some()
        || query.query.is_some()
        || query.feed_id.is_some()
        || query.unread.is_some()
        || query.starred.is_some()
        || query.start.is_some()
        || query.end.is_some()
        || query.sort.is_some();

    let filters = if has_filters {
        Some(QueryFeedsFilters {
            limit: query.limit,
            query: query.query,
            feed_id: query.feed_id,
            unread: query.unread,
            starred: query.starred,
            start: query.start,
            end: query.end,
            sort: query.sort,
        })
    } else {
        None
    };

    let entries = state.data.query_entries(cursor, filters).await?;

    Ok((StatusCode::OK, Json(entries)).into_response())
}
