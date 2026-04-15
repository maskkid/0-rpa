# RPA Automation VM - Task Runner
# Install just: cargo install just
# Usage: just <recipe>

# Default recipe
default:
    @just --list

# ────────────────────────────
# Development
# ────────────────────────────

# Quick check: compile without producing binaries
dev:
    cargo check

# Run all tests
test:
    cargo test --workspace

# Run core + engine tests only (fast feedback)
test-core:
    cargo test -p rpa-core

test-engine:
    cargo test -p rpa-engine

test-fast: test-core test-engine

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run specific test by name
test-one test:
    cargo test --workspace "{{ test }}"

# ────────────────────────────
# Build
# ────────────────────────────

# Debug build
build:
    cargo build

# Release build (optimized)
build-release:
    cargo build --release

# Build server binary only
build-server:
    cargo build --release -p rpa-server

# ────────────────────────────
# Cross-compile: Windows
# ────────────────────────────

# Add Windows target if not already installed
setup-windows:
    rustup target add x86_64-pc-windows-gnu

# Build for Windows (GNU toolchain, cross-compile from macOS/Linux)
build-win: setup-windows
    cargo build --release --target x86_64-pc-windows-gnu

# Build for Windows MSVC (requires Windows host or CI)
build-win-msvc:
    cargo build --release --target x86_64-pc-windows-msvc

# Run Windows tests (only logic tests, platform tests need Windows)
test-win:
    cargo test --workspace --target x86_64-pc-windows-gnu -- --skip windows

# ────────────────────────────
# Code Quality
# ────────────────────────────

# Format code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Full quality check: fmt + clippy + test
check: fmt lint test

# ────────────────────────────
# Code Generation
# ────────────────────────────

# Generate gRPC Rust code from proto
gen-proto:
    cargo build -p rpa-api 2>/dev/null || true

# ────────────────────────────
# Run
# ────────────────────────────

# Run server in dev mode
run:
    cargo run -p rpa-server

# Run server in release mode
run-release:
    cargo run --release -p rpa-server

# ────────────────────────────
# Documentation
# ────────────────────────────

# Generate and open docs
doc:
    cargo doc --workspace --open

# Generate docs without opening
doc-build:
    cargo doc --workspace --no-deps

# ────────────────────────────
# Cleanup
# ────────────────────────────

# Clean all build artifacts
clean:
    cargo clean

# Remove orphaned lock + clean
reset: clean
    rm -f Cargo.lock
    cargo check

# ────────────────────────────
# CI helpers
# ────────────────────────────

# Full CI pipeline
ci: fmt-check lint test build-release

# CI for Windows target
ci-win: setup-windows
    cargo build --release --target x86_64-pc-windows-gnu
    cargo test --workspace --target x86_64-pc-windows-gnu -- --skip windows