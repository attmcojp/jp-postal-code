FROM rust:1-alpine AS build-stage

RUN apk --no-cache add musl-dev mold

WORKDIR /app
COPY . .

ENV CARGO_BUILD_TARGET_DIR=/tmp/cargo-target \
    SQLX_OFFLINE=true
RUN --mount=type=cache,target=/tmp/cargo-target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release \
 && cp /tmp/cargo-target/release/jp-postal-code /bin/server

#---
FROM alpine
LABEL org.opencontainers.image.source=https://github.com/attmcojp/jp-postal-code

COPY --from=build-stage /bin/server /bin/

ENV HTTP_SERVER_ADDR=0.0.0.0:8000
EXPOSE 8000

CMD ["/bin/server"]
