use anyhow::{Context, Result};
use sqlx::{PgPool, migrate};
use tracing::info;

mod entries;
pub use entries::*;

mod feeds;
pub use feeds::*;

mod icons;
pub use icons::*;

mod opml;
pub use opml::*;

mod id;
pub use id::*;

#[derive(Clone)]
pub struct Data {
    pg_pool: sqlx::PgPool,
}

impl Data {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("connecting to db...");

        let pg = PgPool::connect(database_url)
            .await
            .context("error connecting to postgres")?;

        info!("connected to db, running migrations...");

        migrate!("./src/db/migrations")
            .run(&pg)
            .await
            .context("error running migrations")?;

        info!("migrations completed");

        return Ok(Self { pg_pool: pg });
    }
}
