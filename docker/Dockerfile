# Build stage
FROM rust:1.52.1-buster as builder
WORKDIR /usr/src/app

COPY ./crates /usr/src/app/crates
COPY ./migrations /usr/src/app/migrations
COPY Cargo.lock /usr/src/app
COPY Cargo.toml /usr/src/app
COPY diesel.toml /usr/src/app

RUN cargo build --release

# Bundle stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/github-scbot .
RUN apt-get update && apt-get install libpq5 ca-certificates -y && rm -rf /var/lib/apt/lists/*
USER 1000
CMD ["./github-scbot", "server"]
