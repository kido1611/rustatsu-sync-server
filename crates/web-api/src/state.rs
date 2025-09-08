use std::sync::Arc;

use core_application::{
    favourite::usecase::{
        get_user_favourite_resource::GetUserFavouriteResourceUsecase,
        insert_user_favourite_resource::InsertUserFavouriteResourceUsecase,
    },
    history::usecase::{
        get_user_history_resource::GetUserHistoryResourceUsecase,
        insert_user_history_resource::InsertUserHistoryResourceUsecase,
    },
    manga::usecase::{get_manga_by_id::GetMangaByIdUsecase, list_manga::ListMangaUsecase},
    user::usecase::{
        check_or_create_user::CheckOrCreateUserUsecase, check_user_by_id::CheckUserByIdUsecase,
    },
};
use core_infrastructure::{
    category::postgresql_repository::PostgreSQLCategoryRepository,
    favourite::postgresql_repository::PostgreSQLFavouriteRepository,
    history::postgresql_repository::PostgreSQLHistoryRepository,
    manga::postgresql_repository::PostgreSQLMangaRepository,
    manga_tag::postgresql_repository::PostgreSQLMangaTagRepository,
    security::argon_password_manager::ArgonPasswordManager,
    tag::postgresql_repository::PostgreSQLTagRepository,
    user::postgresql_repository::PostgreSQLUserRepository,
};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,

    pub get_manga_by_id_usecase: Arc<GetMangaByIdUsecase>,
    pub list_manga_usecase: Arc<ListMangaUsecase>,
    pub check_or_create_user_usecase: Arc<CheckOrCreateUserUsecase>,
    pub check_user_by_id_usecase: Arc<CheckUserByIdUsecase>,
    pub insert_user_favourite_resource_usecase: Arc<InsertUserFavouriteResourceUsecase>,
    pub get_user_favourite_resource_usecase: Arc<GetUserFavouriteResourceUsecase>,
    pub insert_user_history_resource_usecase: Arc<InsertUserHistoryResourceUsecase>,
    pub get_user_history_resource_usecase: Arc<GetUserHistoryResourceUsecase>,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub async fn init(config: Config) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(30)
            .connect_lazy_with(config.database.with_db());

        if config.application.run_migration {
            tracing::warn!("running database migrations...");
            sqlx::migrate!("../../migrations").run(&pool).await?;
            tracing::info!("successfully running database migrations")
        }

        let password_manager = Arc::new(ArgonPasswordManager {});
        let manga_repository = Arc::new(PostgreSQLMangaRepository { pool: pool.clone() });
        let tag_repository = Arc::new(PostgreSQLTagRepository { pool: pool.clone() });
        let user_repository = Arc::new(PostgreSQLUserRepository { pool: pool.clone() });
        let manga_tag_repository = Arc::new(PostgreSQLMangaTagRepository { pool: pool.clone() });
        let category_repository = Arc::new(PostgreSQLCategoryRepository { pool: pool.clone() });
        let favourite_repository = Arc::new(PostgreSQLFavouriteRepository { pool: pool.clone() });
        let history_repository = Arc::new(PostgreSQLHistoryRepository { pool: pool.clone() });

        let get_manga_by_id_usecase = Arc::new(GetMangaByIdUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
        });
        let list_manga_usecase = Arc::new(ListMangaUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
        });

        let insert_user_favourite_resource_usecase = Arc::new(InsertUserFavouriteResourceUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
            manga_tag_repository: manga_tag_repository.clone(),
            category_repository: category_repository.clone(),
            favourite_repository: favourite_repository.clone(),
            user_repository: user_repository.clone(),
        });
        let get_user_favourite_resource_usecase = Arc::new(GetUserFavouriteResourceUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
            category_repository: category_repository.clone(),
            favourite_repository: favourite_repository.clone(),
            user_repository: user_repository.clone(),
        });

        let insert_user_history_resource_usecase = Arc::new(InsertUserHistoryResourceUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
            manga_tag_repository: manga_tag_repository.clone(),
            history_repository: history_repository.clone(),
            user_repository: user_repository.clone(),
        });
        let get_user_history_resource_usecase = Arc::new(GetUserHistoryResourceUsecase {
            manga_repository: manga_repository.clone(),
            tag_repository: tag_repository.clone(),
            history_repository: history_repository.clone(),
            user_repository: user_repository.clone(),
        });

        let check_or_create_user_usecase = Arc::new(CheckOrCreateUserUsecase {
            user_repository: user_repository.clone(),
            password_manager,
            allow_to_register: config.application.allow_registration,
        });
        let check_user_by_id_usecase = Arc::new(CheckUserByIdUsecase { user_repository });

        Ok(AppState {
            pool,
            config,

            // manga usecase
            get_manga_by_id_usecase,
            list_manga_usecase,

            // user usecase
            check_or_create_user_usecase,
            check_user_by_id_usecase,

            // favourite
            insert_user_favourite_resource_usecase,
            get_user_favourite_resource_usecase,

            // history
            insert_user_history_resource_usecase,
            get_user_history_resource_usecase,
        })
    }
}
