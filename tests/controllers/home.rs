use axum::{body::Body, extract::Request};
use http_body_util::BodyExt;
use reqwest::StatusCode;

use crate::generate_response;

#[tokio::test]
async fn home_index_rendered_correctly() {
    let request = Request::builder().uri("/").body(Body::empty()).unwrap();
    let response = generate_response(request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let response_body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(&response_body[..], b"Alive");
}
