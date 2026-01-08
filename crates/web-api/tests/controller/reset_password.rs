use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::json;

use crate::TestState;

#[tokio::test]
async fn can_reset_user_password() {
    let mut test_state = TestState::new(true).await;

    // create dummy user
    let (user, _) = test_state.generate_jwt_with_user().await;

    // get reset password token first
    let token = test_state
        .app_state
        .request_reset_password_use_case
        .execute(
            &user.email,
            &test_state.app_state.config.application.get_base_url(),
        )
        .await
        .unwrap()
        .unwrap();

    // makesure user password reset token is exist
    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_some());
    assert!(user.password_reset_token_expires_at.is_some());

    let payload = json!({
        "reset_token": token,
        "password": "password"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    // makesure user password reset token was null (reset)
    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_none());
    assert!(user.password_reset_token_expires_at.is_none());

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_not_found_when_token_is_expired() {
    let mut test_state = TestState::new(true).await;

    // create dummy user
    let (user, _) = test_state.generate_jwt_with_user().await;

    let expires_at = Utc::now() - Duration::days(10);
    sqlx::query!(
        r#"
        UPDATE users
        SET 
            password_reset_token_hash = $1,
            password_reset_token_expires_at = $2
        WHERE id = $3;
    "#,
        "4UONXgznKfufO09SEr7GSVmSxGmfr1-JudkwuRMMsuc",
        expires_at.timestamp(),
        user.id,
    )
    .execute(&test_state.app_state.pool)
    .await
    .unwrap();

    // makesure user password reset token is exist
    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_some());
    assert!(user.password_reset_token_expires_at.is_some());

    let payload = json!({
        "reset_token": "7XAyt90RIfGl0PkYvTwT7qagmD93SSdWedVkz_ytJYk",
        "password": "password"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_not_found_when_token_is_missing() {
    let mut test_state = TestState::new(true).await;

    // create dummy user
    let (user, _) = test_state.generate_jwt_with_user().await;

    // get reset password token first
    let _ = test_state
        .app_state
        .request_reset_password_use_case
        .execute(
            &user.email,
            &test_state.app_state.config.application.get_base_url(),
        )
        .await
        .unwrap()
        .unwrap();

    // makesure user password reset token is exist
    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_some());
    assert!(user.password_reset_token_expires_at.is_some());

    let payload = json!({
        "reset_token": "7XAyt90RIfGl0PkYvTwT7qagmD93SSdWedVkz_ytJYk",
        "password": "password"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_bad_request_when_request_is_invalid() {
    let test_state = TestState::new(false).await;

    // --------------------------------------------------------------------------
    let payload = json!({
        "password": "passwo"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // --------------------------------------------------------------------------
    let payload = json!({
        "reset_token": "reset-token",
        "password": "password-should-be-very-long-enough-to-trigger-error-validation"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn should_be_error_bad_request_when_request_body_is_empty() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .method("POST")
        .uri("/reset-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
