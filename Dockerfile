FROM --platform=$BUILDPLATFORM lukemathwalker/cargo-chef:latest-rust-1.86-alpine AS chef

ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /app

# -------------------------------------------------------------------------------------------------------------------------
FROM chef AS planner

COPY . . 
RUN cargo chef prepare --recipe-path recipe.json

# -------------------------------------------------------------------------------------------------------------------------
FROM chef AS builder

COPY --from=planner /app/recipe.json ./recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

ENV SQLX_OFFLINE=true

COPY src ./src
COPY .sqlx ./.sqlx
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY configuration ./configuration
COPY migrations ./migrations

RUN case "$TARGETPLATFORM" in \
  "linux/amd64") echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
  "linux/arm64") echo "aarch64-unknown-linux-musl" > /tmp/target ;; \
  # apk add --no-cache gcc-aarch64-none-elf=14.2.0-r0 && \
  # # Create symlinks for cross-compiler
  # ln -s /usr/bin/aarch64-none-elf-gcc /usr/local/bin/aarch64-linux-musl-gcc && \
  # ln -s /usr/bin/aarch64-none-elf-ar /usr/local/bin/aarch64-linux-musl-ar ;; \
  *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
  esac

RUN apk add --no-cache musl-dev=1.2.5-r9 && \
  rustup target add "$(cat /tmp/target)"

RUN cargo build --release --target="$(cat /tmp/target)" && \
  cp "/app/target/$(cat /tmp/target)/release/rustatsu-sync" "/app/rustatsu-sync"

# -------------------------------------------------------------------------------------------------------------------------
FROM gcr.io/distroless/static-debian12:nonroot AS runtime

LABEL org.opencontainers.image.title=rustatsu-sync \
  org.opencontainers.image.description="Kotatsu sync server alternative written in Rust. Used for personal project." \
  org.opencontainers.image.url=https://github.com/kido1611/rustatsu-sync-server \
  org.opencontainers.image.source=https://github.com/kido1611/rustatsu-sync-server \
  org.opencontainers.image.licenses=GPL-3.0 \ 
  org.opencontainers.image.vendor="Muhammad Abdusy Syukur"

WORKDIR /app

COPY --from=builder /app/rustatsu-sync .
COPY configuration ./configuration
COPY migrations ./migrations

ENV APP_ENVIRONMENT=production

EXPOSE 8000

USER nonroot

ENTRYPOINT ["./rustatsu-sync"]
