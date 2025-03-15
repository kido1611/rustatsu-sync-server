use axum::{body::Body, http::Request};
use reqwest::StatusCode;
use rustatsu_sync::auth::encode_jwt;

use crate::{
    generate_app_state, generate_jwt_with_user, generate_response,
    generate_response_custom_app_state,
};

#[tokio::test]
async fn should_throw_error_when_request_does_not_contain_header_authorization() {
    let request = Request::builder().uri("/me").body(Body::empty()).unwrap();

    let response = generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_auth_header_is_invalid() {
    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "random-string")
        .body(Body::empty())
        .unwrap();

    let response = generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_auth_header_does_not_contain_bearer() {
    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            "not-bearer random-string",
        )
        .body(Body::empty())
        .unwrap();

    let response = generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_jwt_token_is_invalid() {
    let request = Request::builder()
        .uri("/me")
        .header(axum::http::header::AUTHORIZATION, "bearer random-string")
        .body(Body::empty())
        .unwrap();

    let response = generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_throw_error_when_user_is_missing() {
    let app_state = generate_app_state(true).await;

    let token = encode_jwt(1000, &app_state.config.jwt).unwrap();

    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = generate_response_custom_app_state(app_state, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_be_ok_when_user_is_exist() {
    let app_state = generate_app_state(true).await;

    let (_, token) = generate_jwt_with_user(&app_state).await;

    let request = Request::builder()
        .uri("/me")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = generate_response_custom_app_state(app_state, request).await;

    assert_eq!(response.status(), StatusCode::OK);
}
