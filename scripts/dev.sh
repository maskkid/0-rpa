#!/usr/bin/env bash
# RPA Automation VM - Development Helper Script
# Usage: ./scripts/dev.sh <command>
# This script works without `just` installed.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

COMMAND="${1:-help}"
shift 2>/dev/null || true

case "$COMMAND" in
    dev|check)
        info "Checking compilation..."
        cargo check
        ;;
    build)
        info "Building workspace..."
        cargo build
        ;;
    build-release)
        info "Building release..."
        cargo build --release
        ;;
    build-win)
        info "Setting up Windows cross-compile target..."
        rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
        info "Building for Windows (x86_64-pc-windows-gnu)..."
        cargo build --release --target x86_64-pc-windows-gnu
        ;;
    build-win-msvc)
        info "Building for Windows MSVC..."
        cargo build --release --target x86_64-pc-windows-msvc
        ;;
    test)
        info "Running all tests..."
        cargo test --workspace
        ;;
    test-core)
        info "Running rpa-core tests..."
        cargo test -p rpa-core
        ;;
    test-engine)
        info "Running rpa-engine tests..."
        cargo test -p rpa-engine
        ;;
    test-fast)
        info "Running fast tests (core + engine)..."
        cargo test -p rpa-core -p rpa-engine
        ;;
    fmt)
        info "Formatting code..."
        cargo fmt --all
        ;;
    fmt-check)
        info "Checking formatting..."
        cargo fmt --all -- --check
        ;;
    lint)
        info "Running clippy..."
        cargo clippy --workspace --all-targets -- -D warnings
        ;;
    check)
        info "Running full quality check..."
        cargo fmt --all -- --check
        cargo clippy --workspace --all-targets -- -D warnings
        cargo test --workspace
        ;;
    run)
        info "Running server..."
        cargo run -p rpa-server
        ;;
    run-release)
        info "Running server (release)..."
        cargo run --release -p rpa-server
        ;;
    clean)
        info "Cleaning build artifacts..."
        cargo clean
        ;;
    doc)
        info "Building documentation..."
        cargo doc --workspace --no-deps
        ;;
    setup)
        info "Installing tools..."
        rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
        info "Tools installed. Run 'cargo install just' for task runner."
        ;;
    help|*)
        cat << 'EOF'
RPA Automation VM - Development Helper

Usage: ./scripts/dev.sh <command>

Commands:
  dev             Quick compile check (cargo check)
  build           Debug build
  build-release   Release build
  build-win       Cross-compile for Windows (GNU)
  build-win-msvc  Build for Windows MSVC
  test            Run all tests
  test-core       Run rpa-core tests
  test-engine     Run rpa-engine tests
  test-fast       Run core + engine tests
  fmt             Format code
  fmt-check       Check formatting
  lint            Run clippy
  check           Full quality check (fmt + lint + test)
  run             Run server (debug)
  run-release     Run server (release)
  clean           Clean build artifacts
  doc             Build documentation
  setup           Install required tools
  help            Show this help

Recommended: Install 'just' for richer task runner (cargo install just)
Then use: just <recipe>
EOF
        ;;
esac