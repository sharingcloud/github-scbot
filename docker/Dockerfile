# Build stage
FROM clux/muslrust:stable AS builder
COPY . /volume
RUN cargo build --release

# Bundle stage
FROM scratch
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/github-scbot .
USER 1000
CMD ["./github-scbot", "server"]
