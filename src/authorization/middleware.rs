use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
    middleware::Next,
};

use crate::{authorization::util::UserId, error::ApiError, startup::AppState};

use super::util::decode_jwt;

#[tracing::instrument(name = "jwt auth middleware", skip(app_state, req, next))]
pub async fn jwt_authorization_middleware(
    State(app_state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, ApiError> {
    let auth_header = match req.headers_mut().get(axum::http::header::AUTHORIZATION) {
        Some(header) => header
            .to_str()
            .map_err(|_| ApiError::EmptyAuthHeader(anyhow::anyhow!("Auth header is missing")))?,
        None => {
            return Err(ApiError::EmptyAuthToken(anyhow::anyhow!(
                "Auth token is missing"
            )))
        }
    };

    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());
    let token_data = decode_jwt(token.unwrap().to_string(), app_state.config.jwt.clone())?;

    let current_user = UserId(token_data.claims.user_id);

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}
