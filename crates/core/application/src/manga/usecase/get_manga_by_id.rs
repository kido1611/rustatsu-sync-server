use std::sync::Arc;

use core_domain::{manga::repository::MangaRepository, tag::repository::TagRepository};
use tracing::instrument;

use crate::{
    manga::model::{MangaDto, MangaResourceOutput},
    shared::error::ApplicationError,
    tag::model::TagDto,
};

pub struct GetMangaByIdUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
}

impl GetMangaByIdUsecase {
    #[instrument(name = "usecase::get_manga_by_id", skip_all, fields(manga_id = %manga_id))]
    pub async fn execute(
        &self,
        manga_id: i64,
    ) -> Result<Option<MangaResourceOutput>, ApplicationError> {
        let manga: MangaDto = match self.manga_repository.get_by_id(manga_id).await? {
            Some(manga) => manga.into(),
            None => return Ok(None),
        };

        let tags: Vec<(i64, TagDto)> = self
            .tag_repository
            .list_by_manga_ids(&[manga_id])
            .await?
            .into_iter()
            .map(|(manga_id, tag)| (manga_id, tag.into()))
            .collect();

        Ok(Some(MangaResourceOutput { manga, tags }))
    }
}
