use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{auth::decode_jwt, error::Error, model::User, state::SharedAppState};

#[tracing::instrument(name = "middleware::jwt_auth", skip_all)]
pub async fn jwt_auth_middleware(
    State(app_state): State<SharedAppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, Error> {
    let auth_header = match req.headers_mut().get(axum::http::header::AUTHORIZATION) {
        Some(header) => header
            .to_str()
            .map_err(|e| Error::UnexpectedError(e.into()))?,
        None => {
            return Err(Error::Unauthorized);
        }
    };

    let mut header = auth_header.split_whitespace();
    let (bearer_option, token_option) = (header.next(), header.next());

    let bearer = match bearer_option {
        Some(value) => value.to_lowercase(),
        None => {
            return Err(Error::Unauthorized);
        }
    };

    if bearer != *"bearer" {
        return Err(Error::Unauthorized);
    }

    let token = match token_option {
        Some(value) => value,
        None => {
            return Err(Error::Unauthorized);
        }
    };

    let app_state_jwt = app_state.clone();
    let token_data = decode_jwt(token.to_string(), &app_state_jwt.config.jwt)
        .map_err(|_| Error::Unauthorized)?;

    let user: User = app_state
        .check_user_by_id_usecase
        .execute(token_data.claims.user_id)
        .await?
        .ok_or(Error::Unauthorized)?
        .into();

    let user = Arc::new(user);

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
