use std::sync::Arc;

use core_domain::{
    category::repository::CategoryRepository, favourite::repository::FavouriteRepository,
    manga::repository::MangaRepository, tag::repository::TagRepository,
    user::repository::UserRepository,
};
use tracing::instrument;

use crate::{
    category::model::CategoryDto,
    favourite::model::{FavouriteDto, FavouriteResourceOutput},
    manga::model::MangaDto,
    shared::error::ApplicationError,
    tag::model::TagDto,
};

pub struct GetUserFavouriteResourceUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub favourite_repository: Arc<dyn FavouriteRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl GetUserFavouriteResourceUsecase {
    #[instrument(name = "usecase::get_user_favourite_resource", skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: i64) -> Result<FavouriteResourceOutput, ApplicationError> {
        let user = self.user_repository.get_by_id(user_id).await?.ok_or(
            ApplicationError::ResourceNotFound(format!("user {} is missing", user_id)),
        )?;

        let favourites: Vec<FavouriteDto> = self
            .favourite_repository
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
        let categories: Vec<CategoryDto> = self
            .category_repository
            .list_by_user_id(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        if favourites.is_empty() {
            return Ok(FavouriteResourceOutput {
                manga: Vec::new(),
                manga_tags: Vec::new(),
                categories,
                favourites,
                sync_time: user
                    .favourites_sync_timestamp
                    .unwrap_or(chrono::Utc::now().timestamp_millis()),
            });
        }

        let manga_ids: Vec<i64> = favourites.iter().map(|item| item.manga_id).collect();
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

        Ok(FavouriteResourceOutput {
            manga,
            manga_tags,
            categories,
            favourites,
            sync_time: user
                .favourites_sync_timestamp
                .unwrap_or(chrono::Utc::now().timestamp_millis()),
        })
    }
}
