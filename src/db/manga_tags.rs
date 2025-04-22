use sqlx::{Postgres, QueryBuilder, Transaction};

use crate::{error::Error, model::MangaTagEntity};

use super::error::DatabaseError;

pub async fn insert_manga_tags(
    tx: &mut Transaction<'_, Postgres>,
    data: &[MangaTagEntity],
) -> Result<(), Error> {
    for batch in data.chunks(300) {
        let mut manga_tag_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            INSERT INTO manga_tags
                (manga_id, tag_id)
        "#,
        );

        manga_tag_builder.push_values(batch, |mut b, manga_tag| {
            b.push_bind(manga_tag.manga_id).push_bind(manga_tag.tag_id);
        });
        manga_tag_builder.push(" ON CONFLICT (manga_id, tag_id) DO NOTHING;");

        manga_tag_builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::DatabaseError)?;
    }

    Ok(())
}
