FROM rust:1.86.0-alpine AS builder
WORKDIR /app
COPY src ./src
COPY .sqlx ./.sqlx
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY configuration ./configuration
COPY migrations ./migrations
RUN apk add --no-cache musl-dev=1.2.5-r9 && \
  cargo build --release --target=x86_64-unknown-linux-musl

# FROM gcr.io/distroless/static-debian12 AS runtime
FROM gcr.io/distroless/static-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rustatsu-sync .
COPY configuration ./configuration
COPY migrations ./migrations

ENV APP_ENVIRONMENT=production
EXPOSE 8000

USER nonroot

ENTRYPOINT ["./rustatsu-sync"]
