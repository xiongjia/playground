# ─── personal-automation-tools ────────────────────────────────
# Entry point: load backup module
# ────────────────────────────────────────────────────────────

set dotenv-command := "cat config/.env config/.env.dev.local 2>/dev/null; true"

mod backup "modules/backup"

# Show available tasks
default:
    @just --list

# Format all justfiles and markdown
fmt:
    just --fmt
    just --fmt --justfile modules/backup/justfile
    dprint fmt

# Check formatting (CI use)
fmt-check:
    just --fmt --check
    just --fmt --check --justfile modules/backup/justfile
    dprint check
