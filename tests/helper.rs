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

pub struct TestApp {
    pub address: String,
    pub api_client: reqwest::Client,
}

pub struct AppStateTest {
    pub app_state: AppState,
    pub enable_db: bool,
    pub router: Router,
}

impl AppStateTest {
    pub async fn new(enable_db: bool) -> Self {
        let uuid = Uuid::new_v4().to_string().replace("-", "");
        let mut config = Config::new().unwrap();
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

pub async fn spawn_app() -> TestApp {
    //     let config = {
    //         let mut c = Config::new().expect("Failed to read configuration");
    //         c.application.port = 0;
    //         c
    //     };
    //
    //     let application = Application::build(config.clone())
    //         .await
    //         .expect("Failed to build application.");
    //     let application_port = application.port();
    //     let address = format!("http://127.0.0.1:{}", application_port);
    //     tokio::spawn(application.run_until_stopped());
    //
    //     let client = reqwest::Client::builder()
    //         .redirect(reqwest::redirect::Policy::none())
    //         .cookie_store(true)
    //         .build()
    //         .unwrap();
    //
    //     TestApp {
    //         address,
    //         api_client: client,
    //     }

    todo!("not implemented yet")
}

// pub async fn generate_app_state(enable_db: bool) -> AppState {
//     let uuid = Uuid::new_v4().to_string().replace("-", "");
//     let mut config = Config::new().unwrap();
//     config.application.run_migration = false;
//     config.database.database_name = format!("rustatsu_test_{}", uuid);
//
//     if enable_db {
//         let without_db_pool = PgPoolOptions::new()
//             .min_connections(5)
//             .max_connections(30)
//             .connect_with(config.database.without_db())
//             .await
//             .expect("unable to connect database");
//
//         without_db_pool
//             .execute(format!("CREATE DATABASE {};", config.database.database_name).as_str())
//             .await
//             .expect("failed create database");
//     }
//
//     let app_state = AppState::init(config).await.unwrap();
//     if enable_db {
//         sqlx::migrate!("./migrations")
//             .run(&app_state.pool)
//             .await
//             .expect("undo migrations");
//     }
//
//     app_state
// }
//
// pub async fn generate_response(request: Request<Body>) -> Response<Body> {
//     let app_state = generate_app_state(false).await;
//
//     generate_response_custom_app_state(app_state, request).await
// }
//
// pub async fn generate_response_custom_app_state(
//     app_state: AppState,
//     request: Request<Body>,
// ) -> Response<Body> {
//     let db = app_state.config.database.clone();
//
//     let app = init_router(app_state);
//
//     let response = app.oneshot(request).await.unwrap();
//
//     drop_database_test(db).await;
//
//     response
// }
//
// async fn drop_database_test(db: Database) {
//     let without_db_pool = PgPoolOptions::new()
//         .min_connections(5)
//         .max_connections(30)
//         .connect_with(db.without_db())
//         .await
//         .expect("unable to connect database");
//     let _ = without_db_pool
//         .execute(format!("DROP DATABASE IF EXISTS {}", db.database_name,).as_str())
//         .await
//         .expect("Unable drop database");
// }
//
// pub async fn generate_jwt_with_user(app_state: &AppState) -> (User, String) {
//     let (user, _) = create_user(
//         &app_state.pool,
//         "test@email.com".to_string(),
//         "password".into(),
//     )
//     .await
//     .expect("Failed create user");
//
//     let token = encode_jwt(user.id, &app_state.config.jwt).unwrap();
//
//     (user, token)
// }
