use rustatsu_sync::{configuration::Config, startup::Application};

pub struct TestApp {
    pub address: String,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let config = {
        let mut c = Config::new().expect("Failed to read configuration");
        c.application.port = 0;
        c
    };

    let application = Application::build(config.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application_port);
    tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address,
        api_client: client,
    }
}
