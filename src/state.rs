use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::config::Config;

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

        Ok(AppState { pool, config })
    }
}
// impl Application {
//     pub async fn build(config: Config) -> Result<Self, anyhow::Error> {
//         let connection_pool = get_connection_pool(&config.database);
//
//         if config.application.run_migration {
//             println!("Running migrations");
//             sqlx::migrate!("./migrations").run(&connection_pool).await?;
//         }
//
//         let address = format!("{}:{}", config.application.host, config.application.port);
//
//         let listener = TcpListener::bind(address)
//             .await
//             .context("Unable opening port")
//             .unwrap();
//
//         let address = listener.local_addr().unwrap();
//         let port = address.port();
//         let host = address.ip().to_string();
//
//         let server = create_server(listener, connection_pool, config).await?;
//
//         Ok(Application { port, host, server })
//     }
//
//     pub fn port(&self) -> u16 {
//         self.port
//     }
//
//     pub fn host(&self) -> String {
//         self.host.clone()
//     }
//
//     pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
//         self.server.await
//     }
// }
//
// pub fn get_connection_pool(database: &Database) -> MySqlPool {
//     MySqlPoolOptions::new().connect_lazy_with(database.with_db())
// }
