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
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

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

#################
# Docker specific

# Build docker image
docker-build:
	@just docker-build-v {{ version }}

# Build nightly docker image
docker-build-nightly:
	@just docker-build-v nightly

# Build docker image with version
docker-build-v v:
	docker build --rm -t github-scbot:{{ v }} -f ./docker/Dockerfile .

# Build docker image using current branch
docker-build-b:
	#!/usr/bin/env bash
	set -euo pipefail
	BRANCH=`git branch --show-current`
	just build-docker-v "${BRANCH}"

# Tag Docker image
docker-tag-v v t:
	docker tag github-scbot:{{ v }} github-scbot:{{ t }}

# Tag Docker latest image with current version
docker-tag-latest:
	docker tag github-scbot:{{ version }} github-scbot:latest

# Push version to registry
docker-push-v v reg:
	docker tag github-scbot:{{ v }} {{ reg }}/github-scbot:{{ v }}
	docker push {{ reg }}/github-scbot:{{ v }}

docker-push-current reg:
	@just docker-push-v {{ version }} {{ reg }}

docker-push-nightly reg:
	@just docker-push-v nightly {{ reg }}

docker-push-latest reg:
	@just docker-push-v latest {{ reg }}
