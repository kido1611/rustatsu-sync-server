use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use rustatsu_sync::{config::Config, controllers::auth::AuthResponse};
use serde::Serialize;
use serde_json::json;

use crate::AppStateTest;

#[derive(Serialize)]
struct PartialAuthRequest {
    email: String,
}

#[derive(Serialize)]
struct AuthRequest {
    email: String,
    password: String,
}

#[tokio::test]
async fn should_be_error_when_body_is_missing() {
    let test_state = AppStateTest::new(false).await;

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn should_be_error_when_body_is_invalid() {
    let test_state = AppStateTest::new(false).await;

    // -----------------------------------------------------------------------
    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(PartialAuthRequest {
                email: "test@localhost".to_string()
            }))
            .unwrap(),
        ))
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // -----------------------------------------------------------------------
    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: "a".to_string(),
                password: "pass".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // -----------------------------------------------------------------------
    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: "this-email-length-should-be-over-32-characters-to-trigger-error@localhost"
                    .to_string(),
                password: "this-password-length-should-be-over-32-characters-to-trigger-error"
                    .to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn should_be_error_when_has_incorrect_type() {
    let mut test_state = AppStateTest::new(true).await;

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: "test@localhost".to_string(),
                password: "password".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_when_credential_is_invalid() {
    let mut test_state = AppStateTest::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: user.email,
                password: "incorrect-password".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_user_is_exist() {
    let mut test_state = AppStateTest::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(1));

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: user.email,
                password: "password".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(!response_body.is_empty());

    let _: AuthResponse = serde_json::from_slice(&response_body).unwrap();

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(1));

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_create_user_when_user_is_missing() {
    let mut test_state = AppStateTest::new(true).await;

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(0));

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: "test@localhost".to_string(),
                password: "password".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(1));

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_when_user_is_missing_and_registration_is_disabled() {
    let mut config = Config::new().unwrap();
    config.application.allow_registration = false;

    let mut test_state = AppStateTest::new_with_config(true, config).await;

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(0));

    let request = Request::builder()
        .method("POST")
        .uri("/auth")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!(AuthRequest {
                email: "test@localhost".to_string(),
                password: "password".to_string(),
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(response_body.is_empty());

    let result = sqlx::query!(r#"SELECT count(*) FROM users"#)
        .fetch_one(&test_state.app_state.pool)
        .await
        .unwrap();
    assert_eq!(result.count, Some(0));

    test_state.cleanup().await;
}
