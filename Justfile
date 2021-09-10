###########
# Variables

version := `cat ./crates/github_scbot_cli/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

#################
# Format and lint

# Check code style
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

#######
# Tests

# Execute tests
test:
	cargo test --lib

# Execute tests with coverage
test-cov:
	#!/usr/bin/env bash
	set -euo pipefail
	export CARGO_INCREMENTAL=0
	export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
	export RUSTDOCFLAGS='-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
	cargo test --all-features --no-fail-fast
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing --ignore "/*" --ignore "*/tests/*" -o ./target/debug/coverage/

#######
# Tools

# Set crates version
set-version v:
	ls -d crates/github_scbot_*/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"
	sed -i "s/github-scbot:\(.*\)/github-scbot:{{ v }}/" docker/docker-compose.yml

###############
# Documentation

# Generate docs
doc:
	cargo doc --no-deps

###############
# Build and run

# Build app
build:
	cargo build

# Run server
server:
	cargo run -- server

# Run dev-server
dev-server:
	cargo watch -x 'run -- server'
