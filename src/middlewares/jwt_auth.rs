use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{
    auth::{decode_jwt, error::AuthError},
    db::user::get_user_by_id_optional,
    error::Error,
    state::SharedAppState,
};

#[tracing::instrument(name = "[MIDDLEWARE] jwt auth", skip_all)]
pub async fn jwt_auth_middleware(
    State(app_state): State<SharedAppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, Error> {
    let auth_header = match req.headers_mut().get(axum::http::header::AUTHORIZATION) {
        Some(header) => header.to_str().map_err(|e| Error::Other(e.into()))?,
        None => {
            return Err(Error::Auth(AuthError::Unauthenticated));
        }
    };

    let mut header = auth_header.split_whitespace();
    let (bearer_option, token_option) = (header.next(), header.next());

    let bearer = match bearer_option {
        Some(value) => value.to_lowercase(),
        None => {
            return Err(Error::Auth(AuthError::Unauthenticated));
        }
    };

    if bearer != *"bearer" {
        return Err(Error::Auth(AuthError::Unauthenticated));
    }

    let token = match token_option {
        Some(value) => value,
        None => {
            return Err(Error::Auth(AuthError::Unauthenticated));
        }
    };

    let app_state_jwt = app_state.clone();
    let token_data = decode_jwt(token.to_string(), &app_state_jwt.config.jwt)
        .map_err(|_| Error::Auth(AuthError::Unauthenticated))?;

    let user_optional = get_user_by_id_optional(&app_state.pool, token_data.claims.user_id).await?;
    let user = match user_optional {
        Some(user) => Arc::new(user),
        None => {
            return Err(Error::Auth(AuthError::Unauthenticated));
        }
    };

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
