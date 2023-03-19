# Build stage
FROM rust:buster as builder
WORKDIR /usr/src/app

RUN apt update \
    && apt install lld -y \
    && rustup toolchain install stable

COPY ./crates /usr/src/app/crates
COPY Cargo.lock /usr/src/app
COPY Cargo.toml /usr/src/app

RUN cargo build --release

# Bundle stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/github-scbot-cli github-scbot
RUN apt-get update && apt-get install libpq5 ca-certificates -y && rm -rf /var/lib/apt/lists/*
USER 1000

ENTRYPOINT ["/github-scbot"]
CMD ["server"]
