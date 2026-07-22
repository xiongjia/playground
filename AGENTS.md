# AGENTS.md — Context for AI Coding Agents

Modular personal automation repo. Each module in `modules/<name>/` has its own `justfile` +
`README.md`. Root `justfile` loads modules via `mod`.

## Conventions

- **Language**: English for code, commits, docs (except `*-draft.md`)
- **Drafts**: `*-draft.md` — Chinese allowed, **never committed**
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/)
- **AI agents**: single writer at a time; review via read-only subagents
- **Safety**: NEVER create/modify/delete `config/.env*` files. Set vars inline for testing:
  `BACKUP_SOURCE_DIR=/tmp/x just backup::env`
- **Env loading**: `config/.env` → `config/.env.dev.local` (overrides)

## Tech Stack

| Tool            | Lang   | Use                                     |
| --------------- | ------ | --------------------------------------- |
| `just`          | Rust   | Task runner                             |
| `dprint`        | Rust   | Formatter (md, json, ts)                |
| `yt-dlp`        | Python | Video download (CLI only, user-managed) |
| `static-server` | Rust   | HTTP file server (src/bin/)             |

Rust tools pinned in `Cargo.toml`. Shared binaries in `src/bin/`.

## Modules

- `modules/backup/README.md` — restic backup
- `modules/videodl/README.md` — yt-dlp video & subtitle manager

Module recipes use `::` separator (e.g. `just backup::run`). Run `just --list` for all.

## Adding a Module

1. `modules/<name>/justfile` + `README.md`
2. `mod <name> "modules/<name>"` in root `justfile`
3. Env vars in `config/.env.example`
4. `docs/<name>-design.md` for nontrivial design

## Agents

- **pi**: `subagent { action: "list" }` to discover agents
- **Claude Code**: reads this file from project root
- Read `docs/arch.md` and `docs/<module>-design.md` before architecture changes
