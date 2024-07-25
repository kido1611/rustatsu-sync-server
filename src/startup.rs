use anyhow::Context;
use axum::{body::Body, http::Request, serve::Serve, Router};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;
use tower_http::trace::{self, DefaultOnFailure, TraceLayer};
use tracing::Level;

use crate::{
    configuration::{Config, Database},
    router::{get_manga, get_manga_by_id, index},
};

pub struct Application {
    port: u16,
    host: String,
    server: Serve<Router, Router>,
}

impl Application {
    pub async fn build(config: Config) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&config.database);

        let address = format!("{}:{}", config.application.host, config.application.port);

        let listener = TcpListener::bind(address)
            .await
            .context("Unable opening port")
            .unwrap();

        let address = listener.local_addr().unwrap();
        let port = address.port();
        let host = address.ip().to_string();

        let server = create_server(listener, connection_pool).await?;

        Ok(Application { port, host, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(database: &Database) -> MySqlPool {
    MySqlPoolOptions::new().connect_lazy_with(database.with_db())
}

fn create_router(db_pool: MySqlPool) -> Router {
    Router::new()
        .route("/", axum::routing::get(index))
        .nest(
            "/manga",
            Router::new()
                .route("/", axum::routing::get(get_manga))
                .route("/:id", axum::routing::get(get_manga_by_id)),
        )
        .with_state(db_pool)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    |request: &Request<Body>| {
                        let request_id = uuid::Uuid::new_v4();
                        tracing::span!(
                            Level::DEBUG,
                            "request",
                            method = tracing::field::display(request.method()),
                            uri = tracing::field::display(request.uri()),
                            version = tracing::field::debug(request.version()),
                            request_id = tracing::field::display(request_id),
                            headers = tracing::field::debug(request.headers())
                        )
                    }, // trace::DefaultMakeSpan::new()
                       //     .level(Level::INFO)
                       //     .include_headers(true),
                )
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::INFO)),
        )
}

async fn create_server(
    listener: TcpListener,
    db_pool: MySqlPool,
) -> Result<Serve<Router, Router>, anyhow::Error> {
    let router = create_router(db_pool);

    Ok(axum::serve(listener, router))
}
