use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Request},
    middleware,
    routing::{get, post},
};
use tower_http::{
    compression::CompressionLayer,
    trace::{self, DefaultOnFailure, TraceLayer},
};
use tracing::Level;

use crate::{middlewares::jwt_auth_middleware, state::AppState};

pub fn init_router(app_state: AppState) -> Router {
    let state = Arc::new(app_state);

    let app = Router::new()
        .route("/", get(crate::controllers::home::index))
        .route("/auth", post(crate::controllers::auth::store));

    let manga_route = Router::new()
        .route("/", get(crate::controllers::manga::index))
        .route("/{id}", get(crate::controllers::manga::show));

    let resources_favourites_route = Router::new()
        .route("/", post(crate::controllers::resources::favourites::store))
        .layer(DefaultBodyLimit::max(52_428_800)) // 50MB in binary bytes. https://www.gbmb.org/mb-to-bytes
        .route("/", get(crate::controllers::resources::favourites::index))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    let resources_history_route = Router::new()
        .route("/", post(crate::controllers::resources::history::store))
        .layer(DefaultBodyLimit::max(52_428_800)) // 50MB in binary bytes. https://www.gbmb.org/mb-to-bytes
        .route("/", get(crate::controllers::resources::history::index))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    let me_route = Router::new()
        .route("/", get(crate::controllers::me::index))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    app.nest("/manga", manga_route)
        .nest("/me", me_route)
        .nest("/resources/favourites", resources_favourites_route)
        .nest("/resources/history", resources_history_route)
        .with_state(state)
        .layer(CompressionLayer::new())
    // .layer(
    //     TraceLayer::new_for_http()
    //         .make_span_with(
    //             |request: &Request<Body>| {
    //                 let request_id = uuid::Uuid::new_v4();
    //                 tracing::span!(
    //                     Level::INFO,
    //                     "request",
    //                     method = tracing::field::display(request.method()),
    //                     uri = tracing::field::display(request.uri()),
    //                     version = tracing::field::debug(request.version()),
    //                     request_id = tracing::field::display(request_id),
    //                     headers = tracing::field::debug(request.headers())
    //                 )
    //             }, // trace::DefaultMakeSpan::new()
    //                //     .level(Level::INFO)
    //                //     .include_headers(true),
    //         )
    //         .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
    //         .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
    //         .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
    // )
}
