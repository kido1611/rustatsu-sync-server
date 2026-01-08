pub trait TokenGenerator: Send + Sync {
    fn generate(&self, length: usize) -> String;
}
