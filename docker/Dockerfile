# Build stage
FROM rust:1.51.0-buster as builder
WORKDIR /usr/src/app
COPY . /usr/src/app
RUN cargo build --release

# Bundle stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/github-scbot .
USER 1000
CMD ["./github-scbot", "server"]
