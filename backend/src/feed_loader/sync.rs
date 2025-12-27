use std::time::Duration;

use chrono::Utc;
use futures::{StreamExt, stream};

use crate::{
    db::Data,
    feed_loader::{FeedResult, load_feed},
};

static MAX_SYNCING_FEEDS: usize = 10;

pub async fn feed_sync_loop(data: Data) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));

    loop {
        ticker.tick().await;

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
                    sync_feed(&data, feed.feed_url).await;
                }
            })
            .await;
    }
}

#[tracing::instrument(name = "sync_feed", skip(data))]
async fn sync_feed(data: &Data, url: String) {
    let result = load_feed(&url).await;

    match result {
        Ok(FeedResult::Loaded(loaded_feed)) => {
            let _ = data
                .upsert_feed_and_entries_and_icon(
                    &loaded_feed.feed,
                    loaded_feed.entries,
                    loaded_feed.icon,
                )
                .await
                .map_err(|e| tracing::error!("error upserting feed: {e:#}"));

            tracing::info!("feed synced");
        }
        Ok(result) => tracing::warn!("unexpected result syncing feed: {result:?}"),
        Err(err) => tracing::error!("error syncing feed: {err:?}"),
    };
}
