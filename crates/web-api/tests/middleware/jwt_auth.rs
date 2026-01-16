use axum::{body::Body, http::Request, http::StatusCode};
use core_domain::security::{auth_token::AuthToken, model::AuthTokenClaim};
use core_infrastructure::security::jwt_auth_token::JwtAuthToken;
use secrecy::ExposeSecret;

use crate::common::TestState;

#[tokio::test]
async fn should_throw_error_when_request_does_not_contain_header_authorization() {
    let test_state = TestState::new(false).await;

    let request = Request::builder().uri("/me").body(Body::empty()).unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_auth_header_value_is_empty() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_auth_header_is_invalid() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "random-string")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_auth_header_does_not_contain_bearer() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            "not-bearer random-string",
        )
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_jwt_token_is_invalid() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "bearer random-string")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_bearer_value_is_empty() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "bearer")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_user_is_missing() {
    let mut test_state = TestState::new(true).await;

    let auth_token: Box<dyn AuthToken> = Box::new(JwtAuthToken {
        secret: test_state
            .app_state
            .config
            .jwt
            .secret
            .expose_secret()
            .into(),
        iss: test_state.app_state.config.jwt.iss.expose_secret().into(),
        aud: test_state.app_state.config.jwt.aud.expose_secret().into(),
    });

    let token = auth_token
        .encode(&AuthTokenClaim { user_id: 1000 })
        .unwrap();

    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_user_is_exist() {
    let mut test_state = TestState::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;
    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    test_state.cleanup().await;
}
