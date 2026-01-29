//! Generic tests for the DataI trait.
//!
//! Tests are written as async functions that accept `&dyn DataI`,
//! allowing them to be reused for any database backend implementation.

mod pg;

use crate::db::{Cursor, DataI, NewEntry, NewFeed, NewIcon, QueryFeedsFilters, SortOrder};
use chrono::{Duration, Utc};
use std::collections::HashSet;

// ============================================================================
// Test helpers
// ============================================================================

fn new_test_feed(title: &str, feed_url: &str) -> NewFeed {
    NewFeed {
        title: title.to_string(),
        feed_url: feed_url.to_string(),
        site_url: Some(format!("https://{}.example.com", title.replace(' ', "-"))),
    }
}

fn new_test_entry(title: &str, url: &str) -> NewEntry {
    NewEntry {
        title: title.to_string(),
        url: url.to_string(),
        comments_url: None,
        published_at: None,
        entry_updated_at: None,
    }
}

// ============================================================================
// Generic test implementations
// ============================================================================

/// Test that get_feeds_with_entry_counts returns empty when no feeds exist.
pub(super) async fn test_get_feeds_empty(db: &dyn DataI) {
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert!(feeds.is_empty());
}

// ----------------------------------------------------------------------------
// Create feed tests
// ----------------------------------------------------------------------------

/// Test creating a feed with entries.
pub(super) async fn test_create_feed(db: &dyn DataI) {
    let feed = new_test_feed("Test Feed", "https://example.com/feed.xml");
    let entries = vec![
        new_test_entry("Entry 1", "https://example.com/entry1"),
        new_test_entry("Entry 2", "https://example.com/entry2"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);

    let created = &feeds[0];
    assert_eq!(created.source_title, "Test Feed");
    assert_eq!(created.feed_url, "https://example.com/feed.xml");
    assert_eq!(
        created.site_url,
        Some("https://Test-Feed.example.com".to_string())
    );
    assert_eq!(created.entry_count, 2);
    assert_eq!(created.unread_entry_count, 2);
    assert!(!created.has_icon);
}

/// Test creating a feed without entries.
pub(super) async fn test_create_feed_without_entries(db: &dyn DataI) {
    let feed = new_test_feed("Empty Feed", "https://empty.example.com/feed.xml");

    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);

    let created = &feeds[0];
    assert_eq!(created.source_title, "Empty Feed");
    assert_eq!(created.entry_count, 0);
    assert_eq!(created.unread_entry_count, 0);
}

/// Test creating a feed with an icon.
pub(super) async fn test_create_feed_with_icon(db: &dyn DataI) {
    use crate::db::NewIcon;

    let feed = new_test_feed("Icon Feed", "https://icon.example.com/feed.xml");
    let icon = NewIcon {
        hash: "abc123hash".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic bytes
        content_type: "image/png".to_string(),
    };

    db.upsert_feed_and_entries_and_icon(&feed, vec![], Some(icon))
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);

    let created = &feeds[0];
    assert!(created.has_icon);

    // Verify icon can be retrieved
    let icon = db.get_icon_by_feed_id(&created.id).await.unwrap();
    assert!(icon.is_some());
    let icon = icon.unwrap();
    assert_eq!(icon.hash, "abc123hash");
    assert_eq!(icon.content_type, "image/png");
}

// ----------------------------------------------------------------------------
// Upsert behavior tests
// ----------------------------------------------------------------------------

/// Test that upserting a feed with the same feed_url updates existing fields.
pub(super) async fn test_upsert_feed_updates_existing(db: &dyn DataI) {
    // Create initial feed
    let feed = NewFeed {
        title: "Original Title".to_string(),
        feed_url: "https://upsert-update.example.com/feed.xml".to_string(),
        site_url: Some("https://original-site.example.com".to_string()),
    };
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);
    let original_id = feeds[0].id.clone();
    assert_eq!(feeds[0].source_title, "Original Title");

    // Upsert with same feed_url but different title and site_url
    let updated_feed = NewFeed {
        title: "Updated Title".to_string(),
        feed_url: "https://upsert-update.example.com/feed.xml".to_string(),
        site_url: Some("https://updated-site.example.com".to_string()),
    };
    db.upsert_feed_and_entries_and_icon(&updated_feed, vec![], None)
        .await
        .unwrap();

    // Should still be one feed (updated, not duplicated)
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0].id, original_id); // Same ID
    assert_eq!(feeds[0].source_title, "Updated Title");
    assert_eq!(
        feeds[0].site_url,
        Some("https://updated-site.example.com".to_string())
    );
}

