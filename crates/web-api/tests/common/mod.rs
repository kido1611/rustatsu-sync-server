pub mod fake;

use axum::{
    body::Body,
    http::{Request, Response},
};
use figment::{
    Figment,
    providers::{Env, Format, Yaml},
};
use rustatsu_sync::{
    auth::encode_jwt, config::Config, model::User, routes::init_router, state::AppState,
};
use sqlx::{Executor, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

pub struct TestState {
    pub app_state: AppState,
    pub enable_database: bool,
}

impl TestState {
    pub fn create_config() -> Config {
        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let config_directory = base_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("configuration");

        let config: Config = Figment::new()
            .merge(Yaml::file(config_directory.join("base.yaml")))
            .merge(Yaml::file(config_directory.join("local.yaml")))
            .merge(Env::raw().split("__"))
            .extract()
            .map_err(Box::new)
            .unwrap();

        config
    }

    pub async fn new(enable_db: bool) -> Self {
        let config = Self::create_config();

        Self::new_with_config(enable_db, config).await
    }

    pub async fn new_with_config(enable_db: bool, mut config: Config) -> Self {
        let uuid = Uuid::new_v4().to_string().replace("-", "");
        config.application.run_migration = false;
        config.database.database_name = format!("rustatsu_test_{}", uuid);

        if enable_db {
            let without_db_pool = PgPoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .connect_with(config.database.without_db())
                .await
                .expect("unable to connect database");

            without_db_pool
                .execute(format!("CREATE DATABASE {};", config.database.database_name).as_str())
                .await
                .expect("failed create database");

            drop(without_db_pool);
        }

        let app_state = AppState::init(config.clone()).await.unwrap();
        if enable_db {
            sqlx::migrate!("../../migrations")
                .run(&app_state.pool)
                .await
                .expect("undo migrations");
        }

        // let router = init_router(app_state.clone());

        TestState {
            app_state,
            enable_database: enable_db,
        }
    }

    pub async fn generate_jwt_with_user(&self) -> (User, String) {
        let user_dto = self
            .app_state
            .check_or_create_user_usecase
            .execute(core_application::user::model::UserInput {
                email: "test@email.com".to_string(),
                password: "password".into(),
                nickname: None,
            })
            .await
            .expect("failed to create test user");

        let token = encode_jwt(user_dto.id, &self.app_state.config.jwt)
            .expect("failed to create jwt token");

        (user_dto.into(), token)
    }

    pub async fn generate_response(&self, request: Request<Body>) -> Response<Body> {
        let app = init_router(self.app_state.clone());

        app.oneshot(request).await.unwrap()
    }

    pub async fn cleanup(&mut self) {
        self.app_state.pool.close().await;

        let without_db_pool = PgPoolOptions::new()
            .min_connections(1)
            .max_connections(1)
            .connect_with(self.app_state.config.database.without_db())
            .await
            .expect("unable to connect database");
        let _ = without_db_pool
            .execute(
                format!(
                    "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                    self.app_state.config.database.database_name,
                )
                .as_str(),
            )
            .await
            .expect("Unable drop database");

        self.enable_database = false;
    }
}

impl Drop for TestState {
    fn drop(&mut self) {
        assert!(
            !self.enable_database,
            "Database not dropped. Call `app_test_state.cleanup()`"
        );
    }
}
