use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, MatchedPath},
    http::{HeaderName, Request, header},
    middleware,
    routing::{get, post},
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{middlewares::jwt_auth_middleware, state::AppState};

const REQUEST_ID_HEADER: &str = "x-request-id";

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

    let x_request_id_header = HeaderName::from_static(REQUEST_ID_HEADER);
    let request_id_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id_header.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let request_id = match request.headers().get(REQUEST_ID_HEADER) {
                    Some(val) => val.to_str().unwrap(),
                    None => "",
                };
                let user_agent = match request.headers().get(header::USER_AGENT) {
                    Some(val) => val.to_str().unwrap(),
                    None => "",
                };

                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                tracing::info_span!(
                    "http_request",
                    request_id,
                    method = ?request.method(),
                    uri = ?request.uri(),
                    path = matched_path,
                    version = ?request.version(),
                    user_agent,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id_header));

    app.nest("/manga", manga_route)
        .nest("/me", me_route)
        .nest("/resource/favourites", resources_favourites_route)
        .nest("/resource/history", resources_history_route)
        .layer(CompressionLayer::new())
        .layer(request_id_middleware)
        .with_state(state)
}
