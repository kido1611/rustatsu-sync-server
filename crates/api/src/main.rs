use std::process::ExitCode;

use rustatsu_sync::{
    run,
    telemetry::{get_subscriber, init_subscriber},
};
use tracing_panic::panic_hook;

#[tokio::main]
async fn main() -> ExitCode {
    let subscriber = get_subscriber("rustatsu-sync".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    std::panic::set_hook(Box::new(panic_hook));

    match run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error.msg=%e, error.error_chain=?e, "Shutting down due to error");

            ExitCode::FAILURE
        }
    }
}
