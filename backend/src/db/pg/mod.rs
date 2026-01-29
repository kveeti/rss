use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Row, migrate, query, query_as};
use std::{collections::HashSet, sync::Arc};
use tracing::info;

use super::{
    Cursor, CursorOutput, Data, DataI, EntryForList, EntryForQueryList, FeedToSync,
    FeedWithEntryCounts, Icon, NewEntry, NewFeed, NewIcon, OpmlImportItem, OpmlImportJob,
    OpmlImportJobSummary, QueryFeedsFilters, SortOrder, create_id,
};

#[derive(Clone)]
pub(super) struct PgData {
    pg_pool: PgPool,
}

pub(super) async fn new_pg_data(database_url: &str) -> Result<Data> {
    info!("connecting to pg...");

    let pg = PgPool::connect(database_url)
        .await
        .context("error connecting to postgres")?;

    info!("connected to pg, running migrations...");

    migrate!("./src/db/pg/migrations")
        .run(&pg)
        .await
        .context("error running migrations")?;

    info!("migrations completed");

    Ok(Arc::new(PgData { pg_pool: pg }))
}

#[async_trait]
impl DataI for PgData {
    async fn upsert_feed_and_entries_and_icon(
        &self,
        feed: &NewFeed,
        entries: Vec<NewEntry>,
        icon: Option<NewIcon>,
    ) -> Result<(), anyhow::Error> {
        let mut seen = HashSet::new();
        let unique_entries: Vec<_> = entries
            .iter()
            .filter(|entry| seen.insert(entry.url.clone()))
            .cloned()
            .collect();

        let mut tx = self
            .pg_pool
            .begin()
            .await
            .context("error starting transaction")?;

        let feed_id = query!(
            r#"
            insert into feeds (
                id,
                source_title,
                feed_url,
                site_url,
                last_synced_at,
                last_sync_result,
                sync_started_at
            ) values ($1, $2, $3, $4, now(), 'success', NULL)
            on conflict (feed_url) do update set
                source_title = $2,
                site_url = $4,
                updated_at = now(),
                sync_started_at = NULL,
                last_synced_at = now(),
                last_sync_result = 'success'
            returning id
            "#,
            create_id(),
            feed.title,
            feed.feed_url,
            feed.site_url
        )
        .fetch_one(&mut *tx)
        .await
        .context("error upserting feed")?
        .id;

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into entries (id, feed_id, title, url, comments_url, published_at, entry_updated_at)",
        );

        builder.push_values(unique_entries, |mut b, entry| {
            b.push_bind(create_id());
            b.push_bind(&feed_id);
            b.push_bind(entry.title);
            b.push_bind(entry.url);
            b.push_bind(entry.comments_url);
            b.push_bind(entry.published_at);
            b.push_bind(entry.entry_updated_at);
        });

        builder.push(
            r#"
            on conflict (feed_id, url) do update set
                title = excluded.title,
                url = excluded.url,
                comments_url = excluded.comments_url,
                published_at = excluded.published_at,
                entry_updated_at = excluded.entry_updated_at
            "#,
        );

        builder
            .build()
            .execute(&mut *tx)
            .await
            .context("error inserting entries")?;

        if let Some(icon) = icon {
            let icon_id = create_id();

            query!(
                r#"
                with icon as (
                    insert into icons (id, hash, data, content_type) values ($1, $2, $3, $4)
                    on conflict (hash) do update
                        set hash = excluded.hash
                    returning id
                )
                insert into feeds_icons (feed_id, icon_id)
                select $5, id from icon
                on conflict (feed_id, icon_id) do nothing
                "#,
                icon_id,
                icon.hash,
                icon.data,
                icon.content_type,
                feed_id
            )
            .execute(&mut *tx)
            .await
            .context("error upserting icon and feeds_icons")?;
        }

        tx.commit().await.context("error committing transaction")?;

