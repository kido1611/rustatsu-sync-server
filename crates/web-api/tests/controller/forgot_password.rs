use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use serde_json::json;

use crate::TestState;

#[tokio::test]
async fn can_set_password_reset_token() {
    let mut test_state = TestState::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_none());
    assert!(user.password_reset_token_expires_at.is_none());

    let payload = json!({
        "email": user.email
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_some());
    assert!(user.password_reset_token_expires_at.is_some());

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_do_nothing_when_request_again() {
    let mut test_state = TestState::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_none());
    assert!(user.password_reset_token_expires_at.is_none());

    let payload = json!({
        "email": user.email
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_some());
    assert!(user.password_reset_token_expires_at.is_some());

    // --------------------------------------------------------------- request again
    let payload = json!({
        "email": user.email
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let latest_user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        latest_user.password_reset_token_hash,
        user.password_reset_token_hash
    );
    assert_eq!(
        latest_user.password_reset_token_expires_at,
        user.password_reset_token_expires_at
    );

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_user_is_missing() {
    let mut test_state = TestState::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_none());
    assert!(user.password_reset_token_expires_at.is_none());

    let payload = json!({
        "email": "test-random-email@email.com"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

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
async fn should_be_error_bad_request_when_body_is_missing() {
    let mut test_state = TestState::new(true).await;

    let (user, _) = test_state.generate_jwt_with_user().await;

    let user = test_state
        .app_state
        .check_user_by_id_usecase
        .execute(user.id)
        .await
        .unwrap()
        .unwrap();

    assert!(user.password_reset_token_hash.is_none());
    assert!(user.password_reset_token_expires_at.is_none());

    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

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
async fn should_be_error_bad_request_when_body_invalid() {
    let test_state = TestState::new(false).await;

    // ------------------------------- NOT EMAIL
    let payload = json!({
        "email": "test-random-email"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // ------------------------------- email too long
    let payload = json!({
        "email": "this-email-length-should-be-over-100-characters-to-trigger-validation-error-and-this-should-be-very-long-enough@localhost"
    });
    let request = Request::builder()
        .method("POST")
        .uri("/forgot-password")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
