FROM alpine:3.21.2 AS builder

RUN apk add build-base openssl-dev pkgconfig cargo

COPY ./server /build/server

WORKDIR /build/server
RUN cargo build --release

FROM alpine:3.21.2

RUN apk add libgcc

COPY --from=builder /build/server/target/release /app/server
RUN chmod +x /app/server/picocmt
COPY ./server/config.toml /app/server/config.toml

EXPOSE 3000

CMD ["/app/server/picocmt", "--config-path", "/app/server/config.toml"]
