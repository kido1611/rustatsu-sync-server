#[tracing::instrument(name = "Index Page")]
pub async fn index() -> &'static str {
    "Alive"
}
