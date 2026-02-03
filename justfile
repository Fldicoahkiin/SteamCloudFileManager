# Format all code
fmt:
	cargo fmt --all && fama "**/*.yml" "**/*.yaml" "**/*.toml" "**/*.md" "**/*.json"

# Check formatting (for CI)
fmt-check:
	cargo fmt --all -- --check
	fama "**/*.yml" "**/*.yaml" "**/*.toml" "**/*.md" "**/*.json"
	git diff --exit-code

# Run all checks (format, clippy, tests)
check: fmt-check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features

# Build release version
build:
	cargo build --release
