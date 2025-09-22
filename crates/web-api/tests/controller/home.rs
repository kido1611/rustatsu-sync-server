use axum::{body::Body, extract::Request, http::StatusCode};
use http_body_util::BodyExt;

use crate::common::TestState;

#[tokio::test]
async fn should_be_ok() {
    let test_state = TestState::new(false).await;

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(&response_body[..], b"Alive");
}

#[tokio::test]
async fn should_be_error_not_found_when_access_undefined_route() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/random-route-should-be-404")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn should_be_error_not_found_when_access_undefined_route_on_protected_routes() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/resources/random-route-404")
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
