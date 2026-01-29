//! PostgreSQL-specific tests for the DataI trait.
//!
//! Each test creates an isolated database using TestDb, calls the generic
//! test function, and automatically cleans up when done.

use crate::db::pg::test_utils::TestDb;

use super::{
    test_create_feed, test_create_feed_with_icon, test_create_feed_without_entries,
    test_create_opml_import_job, test_delete_feed, test_delete_feed_cascades_entries,
    test_delete_feed_not_found, test_feed_icon_update, test_get_existing_feed_urls,
    test_get_existing_feed_urls_empty, test_get_feed_by_id, test_get_feed_by_id_not_found,
    test_get_feed_entries_cursor, test_get_feed_entries_cursor_left, test_get_feed_entries_empty,
    test_get_feed_entries_limit, test_get_feeds_empty, test_get_feeds_to_sync_empty,
    test_get_feeds_to_sync_excludes_parse_error, test_get_feeds_to_sync_respects_sync_timeout,
    test_get_feeds_to_sync_returns_stale, test_get_one_feed_to_sync,
    test_get_opml_import_job_not_found, test_get_opml_import_recent_items,
    test_get_similar_named_feed, test_get_similar_named_feed_no_match,
    test_icon_deduplication_by_hash, test_insert_stub_feeds, test_query_entries_cursor_pagination,
    test_query_entries_empty, test_query_entries_filter_date_range,
    test_query_entries_filter_feed_id, test_query_entries_filter_query_search,
    test_query_entries_filter_sort_and_limit, test_query_entries_filter_starred,
    test_query_entries_filter_unread, test_query_entries_no_filters, test_set_feed_sync_result,
    test_update_feed, test_update_feed_clear_user_title, test_update_feed_not_found,
    test_update_opml_import_item_and_job_status, test_upsert_entries,
    test_upsert_entries_updates_existing, test_upsert_feed_deduplicates_entries,
    test_upsert_feed_updates_existing, test_upsert_icon,
};

