FROM rust:1-alpine AS base

RUN apk --no-cache add musl-dev
RUN cargo install --locked cargo-chef
RUN apk --no-cache add mold sccache

ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache

#---
FROM base AS planner

WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

#---
FROM base AS builder

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/usr/local/cargo/git/ \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .

ENV CARGO_BUILD_TARGET_DIR=/tmp/cargo-target \
    SQLX_OFFLINE=true
RUN --mount=type=cache,target=/tmp/cargo-target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/usr/local/cargo/git/ \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo build --locked --release \
 && cp /tmp/cargo-target/release/jp-postal-code /bin/server

#---
FROM alpine
LABEL org.opencontainers.image.source=https://github.com/attmcojp/jp-postal-code

COPY --from=builder /bin/server /bin/

ENV HTTP_SERVER_ADDR=0.0.0.0:8000
EXPOSE 8000

CMD ["/bin/server"]
