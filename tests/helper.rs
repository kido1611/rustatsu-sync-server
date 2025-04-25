use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use rustatsu_sync::{
    auth::encode_jwt, config::Config, db::user::create_user, model::User, routes::init_router,
    state::AppState,
};
use sqlx::{Executor, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

pub struct AppStateTest {
    pub app_state: AppState,
    pub enable_db: bool,
    pub router: Router,
}

impl AppStateTest {
    pub async fn new(enable_db: bool) -> Self {
        let config = Config::new().unwrap();

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
            sqlx::migrate!("./migrations")
                .run(&app_state.pool)
                .await
                .expect("undo migrations");
        }

        let router = init_router(app_state.clone());

        AppStateTest {
            app_state,
            enable_db,
            router,
        }
    }

    pub async fn generate_jwt_with_user(&self) -> (User, String) {
        let (user, _) = create_user(
            &self.app_state.pool,
            "test@email.com".to_string(),
            "password".into(),
        )
        .await
        .expect("Failed create user");

        let token = encode_jwt(user.id, &self.app_state.config.jwt).unwrap();

        (user, token)
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
                    "DROP DATABASE IF EXISTS {}",
                    self.app_state.config.database.database_name,
                )
                .as_str(),
            )
            .await
            .expect("Unable drop database");

        self.enable_db = false;
    }
}

impl Drop for AppStateTest {
    fn drop(&mut self) {
        assert!(
            !self.enable_db,
            "Database not dropped. Call `app_test_state.cleanup()`"
        );
    }
}
