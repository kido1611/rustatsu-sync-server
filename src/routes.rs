use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};

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
}
