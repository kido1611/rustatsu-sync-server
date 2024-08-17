use anyhow::Context;
use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    http::Request,
    middleware::{self},
    serve::Serve,
    Router,
};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer,
    trace::{self, DefaultOnFailure, TraceLayer},
};
use tracing::Level;

use crate::{
    authorization::jwt_authorization_middleware,
    configuration::{Config, Database},
    router::{
        get_favourites_route, get_history_route, get_manga_id_route, get_manga_route, get_me_route,
        index, post_auth_route, post_favourites_route, post_history_route,
    },
};

pub struct Application {
    port: u16,
    host: String,
    server: Serve<Router, Router>,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Config,
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

        let server = create_server(listener, connection_pool, config).await?;

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

fn create_router(db_pool: MySqlPool, config: Config) -> Router {
    let state = AppState {
        pool: db_pool,
        config,
    };

    Router::new()
        .route("/", axum::routing::get(index))
        .nest(
            "/manga",
            Router::new()
                .route("/", axum::routing::get(get_manga_route))
                .route("/:id", axum::routing::get(get_manga_id_route)),
        )
        .route("/auth", axum::routing::post(post_auth_route))
        .nest(
            "/",
            Router::new()
                .route("/me", axum::routing::get(get_me_route))
                .nest(
                    "/resource",
                    Router::new()
                        .nest(
                            "/favourites",
                            Router::new()
                                .route("/", axum::routing::get(get_favourites_route))
                                .route(
                                    "/",
                                    axum::routing::post(post_favourites_route)
                                        .layer(DefaultBodyLimit::max(52_428_800)), // 50MB in binary bytes. https://www.gbmb.org/mb-to-bytes
                                ),
                        )
                        .nest(
                            "/history",
                            Router::new()
                                .route("/", axum::routing::get(get_history_route))
                                .route(
                                    "/",
                                    axum::routing::post(post_history_route)
                                        .layer(DefaultBodyLimit::max(52_428_800)), // 50MB in binary bytes. https://www.gbmb.org/mb-to-bytes
                                ),
                        ),
                )
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    jwt_authorization_middleware,
                )),
        )
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    |request: &Request<Body>| {
                        let request_id = uuid::Uuid::new_v4();
                        tracing::span!(
                            Level::INFO,
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
    // TODO: custom body limit size from config
    // TODO: Tracing to jaeger
}

async fn create_server(
    listener: TcpListener,
    db_pool: MySqlPool,
    config: Config,
) -> Result<Serve<Router, Router>, anyhow::Error> {
    let router = create_router(db_pool, config);

    Ok(axum::serve(listener, router))
}
