use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, query, query_as};

use crate::{Data, create_id};

impl Data {
    pub async fn add_feed_and_entries(
        &self,
        feed: NewFeed,
        entries: Vec<NewEntry>,
    ) -> anyhow::Result<()> {
        let mut tx = self
            .pg_pool
            .begin()
            .await
            .context("error starting transaction")?;

        let feed_id = create_id();

        query!(
            r#"
            insert into feeds (id, title, url) values ($1, $2, $3)
            "#,
            feed_id,
            feed.title,
            feed.url
        )
        .execute(&mut *tx)
        .await
        .context("error inserting feed")?;

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into entries (id, feed_id, title, url, comments_url, published_at)",
        );

        builder.push_values(entries, |mut b, entry| {
            b.push_bind(create_id());
            b.push_bind(&feed_id);
            b.push_bind(entry.title);
            b.push_bind(entry.url);
            b.push_bind(entry.comments_url);
            b.push_bind(entry.published_at);
        });

        builder
            .build()
            .execute(&mut *tx)
            .await
            .context("error inserting entries")?;

        tx.commit().await.context("error committing transaction")?;

        Ok(())
    }

    pub async fn upsert_entries(
        &self,
        feed_id: &str,
        entries: Vec<NewEntry>,
    ) -> Result<(), sqlx::Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into entries (id, feed_id, title, url, comments_url, published_at)",
        );

        builder.push_values(entries, |mut b, entry| {
            b.push_bind(create_id());
            b.push_bind(feed_id);
            b.push_bind(entry.title);
            b.push_bind(entry.url);
            b.push_bind(entry.comments_url);
            b.push_bind(entry.published_at);
        });

        builder.build().execute(&self.pg_pool).await?;

        Ok(())
    }

    pub async fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>, sqlx::Error> {
        let feed = query_as!(
            Feed,
            r#"select id, title, url, created_at, updated_at from feeds where url = $1"#,
            url
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(feed)
    }
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
pub struct Feed {
    pub id: String,
    pub title: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct NewEntry {
    pub title: String,
    pub url: String,
    pub comments_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize)]
pub struct NewFeed {
    pub title: String,
    pub url: String,
}
