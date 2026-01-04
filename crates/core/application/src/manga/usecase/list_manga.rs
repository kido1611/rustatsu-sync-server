use std::sync::Arc;

use core_domain::{manga::repository::MangaRepository, tag::repository::TagRepository};
use tracing::{Level, instrument, warn};

use crate::{
    manga::model::{ListMangaQuery, ListMangaResourceOutput, MangaDto},
    shared::error::ApplicationError,
    tag::model::TagDto,
};

pub struct ListMangaUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
}

impl ListMangaUsecase {
    #[instrument(name = "usecase::list_manga", skip_all, fields(limit = %query.limit, offset = %query.offset), level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(
        &self,
        query: ListMangaQuery,
    ) -> Result<ListMangaResourceOutput, ApplicationError> {
        let manga: Vec<MangaDto> = self
            .manga_repository
            .list_with_pagination(query.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        if manga.is_empty() {
            warn!("manga list is empty");

            return Ok(ListMangaResourceOutput {
                manga,
                manga_tags: Vec::new(),
            });
        }

        let manga_ids: Vec<i64> = manga.iter().map(|manga| manga.id).collect();

        let tags: Vec<(i64, TagDto)> = self
            .tag_repository
            .list_by_manga_ids(&manga_ids)
            .await?
            .into_iter()
            .map(|(manga_id, tag)| (manga_id, tag.into()))
            .collect();

        Ok(ListMangaResourceOutput {
            manga,
            manga_tags: tags,
        })
    }
}
