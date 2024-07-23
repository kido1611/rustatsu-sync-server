use crate::spawn_app;

#[tokio::test]
async fn home_page_should_return_ok_and_alive() {
    let app = spawn_app().await;

    let response = app
        .api_client
        .get(format!("{}/", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());

    let html_page = response.text().await.unwrap();
    assert!(html_page.contains("Alive"));
}
