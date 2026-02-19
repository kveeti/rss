use std::time::Duration;

use chrono::Utc;
use futures::{StreamExt, stream};
use tokio::sync::watch;

use crate::{
    db::Data,
    feed_loader::{
        FeedResult, SYNC_RESULT_DB_ERROR, SYNC_RESULT_NOT_MODIFIED, load_feed,
        sync_result_for_error, sync_result_for_feed_result,
    },
};

static MAX_SYNCING_FEEDS: usize = 10;

pub async fn feed_sync_loop(
    data: Data,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = shutdown_rx.wait_for(|&v| v) => {
                tracing::info!("feed sync loop shutting down");
                return Ok(());
            }
        }

        let feeds = data
            .get_feeds_to_sync(Utc::now() - chrono::Duration::hours(1))
            .await?;

        if feeds.len() == 0 {
            tracing::info!("no feeds to sync");
            continue;
        }

        tracing::info!("syncing {} feeds", feeds.len());

        stream::iter(feeds)
            .for_each_concurrent(MAX_SYNCING_FEEDS, |feed| {
                let data = data.clone();
                async move {
                    sync_feed(
                        &data,
                        feed.feed_url,
                        feed.http_etag,
                        feed.http_last_modified,
                    )
                    .await;
                }
            })
            .await;
    }
}

#[tracing::instrument(name = "sync_feed", skip(data, etag, last_modified))]
async fn sync_feed(data: &Data, url: String, etag: Option<String>, last_modified: Option<String>) {
    let result = load_feed(&url, etag, last_modified).await;

    match result {
        Ok(FeedResult::Loaded(loaded_feed)) => {
            // Store the new cache headers
            if let Err(e) = data
                .update_feed_headers(
                    &url,
                    loaded_feed.http_etag.as_deref(),
                    loaded_feed.http_last_modified.as_deref(),
                )
                .await
            {
                tracing::error!("error updating feed headers: {e:#}");
            }

            let upsert_result = data
                .upsert_feed_and_entries_and_icon(
                    &loaded_feed.feed,
                    loaded_feed.entries,
                    loaded_feed.icon,
                )
                .await
                .map_err(|e| tracing::error!("error upserting feed: {e:#}"));

            if upsert_result.is_err() {
                set_sync_result(data, &url, SYNC_RESULT_DB_ERROR).await;
            }

            tracing::info!("feed synced");
        }
        Ok(FeedResult::NotModified) => {
            tracing::info!("feed not modified, skipping");
            set_sync_result(data, &url, SYNC_RESULT_NOT_MODIFIED).await;
        }
        Ok(result) => {
            match result {
                FeedResult::Loaded(_) | FeedResult::NotModified => {}
                _ => tracing::warn!("unexpected result syncing feed: {result:?}"),
            }
            set_sync_result(data, &url, sync_result_for_feed_result(&result)).await;
        }
        Err(err) => {
            tracing::error!("error syncing feed: {err:?}");
            set_sync_result(data, &url, sync_result_for_error(&err)).await;
        }
    };
}

async fn set_sync_result(data: &Data, url: &str, result: &str) {
    let _ = data
        .set_feed_sync_result(url, result)
        .await
        .map_err(|e| tracing::error!("error updating sync result: {e:#}"));
}
