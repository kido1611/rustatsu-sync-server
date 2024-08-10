FROM rust:1.80.1-slim-bookworm AS builder
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime
COPY --from=builder /app/target/release/rustatsu-sync /
COPY configuration configuration

ENV APP_ENVIRONMENT=production
EXPOSE 8000

ENTRYPOINT ["./rustatsu-sync"]
