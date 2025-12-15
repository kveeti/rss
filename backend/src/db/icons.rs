use sqlx::{query, query_as};

use crate::db::{Data, create_id};

impl Data {
    pub async fn upsert_icon(&self, icon: NewIcon) -> Result<(), sqlx::Error> {
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

    pub async fn get_icon_by_feed_id(&self, feed_id: &str) -> Result<Option<Icon>, sqlx::Error> {
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
}

#[derive(Debug, serde::Serialize)]
pub struct NewIcon {
    pub hash: String,
    pub data: Vec<u8>,
    pub content_type: String,
}

pub struct Icon {
    pub id: String,
    pub hash: String,
    pub data: Vec<u8>,
    pub content_type: String,
}
