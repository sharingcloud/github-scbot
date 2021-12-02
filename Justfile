set dotenv-load := false
version := `cat ./crates/github_scbot_cli/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

_default:
	@just -l

# # Check code style
fmt:
	cargo fmt --all

# Check code style and error if changes are needed
fmt-check:
	cargo fmt --all -- --check

# Lint files
lint:
	cargo clippy --all-features --all --tests

# Lint files and error on warnings
lint-err:
	cargo clippy --all-features --all --tests -- -D warnings

# Debug build
build:
	cargo build --all

# Release build
build-release:
	cargo build --all --release

# Test
test:
	cargo test --all

test-coverage:
	cargo tarpaulin -- --test-threads 1

# Set crates version
set-version v:
	ls -d crates/github_scbot_*/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"

# Show version
show-version:
	@echo {{ version }}

# Run server (debug)
run-server:
	#!/bin/bash
	RUST_LOG=info,github_scbot=debug cargo run -q -- server

# Run server (release)
run-server-release:
	#!/bin/bash
	RUST_LOG=info,github_scbot=debug cargo run -q --release -- server

# Run server (watch)
run-server-watch:
	#!/bin/bash
	RUST_LOG=info,github_scbot=debug cargo watch -x "run -- server"

# Build Docker image
docker-build:
	docker build --rm -t github-scbot:{{ version }} -f ./docker/Dockerfile .

# Push Docker image
docker-push reg:
	docker push {{ reg }}/github-scbot:{{ version }}
