use std::sync::Arc;

use core_domain::{
    history::{model::History, repository::HistoryRepository},
    manga::{model::Manga, repository::MangaRepository},
    manga_tag::{model::MangaTag, repository::MangaTagRepository},
    tag::{model::Tag, repository::TagRepository},
    user::{model::UserUpdateSyncTime, repository::UserRepository},
};
use tracing::{Level, instrument};

use crate::{history::model::HistoryResourceInput, shared::error::ApplicationError};

pub struct InsertUserHistoryResourceUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    pub manga_tag_repository: Arc<dyn MangaTagRepository>,
    pub history_repository: Arc<dyn HistoryRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl InsertUserHistoryResourceUsecase {
    #[instrument(name = "usecase::insert_user_history_resource", skip_all, fields(user_id = %user_id), level = Level::DEBUG, err(level = Level::ERROR))]
    pub async fn execute(
        &self,
        user_id: i64,
        data: HistoryResourceInput,
    ) -> Result<(), ApplicationError> {
        let manga: Vec<Manga> = data.manga.into_iter().map(Into::into).collect();
        if !manga.is_empty() {
            self.manga_repository.insert(manga).await?;
        }

        let tags: Vec<Tag> = data.tags.into_iter().map(Into::into).collect();
        if !tags.is_empty() {
            self.tag_repository.insert(tags).await?;
        }

        let manga_tags: Vec<MangaTag> = data.manga_tags.into_iter().map(Into::into).collect();
        if !manga_tags.is_empty() {
            self.manga_tag_repository.insert(manga_tags).await?;
        }

        let history: Vec<History> = data
            .history
            .into_iter()
            .map(|history_dto| history_dto.to_history(user_id))
            .collect();
        if !history.is_empty() {
            self.history_repository.insert(history).await?;

            self.user_repository
                .update_history_sync_time(UserUpdateSyncTime {
                    user_id,
                    time: data.sync_time,
                })
                .await?;
        }

        Ok(())
    }
}
