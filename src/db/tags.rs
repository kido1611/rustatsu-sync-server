use std::sync::Arc;

use sqlx::{Postgres, Transaction};

use crate::{error::Error, model::Tag};

use super::error::DatabaseError;

pub async fn insert_tags(
    tx: &mut Transaction<'_, Postgres>,
    data: &[Arc<Tag>],
) -> Result<(), Error> {
    for tag in data {
        sqlx::query!(
            r#"
            INSERT INTO tags 
                (id, title, "key", source)
            VALUES
                ($1, $2, $3, $4)
            ON CONFLICT (id)
            DO UPDATE SET
                title = $2,
                "key" = $3,
                source = $4;
        "#,
            tag.tag_id,
            tag.title,
            tag.key,
            tag.source,
        )
        .execute(&mut **tx)
        .await
        .map_err(DatabaseError::DatabaseError)?;
    }

    Ok(())
}
