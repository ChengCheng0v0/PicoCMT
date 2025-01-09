FROM alpine:3.21.2 AS builder

RUN apk add build-base openssl-dev pkgconfig cargo

COPY ./server /build/server

WORKDIR /build/server
RUN cargo build --release

FROM alpine:3.21.2

RUN apk add supervisor libgcc nginx

COPY ./docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

COPY ./docker/supervisord.conf /etc/supervisor/supervisord.conf
COPY ./docker/supervisord.only_server.conf /etc/supervisor/supervisord.only_server.conf

COPY --from=builder /build/server/target/release /app/server
RUN chmod +x /app/server/picocmt
COPY ./server/config.toml /app/server/config.toml

COPY ./client /app/client
COPY ./docker/nginx.conf /etc/nginx/nginx.conf

EXPOSE 3000 80

CMD ["/entrypoint.sh"]
