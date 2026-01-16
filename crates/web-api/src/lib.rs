use anyhow::Context;
use axum::serve;
use config::Config;
use routes::init_router;
use state::AppState;
use tokio::{net::TcpListener, signal};
use tracing::info;

pub mod config;
pub mod controllers;
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
    serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    // Wait for either Ctrl+C (SIGINT) or SIGTERM
    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C"),
        _ = terminate => info!("Received SIGTERM"),
    }

    info!("🛑 Shutdown signal received, starting graceful shutdown...");
}