        Ok(())
    }

    async fn upsert_entries(
        &self,
        feed_id: &str,
        entries: Vec<NewEntry>,
    ) -> Result<(), sqlx::Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into entries (id, feed_id, title, url, comments_url, published_at, entry_updated_at)",
        );

        builder.push_values(entries, |mut b, entry| {
            b.push_bind(create_id());
            b.push_bind(feed_id);
            b.push_bind(entry.title);
            b.push_bind(entry.url);
            b.push_bind(entry.comments_url);
            b.push_bind(entry.published_at);
            b.push_bind(entry.entry_updated_at);
        });

        builder.build().execute(&self.pg_pool).await?;

        Ok(())
    }

    async fn get_feed_by_id_with_entry_counts(
        &self,
        id: &str,
    ) -> Result<Option<FeedWithEntryCounts>, sqlx::Error> {
        let feed = query_as!(
            FeedWithEntryCounts,
            r#"select
                f.id,
                coalesce(f.user_title, f.source_title) as "title!",
                f.source_title as "source_title!",
                f.user_title,
                f.feed_url,
                f.site_url,
                f.created_at,
                f.last_synced_at,
                f.last_sync_result,
                count(e.id) as "entry_count!",
                count(e.id) filter (where e.read_at is null) as "unread_entry_count!",
                exists (
                    select 1
                    from feeds_icons fi
                    where fi.feed_id = f.id
                ) as "has_icon!"
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

    async fn get_feeds_with_entry_counts(&self) -> Result<Vec<FeedWithEntryCounts>, sqlx::Error> {
        let rows = query_as!(
            FeedWithEntryCounts,
            r#"
            select
                f.id,
                coalesce(f.user_title, f.source_title) as "title!",
                f.source_title as "source_title!",
                f.user_title,
                f.feed_url,
                f.site_url,
                f.created_at,
                f.last_synced_at,
                f.last_sync_result,
                count(e.id) as "entry_count!",
                count(e.id) filter (where e.read_at is null) as "unread_entry_count!",
                exists (
                    select 1
                    from feeds_icons fi
                    where fi.feed_id = f.id
                ) as "has_icon!"
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

    async fn get_feed_entries(
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
                e.entry_updated_at,
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
                    .push("( coalesce(e.entry_updated_at, e.published_at, e.created_at) = ( select coalesce(entry_updated_at, published_at, created_at) from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id > ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or coalesce(e.entry_updated_at, e.published_at, e.created_at) > ( select coalesce(entry_updated_at, published_at, created_at) from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                "asc"
            }
            Some(Cursor::Right(ref id)) => {
                query
                    .push(" and (")
                    .push("( coalesce(e.entry_updated_at, e.published_at, e.created_at) = ( select coalesce(entry_updated_at, published_at, created_at) from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id < ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or coalesce(e.entry_updated_at, e.published_at, e.created_at) < ( select coalesce(entry_updated_at, published_at, created_at) from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                "desc"
            }
            None => "desc",
        };

        query
            .push(" order by coalesce(e.entry_updated_at, e.published_at, e.created_at) ")
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
                entry_updated_at: row.get_unchecked("entry_updated_at"),
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

    async fn query_entries(
        &self,
        cursor: Option<Cursor>,
        filters: Option<QueryFeedsFilters>,
    ) -> Result<CursorOutput<EntryForQueryList>, sqlx::Error> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            select
                e.id,
                e.feed_id,
                e.title,
                e.url,
                e.comments_url,
                e.published_at,
                e.entry_updated_at,
                e.read_at,
                e.starred_at,
                e.created_at,
                e.updated_at,
                exists (
                    select 1
                    from feeds_icons fi
                    where fi.feed_id = e.feed_id
                ) as "has_icon"
            from entries e
            where 1=1
            "#,
        );

        let (limit, sort_order) = if let Some(ref filters) = filters {
            if let Some(ref feed_id) = filters.feed_id {
                query.push(" and e.feed_id = ").push_bind(feed_id);
            }

            if let Some(ref search_query) = filters.query {
                query
                    .push(" and (e.title ilike ")
                    .push_bind(format!("%{}%", search_query))
                    .push(" or e.url ilike ")
                    .push_bind(format!("%{}%", search_query))
                    .push(")");
            }

            if filters.unread == Some(true) {
                query.push(" and e.read_at is null");
            }

            if filters.starred == Some(true) {
                query.push(" and e.starred_at is not null");
            }

            if let Some(ref start) = filters.start {
                query
                    .push(" and coalesce(e.published_at, e.entry_updated_at, e.created_at) >= ")
                    .push_bind(*start);
            }

            if let Some(ref end) = filters.end {
                query
                    .push(" and coalesce(e.published_at, e.entry_updated_at, e.created_at) <= ")
                    .push_bind(*end);
            }

            (filters.limit, filters.sort.unwrap_or_default())
        } else {
            (None, SortOrder::default())
        };

        let base_order = match sort_order {
            SortOrder::Newest => "desc",
            SortOrder::Oldest => "asc",
        };

        let (gt, lt) = match sort_order {
            SortOrder::Newest => ("<", ">"),
            SortOrder::Oldest => (">", "<"),
        };

        let order = match cursor {
            Some(Cursor::Left(ref id)) => {
                query
                    .push(" and (")
                    .push("( coalesce(e.published_at, e.entry_updated_at, e.created_at) = ( select coalesce(published_at, entry_updated_at, created_at) from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id ")
                    .push(lt)
                    .push(" ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or coalesce(e.published_at, e.entry_updated_at, e.created_at) ")
                    .push(lt)
                    .push(" ( select coalesce(published_at, entry_updated_at, created_at) from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                if base_order == "desc" { "asc" } else { "desc" }
            }
            Some(Cursor::Right(ref id)) => {
                query
                    .push(" and (")
                    .push("( coalesce(e.published_at, e.entry_updated_at, e.created_at) = ( select coalesce(published_at, entry_updated_at, created_at) from entries where id = ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" and e.id ")
                    .push(gt)
                    .push(" ")
                    .push_bind(id.to_owned())
                    .push(")")
                    .push(" or coalesce(e.published_at, e.entry_updated_at, e.created_at) ")
                    .push(gt)
                    .push(" ( select coalesce(published_at, entry_updated_at, created_at) from entries where id = ")
                    .push_bind(id)
                    .push(")")
                    .push(")");

                base_order
            }
            None => base_order,
        };

        query
            .push(" order by coalesce(e.published_at, e.entry_updated_at, e.created_at) ")
            .push(order)
            .push(", e.id ")
            .push(order);

        let limit = limit.unwrap_or(20) + 1;
        query.push(" limit ").push(limit);

        let rows = query.build().fetch_all(&self.pg_pool).await?;

        let mut entries: Vec<EntryForQueryList> = rows
            .into_iter()
            .map(|row| EntryForQueryList {
                id: row.get_unchecked("id"),
                feed_id: row.get_unchecked("feed_id"),
                title: row.get_unchecked("title"),
                url: row.get_unchecked("url"),
                comments_url: row.get_unchecked("comments_url"),
                read_at: row.get_unchecked("read_at"),
                starred_at: row.get_unchecked("starred_at"),
                published_at: row.get_unchecked("published_at"),
                entry_updated_at: row.get_unchecked("entry_updated_at"),
                has_icon: row.get_unchecked("has_icon"),
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

    async fn get_existing_feed_urls(
        &self,
        feed_urls: &[String],
    ) -> Result<HashSet<String>, sqlx::Error> {
        if feed_urls.is_empty() {
            return Ok(HashSet::new());
        }

        let rows = sqlx::query!(
            r#"
            select feed_url
            from feeds
            where feed_url = any($1)
            "#,
            feed_urls
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.feed_url).collect())
    }

    async fn get_feeds_to_sync(
        &self,
        last_synced_before: DateTime<Utc>,
    ) -> anyhow::Result<Vec<FeedToSync>> {
        let feeds = sqlx::query_as!(
            FeedToSync,
            r#"
            update feeds f
            set sync_started_at = now()
            where id in (
                select id
                from feeds f
                where f.last_sync_result is distinct from 'parse_error'
                and (
                    (f.sync_started_at is null and (f.last_synced_at < $1 or f.last_synced_at is null))
                    or f.sync_started_at < now() - interval '5 minutes'
                )
                order by f.last_synced_at desc nulls first
                for update skip locked
            )
            returning f.id, f.feed_url, f.site_url
            "#,
            last_synced_before
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(feeds)
    }

    async fn set_feed_sync_result(&self, feed_url: &str, result: &str) -> Result<(), sqlx::Error> {
        query!(
            r#"
            update feeds
            set last_sync_result = $2,
                sync_started_at = null,
                updated_at = now()
            where feed_url = $1
            "#,
            feed_url,
            result
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn get_one_feed_to_sync(&self, feed_id: &str) -> Result<Option<FeedToSync>, sqlx::Error> {
        let feed = sqlx::query_as!(
            FeedToSync,
            r#"
            update feeds f
            set sync_started_at = now()
            where id in (
                select id
                from feeds f
                where id = $1
                for update skip locked
            )
            returning f.id, f.feed_url, f.site_url
            "#,
            feed_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(feed)
    }

    async fn get_similar_named_feed(
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

    async fn update_feed(
        &self,
        feed_id: &str,
        user_title: Option<&str>,
        feed_url: &str,
        site_url: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let updated = query!(
            r#"
            update feeds
            set user_title = $2,
                feed_url = $3,
                site_url = $4,
                updated_at = now()
            where id = $1
            returning id
            "#,
            feed_id,
            user_title,
            feed_url,
            site_url
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        if updated.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        Ok(())
    }

    async fn delete_feed(&self, feed_id: &str) -> Result<bool, anyhow::Error> {
        let mut tx = self
            .pg_pool
            .begin()
            .await
            .context("error starting transaction")?;

        query!(
            r#"
            delete from entries
            where feed_id = $1
            "#,
            feed_id
        )
        .execute(&mut *tx)
        .await
        .context("error deleting entries")?;

        query!(
            r#"
            delete from feeds_icons
            where feed_id = $1
            "#,
            feed_id
        )
        .execute(&mut *tx)
        .await
        .context("error deleting feeds_icons")?;

        let deleted = query!(
            r#"
            delete from feeds
            where id = $1
            returning id
            "#,
            feed_id
        )
        .fetch_optional(&mut *tx)
        .await
        .context("error deleting feed")?;

        tx.commit().await.context("error committing transaction")?;

        Ok(deleted.is_some())
    }

    async fn upsert_icon(&self, icon: NewIcon) -> Result<(), sqlx::Error> {
        let id = create_id();
        query!(
            r#"
            insert into icons (id, hash, data, content_type) values ($1, $2, $3, $4)
            on conflict (hash) do nothing
            "#,
            id,
            icon.hash,
            icon.data,
            icon.content_type
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn get_icon_by_feed_id(&self, feed_id: &str) -> Result<Option<Icon>, sqlx::Error> {
        let icon = query_as!(
            Icon,
            r#"
            select i.id, i.hash, i.data, i.content_type
            from icons as i
            inner join feeds_icons as fi
                on i.id = fi.icon_id
            where fi.feed_id = $1
            "#,
            feed_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(icon)
    }

    async fn create_opml_import_job(
        &self,
        feed_urls: &[String],
        existing_urls: &HashSet<String>,
    ) -> Result<OpmlImportJobSummary, sqlx::Error> {
        let job_id = create_id();
        let total = feed_urls.len() as i64;
        let skipped = feed_urls
            .iter()
            .filter(|url| existing_urls.contains(*url))
            .count() as i64;

        query!(
            r#"
            insert into opml_import_jobs (id, status, total, imported, skipped, failed)
            values ($1, $2, $3, 0, $4, 0)
            "#,
            job_id,
            "running",
            total,
            skipped
        )
        .execute(&self.pg_pool)
        .await?;

        if !feed_urls.is_empty() {
            let mut builder: QueryBuilder<Postgres> =
                QueryBuilder::new("insert into opml_import_items (id, job_id, feed_url, status)");

            builder.push_values(feed_urls, |mut b, url| {
                let status = if existing_urls.contains(url) {
                    "skipped"
                } else {
                    "queued"
                };
                b.push_bind(create_id());
                b.push_bind(&job_id);
                b.push_bind(url);
                b.push_bind(status);
            });

            builder.build().execute(&self.pg_pool).await?;
        }

        Ok(OpmlImportJobSummary {
            job_id,
            total,
            skipped,
        })
    }

    async fn insert_stub_feeds(&self, feed_urls: &[String]) -> Result<(), sqlx::Error> {
        if feed_urls.is_empty() {
            return Ok(());
        }

        let now = Utc::now();
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into feeds (id, source_title, user_title, feed_url, site_url, last_synced_at, sync_started_at)",
        );

        builder.push_values(feed_urls, |mut b, url| {
            b.push_bind(create_id());
            b.push_bind(url);
            b.push_bind::<Option<String>>(None);
            b.push_bind(url);
            b.push_bind::<Option<String>>(None);
            b.push_bind::<Option<DateTime<Utc>>>(None);
            b.push_bind(now);
        });

        builder.push(" on conflict (feed_url) do nothing");

        builder.build().execute(&self.pg_pool).await?;

        Ok(())
    }

    async fn update_opml_import_item(
        &self,
        job_id: &str,
        feed_url: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        query!(
            r#"
            update opml_import_items
            set status = $1,
                error = $2,
                updated_at = now()
            where job_id = $3 and feed_url = $4
            "#,
            status,
            error,
            job_id,
            feed_url
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn increment_opml_import_job_counts(
        &self,
        job_id: &str,
        imported: i64,
        skipped: i64,
        failed: i64,
    ) -> Result<(), sqlx::Error> {
        query!(
            r#"
            update opml_import_jobs
            set imported = imported + $1,
                skipped = skipped + $2,
                failed = failed + $3,
                updated_at = now()
            where id = $4
            "#,
            imported,
            skipped,
            failed,
            job_id
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn update_opml_import_job_status(
        &self,
        job_id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        query!(
            r#"
            update opml_import_jobs
            set status = $1,
                updated_at = now()
            where id = $2
            "#,
            status,
            job_id
        )
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn get_opml_import_job(
        &self,
        job_id: &str,
    ) -> Result<Option<OpmlImportJob>, sqlx::Error> {
        let job = query_as!(
            OpmlImportJob,
            r#"
            select id, status, total, imported, skipped, failed
            from opml_import_jobs
            where id = $1
            "#,
            job_id
        )
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(job)
    }

    async fn get_opml_import_recent_items(
        &self,
        job_id: &str,
        limit: i64,
    ) -> Result<Vec<OpmlImportItem>, sqlx::Error> {
        let rows = query_as!(
            OpmlImportItem,
            r#"
            select feed_url, status, error, updated_at
            from opml_import_items
            where job_id = $1
            order by coalesce(updated_at, created_at) desc
            limit $2
            "#,
            job_id,
            limit
        )
        .fetch_all(&self.pg_pool)
        .await?;

        Ok(rows)
    }
}
