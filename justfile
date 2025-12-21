# Guts Development Justfile
# Run `just --list` to see available commands

# Default recipe
default:
    @just --list

# =============================================================================
# Building
# =============================================================================

# Build all crates in release mode
build:
    cargo build --workspace --release

# Build all crates in debug mode
build-debug:
    cargo build --workspace

# Build specific crate
build-crate CRATE:
    cargo build -p {{CRATE}}

# =============================================================================
# Testing
# =============================================================================

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run tests for a specific crate
test-crate CRATE:
    cargo test -p {{CRATE}}

# Run only unit tests (no integration tests)
test-unit:
    cargo test --workspace --lib

# Run only integration tests
test-integration:
    cargo test --workspace --test '*'

# Run tests with coverage (requires cargo-llvm-cov)
test-coverage:
    cargo llvm-cov --workspace --html
    @echo "Coverage report generated in target/llvm-cov/html/index.html"

# Run property-based tests specifically
test-proptest:
    cargo test --workspace -- proptest

# =============================================================================
# Linting & Formatting
# =============================================================================

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --workspace --all-targets

# Run clippy with strict settings
lint-strict:
    cargo clippy --workspace --all-targets -- -D warnings -D clippy::all -D clippy::pedantic

# Fix clippy warnings automatically where possible
lint-fix:
    cargo clippy --workspace --all-targets --fix --allow-staged

# =============================================================================
# All-in-one checks
# =============================================================================

# Run all checks (format, lint, test)
check: fmt-check lint test

# Run all checks and fix what can be fixed
check-fix: fmt lint-fix test

# Pre-commit check (recommended before committing)
pre-commit: fmt lint test
    @echo "All checks passed!"

# =============================================================================
# Documentation
# =============================================================================

# Generate documentation
doc:
    cargo doc --workspace --no-deps

# Generate and open documentation
doc-open:
    cargo doc --workspace --no-deps --open

# =============================================================================
# Running
# =============================================================================

# Run a local node on port 8080
run:
    cargo run --bin guts-node -- --api-addr 127.0.0.1:8080

# Run a local node in release mode
run-release:
    cargo run --release --bin guts-node -- --api-addr 127.0.0.1:8080

# Run CLI
cli *ARGS:
    cargo run --bin guts -- {{ARGS}}

# =============================================================================
# Docker & Devnet
# =============================================================================

# Start local devnet (5 nodes)
devnet-up:
    cd infra/docker && docker compose -f docker-compose.devnet.yml up -d

# Stop local devnet
devnet-down:
    cd infra/docker && docker compose -f docker-compose.devnet.yml down

# Show devnet logs
devnet-logs:
    cd infra/docker && docker compose -f docker-compose.devnet.yml logs -f

# Run E2E tests against devnet
devnet-test:
    ./infra/scripts/devnet-e2e-test.sh

# Build Docker image
docker-build:
    docker build -t guts-node -f infra/docker/Dockerfile .

# =============================================================================
# Fuzzing (requires nightly Rust and cargo-fuzz)
# =============================================================================

# List available fuzz targets
fuzz-list:
    @echo "Available fuzz targets:"
    @echo "  - fuzz_p2p_message"
    @echo "  - fuzz_pack_parser"
    @echo "  - fuzz_username_validation"
    @echo "  - fuzz_pktline"

# Run a fuzz target for 60 seconds
fuzz TARGET:
    cd fuzz && cargo +nightly fuzz run {{TARGET}} -- -max_total_time=60

# Run all fuzz targets briefly (10 seconds each)
fuzz-all:
    cd fuzz && cargo +nightly fuzz run fuzz_p2p_message -- -max_total_time=10
    cd fuzz && cargo +nightly fuzz run fuzz_pack_parser -- -max_total_time=10
    cd fuzz && cargo +nightly fuzz run fuzz_username_validation -- -max_total_time=10
    cd fuzz && cargo +nightly fuzz run fuzz_pktline -- -max_total_time=10

# =============================================================================
# Security
# =============================================================================

# Run security audit
audit:
    cargo audit

# Check for known vulnerabilities in dependencies
audit-deny:
    cargo deny check

# =============================================================================
# Cleanup
# =============================================================================

# Clean build artifacts
clean:
    cargo clean

# Clean fuzz corpus
clean-fuzz:
    rm -rf fuzz/corpus fuzz/artifacts

# =============================================================================
# Development helpers
# =============================================================================

# Watch and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x build

# Watch and test on changes
watch-test:
    cargo watch -x test

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree --workspace

# Show duplicate dependencies
deps-duplicates:
    cargo tree --workspace --duplicates

# =============================================================================
# Release
# =============================================================================

# Check if ready for release
release-check:
    @just fmt-check
    @just lint-strict
    @just test
    @just audit
    @echo "Ready for release!"

# Create a new version tag
release-tag VERSION:
    git tag -a v{{VERSION}} -m "Release v{{VERSION}}"
    git push origin v{{VERSION}}
