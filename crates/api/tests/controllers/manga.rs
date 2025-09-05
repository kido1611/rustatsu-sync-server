use axum::{body::Body, http::Request, http::StatusCode};
use http_body_util::BodyExt;
use rustatsu_sync::model::Manga;

use crate::{AppStateTest, insert_fake_manga};

#[tokio::test]
async fn index_should_be_ok_with_manga_is_empty() {
    let mut test_state = AppStateTest::new(true).await;

    let request = Request::builder()
        .uri("/manga")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(!response_body.is_empty());

    let mangas: Vec<Manga> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(mangas.len(), 0);

    test_state.cleanup().await;
}

#[tokio::test]
async fn index_should_be_ok_with_manga_not_empty() {
    let mut test_state = AppStateTest::new(true).await;

    insert_fake_manga(&test_state.app_state.pool, None).await;
    insert_fake_manga(&test_state.app_state.pool, None).await;

    let request = Request::builder()
        .uri("/manga")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let mangas: Vec<Manga> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(mangas.len(), 2);
    assert_eq!(mangas[0].tags.len(), 2);

    test_state.cleanup().await;
}

#[tokio::test]
async fn index_should_be_ok_when_accessed_with_query() {
    let mut test_state = AppStateTest::new(true).await;

    insert_fake_manga(&test_state.app_state.pool, Some(0)).await;
    insert_fake_manga(&test_state.app_state.pool, None).await;
    insert_fake_manga(&test_state.app_state.pool, None).await;

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?limit=1")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let mangas: Vec<Manga> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(mangas.len(), 1);

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?offset=10")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let mangas: Vec<Manga> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(mangas.len(), 0);

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?offset=1&limit=2")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    let mangas: Vec<Manga> = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(mangas.len(), 1);

    test_state.cleanup().await;
}

#[tokio::test]
async fn index_should_be_error_when_query_invalid() {
    let mut test_state = AppStateTest::new(true).await;

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?offset=-1")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?limit=-1")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // -----------------------------------------------------------------------------
    let request = Request::builder()
        .uri("/manga?offset=-1&limit=-1")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    test_state.cleanup().await;
}

#[tokio::test]
async fn show_should_be_ok_when_manga_is_exist() {
    let mut test_state = AppStateTest::new(true).await;

    let manga_id = insert_fake_manga(&test_state.app_state.pool, Some(3)).await;

    let request = Request::builder()
        .uri(format!("/manga/{}", manga_id))
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response_body = response.into_body().collect().await.unwrap().to_bytes();
    assert!(!response_body.is_empty());

    let manga: Manga = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(manga.tags.len(), 3);

    test_state.cleanup().await;
}

#[tokio::test]
async fn show_should_be_error_when_manga_is_missing() {
    let mut test_state = AppStateTest::new(true).await;

    let request = Request::builder()
        .uri("/manga/-99999999999")
        .body(Body::empty())
        .unwrap();
    let response = test_state.generate_response(request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    test_state.cleanup().await;
}