#[tokio::test]
async fn pg_get_feeds_empty() {
    let test_db = TestDb::new().await;
    test_get_feeds_empty(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Create feed tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_create_feed() {
    let test_db = TestDb::new().await;
    test_create_feed(&*test_db.data).await;
}

#[tokio::test]
async fn pg_create_feed_without_entries() {
    let test_db = TestDb::new().await;
    test_create_feed_without_entries(&*test_db.data).await;
}

#[tokio::test]
async fn pg_create_feed_with_icon() {
    let test_db = TestDb::new().await;
    test_create_feed_with_icon(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Upsert behavior tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_upsert_feed_updates_existing() {
    let test_db = TestDb::new().await;
    test_upsert_feed_updates_existing(&*test_db.data).await;
}

#[tokio::test]
async fn pg_upsert_entries_updates_existing() {
    let test_db = TestDb::new().await;
    test_upsert_entries_updates_existing(&*test_db.data).await;
}

#[tokio::test]
async fn pg_upsert_feed_deduplicates_entries() {
    let test_db = TestDb::new().await;
    test_upsert_feed_deduplicates_entries(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Read feed tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_get_feed_by_id() {
    let test_db = TestDb::new().await;
    test_get_feed_by_id(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feed_by_id_not_found() {
    let test_db = TestDb::new().await;
    test_get_feed_by_id_not_found(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_existing_feed_urls() {
    let test_db = TestDb::new().await;
    test_get_existing_feed_urls(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_existing_feed_urls_empty() {
    let test_db = TestDb::new().await;
    test_get_existing_feed_urls_empty(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Entries tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_upsert_entries() {
    let test_db = TestDb::new().await;
    test_upsert_entries(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feed_entries_empty() {
    let test_db = TestDb::new().await;
    test_get_feed_entries_empty(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feed_entries_limit() {
    let test_db = TestDb::new().await;
    test_get_feed_entries_limit(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feed_entries_cursor() {
    let test_db = TestDb::new().await;
    test_get_feed_entries_cursor(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feed_entries_cursor_left() {
    let test_db = TestDb::new().await;
    test_get_feed_entries_cursor_left(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_cursor_pagination() {
    let test_db = TestDb::new().await;
    test_query_entries_cursor_pagination(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_no_filters() {
    let test_db = TestDb::new().await;
    test_query_entries_no_filters(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_feed_id() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_feed_id(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_sort_and_limit() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_sort_and_limit(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_unread() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_unread(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_empty() {
    let test_db = TestDb::new().await;
    test_query_entries_empty(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_starred() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_starred(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_query_search() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_query_search(&*test_db.data).await;
}

#[tokio::test]
async fn pg_query_entries_filter_date_range() {
    let test_db = TestDb::new().await;
    test_query_entries_filter_date_range(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Update feed tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_update_feed() {
    let test_db = TestDb::new().await;
    test_update_feed(&*test_db.data).await;
}

#[tokio::test]
async fn pg_update_feed_not_found() {
    let test_db = TestDb::new().await;
    test_update_feed_not_found(&*test_db.data).await;
}

#[tokio::test]
async fn pg_update_feed_clear_user_title() {
    let test_db = TestDb::new().await;
    test_update_feed_clear_user_title(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Delete feed tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_delete_feed() {
    let test_db = TestDb::new().await;
    test_delete_feed(&*test_db.data).await;
}

#[tokio::test]
async fn pg_delete_feed_not_found() {
    let test_db = TestDb::new().await;
    test_delete_feed_not_found(&*test_db.data).await;
}

#[tokio::test]
async fn pg_delete_feed_cascades_entries() {
    let test_db = TestDb::new().await;
    test_delete_feed_cascades_entries(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Sync tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_get_feeds_to_sync_empty() {
    let test_db = TestDb::new().await;
    test_get_feeds_to_sync_empty(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feeds_to_sync_returns_stale() {
    let test_db = TestDb::new().await;
    test_get_feeds_to_sync_returns_stale(&*test_db.data).await;
}

#[tokio::test]
async fn pg_set_feed_sync_result() {
    let test_db = TestDb::new().await;
    test_set_feed_sync_result(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_one_feed_to_sync() {
    let test_db = TestDb::new().await;
    test_get_one_feed_to_sync(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_similar_named_feed() {
    let test_db = TestDb::new().await;
    test_get_similar_named_feed(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_similar_named_feed_no_match() {
    let test_db = TestDb::new().await;
    test_get_similar_named_feed_no_match(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feeds_to_sync_excludes_parse_error() {
    let test_db = TestDb::new().await;
    test_get_feeds_to_sync_excludes_parse_error(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_feeds_to_sync_respects_sync_timeout() {
    let test_db = TestDb::new().await;
    test_get_feeds_to_sync_respects_sync_timeout(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// Icon tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_upsert_icon() {
    let test_db = TestDb::new().await;
    test_upsert_icon(&*test_db.data).await;
}

#[tokio::test]
async fn pg_icon_deduplication_by_hash() {
    let test_db = TestDb::new().await;
    test_icon_deduplication_by_hash(&*test_db.data).await;
}

#[tokio::test]
async fn pg_feed_icon_update() {
    let test_db = TestDb::new().await;
    test_feed_icon_update(&*test_db.data).await;
}

// ----------------------------------------------------------------------------
// OPML import tests
// ----------------------------------------------------------------------------

#[tokio::test]
async fn pg_create_opml_import_job() {
    let test_db = TestDb::new().await;
    test_create_opml_import_job(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_opml_import_job_not_found() {
    let test_db = TestDb::new().await;
    test_get_opml_import_job_not_found(&*test_db.data).await;
}

#[tokio::test]
async fn pg_update_opml_import_item_and_job_status() {
    let test_db = TestDb::new().await;
    test_update_opml_import_item_and_job_status(&*test_db.data).await;
}

#[tokio::test]
async fn pg_get_opml_import_recent_items() {
    let test_db = TestDb::new().await;
    test_get_opml_import_recent_items(&*test_db.data).await;
}

#[tokio::test]
async fn pg_insert_stub_feeds() {
    let test_db = TestDb::new().await;
    test_insert_stub_feeds(&*test_db.data).await;
}
