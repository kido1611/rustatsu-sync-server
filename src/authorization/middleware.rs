use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
    middleware::Next,
};

use crate::{authorization::util::UserId, startup::AppState, util::AuthError};

use super::util::decode_jwt;

#[tracing::instrument(name = "jwt auth middleware", skip(app_state, req, next))]
pub async fn jwt_authorization_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AuthError> {
    let auth_header = req.headers_mut().get(axum::http::header::AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header
            .to_str()
            .map_err(|_| AuthError::EmptyAuthHeader(anyhow::anyhow!("Auth header is missing")))?,
        None => {
            return Err(AuthError::EmptyAuthToken(anyhow::anyhow!(
                "Auth token is missing"
            )))
        }
    };

    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());
    let token_data = match decode_jwt(token.unwrap().to_string(), app_state.config.jwt) {
        Ok(data) => data,
        Err(e) => return Err(AuthError::InvalidCredential(e.into())),
    };

    let current_user = UserId(token_data.claims.user_id);

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}
