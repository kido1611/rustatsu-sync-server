[doc("Show this help message")]
help:
  @just --list

[doc("Generate SQLX metadata")]
prepare:
  @echo "Generate SQLX metadata"
  @cargo sqlx prepare --workspace
  @cargo sqlx prepare --workspace -- --tests

[doc("Running tests")]
test:
  @echo "Running tests"
  @cargo test

[doc("Running test and check code coverage")]
tarpaulin:
  @echo "Running test and check code coverage"
  @cargo tarpaulin
