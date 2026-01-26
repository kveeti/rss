use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder, query, query_as};

use crate::db::{Data, create_id};

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

impl Data {
    pub async fn create_opml_import_job(
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
            let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
                "insert into opml_import_items (id, job_id, feed_url, status)",
            );

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

            builder
                .build()
                .execute(&self.pg_pool)
                .await?;
        }

        Ok(OpmlImportJobSummary {
            job_id,
            total,
            skipped,
        })
    }

    pub async fn insert_stub_feeds(
        &self,
        feed_urls: &[String],
    ) -> Result<(), sqlx::Error> {
        if feed_urls.is_empty() {
            return Ok(());
        }

        let now = Utc::now();
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into feeds (id, title, feed_url, site_url, last_synced_at, sync_started_at)",
        );

        builder.push_values(feed_urls, |mut b, url| {
            b.push_bind(create_id());
            b.push_bind(url);
            b.push_bind(url);
            b.push_bind::<Option<String>>(None);
            b.push_bind::<Option<DateTime<Utc>>>(None);
            b.push_bind(now);
        });

        builder.push(" on conflict (feed_url) do nothing");

        builder.build().execute(&self.pg_pool).await?;

        Ok(())
    }

    pub async fn update_opml_import_item(
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

    pub async fn increment_opml_import_job_counts(
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

    pub async fn update_opml_import_job_status(
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

    pub async fn get_opml_import_job(
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

    pub async fn get_opml_import_recent_items(
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
