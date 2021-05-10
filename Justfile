#########
# Options

opt_fmt_check := "false"
opt_lint_err := "false"
opt_doc_open := "false"

###########
# Variables

version := `cat ./crates/github_scbot/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

#################
# Format and lint

# Check code style
fmt:
	cargo +nightly fmt --all {{ if opt_fmt_check == "true" { "-- --check" } else { "" } }}

# Check code style and error if changes are needed
fmt-check:
	@just opt_fmt_check=true fmt

# Lint files
lint:
	ls -d crates/*/src/lib.rs | xargs touch && cargo clippy --tests {{ if opt_lint_err == "true" { "-- -D warnings" } else { "" } }}

# Lint files and error on warnings
lint-err:
	@just opt_lint_err=true lint

#######
# Tests

# Execute tests
test:
	cargo test --lib

# Execute tests with coverage
test-cov:
	cargo tarpaulin --out Html

#######
# Tools

# Set crates version
set-version v:
	ls -d crates/github_scbot_*/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"
	ls -d crates/github_scbot/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"
	sed -i "s/github-scbot:\(.*\)/github-scbot:{{ v }}/" docker/docker-compose.yml

###############
# Documentation

# Generate docs
doc:
	cargo doc --no-deps {{ if opt_doc_open == "true" { "--open" } else { "" } }}

# Generate docs and open in browser
doc-open:
	@just opt_doc_open=true doc

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

# Push current version and latest image to registry
docker-push reg:
	docker tag github-scbot:{{ version }} {{ reg }}/github-scbot:{{ version }}
	docker tag github-scbot:latest {{ reg }}/github-scbot:latest
	docker push {{ reg }}/github-scbot:{{ version }}
	docker push {{ reg }}/github-scbot:latest
