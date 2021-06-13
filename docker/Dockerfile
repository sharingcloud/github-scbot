# Build stage
FROM rust:1.52.1-buster as builder
WORKDIR /usr/src/app

RUN rustup toolchain install nightly-2021-06-04 --profile default && rustup default nightly-2021-06-04

COPY ./crates /usr/src/app/crates
COPY ./migrations /usr/src/app/migrations
COPY Cargo.lock /usr/src/app
COPY Cargo.toml /usr/src/app
COPY diesel.toml /usr/src/app

RUN cargo build --release

# Bundle stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/github-scbot-cli github-scbot
RUN apt-get update && apt-get install libpq5 ca-certificates -y && rm -rf /var/lib/apt/lists/*
USER 1000

ENTRYPOINT ["/github-scbot"]
CMD ["server"]
