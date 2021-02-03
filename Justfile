#########
# Options

opt_fmt_check := "false"
opt_lint_err := "false"
opt_doc_open := "false"

#################
# Format and lint

# Check code style
fmt:
	cargo fmt --all {{ if opt_fmt_check == "true" { "-- --check" } else { "" } }}

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
	cargo test --all

# Execute tests with coverage analysis
cov:
	cargo tarpaulin --out Lcov

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
