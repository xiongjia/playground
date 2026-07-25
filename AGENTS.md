# AGENTS.md — Context for AI Coding Agents

Modular personal automation repo. Each module in `modules/<name>/` has its own `justfile` +
`README.md`. Root `justfile` loads modules via `mod`.

## Conventions

- **Language**: English for code, commits, docs (except `*-draft.md`)
- **Drafts**: `*-draft.md` — Chinese allowed, **never committed**
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/)
- **AI agents**: single writer at a time; review via read-only subagents
- **Safety**: NEVER create/modify/delete `config/.env` or `config/.env.dev.local`. These contain
  user secrets and local overrides. `config/.env.example` **is** allowed to edit — it's a committed
  template. For testing, set vars inline: `NOTIFY_TELEGRAM_CHAT_ID=123 just notify::test` If you
  accidentally touch an `.env` file, stop and ask the user to restore it. **Do not use `rm`, `cp`,
  `echo >`, or any file-write tool on `config/.env` or `config/.env.dev.local`.**
- **Env loading**: `config/.env` → `config/.env.dev.local` (overrides)
- **Build**: `just build` (all Rust bins, release), `just build-debug` (debug), `just build-notify`
  (single bin, release), `just build-debug-notify` (single bin, debug) `profile=debug just build`
  (override profile inline)
- **Test**: `just test` (run all Rust tests)

## Tech Stack

| Tool            | Lang   | Use                                       |
| --------------- | ------ | ----------------------------------------- |
| `just`          | Rust   | Task runner                               |
| `dprint`        | Rust   | Formatter (md, json, ts)                  |
| `yt-dlp`        | Python | Video download (CLI only, user-managed)   |
| `notify`        | Rust   | Notification CLI (Telegram, watch, audit) |
| `static-server` | Rust   | HTTP file server                          |

Rust tools in `src/bin/`. Build: `just build` (all, release), `just build-debug` (all, debug),
`just build-notify` (one).

## Modules

- `modules/backup/README.md` — restic backup
- `modules/videodl/README.md` — yt-dlp video & subtitle manager
- `modules/notify/README.md` — notification CLI (Telegram, watch, audit log)

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
