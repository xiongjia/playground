# AGENTS.md — Context for AI Coding Agents

Modular personal automation repo. Each module lives in `modules/<name>/` with its own `justfile` +
`README.md`. Root `justfile` loads modules via `mod`.

## Conventions

- **Language**: English for code, commits, docs (except `*-draft.md`)
- **Draft files**: `*-draft.md` — Chinese allowed, **never committed**
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/) — prefix with type:
  `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`
- **AI agents**: single writer principle — only one agent writes at a time
- **Review tasks**: use read-only subagents; main agent applies changes
- **Commit**: requires dev review approval
- **Push**: developer does it themselves (no auto-push)
- **Safety**: NEVER create, modify, or delete any `config/.env*` file — set vars inline for testing:
  `BACKUP_SOURCE_DIR=/tmp/x just backup::env`
- **Env loading order**: `config/.env` (base) → `config/.env.dev.local` (overrides)

## Tech Stack

- `just` (Rust) — task runner | `dprint` (Rust) — formatter | `restic` (Go) — backup

All Rust tools pinned in `Cargo.toml` (`[dev-dependencies]`).

## Available Commands

```bash
just --list              # all recipes
just backup::run         # daily: backup → forget(7d+4w+6m) → prune
just backup::init        # init restic repo (one-time)
just backup::list        # list snapshots
just backup::status      # latest backup stats
just backup::restore ./x       # restore latest snapshot to ./x
just backup::restore ./x <id>  # restore specific snapshot by ID
just backup::forget      # manual cleanup + prune
just backup::forget-dry-run  # preview cleanup
just backup::check       # verify repo integrity
just fmt                 # format all files
just fmt-check           # CI check
```

Module recipes use `::` separator: `just backup::run`, not `just backup run`.

## Config & Secrets

`config/.env` + `config/.env.dev.local` (both gitignored) loaded via `set dotenv-command`.
`config/.env.example` is the committed template. Never write secrets into justfiles or scripts.

## Adding a New Module

1. `modules/<name>/justfile` + `README.md` + optional `scripts/`
2. Add `mod <name> "modules/<name>"` to root `justfile`
3. Add env vars to `config/.env.example`
4. Add `docs/<name>-design.md` for nontrivial decisions
5. Recipes callable as `just <name>::<recipe>`

## Agent Notes

- **pi agent**: use `subagent { action: "list" }` to discover available agents
- **Claude Code**: reads this file automatically from project root
- **Both**: read `docs/arch.md` and `docs/<module>-design.md` before architectural changes
