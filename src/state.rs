use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

pub type SharedAppState = Arc<AppState>;

impl AppState {
    pub async fn init(config: Config) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(30)
            .connect_lazy_with(config.database.with_db());

        if config.application.run_migration {
            tracing::warn!("Running database migrations...");
            sqlx::migrate!("./migrations").run(&pool).await?;
        }

        Ok(AppState { pool, config })
    }
}
