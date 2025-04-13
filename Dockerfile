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

SHELL ["/bin/ash", "-eo", "pipefail", "-c"]

RUN case "$TARGETPLATFORM" in \
  "linux/amd64") echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
  "linux/arm64") echo "aarch64-unknown-linux-musl" > /tmp/target ;; \
  *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
  esac

# # Install the correct cross-compilation tools based on target
# RUN apk add --no-cache musl-dev=1.2.5-r9  && \
#   if [ "$TARGETPLATFORM" = "linux/arm64" ]; then \
#   wget -qO- https://musl.cc/aarch64-linux-musl-cross.tgz | tar -xz -C /opt && \
#   ln -s /opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc /usr/local/bin/ && \
#   ln -s /opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-g++ /usr/local/bin/ && \
#   ln -s /opt/aarch64-linux-musl-cross/bin/aarch64-linux-musl-ar /usr/local/bin/ && \
#   # Create necessary symlinks and configuration
#   mkdir -p ~/.cargo && \
#   echo '[target.aarch64-unknown-linux-musl]' >> ~/.cargo/config && \
#   echo 'linker = "aarch64-linux-musl-gcc"' >> ~/.cargo/config && \
#   echo 'rustflags = ["-C", "target-feature=+crt-static"]' >> ~/.cargo/config.toml && \
#   export CC_aarch64_unknown_linux_musl="aarch64-linux-musl-gcc" && \
#   export CXX_aarch64_unknown_linux_musl="aarch64-linux-musl-g++" && \
#   export AR_aarch64_unknown_linux_musl="aarch64-linux-musl-ar" && \
#   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="aarch64-linux-musl-gcc"; \
#   fi

ENV SQLX_OFFLINE=true

COPY --from=planner /app/recipe.json ./recipe.json
# Build dependencies - this is the caching Docker layer!
RUN rustup target add "$(cat /tmp/target)" && \
  cargo chef cook --release --recipe-path recipe.json --target "$(cat /tmp/target)"

COPY src ./src
COPY .sqlx ./.sqlx
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY configuration ./configuration
COPY migrations ./migrations

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
