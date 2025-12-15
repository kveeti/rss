use chrono::{DateTime, Utc};

use crate::db::Data;

impl Data {
    pub async fn get_feeds_to_sync(
        &self,
        last_synced_before: DateTime<Utc>,
        limit: i64,
    ) -> anyhow::Result<Vec<FeedToSync>> {
        let feeds = sqlx::query_as!(
            FeedToSync,
            r#"
            update feeds f
            set sync_started_at = now()
            where id in (
                select id
                from feeds f
                where f.sync_started_at is null
                    and (f.last_synced_at < $1 or f.last_synced_at is null)
                order by f.last_synced_at desc nulls first
                limit $2
                for update skip locked
            )
            returning f.id, f.feed_url, f.site_url
            "#,
            last_synced_before,
            limit
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(feeds)
    }

    pub async fn get_syncing_feeds_count(&self) -> Result<i64, sqlx::Error> {
        Ok(sqlx::query_scalar!(
            r#"
            select count(*)
            from feeds f
            where f.sync_started_at is not null
            "#
        )
        .fetch_one(&self.pg_pool)
        .await?
        .expect("syncing feeds count"))
    }

    pub async fn get_similar_named_feed(
        &self,
        feed_url: &str,
    ) -> Result<Option<FeedToSync>, sqlx::Error> {
        let feed_url = format!("%{}%", feed_url);

        let feed = sqlx::query_as!(
            FeedToSync,
            r#"
            select f.id, f.feed_url, f.site_url
            from feeds f
            where f.feed_url like $1
            limit 1
            "#,
            feed_url
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(feed)
    }
}

pub struct FeedToSync {
    pub id: String,
    pub feed_url: String,
    pub site_url: Option<String>,
}
