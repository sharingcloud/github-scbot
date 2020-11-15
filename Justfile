build:
	cargo build

lint:
	touch src/lib.rs && cargo clippy --all --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

test:
	cargo test --all