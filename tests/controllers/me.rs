use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use reqwest::StatusCode;
use rustatsu_sync::model::User;

use crate::AppStateTest;

#[tokio::test]
async fn should_be_ok() {
    let mut test_state = AppStateTest::new(true).await;

    let (user, token) = test_state.generate_jwt_with_user().await;

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

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let user_result: User = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(user.id, user_result.id);
    assert_eq!(user.email, user_result.email);
    assert_eq!(user.nickname, user_result.nickname);

    test_state.cleanup().await;
}
