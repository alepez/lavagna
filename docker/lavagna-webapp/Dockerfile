################################################################################
# Build the web app

FROM rust:1.70 as builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install -f wasm-bindgen-cli

WORKDIR /src
ADD tools tools 
ADD assets assets 
ADD Cargo.lock Cargo.lock
ADD Cargo.toml Cargo.toml
ADD src src 
ADD www www

RUN ./tools/build-web

################################################################################
# Build a lightweight http server

FROM alpine:3.13.2 AS server-builder
RUN apk add gcc musl-dev make perl
RUN wget https://busybox.net/downloads/busybox-1.35.0.tar.bz2 \
  && tar xf busybox-1.35.0.tar.bz2 \
  && mv /busybox-1.35.0 /busybox

WORKDIR /busybox
COPY docker/lavagna-webapp/.config .
RUN make && make install
RUN adduser -D static

################################################################################
# Merge the web app and the http server

FROM scratch
EXPOSE 3000
COPY --from=server-builder /etc/passwd /etc/passwd
COPY --from=server-builder /busybox/_install/bin/busybox /
USER static
WORKDIR /home/static
COPY docker/lavagna-webapp/httpd.conf .

# Copy the static website
COPY --from=builder /src/www ./

CMD ["/busybox", "httpd", "-f", "-v", "-p", "8000", "-c", "httpd.conf"]
