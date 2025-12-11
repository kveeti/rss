use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, Row, query, query_as};

use crate::{Data, create_id, icons::NewIcon};

#[derive(Debug, thiserror::Error)]
pub enum InsertFeedError {
    #[error("duplicate feed")]
    DuplicateFeed,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl Data {
    pub async fn add_feed_and_entries_and_icon(
        &self,
        feed: NewFeed,
        entries: Vec<NewEntry>,
        icon: Option<NewIcon>,
    ) -> Result<(), InsertFeedError> {
        let mut tx = self
            .pg_pool
            .begin()
            .await
            .context("error starting transaction")?;

        let feed_id = create_id();

        let res = query!(
            r#"
            insert into feeds (id, title, feed_url, site_url) values ($1, $2, $3, $4)
            "#,
            feed_id,
            feed.title,
            feed.feed_url,
            feed.site_url
        )
        .execute(&mut *tx)
        .await;

        match res {
            Ok(_) => {}
            Err(e) => {
                if let Some(e) = e.as_database_error() {
                    if e.code() == Some("23505".into()) {
                        return Err(InsertFeedError::DuplicateFeed);
                    }
                } else {
                    return Err(InsertFeedError::UnexpectedError(e.into()));
                }
            }
        }

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

        if let Some(icon) = icon {
            let icon_id = create_id();
            query!(
                r#"
                insert into icons (id, hash, data, content_type) values ($1, $2, $3, $4)
                on conflict (hash) do nothing
                "#,
                icon_id,
                icon.hash,
                icon.data,
                icon.content_type
            )
            .execute(&mut *tx)
            .await
            .context("error inserting icon")?;

            query!(
                r#"
                insert into feeds_icons (feed_id, icon_id) values ($1, $2)
                "#,
                feed_id,
                icon_id
            )
            .execute(&mut *tx)
            .await
            .context("error inserting feed icon")?;
        }

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
            r#"select id, title, feed_url, site_url, created_at, updated_at from feeds where feed_url = $1"#,
            url
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(feed)
    }

    pub async fn get_feed_by_id_with_entry_counts(
        &self,
        id: &str,
    ) -> Result<Option<FeedWithEntryCounts>, sqlx::Error> {
        let feed = query_as!(
            FeedWithEntryCounts,
            r#"select
                f.id,
                f.title,
                f.feed_url,
                f.site_url,
                f.created_at,
                count(e.id) as "entry_count!",
                count(e.id) filter (where e.read_at is null) as "unread_entry_count!"
            from feeds f
            left join entries e on e.feed_id = f.id
            where f.id = $1
            group by f.id
            order by f.created_at desc
            "#,
            id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(feed)
    }

    pub async fn get_feeds_with_entry_counts(
        &self,
    ) -> Result<Vec<FeedWithEntryCounts>, sqlx::Error> {
        let rows = query_as!(
            FeedWithEntryCounts,
            r#"
            select 
                f.id,
                f.title,
                f.feed_url,
                f.site_url,
                f.created_at,
                count(e.id) as "entry_count!",
                count(e.id) filter (where e.read_at is null) as "unread_entry_count!"
            from feeds f
            left join entries e on e.feed_id = f.id
            group by f.id
            order by f.created_at desc
            "#
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_feed_entries(
        &self,
        feed_id: &str,
        cursor: Option<Cursor>,
        limit: Option<i64>,
    ) -> Result<CursorOutput<EntryForList>, sqlx::Error> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            select 
                e.id,
                e.feed_id,
                e.title,
                e.url,
                e.comments_url,
                e.published_at,
                e.read_at,
                e.starred_at,
                e.created_at,
                e.updated_at
            from entries e
            "#,
        );

        query.push("where e.feed_id = ").push_bind(feed_id);

        let order = match cursor {
            Some(Cursor::Left(ref id)) => {
                query
                    .push(" and (")
                    .push("( e.published_at = ( select published_at from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id > ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or e.published_at > ( select published_at from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                "asc"
            }
            Some(Cursor::Right(ref id)) => {
                query
                    .push(" and (")
                    .push("( e.published_at = ( select published_at from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id < ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or e.published_at < ( select published_at from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                "desc"
            }
            None => "desc",
        };

        query
            .push(" order by e.published_at ")
            .push(order)
            .push(", e.id ")
            .push(order);

        let limit = limit.unwrap_or(20) + 1;
        query.push(" limit ").push(limit);

        let rows = query.build().fetch_all(&self.pg_pool).await?;

        let mut entries: Vec<EntryForList> = rows
            .into_iter()
            .map(|row| EntryForList {
                id: row.get_unchecked("id"),
                title: row.get_unchecked("title"),
                url: row.get_unchecked("url"),
                comments_url: row.get_unchecked("comments_url"),
                read_at: row.get_unchecked("read_at"),
                starred_at: row.get_unchecked("starred_at"),
                published_at: row.get_unchecked("published_at"),
            })
            .collect();

        let has_more = entries.len() == limit as usize;
        if has_more {
            entries.pop();
        }

        match cursor {
            Some(Cursor::Left(_)) => entries.reverse(),
            _ => {}
        }

        let (next_id, prev_id) = if let [first, _second, ..] = &entries[..] {
            let first_id = first.id.to_owned();
            let last_id = entries.last().expect("last").id.to_owned();

            let (next_id, prev_id) = match (has_more, cursor) {
                (true, None) => (Some(last_id), None),
                (false, None) => (None, None),
                (true, Some(_)) => (Some(last_id), Some(first_id)),
                (false, Some(Cursor::Left(_))) => (Some(last_id), None),
                (false, Some(Cursor::Right(_))) => (None, Some(first_id)),
            };
            (next_id, prev_id)
        } else {
            (None, None)
        };

        Ok(CursorOutput {
            entries,
            next_id,
            prev_id,
        })
    }
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
    pub site_url: Option<String>,
    pub feed_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct FeedWithEntryCounts {
    pub id: String,
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub entry_count: i64,
    pub unread_entry_count: i64,
}
