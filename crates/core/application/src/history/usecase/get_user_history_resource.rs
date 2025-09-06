use std::sync::Arc;

use core_domain::{
    history::repository::HistoryRepository, manga::repository::MangaRepository,
    tag::repository::TagRepository, user::repository::UserRepository,
};
use tracing::instrument;

use crate::{
    history::model::{HistoryDto, HistoryResourceOutput},
    manga::model::MangaDto,
    shared::error::ApplicationError,
    tag::model::TagDto,
};

pub struct GetUserHistoryResourceUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    pub history_repository: Arc<dyn HistoryRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl GetUserHistoryResourceUsecase {
    #[instrument(name = "usecase::get_user_history_resource", skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: i64) -> Result<HistoryResourceOutput, ApplicationError> {
        let user = self.user_repository.get_by_id(user_id).await?.ok_or(
            ApplicationError::ResourceNotFound(format!("user {} is missing", user_id)),
        )?;

        let history: Vec<HistoryDto> = self
            .history_repository
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        if history.is_empty() {
            return Ok(HistoryResourceOutput {
                manga: Vec::new(),
                manga_tags: Vec::new(),
                history,
                sync_time: user
                    .favourites_sync_timestamp
                    .unwrap_or(chrono::Utc::now().timestamp_millis()),
            });
        }

        let manga_ids: Vec<i64> = history.iter().map(|item| item.manga_id).collect();
        let manga: Vec<MangaDto> = self
            .manga_repository
            .list_by_ids(&manga_ids)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let manga_tags: Vec<(i64, TagDto)> = self
            .tag_repository
            .list_by_manga_ids(&manga_ids)
            .await?
            .into_iter()
            .map(|(manga_id, tag)| (manga_id, tag.into()))
            .collect();

        Ok(HistoryResourceOutput {
            manga,
            manga_tags,
            history,
            sync_time: user
                .history_sync_timestamp
                .unwrap_or(chrono::Utc::now().timestamp_millis()),
        })
    }
}
