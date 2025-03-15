use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use reqwest::StatusCode;
use rustatsu_sync::model::User;

use crate::{generate_app_state, generate_jwt_with_user, generate_response_custom_app_state};

#[tokio::test]
async fn should_be_ok() {
    let app_state = generate_app_state(true).await;

    let (user, token) = generate_jwt_with_user(&app_state).await;

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

    let response_body = response.into_body().collect().await.unwrap().to_bytes();

    let user_result: User = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(user.id, user_result.id);
    assert_eq!(user.email, user_result.email);
    assert_eq!(user.nickname, user_result.nickname);
}
