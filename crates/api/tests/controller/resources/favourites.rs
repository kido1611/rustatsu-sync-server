use std::fs::File;

use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use shared::model::{request::UserFavouriteRequest, response::UserFavouriteResponse};

use crate::common::TestState;

#[tokio::test]
async fn should_be_error_when_accessed_list_user_favourites_without_auth() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .uri("/resource/favourites")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_be_ok_when_accessed_list_user_favourites_with_auth() {
    let mut test_state = TestState::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .uri("/resource/favourites")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserFavouriteResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.favourites.len(), 0);
    assert_eq!(result.favourite_categories.len(), 0);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_ok_when_accessed_list_user_favourites_with_auth_and_user_have_data() {
    let example_file = File::open("tests/assets/user_favourites.json").unwrap();
    let user_favourite: UserFavouriteRequest = serde_json::from_reader(example_file).unwrap();
    assert_eq!(user_favourite.favourites.len(), 24);
    assert_eq!(user_favourite.favourite_categories.len(), 2);

    let mut test_state = TestState::new(true).await;

    let (user, token) = test_state.generate_jwt_with_user().await;

    test_state
        .app_state
        .insert_user_favourite_resource_usecase
        .execute(user.id, user_favourite.into())
        .await
        .expect("failed to insert dummy user favourite data");

    let request = Request::builder()
        .uri("/resource/favourites")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::empty())
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserFavouriteResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.favourites.len(), 24);
    assert_eq!(result.favourite_categories.len(), 2);

    test_state.cleanup().await;
}

#[tokio::test]
async fn should_be_error_when_update_user_favourites_without_auth() {
    let test_state = TestState::new(false).await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/favourites")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_be_error_when_update_user_favourites_without_data() {
    let mut test_state = TestState::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/favourites")
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
        .uri("/resource/favourites")
        .header(http::header::CONTENT_TYPE, "application/json")
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
async fn should_be_ok_when_update_user_favourites_with_data() {
    let example_file = File::open("tests/assets/user_favourites.json").unwrap();
    let user_favourite: UserFavouriteRequest = serde_json::from_reader(example_file).unwrap();
    assert_eq!(user_favourite.favourites.len(), 24);
    assert_eq!(user_favourite.favourite_categories.len(), 2);

    let mut test_state = TestState::new(true).await;

    let (_, token) = test_state.generate_jwt_with_user().await;

    let request = Request::builder()
        .method("POST")
        .uri("/resource/favourites")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            axum::http::header::AUTHORIZATION,
            format!("bearer {}", token),
        )
        .body(Body::from(serde_json::to_string(&user_favourite).unwrap()))
        .unwrap();

    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let result: UserFavouriteResponse = serde_json::from_slice(&response_body).unwrap();

    assert_eq!(result.favourites.len(), user_favourite.favourites.len());
    assert_eq!(
        result.favourite_categories.len(),
        user_favourite.favourite_categories.len()
    );

    test_state.cleanup().await;
}
