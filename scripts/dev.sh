#!/usr/bin/env bash
# RPA Automation VM - Development Helper Script
# Usage: ./scripts/dev.sh <command>
# This script works without `just` installed.

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
desktop() { echo -e "${CYAN}[DESKTOP]${NC} $*"; }

COMMAND="${1:-help}"
shift 2>/dev/null || true

case "$COMMAND" in
    dev|check)
        info "Checking compilation..."
        cargo check
        ;;
    # ────────────────────────────
    # Desktop Client (Tauri)
    # ────────────────────────────
    desktop-dev)
        desktop "Starting Tauri desktop client with hot-reload..."
        cd apps/desktop && yarn tauri:dev
        ;;
    desktop-fe)
        desktop "Starting frontend dev server only (Vite HMR)..."
        cd apps/desktop && yarn dev
        ;;
    desktop-build)
        desktop "Building Tauri desktop client for production..."
        cd apps/desktop && yarn tauri:build
        ;;
    desktop-build-fe)
        desktop "Building frontend only..."
        cd apps/desktop && yarn build
        ;;
    desktop-check)
        desktop "Checking desktop Rust backend..."
        cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
        ;;
    desktop-install)
        desktop "Installing frontend dependencies..."
        cd apps/desktop && yarn
        ;;
    desktop-init)
        info "Initializing desktop project (first-time setup)..."
        cd apps/desktop && yarn
        desktop "Desktop client initialized. Run './scripts/dev.sh desktop-dev' to start development."
        ;;
    # ────────────────────────────
    # Build
    # ────────────────────────────
    build)
        info "Building workspace..."
        cargo build
        ;;
    build-release)
        info "Building release..."
        cargo build --release
        ;;
    build-server)
        info "Building server binary..."
        cargo build --release -p rpa-server
        ;;
    build-desktop)
        info "Building desktop binary..."
        cargo build --release -p rpa-desktop
        ;;
    # ────────────────────────────
    # Cross-compile: Windows
    # ────────────────────────────
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
    # ────────────────────────────
    # Testing
    # ────────────────────────────
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
    test-desktop)
        info "Running desktop backend tests..."
        cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
        ;;
    # ────────────────────────────
    # Code Quality
    # ────────────────────────────
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
        cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml -- -D warnings
        ;;
    check)
        info "Running full quality check..."
        cargo fmt --all -- --check
        cargo clippy --workspace --all-targets -- -D warnings
        cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml -- -D warnings
        cargo test --workspace
        ;;
    # ────────────────────────────
    # Run
    # ────────────────────────────
    run)
        info "Running server..."
        cargo run -p rpa-server
        ;;
    run-release)
        info "Running server (release)..."
        cargo run --release -p rpa-server
        ;;
    # ────────────────────────────
    # Code Generation
    # ────────────────────────────
    gen-proto)
        info "Generating gRPC Rust code..."
        cargo build -p rpa-api 2>/dev/null || true
        ;;
    # ────────────────────────────
    # Documentation
    # ────────────────────────────
    doc)
        info "Building documentation..."
        cargo doc --workspace --open
        ;;
    doc-build)
        info "Building documentation..."
        cargo doc --workspace --no-deps
        ;;
    # ────────────────────────────
    # Cleanup
    # ────────────────────────────
    clean)
        info "Cleaning build artifacts..."
        cargo clean
        ;;
    clean-desktop)
        info "Cleaning desktop frontend artifacts..."
        rm -rf apps/desktop/dist apps/desktop/node_modules/.vite
        ;;
    reset)
        info "Removing orphaned lock + cleaning..."
        cargo clean
        rm -f Cargo.lock
        cargo check
        ;;
    # ────────────────────────────
    # CI helpers
    # ────────────────────────────
    ci)
        cargo fmt --all -- --check
        cargo clippy --workspace --all-targets -- -D warnings
        cargo clippy --manifest-path apps/desktop/src-tauri/Cargo.toml -- -D warnings
        cargo test --workspace
        cargo build --release
        ;;
    ci-win)
        rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
        cargo build --release --target x86_64-pc-windows-gnu
        cargo test --workspace --target x86_64-pc-windows-gnu -- --skip windows
        ;;
    # ────────────────────────────
    # Setup
    # ────────────────────────────
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
  dev               Quick compile check (cargo check)
  build             Debug build
  build-release     Release build
  build-server      Build server binary only
  build-desktop     Build desktop binary only
  build-win         Cross-compile for Windows (GNU)
  build-win-msvc    Build for Windows MSVC
  test              Run all tests
  test-core         Run rpa-core tests
  test-engine       Run rpa-engine tests
  test-fast         Run core + engine tests
  test-desktop      Run desktop backend tests
  fmt               Format code
  fmt-check         Check formatting
  lint              Run clippy (workspace + desktop)
  check             Full quality check (fmt + lint + test)
  run               Run server (debug)
  run-release       Run server (release)
  gen-proto         Generate gRPC Rust code
  doc               Build and open docs
  doc-build         Build docs without opening
  clean             Clean build artifacts
  clean-desktop     Clean desktop frontend artifacts
  reset             Remove lock + clean + recheck
  setup             Install required tools

Desktop Commands:
  desktop-dev       Start Tauri dev mode (hot-reload frontend + Rust)
  desktop-fe        Start Vite frontend dev server only
  desktop-build     Build Tauri desktop client for production
  desktop-build-fe  Build frontend only (Vite)
  desktop-check     Check desktop Rust backend compilation
  desktop-install   Install frontend dependencies (yarn)
  desktop-init      First-time setup (yarn + init)

Recommended: Install 'just' for richer task runner (cargo install just)
Then use: just <recipe>
EOF
        ;;
esac