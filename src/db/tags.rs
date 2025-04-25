use std::sync::Arc;

use sqlx::{Postgres, QueryBuilder};

use crate::{error::Error, model::Tag};

use super::{PostgresTransaction, error::DatabaseError};

pub async fn insert_tags(tx: &mut PostgresTransaction, data: &[Arc<Tag>]) -> Result<(), Error> {
    for batch in data.chunks(300) {
        let mut tags_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO tags 
                (id, title, "key", source)
        "#,
        );

        tags_builder.push_values(batch, |mut b, tag| {
            b.push_bind(tag.tag_id)
                .push_bind(&tag.title)
                .push_bind(&tag.key)
                .push_bind(&tag.source);
        });

        tags_builder.push(
            r#" ON CONFLICT (id)
            DO UPDATE SET 
                title = EXCLUDED.title,
                "key" = EXCLUDED.key,
                source = EXCLUDED.source;
        "#,
        );

        tags_builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    Ok(())
}
