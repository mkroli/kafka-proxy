FROM rust:alpine3.17 AS builder
WORKDIR /usr/src/kafka-proxy
RUN apk add --no-cache musl-dev openssl-dev zstd-dev cyrus-sasl-dev cmake make g++
COPY . .
RUN RUSTFLAGS=-Ctarget-feature=-crt-static cargo install --all-features --path . --root /tmp

FROM alpine:3.17
RUN apk add --no-cache zstd zstd-libs cyrus-sasl
COPY --from=builder /tmp/bin/kafka-proxy /usr/bin/kafka-proxy
ENTRYPOINT ["/usr/bin/kafka-proxy"]
