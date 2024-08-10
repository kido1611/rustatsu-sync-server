#[tracing::instrument(name = "get index route")]
pub async fn index() -> &'static str {
    "Alive"
}
