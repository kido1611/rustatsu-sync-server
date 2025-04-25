use axum::{body::Body, extract::Request, http::StatusCode};
use http_body_util::BodyExt;

use crate::AppStateTest;

#[tokio::test]
async fn should_be_ok() {
    let test_state = AppStateTest::new(false).await;

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(&response_body[..], b"Alive");
}
