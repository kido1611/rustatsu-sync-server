use std::sync::Arc;

use core_domain::{
    category::{model::Category, repository::CategoryRepository},
    favourite::{model::Favourite, repository::FavouriteRepository},
    manga::{model::Manga, repository::MangaRepository},
    manga_tag::{model::MangaTag, repository::MangaTagRepository},
    tag::{model::Tag, repository::TagRepository},
    user::{model::UserUpdateSyncTime, repository::UserRepository},
};
use tracing::instrument;

use crate::{favourite::model::FavouriteResourceInput, shared::error::ApplicationError};

pub struct InsertUserFavouriteResourceUsecase {
    pub manga_repository: Arc<dyn MangaRepository>,
    pub tag_repository: Arc<dyn TagRepository>,
    pub manga_tag_repository: Arc<dyn MangaTagRepository>,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub favourite_repository: Arc<dyn FavouriteRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

impl InsertUserFavouriteResourceUsecase {
    #[instrument(name = "usecase::insert_user_favourite_resource", skip_all, fields(user_id = %user_id))]
    pub async fn execute(
        &self,
        user_id: i64,
        data: FavouriteResourceInput,
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

        let categories: Vec<Category> = data
            .categories
            .into_iter()
            .map(|category| category.to_category(user_id))
            .collect();
        if !categories.is_empty() {
            self.category_repository.insert(categories).await?;
        }

        let favourites: Vec<Favourite> = data
            .favourites
            .into_iter()
            .map(|favourite| favourite.to_favourite(user_id))
            .collect();
        if !favourites.is_empty() {
            self.favourite_repository.insert(favourites).await?;

            self.user_repository
                .update_favourite_sync_time(UserUpdateSyncTime {
                    user_id,
                    time: data.sync_time,
                })
                .await?;
        }

        Ok(())
    }
}
