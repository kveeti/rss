use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{collections::HashSet, sync::Arc};

mod id;
pub use id::*;

pub(crate) mod pg;

#[cfg(test)]
mod tests;

#[async_trait]
pub trait DataI: Send + Sync {
    async fn upsert_feed_and_entries_and_icon(
        &self,
        feed: &NewFeed,
        entries: Vec<NewEntry>,
        icon: Option<NewIcon>,
    ) -> Result<(), anyhow::Error>;

    async fn upsert_entries(
        &self,
        feed_id: &str,
        entries: Vec<NewEntry>,
    ) -> Result<(), sqlx::Error>;

    async fn get_feed_by_id_with_entry_counts(
        &self,
        id: &str,
    ) -> Result<Option<FeedWithEntryCounts>, sqlx::Error>;

    async fn get_feeds_with_entry_counts(&self) -> Result<Vec<FeedWithEntryCounts>, sqlx::Error>;

    async fn get_feed_entries(
        &self,
        feed_id: &str,
        cursor: Option<Cursor>,
        limit: Option<i64>,
    ) -> Result<CursorOutput<EntryForList>, sqlx::Error>;

    async fn query_entries(
        &self,
        cursor: Option<Cursor>,
        filters: Option<QueryFeedsFilters>,
    ) -> Result<CursorOutput<EntryForQueryList>, sqlx::Error>;

    async fn get_existing_feed_urls(
        &self,
        feed_urls: &[String],
    ) -> Result<HashSet<String>, sqlx::Error>;

    async fn get_feeds_to_sync(
        &self,
        last_synced_before: DateTime<Utc>,
    ) -> anyhow::Result<Vec<FeedToSync>>;

    async fn set_feed_sync_result(&self, feed_url: &str, result: &str) -> Result<(), sqlx::Error>;

    async fn update_feed_headers(
        &self,
        feed_url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<(), sqlx::Error>;

    async fn get_one_feed_to_sync(&self, feed_id: &str) -> Result<Option<FeedToSync>, sqlx::Error>;

    async fn get_similar_named_feed(
        &self,
        feed_url: &str,
    ) -> Result<Option<FeedToSync>, sqlx::Error>;

    async fn update_feed(
        &self,
        feed_id: &str,
        user_title: Option<&str>,
        feed_url: &str,
        site_url: Option<&str>,
    ) -> Result<(), sqlx::Error>;

    async fn delete_feed(&self, feed_id: &str) -> Result<bool, anyhow::Error>;

    async fn upsert_icon(&self, icon: NewIcon) -> Result<(), sqlx::Error>;

    async fn get_icon_by_feed_id(&self, feed_id: &str) -> Result<Option<Icon>, sqlx::Error>;

    async fn create_opml_import_job(
        &self,
        feed_urls: &[String],
        existing_urls: &HashSet<String>,
    ) -> Result<OpmlImportJobSummary, sqlx::Error>;

    async fn insert_stub_feeds(&self, feed_urls: &[String]) -> Result<(), sqlx::Error>;

    async fn update_opml_import_item(
        &self,
        job_id: &str,
        feed_url: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error>;

    async fn increment_opml_import_job_counts(
        &self,
        job_id: &str,
        imported: i64,
        skipped: i64,
        failed: i64,
    ) -> Result<(), sqlx::Error>;

    async fn update_opml_import_job_status(
        &self,
        job_id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error>;

    async fn get_opml_import_job(&self, job_id: &str)
    -> Result<Option<OpmlImportJob>, sqlx::Error>;
    async fn get_opml_import_recent_items(
        &self,
        job_id: &str,
        limit: i64,
    ) -> Result<Vec<OpmlImportItem>, sqlx::Error>;

    async fn update_entry_read_status(&self, entry_id: &str, read: bool)
    -> Result<(), sqlx::Error>;
}

pub type Data = Arc<dyn DataI>;

pub async fn new_pg_data(database_url: &str) -> Result<Data> {
    pg::new_pg_data(database_url).await
}

#[derive(Debug, serde::Serialize)]
pub struct NewIcon {
    pub hash: String,
    pub data: Vec<u8>,
    pub content_type: String,
}

pub struct Icon {
    pub id: String,
    pub hash: String,
    pub data: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, Clone)]
pub struct OpmlImportJobSummary {
    pub job_id: String,
    pub total: i64,
    pub skipped: i64,
}

#[derive(Debug, Clone)]
pub struct OpmlImportJob {
    pub id: String,
    pub status: String,
    pub total: i64,
    pub imported: i64,
    pub skipped: i64,
    pub failed: i64,
}

#[derive(Debug, Clone)]
pub struct OpmlImportItem {
    pub feed_url: String,
    pub status: String,
    pub error: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Newest,
    Oldest,
}

pub struct QueryFeedsFilters {
    pub limit: Option<u64>,
    pub query: Option<String>,
    pub feed_id: Option<String>,
    pub unread: Option<bool>,
    pub starred: Option<bool>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub sort: Option<SortOrder>,
}

pub enum Cursor {
    Left(String),
    Right(String),
}

#[derive(Debug, serde::Serialize)]
pub struct CursorOutput<T> {
    pub entries: Vec<T>,
    pub next_id: Option<String>,
    pub prev_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Entry {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub comments_url: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub starred_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct EntryForList {
    pub id: String,
    pub title: String,
    pub url: String,
    pub comments_url: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub starred_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub entry_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct EntryForQueryList {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub url: String,
    pub comments_url: Option<String>,
    pub read_at: Option<DateTime<Utc>>,
    pub starred_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub entry_updated_at: Option<DateTime<Utc>>,
    pub has_icon: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NewEntry {
    pub title: String,
    pub url: String,
    pub comments_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub entry_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct NewFeed {
    pub title: String,
    pub site_url: Option<String>,
    pub feed_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct FeedWithEntryCounts {
    pub id: String,
    pub title: String,
    pub source_title: String,
    pub user_title: Option<String>,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub entry_count: i64,
    pub unread_entry_count: i64,
    pub has_icon: bool,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_sync_result: Option<String>,
}

pub struct FeedToSync {
    pub id: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub http_etag: Option<String>,
    pub http_last_modified: Option<String>,
}
