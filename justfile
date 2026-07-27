# ─── personal-automation-tools ────────────────────────────────
# Entry point: load backup module
# ────────────────────────────────────────────────────────────

set dotenv-command := "cat config/.env config/.env.dev.local 2>/dev/null; true"

mod backup "modules/backup"
mod videodl "modules/videodl"
mod notify "modules/notify"
mod finance "modules/finance"
mod robot "modules/robot"
mod md-export "modules/md-export"

# Build profile: 'release' (default) or 'debug'
# Override:  profile=debug just build
profile := "release"

# Show available tasks
default:
    @just --list

# ── build ────────────────────────────────────────────────

# Build all Rust binaries (profile: release | debug)
build:
    @echo "🔨 Building all (notify, static-server) [{{ profile }}] ..." && \
    cargo build {{ if profile == "release" { "--release" } else { "" } }} --bin notify --bin static-server --quiet && \
    echo "✓ notify ready" && \
    echo "✓ static-server ready"

# Build notify only
build-notify:
    @echo "🔨 Building notify [{{ profile }}] ..." && \
    cargo build {{ if profile == "release" { "--release" } else { "" } }} --bin notify --quiet && \
    echo "✓ notify ready"

# Build static-server only
build-static-server:
    @echo "🔨 Building static-server [{{ profile }}] ..." && \
    cargo build {{ if profile == "release" { "--release" } else { "" } }} --bin static-server --quiet && \
    echo "✓ static-server ready"

# Debug build variants (no --release)
build-debug:
    @echo "🔨 Building all (notify, static-server) [debug] ..." && \
    cargo build --bin notify --bin static-server --quiet && \
    echo "✓ notify ready (debug)" && \
    echo "✓ static-server ready (debug)"

build-debug-notify:
    @echo "🔨 Building notify [debug] ..." && \
    cargo build --bin notify --quiet && \
    echo "✓ notify ready (debug)"

build-debug-static-server:
    @echo "🔨 Building static-server [debug] ..." && \
    cargo build --bin static-server --quiet && \
    echo "✓ static-server ready (debug)"

# ── test ──────────────────────────────────────────────────

# Run all Rust tests
test:
    @cargo test --bin notify --bin static-server

# ── fmt ──────────────────────────────────────────────────

# Format all files
fmt:
    just --fmt
    just --fmt --justfile modules/backup/justfile
    just --fmt --justfile modules/notify/justfile
    just --fmt --justfile modules/finance/justfile
    just --fmt --justfile modules/robot/justfile
    just --fmt --justfile modules/md-export/justfile
    dprint fmt

# Check formatting (CI use)
fmt-check:
    just --fmt --check
    just --fmt --check --justfile modules/backup/justfile
    just --fmt --check --justfile modules/notify/justfile
    just --fmt --check --justfile modules/finance/justfile
    just --fmt --check --justfile modules/robot/justfile
    just --fmt --check --justfile modules/md-export/justfile
    dprint check