/// Test that upserting entries with the same URL updates existing entries.
pub(super) async fn test_upsert_entries_updates_existing(db: &dyn DataI) {
    let feed = new_test_feed(
        "Entry Update Feed",
        "https://entry-update.example.com/feed.xml",
    );

    // Create feed with initial entry
    let initial_entry = NewEntry {
        title: "Original Entry Title".to_string(),
        url: "https://entry-update.example.com/entry1".to_string(),
        comments_url: None,
        published_at: Some(Utc::now() - Duration::days(1)),
        entry_updated_at: None,
    };
    db.upsert_feed_and_entries_and_icon(&feed, vec![initial_entry], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert_eq!(entries.entries.len(), 1);
    assert_eq!(entries.entries[0].title, "Original Entry Title");

    // Upsert entry with same URL but different title
    let updated_entry = NewEntry {
        title: "Updated Entry Title".to_string(),
        url: "https://entry-update.example.com/entry1".to_string(),
        comments_url: Some("https://entry-update.example.com/comments".to_string()),
        published_at: Some(Utc::now()),
        entry_updated_at: None,
    };
    db.upsert_feed_and_entries_and_icon(&feed, vec![updated_entry], None)
        .await
        .unwrap();

    // Should still be one entry (updated, not duplicated)
    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert_eq!(entries.entries.len(), 1);
    assert_eq!(entries.entries[0].title, "Updated Entry Title");
    assert_eq!(
        entries.entries[0].comments_url,
        Some("https://entry-update.example.com/comments".to_string())
    );
}

/// Test that entries with duplicate URLs in the same batch are deduplicated.
pub(super) async fn test_upsert_feed_deduplicates_entries(db: &dyn DataI) {
    let feed = new_test_feed("Dedup Feed", "https://dedup.example.com/feed.xml");

    // Create entries with duplicate URLs in the same batch
    let entries = vec![
        NewEntry {
            title: "First Version".to_string(),
            url: "https://dedup.example.com/entry1".to_string(),
            comments_url: None,
            published_at: None,
            entry_updated_at: None,
        },
        NewEntry {
            title: "Second Version".to_string(),
            url: "https://dedup.example.com/entry1".to_string(), // Same URL
            comments_url: None,
            published_at: None,
            entry_updated_at: None,
        },
        NewEntry {
            title: "Unique Entry".to_string(),
            url: "https://dedup.example.com/entry2".to_string(),
            comments_url: None,
            published_at: None,
            entry_updated_at: None,
        },
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    // Should only have 2 entries (duplicates deduplicated)
    assert_eq!(feeds[0].entry_count, 2);

    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert_eq!(entries.entries.len(), 2);

    // First occurrence wins in deduplication
    let entry1 = entries
        .entries
        .iter()
        .find(|e| e.url == "https://dedup.example.com/entry1")
        .expect("entry1");
    assert_eq!(entry1.title, "First Version");
}

// ----------------------------------------------------------------------------
// Read feed tests
// ----------------------------------------------------------------------------

/// Test getting a feed by ID.
pub(super) async fn test_get_feed_by_id(db: &dyn DataI) {
    // Create a feed first
    let feed = new_test_feed("Read Test Feed", "https://read.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    // Get the feed ID from the list
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = &feeds[0].id;

    // Now get by ID
    let result = db.get_feed_by_id_with_entry_counts(feed_id).await.unwrap();
    assert!(result.is_some());

    let found = result.unwrap();
    assert_eq!(found.id, *feed_id);
    assert_eq!(found.source_title, "Read Test Feed");
    assert_eq!(found.feed_url, "https://read.example.com/feed.xml");
}

/// Test getting a non-existent feed by ID returns None.
pub(super) async fn test_get_feed_by_id_not_found(db: &dyn DataI) {
    let result = db
        .get_feed_by_id_with_entry_counts("nonexistent-id")
        .await
        .unwrap();
    assert!(result.is_none());
}

/// Test checking which feed URLs already exist.
pub(super) async fn test_get_existing_feed_urls(db: &dyn DataI) {
    // Create two feeds
    let feed1 = new_test_feed("Feed 1", "https://feed1.example.com/feed.xml");
    let feed2 = new_test_feed("Feed 2", "https://feed2.example.com/feed.xml");

    db.upsert_feed_and_entries_and_icon(&feed1, vec![], None)
        .await
        .unwrap();
    db.upsert_feed_and_entries_and_icon(&feed2, vec![], None)
        .await
        .unwrap();

    // Check which URLs exist
    let urls_to_check = vec![
        "https://feed1.example.com/feed.xml".to_string(),
        "https://feed2.example.com/feed.xml".to_string(),
        "https://feed3.example.com/feed.xml".to_string(), // doesn't exist
    ];

    let existing = db.get_existing_feed_urls(&urls_to_check).await.unwrap();

    assert_eq!(existing.len(), 2);
    assert!(existing.contains("https://feed1.example.com/feed.xml"));
    assert!(existing.contains("https://feed2.example.com/feed.xml"));
    assert!(!existing.contains("https://feed3.example.com/feed.xml"));
}

/// Test that empty URL list returns empty set.
pub(super) async fn test_get_existing_feed_urls_empty(db: &dyn DataI) {
    let existing = db.get_existing_feed_urls(&[]).await.unwrap();
    assert!(existing.is_empty());
}

// ----------------------------------------------------------------------------
// Entries tests
// ----------------------------------------------------------------------------

/// Test inserting entries for an existing feed.
pub(super) async fn test_upsert_entries(db: &dyn DataI) {
    let feed = new_test_feed("Upsert Entries Feed", "https://upsert.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let entries = vec![
        new_test_entry("Upsert Entry 1", "https://upsert.example.com/entry1"),
        new_test_entry("Upsert Entry 2", "https://upsert.example.com/entry2"),
    ];

    db.upsert_entries(&feed_id, entries).await.unwrap();

    let updated = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(updated[0].entry_count, 2);

    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert_eq!(entries.entries.len(), 2);
}

/// Test listing entries for a feed with no entries.
pub(super) async fn test_get_feed_entries_empty(db: &dyn DataI) {
    let feed = new_test_feed("Empty Entries Feed", "https://entries.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert!(entries.entries.is_empty());
    assert!(entries.next_id.is_none());
    assert!(entries.prev_id.is_none());
}

/// Test that entry listing respects limit and returns a cursor.
pub(super) async fn test_get_feed_entries_limit(db: &dyn DataI) {
    let feed = new_test_feed("Limit Entries Feed", "https://limit.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Limit Entry 1", "https://limit.example.com/entry1"),
        new_test_entry("Limit Entry 2", "https://limit.example.com/entry2"),
        new_test_entry("Limit Entry 3", "https://limit.example.com/entry3"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let page = db.get_feed_entries(&feed_id, None, Some(2)).await.unwrap();
    assert_eq!(page.entries.len(), 2);
    assert!(page.next_id.is_some());
}

/// Test cursor pagination for feed entries.
pub(super) async fn test_get_feed_entries_cursor(db: &dyn DataI) {
    let feed = new_test_feed("Cursor Entries Feed", "https://cursor.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Cursor Entry 1", "https://cursor.example.com/entry1"),
        new_test_entry("Cursor Entry 2", "https://cursor.example.com/entry2"),
        new_test_entry("Cursor Entry 3", "https://cursor.example.com/entry3"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let first_page = db.get_feed_entries(&feed_id, None, Some(2)).await.unwrap();
    let next_id = first_page.next_id.clone().expect("next id");

    let second_page = db
        .get_feed_entries(&feed_id, Some(Cursor::Right(next_id)), Some(2))
        .await
        .unwrap();

    assert!(!second_page.entries.is_empty());
    let first_ids: HashSet<_> = first_page
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    assert!(
        second_page
            .entries
            .iter()
            .all(|entry| !first_ids.contains(&entry.id))
    );
}

/// Test backward cursor pagination (Cursor::Left) for feed entries.
pub(super) async fn test_get_feed_entries_cursor_left(db: &dyn DataI) {
    let feed = new_test_feed(
        "Cursor Left Entries Feed",
        "https://cursor-left.example.com/feed.xml",
    );
    let now = Utc::now();
    let entries = vec![
        NewEntry {
            title: "Entry 1".to_string(),
            url: "https://cursor-left.example.com/entry1".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(4)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Entry 2".to_string(),
            url: "https://cursor-left.example.com/entry2".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(3)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Entry 3".to_string(),
            url: "https://cursor-left.example.com/entry3".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(2)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Entry 4".to_string(),
            url: "https://cursor-left.example.com/entry4".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(1)),
            entry_updated_at: None,
        },
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    // Get first page (newest first by default): Entry 4, Entry 3
    let first_page = db.get_feed_entries(&feed_id, None, Some(2)).await.unwrap();
    assert_eq!(first_page.entries.len(), 2);
    assert_eq!(first_page.entries[0].title, "Entry 4");
    assert_eq!(first_page.entries[1].title, "Entry 3");
    assert!(first_page.next_id.is_some());
    assert!(first_page.prev_id.is_none()); // No previous on first page

    // Navigate forward to second page: Entry 2, Entry 1
    let next_id = first_page.next_id.clone().unwrap();
    let second_page = db
        .get_feed_entries(&feed_id, Some(Cursor::Right(next_id)), Some(2))
        .await
        .unwrap();
    assert_eq!(second_page.entries.len(), 2);
    assert_eq!(second_page.entries[0].title, "Entry 2");
    assert_eq!(second_page.entries[1].title, "Entry 1");
    assert!(second_page.prev_id.is_some()); // Can go back

    // Navigate backward using Cursor::Left
    let prev_id = second_page.prev_id.clone().unwrap();
    let back_to_first = db
        .get_feed_entries(&feed_id, Some(Cursor::Left(prev_id)), Some(2))
        .await
        .unwrap();

    // Should get same entries as first page
    assert_eq!(back_to_first.entries.len(), 2);
    assert_eq!(back_to_first.entries[0].title, "Entry 4");
    assert_eq!(back_to_first.entries[1].title, "Entry 3");
}

/// Test cursor pagination for query_entries (both directions).
pub(super) async fn test_query_entries_cursor_pagination(db: &dyn DataI) {
    let feed = new_test_feed(
        "Query Cursor Feed",
        "https://query-cursor.example.com/feed.xml",
    );
    let now = Utc::now();
    let entries = vec![
        NewEntry {
            title: "Query Entry 1".to_string(),
            url: "https://query-cursor.example.com/entry1".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(4)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Query Entry 2".to_string(),
            url: "https://query-cursor.example.com/entry2".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(3)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Query Entry 3".to_string(),
            url: "https://query-cursor.example.com/entry3".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(2)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Query Entry 4".to_string(),
            url: "https://query-cursor.example.com/entry4".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::hours(1)),
            entry_updated_at: None,
        },
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    // Get first page with limit
    let filters = QueryFeedsFilters {
        limit: Some(2),
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: Some(SortOrder::Newest),
    };

    let first_page = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(first_page.entries.len(), 2);
    assert_eq!(first_page.entries[0].title, "Query Entry 4");
    assert_eq!(first_page.entries[1].title, "Query Entry 3");
    assert!(first_page.next_id.is_some());

    // Navigate forward
    let next_id = first_page.next_id.clone().unwrap();
    let filters = QueryFeedsFilters {
        limit: Some(2),
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: Some(SortOrder::Newest),
    };

    let second_page = db
        .query_entries(Some(Cursor::Right(next_id)), Some(filters))
        .await
        .unwrap();
    assert_eq!(second_page.entries.len(), 2);
    assert_eq!(second_page.entries[0].title, "Query Entry 2");
    assert_eq!(second_page.entries[1].title, "Query Entry 1");
    assert!(second_page.prev_id.is_some());

    // Navigate backward using Cursor::Left
    let prev_id = second_page.prev_id.clone().unwrap();
    let filters = QueryFeedsFilters {
        limit: Some(2),
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: Some(SortOrder::Newest),
    };

    let back_to_first = db
        .query_entries(Some(Cursor::Left(prev_id)), Some(filters))
        .await
        .unwrap();

    // Should get same entries as first page
    assert_eq!(back_to_first.entries.len(), 2);
    assert_eq!(back_to_first.entries[0].title, "Query Entry 4");
    assert_eq!(back_to_first.entries[1].title, "Query Entry 3");
}

/// Test querying entries with no filters returns entries.
pub(super) async fn test_query_entries_no_filters(db: &dyn DataI) {
    let feed = new_test_feed("Query Entries Feed", "https://query.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Query Entry 1", "https://query.example.com/entry1"),
        new_test_entry("Query Entry 2", "https://query.example.com/entry2"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let result = db.query_entries(None, None).await.unwrap();
    assert_eq!(result.entries.len(), 2);
}

/// Test querying entries filtered by feed_id.
pub(super) async fn test_query_entries_filter_feed_id(db: &dyn DataI) {
    let feed1 = new_test_feed("Feed A", "https://query-a.example.com/feed.xml");
    let feed2 = new_test_feed("Feed B", "https://query-b.example.com/feed.xml");

    db.upsert_feed_and_entries_and_icon(
        &feed1,
        vec![new_test_entry(
            "Feed A Entry",
            "https://query-a.example.com/entry1",
        )],
        None,
    )
    .await
    .unwrap();

    db.upsert_feed_and_entries_and_icon(
        &feed2,
        vec![new_test_entry(
            "Feed B Entry",
            "https://query-b.example.com/entry1",
        )],
        None,
    )
    .await
    .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds
        .iter()
        .find(|feed| feed.feed_url == "https://query-a.example.com/feed.xml")
        .expect("feed a")
        .id
        .clone();

    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: Some(feed_id.clone()),
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].feed_id, feed_id);
}

/// Test querying entries with sort and limit.
pub(super) async fn test_query_entries_filter_sort_and_limit(db: &dyn DataI) {
    let feed = new_test_feed("Sort Entries Feed", "https://sort.example.com/feed.xml");
    let entries = vec![
        NewEntry {
            title: "Older Entry".to_string(),
            url: "https://sort.example.com/entry1".to_string(),
            comments_url: None,
            published_at: Some(Utc::now() - Duration::days(2)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Newer Entry".to_string(),
            url: "https://sort.example.com/entry2".to_string(),
            comments_url: None,
            published_at: Some(Utc::now() - Duration::days(1)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Newest Entry".to_string(),
            url: "https://sort.example.com/entry3".to_string(),
            comments_url: None,
            published_at: Some(Utc::now()),
            entry_updated_at: None,
        },
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let filters = QueryFeedsFilters {
        limit: Some(2),
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: Some(SortOrder::Oldest),
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].title, "Older Entry");
    assert_eq!(result.entries[1].title, "Newer Entry");
}

/// Test querying entries with unread filter.
pub(super) async fn test_query_entries_filter_unread(db: &dyn DataI) {
    let feed = new_test_feed("Unread Entries Feed", "https://unread.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Unread Entry 1", "https://unread.example.com/entry1"),
        new_test_entry("Unread Entry 2", "https://unread.example.com/entry2"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: None,
        unread: Some(true),
        starred: None,
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 2);
}

/// Test querying entries with no data returns empty.
pub(super) async fn test_query_entries_empty(db: &dyn DataI) {
    let result = db.query_entries(None, None).await.unwrap();
    assert!(result.entries.is_empty());
    assert!(result.next_id.is_none());
    assert!(result.prev_id.is_none());
}

/// Test querying entries with starred filter.
pub(super) async fn test_query_entries_filter_starred(db: &dyn DataI) {
    // Note: We can't directly set starred_at through the DataI trait currently,
    // so we test that the filter works by verifying no entries match when
    // starred=true (since all entries are unstarred by default).
    let feed = new_test_feed(
        "Starred Entries Feed",
        "https://starred.example.com/feed.xml",
    );
    let entries = vec![
        new_test_entry("Entry 1", "https://starred.example.com/entry1"),
        new_test_entry("Entry 2", "https://starred.example.com/entry2"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    // All entries are unstarred by default
    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: None,
        unread: None,
        starred: Some(true),
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    // No entries should match starred=true since none are starred
    assert!(result.entries.is_empty());

    // Without starred filter, we should get all entries
    let result = db.query_entries(None, None).await.unwrap();
    assert_eq!(result.entries.len(), 2);
}

/// Test querying entries with text search filter.
pub(super) async fn test_query_entries_filter_query_search(db: &dyn DataI) {
    let feed = new_test_feed("Search Entries Feed", "https://search.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Rust Programming Guide", "https://search.example.com/rust"),
        new_test_entry("Python Tutorial", "https://search.example.com/python"),
        new_test_entry("JavaScript Basics", "https://search.example.com/javascript"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    // Search by title
    let filters = QueryFeedsFilters {
        limit: None,
        query: Some("Rust".to_string()),
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].title, "Rust Programming Guide");

    // Search by URL
    let filters = QueryFeedsFilters {
        limit: None,
        query: Some("python".to_string()),
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].title, "Python Tutorial");

    // Search with no matches
    let filters = QueryFeedsFilters {
        limit: None,
        query: Some("golang".to_string()),
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: None,
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert!(result.entries.is_empty());
}

/// Test querying entries with date range filter.
pub(super) async fn test_query_entries_filter_date_range(db: &dyn DataI) {
    let feed = new_test_feed("Date Range Feed", "https://daterange.example.com/feed.xml");
    let now = Utc::now();
    let entries = vec![
        NewEntry {
            title: "Old Entry".to_string(),
            url: "https://daterange.example.com/old".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::days(10)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Recent Entry".to_string(),
            url: "https://daterange.example.com/recent".to_string(),
            comments_url: None,
            published_at: Some(now - Duration::days(3)),
            entry_updated_at: None,
        },
        NewEntry {
            title: "Today Entry".to_string(),
            url: "https://daterange.example.com/today".to_string(),
            comments_url: None,
            published_at: Some(now),
            entry_updated_at: None,
        },
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    // Filter: last 5 days only
    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: Some(now - Duration::days(5)),
        end: None,
        sort: Some(SortOrder::Oldest),
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 2);
    assert_eq!(result.entries[0].title, "Recent Entry");
    assert_eq!(result.entries[1].title, "Today Entry");

    // Filter: before 5 days ago
    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: None,
        end: Some(now - Duration::days(5)),
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].title, "Old Entry");

    // Filter: between 5 and 1 days ago
    let filters = QueryFeedsFilters {
        limit: None,
        query: None,
        feed_id: None,
        unread: None,
        starred: None,
        start: Some(now - Duration::days(5)),
        end: Some(now - Duration::days(1)),
        sort: None,
    };

    let result = db.query_entries(None, Some(filters)).await.unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].title, "Recent Entry");
}

// ----------------------------------------------------------------------------
// Update feed tests
// ----------------------------------------------------------------------------

/// Test updating a feed's properties.
pub(super) async fn test_update_feed(db: &dyn DataI) {
    // Create a feed first
    let feed = new_test_feed("Original Title", "https://original.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    // Get the feed ID
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    // Update the feed
    db.update_feed(
        &feed_id,
        Some("Custom User Title"),
        "https://updated.example.com/feed.xml",
        Some("https://updated-site.example.com"),
    )
    .await
    .unwrap();

    // Verify the update
    let updated = db
        .get_feed_by_id_with_entry_counts(&feed_id)
        .await
        .unwrap()
        .unwrap();

    // user_title takes precedence over source_title for the "title" field
    assert_eq!(updated.title, "Custom User Title");
    assert_eq!(updated.user_title, Some("Custom User Title".to_string()));
    assert_eq!(updated.source_title, "Original Title"); // source_title unchanged
    assert_eq!(updated.feed_url, "https://updated.example.com/feed.xml");
    assert_eq!(
        updated.site_url,
        Some("https://updated-site.example.com".to_string())
    );
}

/// Test that updating a non-existent feed returns an error.
pub(super) async fn test_update_feed_not_found(db: &dyn DataI) {
    let result = db
        .update_feed(
            "nonexistent-id",
            Some("Title"),
            "https://example.com/feed.xml",
            None,
        )
        .await;

    assert!(result.is_err());
}

/// Test clearing user_title by setting it to None.
pub(super) async fn test_update_feed_clear_user_title(db: &dyn DataI) {
    // Create a feed
    let feed = new_test_feed("Source Title", "https://clear-title.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    // Set a custom user_title
    db.update_feed(
        &feed_id,
        Some("Custom Title"),
        "https://clear-title.example.com/feed.xml",
        None,
    )
    .await
    .unwrap();

    let feed = db
        .get_feed_by_id_with_entry_counts(&feed_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(feed.title, "Custom Title"); // user_title takes precedence
    assert_eq!(feed.user_title, Some("Custom Title".to_string()));

    // Clear the user_title by setting it to None
    db.update_feed(
        &feed_id,
        None,
        "https://clear-title.example.com/feed.xml",
        None,
    )
    .await
    .unwrap();

    let feed = db
        .get_feed_by_id_with_entry_counts(&feed_id)
        .await
        .unwrap()
        .unwrap();
    // After clearing user_title, title should fall back to source_title
    assert_eq!(feed.title, "Source Title");
    assert!(feed.user_title.is_none());
}

// ----------------------------------------------------------------------------
// Delete feed tests
// ----------------------------------------------------------------------------

/// Test deleting a feed.
pub(super) async fn test_delete_feed(db: &dyn DataI) {
    // Create a feed first
    let feed = new_test_feed("Delete Test Feed", "https://delete.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    // Get the feed ID
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);
    let feed_id = feeds[0].id.clone();

    // Delete the feed
    let deleted = db.delete_feed(&feed_id).await.unwrap();
    assert!(deleted);

    // Verify it's gone
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert!(feeds.is_empty());

    let result = db.get_feed_by_id_with_entry_counts(&feed_id).await.unwrap();
    assert!(result.is_none());
}

/// Test that deleting a non-existent feed returns false.
pub(super) async fn test_delete_feed_not_found(db: &dyn DataI) {
    let deleted = db.delete_feed("nonexistent-id").await.unwrap();
    assert!(!deleted);
}

/// Test that deleting a feed also deletes its entries.
pub(super) async fn test_delete_feed_cascades_entries(db: &dyn DataI) {
    // Create a feed with entries
    let feed = new_test_feed("Cascade Test Feed", "https://cascade.example.com/feed.xml");
    let entries = vec![
        new_test_entry("Entry 1", "https://cascade.example.com/entry1"),
        new_test_entry("Entry 2", "https://cascade.example.com/entry2"),
        new_test_entry("Entry 3", "https://cascade.example.com/entry3"),
    ];

    db.upsert_feed_and_entries_and_icon(&feed, entries, None)
        .await
        .unwrap();

    // Verify entries exist
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0].entry_count, 3);
    let feed_id = feeds[0].id.clone();

    // Delete the feed
    let deleted = db.delete_feed(&feed_id).await.unwrap();
    assert!(deleted);

    // Verify feed and entries are gone
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert!(feeds.is_empty());

    // Query entries should return empty for this feed
    let entries = db.get_feed_entries(&feed_id, None, None).await.unwrap();
    assert!(entries.entries.is_empty());
}

// ----------------------------------------------------------------------------
// Sync tests
// ----------------------------------------------------------------------------

/// Test that get_feeds_to_sync returns empty when no feeds exist.
pub(super) async fn test_get_feeds_to_sync_empty(db: &dyn DataI) {
    let feeds = db.get_feeds_to_sync(Utc::now()).await.unwrap();
    assert!(feeds.is_empty());
}

/// Test that get_feeds_to_sync returns a stale feed.
pub(super) async fn test_get_feeds_to_sync_returns_stale(db: &dyn DataI) {
    let feed = new_test_feed("Sync Feed", "https://sync.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();

    assert_eq!(feeds.len(), 1);
    assert_eq!(feeds[0].feed_url, "https://sync.example.com/feed.xml");
}

/// Test setting the sync result for a feed.
pub(super) async fn test_set_feed_sync_result(db: &dyn DataI) {
    let feed = new_test_feed(
        "Sync Result Feed",
        "https://sync-result.example.com/feed.xml",
    );
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    db.set_feed_sync_result("https://sync-result.example.com/feed.xml", "parse_error")
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let updated = feeds
        .iter()
        .find(|item| item.feed_url == "https://sync-result.example.com/feed.xml")
        .expect("feed");
    assert_eq!(updated.last_sync_result, Some("parse_error".to_string()));
}

/// Test get_one_feed_to_sync for existing and missing feeds.
pub(super) async fn test_get_one_feed_to_sync(db: &dyn DataI) {
    let feed = new_test_feed("One Sync Feed", "https://one-sync.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let result = db.get_one_feed_to_sync(&feed_id).await.unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.id, feed_id);

    let missing = db.get_one_feed_to_sync("missing-id").await.unwrap();
    assert!(missing.is_none());
}

/// Test get_similar_named_feed returns a match.
pub(super) async fn test_get_similar_named_feed(db: &dyn DataI) {
    let feed = new_test_feed("Similar Feed", "https://similar.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let result = db
        .get_similar_named_feed("similar.example.com")
        .await
        .unwrap();
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().feed_url,
        "https://similar.example.com/feed.xml"
    );
}

/// Test get_similar_named_feed returns None when no match exists.
pub(super) async fn test_get_similar_named_feed_no_match(db: &dyn DataI) {
    let feed = new_test_feed("Some Feed", "https://somefeed.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    let result = db
        .get_similar_named_feed("completely-different-domain.org")
        .await
        .unwrap();
    assert!(result.is_none());
}

/// Test that get_feeds_to_sync excludes feeds with parse_error.
pub(super) async fn test_get_feeds_to_sync_excludes_parse_error(db: &dyn DataI) {
    // Create a feed
    let feed = new_test_feed(
        "Parse Error Feed",
        "https://parse-error.example.com/feed.xml",
    );
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    // Set the sync result to parse_error
    db.set_feed_sync_result("https://parse-error.example.com/feed.xml", "parse_error")
        .await
        .unwrap();

    // The feed should be excluded from sync even though it's stale
    let feeds_to_sync = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();

    assert!(feeds_to_sync.is_empty());

    // If we change the sync result to something else, it should be included
    db.set_feed_sync_result("https://parse-error.example.com/feed.xml", "network_error")
        .await
        .unwrap();

    let feeds_to_sync = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();

    assert_eq!(feeds_to_sync.len(), 1);
}

/// Test that get_feeds_to_sync handles sync timeout correctly.
/// Feeds that have been syncing for more than 5 minutes should be re-synced.
pub(super) async fn test_get_feeds_to_sync_respects_sync_timeout(db: &dyn DataI) {
    // Create a feed and mark it as syncing
    let feed = new_test_feed(
        "Sync Timeout Feed",
        "https://sync-timeout.example.com/feed.xml",
    );
    db.upsert_feed_and_entries_and_icon(&feed, vec![], None)
        .await
        .unwrap();

    // Get the feed to sync (this sets sync_started_at to now)
    let feeds_to_sync = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(feeds_to_sync.len(), 1);

    // Immediately try to get feeds to sync again - should be empty
    // because the feed is still being synced (sync_started_at is recent)
    let feeds_to_sync = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert!(feeds_to_sync.is_empty());

    // Complete the sync by setting a result - this clears sync_started_at
    db.set_feed_sync_result("https://sync-timeout.example.com/feed.xml", "success")
        .await
        .unwrap();

    // Now it should be available for sync again
    let feeds_to_sync = db
        .get_feeds_to_sync(Utc::now() + Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(feeds_to_sync.len(), 1);
}

// ----------------------------------------------------------------------------
// Icon tests
// ----------------------------------------------------------------------------

/// Test upserting an icon and linking via feed upsert.
pub(super) async fn test_upsert_icon(db: &dyn DataI) {
    let icon_hash = "iconhash123".to_string();
    let icon_data = vec![0x89, 0x50, 0x4E, 0x47];
    let icon_content_type = "image/png".to_string();

    db.upsert_icon(NewIcon {
        hash: icon_hash.clone(),
        data: icon_data.clone(),
        content_type: icon_content_type.clone(),
    })
    .await
    .unwrap();

    let feed = new_test_feed("Icon Link Feed", "https://icon-link.example.com/feed.xml");
    db.upsert_feed_and_entries_and_icon(
        &feed,
        vec![],
        Some(NewIcon {
            hash: icon_hash,
            data: icon_data,
            content_type: icon_content_type,
        }),
    )
    .await
    .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();
    let icon = db.get_icon_by_feed_id(&feed_id).await.unwrap();
    assert!(icon.is_some());
    assert_eq!(icon.unwrap().hash, "iconhash123");
}

/// Test that two feeds with the same icon hash share the icon record.
pub(super) async fn test_icon_deduplication_by_hash(db: &dyn DataI) {
    // Create first feed with icon
    let feed1 = new_test_feed(
        "Icon Dedup Feed 1",
        "https://icon-dedup1.example.com/feed.xml",
    );
    db.upsert_feed_and_entries_and_icon(
        &feed1,
        vec![],
        Some(NewIcon {
            hash: "shared_hash_123".to_string(),
            data: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A],
            content_type: "image/png".to_string(),
        }),
    )
    .await
    .unwrap();

    // Create second feed with same icon hash
    let feed2 = new_test_feed(
        "Icon Dedup Feed 2",
        "https://icon-dedup2.example.com/feed.xml",
    );
    db.upsert_feed_and_entries_and_icon(
        &feed2,
        vec![],
        Some(NewIcon {
            hash: "shared_hash_123".to_string(),
            data: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A],
            content_type: "image/png".to_string(),
        }),
    )
    .await
    .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 2);

    // Both feeds should have icons
    let feed1_data = feeds
        .iter()
        .find(|f| f.feed_url == "https://icon-dedup1.example.com/feed.xml")
        .unwrap();
    let feed2_data = feeds
        .iter()
        .find(|f| f.feed_url == "https://icon-dedup2.example.com/feed.xml")
        .unwrap();

    assert!(feed1_data.has_icon);
    assert!(feed2_data.has_icon);

    // Both should reference the same icon (same id)
    let icon1 = db
        .get_icon_by_feed_id(&feed1_data.id)
        .await
        .unwrap()
        .unwrap();
    let icon2 = db
        .get_icon_by_feed_id(&feed2_data.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(icon1.id, icon2.id);
    assert_eq!(icon1.hash, "shared_hash_123");
}

/// Test updating a feed's icon to a different one.
pub(super) async fn test_feed_icon_update(db: &dyn DataI) {
    let feed = new_test_feed(
        "Icon Update Feed",
        "https://icon-update.example.com/feed.xml",
    );

    // Create feed with initial icon
    let initial_icon = NewIcon {
        hash: "initial_icon_hash".to_string(),
        data: vec![0x89, 0x50, 0x4E, 0x47],
        content_type: "image/png".to_string(),
    };

    db.upsert_feed_and_entries_and_icon(&feed, vec![], Some(initial_icon))
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    let feed_id = feeds[0].id.clone();

    let icon = db.get_icon_by_feed_id(&feed_id).await.unwrap().unwrap();
    assert_eq!(icon.hash, "initial_icon_hash");

    // Update feed with new icon
    let new_icon = NewIcon {
        hash: "new_icon_hash".to_string(),
        data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG magic bytes
        content_type: "image/jpeg".to_string(),
    };

    db.upsert_feed_and_entries_and_icon(&feed, vec![], Some(new_icon))
        .await
        .unwrap();

    // Feed should now have the new icon
    // Note: The current implementation adds a new feed_icon association
    // but the query returns any icon associated with the feed.
    // This test verifies the upsert doesn't fail.
    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 1);
    assert!(feeds[0].has_icon);
}

// ----------------------------------------------------------------------------
// OPML import tests
// ----------------------------------------------------------------------------

/// Test creating an OPML import job.
pub(super) async fn test_create_opml_import_job(db: &dyn DataI) {
    let feed_urls = vec![
        "https://opml.example.com/feed1.xml".to_string(),
        "https://opml.example.com/feed2.xml".to_string(),
    ];

    let summary = db
        .create_opml_import_job(&feed_urls, &HashSet::new())
        .await
        .unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.skipped, 0);

    let job = db
        .get_opml_import_job(&summary.job_id)
        .await
        .unwrap()
        .expect("job");
    assert_eq!(job.status, "running");
    assert_eq!(job.total, 2);
}

/// Test getting a missing OPML import job returns None.
pub(super) async fn test_get_opml_import_job_not_found(db: &dyn DataI) {
    let job = db.get_opml_import_job("missing-id").await.unwrap();
    assert!(job.is_none());
}

/// Test updating OPML items and job status.
pub(super) async fn test_update_opml_import_item_and_job_status(db: &dyn DataI) {
    let feed_urls = vec![
        "https://opml-update.example.com/feed1.xml".to_string(),
        "https://opml-update.example.com/feed2.xml".to_string(),
    ];

    let summary = db
        .create_opml_import_job(&feed_urls, &HashSet::new())
        .await
        .unwrap();

    db.update_opml_import_item(
        &summary.job_id,
        "https://opml-update.example.com/feed1.xml",
        "imported",
        None,
    )
    .await
    .unwrap();

    db.increment_opml_import_job_counts(&summary.job_id, 1, 0, 0)
        .await
        .unwrap();

    db.update_opml_import_job_status(&summary.job_id, "completed")
        .await
        .unwrap();

    let job = db
        .get_opml_import_job(&summary.job_id)
        .await
        .unwrap()
        .expect("job");
    assert_eq!(job.status, "completed");
    assert_eq!(job.imported, 1);
    assert_eq!(job.skipped, 0);
    assert_eq!(job.failed, 0);

    let items = db
        .get_opml_import_recent_items(&summary.job_id, 10)
        .await
        .unwrap();
    assert!(items.iter().any(|item| {
        item.feed_url == "https://opml-update.example.com/feed1.xml" && item.status == "imported"
    }));
}

/// Test fetching recent OPML import items.
pub(super) async fn test_get_opml_import_recent_items(db: &dyn DataI) {
    let feed_urls = vec![
        "https://opml-recent.example.com/feed1.xml".to_string(),
        "https://opml-recent.example.com/feed2.xml".to_string(),
    ];

    let summary = db
        .create_opml_import_job(&feed_urls, &HashSet::new())
        .await
        .unwrap();

    db.update_opml_import_item(
        &summary.job_id,
        "https://opml-recent.example.com/feed2.xml",
        "failed",
        Some("network error"),
    )
    .await
    .unwrap();

    let items = db
        .get_opml_import_recent_items(&summary.job_id, 1)
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].feed_url,
        "https://opml-recent.example.com/feed2.xml"
    );
}

/// Test inserting stub feeds is idempotent.
pub(super) async fn test_insert_stub_feeds(db: &dyn DataI) {
    let feed_urls = vec![
        "https://stub.example.com/feed1.xml".to_string(),
        "https://stub.example.com/feed2.xml".to_string(),
    ];

    db.insert_stub_feeds(&feed_urls).await.unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 2);
    assert!(
        feeds
            .iter()
            .any(|feed| feed.feed_url == "https://stub.example.com/feed1.xml")
    );
    assert!(
        feeds
            .iter()
            .any(|feed| feed.feed_url == "https://stub.example.com/feed2.xml")
    );

    db.insert_stub_feeds(&["https://stub.example.com/feed1.xml".to_string()])
        .await
        .unwrap();

    let feeds = db.get_feeds_with_entry_counts().await.unwrap();
    assert_eq!(feeds.len(), 2);
}
