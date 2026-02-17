use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use tracing::{debug, error};

use crate::{error::Error, model::User, state::SharedAppState};

#[tracing::instrument(name = "middleware::jwt_auth", skip_all)]
pub async fn jwt_auth_middleware(
    State(app_state): State<SharedAppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, Error> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            let parts: Vec<&str> = h.split_whitespace().collect();

            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("bearer") {
                Some(parts[1])
            } else {
                None
            }
        })
        .ok_or({
            debug!("bearer token is missing");

            Error::Unauthorized
        })?;

    let user: User = app_state
        .get_user_by_auth_token_use_case
        .execute(token)
        .await
        .map_err(|e| {
            error!("error: {}", e);

            Error::Unauthorized
        })?
        .ok_or_else(|| {
            error!("user is missing but token is valid");

            Error::Unauthorized
        })?
        .into();

    let user = Arc::new(user);

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
