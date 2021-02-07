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
	TEST_DATABASE_URL=postgresql://user:pass@localhost:5432/test-bot cargo test --all

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

# Build release
export:
	@echo "Exporting github-scbot {{ version }} ..."
	@cargo build --release
	@mkdir -p ./export
	@cp ./target/release/github-scbot ./export
	@cp ./.env.sample ./export
	@cp ./README.md ./export
	@tar -zcf ./export/github-scbot-{{ version }}.tar.gz ./export/github-scbot ./export/.env.sample ./export/README.md
	@echo "Exported to ./export/github-scbot-{{ version }}.tar.gz"

# Run server
server:
	cargo run -- server
