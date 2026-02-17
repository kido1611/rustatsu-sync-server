# ================================================================================= RUST Section
FROM clux/muslrust:stable AS chef
ARG TARGETARCH
RUN case "$TARGETARCH" in \
  amd64)  echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
  arm64)  echo "aarch64-unknown-linux-musl" > /tmp/target ;; \
  *) echo "unsupported arch: $TARGETARCH" && exit 1 ;; \
esac
RUN rustup target add "$(cat /tmp/target)"
USER root
RUN cargo install cargo-chef
WORKDIR /app

# ================================================================================= CHEF PREPARE
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ================================================================================= BUILD APP
FROM chef AS builder
ARG TARGETARCH

ENV SQLX_OFFLINE=true

COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,id=cargo-registry-${TARGETARCH},target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git-${TARGETARCH},target=/usr/local/cargo/git \
    cargo chef cook --release --target "$(cat /tmp/target)" --recipe-path recipe.json

COPY . .

RUN --mount=type=cache,id=cargo-registry-${TARGETARCH},target=/usr/local/cargo/registry \
    --mount=type=cache,id=cargo-git-${TARGETARCH},target=/usr/local/cargo/git \
    cargo build --release --target "$(cat /tmp/target)" && \
    cp "/app/target/$(cat /tmp/target)/release/rustatsu-sync" "/app/rustatsu-sync"

# ================================================================================= RUNTIME
FROM gcr.io/distroless/static-debian13:nonroot AS runtime

LABEL org.opencontainers.image.title=rustatsu-sync \
  org.opencontainers.image.description="Kotatsu sync server alternative written in Rust. Used for personal project." \
  org.opencontainers.image.url=https://github.com/kido1611/rustatsu-sync-server \
  org.opencontainers.image.source=https://github.com/kido1611/rustatsu-sync-server \
  org.opencontainers.image.licenses=GPL-3.0 \ 
  org.opencontainers.image.vendor="Muhammad Abdusy Syukur"

WORKDIR /app

COPY --from=builder /app/rustatsu-sync /app/
COPY configuration ./configuration
COPY migrations ./migrations

USER nonroot

ENV APP_ENVIRONMENT=production

EXPOSE 8080

ENTRYPOINT ["./rustatsu-sync"]
