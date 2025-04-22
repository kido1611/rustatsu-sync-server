use anyhow::Context;
use axum::serve;
use config::Config;
use routes::init_router;
use state::AppState;
use tokio::net::TcpListener;

pub mod auth;
pub mod config;
pub mod controllers;
pub mod db;
pub mod error;
pub mod middlewares;
pub mod model;
pub mod routes;
pub mod state;
pub mod telemetry;

pub async fn run() -> Result<(), anyhow::Error> {
    let config = Config::new().context("Failed to read configuration.")?;
    let address = config.application.get_address();
    let state = AppState::init(config).await?;
    let router = init_router(state);

    tracing::info!("Starting server: {}", address);

    let listener = TcpListener::bind(address).await?;
    serve(listener, router.into_make_service()).await?;

    Ok(())
}
