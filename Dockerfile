FROM rust:1.84-alpine3.21 AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev

COPY . .
RUN SQLX_OFFLINE=true cargo build --release

FROM alpine:3.21
COPY --from=builder /app/target/release/jp-postal-code .
CMD ["/jp-postal-code"]
