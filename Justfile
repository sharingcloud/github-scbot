set dotenv-load := false
version := `cat ./crates/github-scbot-cli/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

_default:
	@just -l

# Check code style
fmt:
	cargo +nightly fmt --all -- \
		--unstable-features \
		--config \
			imports_granularity=Crate,\
			group_imports=StdExternalCrate

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

# Test with coverage
test-cov:
	rm -rf .cov
	RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE=".cov/test-%p-%m.profraw" cargo build
	RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE=".cov/test-%p-%m.profraw" cargo test
	grcov . --binary-path ./target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "/*" -o lcov.info

# Build HTML coverage
test-cov-html:
	rm -rf .cov
	rm -rf htmlcov
	RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE=".cov/test-%p-%m.profraw" cargo build
	RUSTFLAGS="-Cinstrument-coverage" LLVM_PROFILE_FILE=".cov/test-%p-%m.profraw" cargo test
	grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing --ignore "/*" -o htmlcov

# Set crates version
set-version v:
	ls -d crates/github-scbot-*/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"

# Show version
show-version:
	@echo {{ version }}

# Run server (debug)
run-server:
	RUST_LOG=info,github_scbot=debug,sqlx=warn cargo run -q -- server

# Run server (release)
run-server-release:
	RUST_LOG=info,github_scbot=debug,sqlx=warn cargo run -q --release -- server

# Run server (watch)
run-server-watch:
	RUST_LOG=info,github_scbot=debug,sqlx=warn cargo watch -x "run -- server"

# Build Docker image
docker-build:
	docker build --rm -t github-scbot:v{{ version }} -f ./docker/Dockerfile .

# Push Docker image
docker-push reg:
	docker push {{ reg }}/github-scbot:v{{ version }}
