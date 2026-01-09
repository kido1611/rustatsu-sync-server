use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use crate::TestState;

#[tokio::test]
async fn should_be_ok() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/deeplink/reset-password?token=token")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn should_be_error_bad_request_when_token_is_missing() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/deeplink/reset-password")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
