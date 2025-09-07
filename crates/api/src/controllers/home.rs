#[tracing::instrument(name = "request::home", skip_all)]
pub async fn index() -> &'static str {
    "Alive"
}
