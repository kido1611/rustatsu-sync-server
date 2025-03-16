FROM rust:1.85.0-slim-bookworm AS builder
WORKDIR /app
COPY . /app
RUN ls -la /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/rustatsu-sync /app
COPY configuration configuration
COPY migrations migrations

ENV APP_ENVIRONMENT=production
EXPOSE 8000

ENTRYPOINT ["./rustatsu-sync"]
