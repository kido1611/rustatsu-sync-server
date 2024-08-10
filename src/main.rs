use rustatsu_sync::{
    configuration::read_config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("rustatsu-sync".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = read_config().expect("Failed to read configuration.");

    let application = Application::build(config)
        .await
        .expect("Failed creating server.");

    println!(
        "Started at http://{}:{}",
        application.host(),
        application.port()
    );

    application.run_until_stopped().await.unwrap();
}
