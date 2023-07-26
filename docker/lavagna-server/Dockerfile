FROM rust:1.70 as builder

WORKDIR /src

ADD https://github.com/johanhelsing/matchbox/archive/refs/tags/v0.6.1.tar.gz /src
RUN tar -xzf v0.6.1.tar.gz
RUN cd matchbox-0.6.1 && cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/matchbox-0.6.1/target/release/matchbox_server /usr/local/bin/matchbox_server
CMD ["matchbox_server"]
