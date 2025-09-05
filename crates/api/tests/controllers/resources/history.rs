use std::fs::File;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rustatsu_sync::{db::user_history::update_user_history, model::UserHistory};

use crate::AppStateTest;

#[tokio::test]
async fn should_be_error_when_accessed_list_user_history_without_auth() {
    let test_state = AppStateTest::new(false).await;

    let request = Request::builder()
        .uri("/resource/history")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_be_ok_when_accessed_list_user_history_with_auth() {
    let mut test_state = AppStateTest::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .uri("/resource/history")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserHistory = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.history.len(), 0);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_accessed_list_user_history_with_auth_and_user_have_data() {
    let example_file = File::open("tests/assets/user_history.json").unwrap();
    let user_history: UserHistory = serde_json::from_reader(example_file).unwrap();
    assert_eq!(user_history.history.len(), 15);

    let mut test_state = AppStateTest::new(true).await;

    let (user, token) = test_state.generate_jwt_with_user().await;

    let _ = update_user_history(&test_state.app_state.pool, user.id, user_history).await;

    let request = Request::builder()
        .uri("/resource/history")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserHistory = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.history.len(), 15);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_when_update_user_history_without_auth() {
    let test_state = AppStateTest::new(false).await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/history")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_be_error_when_update_user_history_without_data() {
    let mut test_state = AppStateTest::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/history")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let request = Request::builder()
        .method("POST")
        .uri("/resource/history")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_update_user_history_with_data() {
    let example_file = File::open("tests/assets/user_history.json").unwrap();
    let user_history: UserHistory = serde_json::from_reader(example_file).unwrap();
    assert_eq!(user_history.history.len(), 15);

    let mut test_state = AppStateTest::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/history")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::from(serde_json::to_string(&user_history).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserHistory = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.history.len(), 15);

    test_state.cleanup().await;
}
