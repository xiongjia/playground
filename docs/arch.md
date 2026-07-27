# Architecture

> Design overview of the personal-automation-tools repository.

---

## 1. Philosophy

This repository aggregates personal daily automation tasks in a **modular, self-documenting** way.
Each functional domain lives in its own `modules/<name>/` directory with its own justfile, README,
and scripts. The root justfile acts as a thin discovery layer.

**Key principles:**

| Principle           | Rationale                                                                                                   |
| ------------------- | ----------------------------------------------------------------------------------------------------------- |
| Module-per-domain   | Each concern is self-contained; adding a new module does not touch existing ones                            |
| Config out of repo  | Secrets and machine-local paths stay in `config/.env` (gitignored); only `.example` templates are committed |
| Uniform entry point | `just <module> <task>` is the only interface — no memorizing script paths                                   |
| POSIX-first         | Scripts target Linux and macOS; Windows is out of scope                                                     |

---

## 2. Directory Structure

```
.
├── justfile                 # Root entry: loads modules via `mod`
│
├── config/                  # Environment configuration
│   ├── .env                 # Local secrets & paths (gitignored)
│   └── .env.example         # Template with placeholder values (committed)
│
├── modules/                 # Feature modules
│   └── <name>/
│       ├── justfile         # Module-specific tasks
│       ├── README.md        # Usage & prerequisites
│       ├── .<name>_exclude  # Module-specific exclude patterns (if needed)
│       └── scripts/         # Shell scripts (optional)
│
├── docs/                    # Documentation
│   ├── arch.md              # This file — repo-level design
│   ├── <module>-design.md   # Per-module design docs
│   └── *-draft.md           # Local planning notes (not committed)
│
└── .github/
    └── workflows/           # CI workflows
```

### 2.1 Root justfile

Minimal — loads modules via `mod`:

```make
mod backup "modules/backup"
mod infra "modules/infra"      # future
```

Module tasks are namespaced with `::` separator: `just backup::run`, `just infra::doctor`.

### 2.2 Config layer

`config/.env` is loaded automatically by `set dotenv-path := "config/.env"` in the root justfile.
All module-specific variables follow the `<MODULE>_<KEY>` naming convention to avoid collisions.

---

## 3. Adding a New Module

1. Create `modules/<name>/` with `justfile`, `README.md`, and optional `scripts/`
2. Add `mod <name> "modules/<name>"` to the root `justfile`
3. Add any module-specific env vars to `config/.env.example`
4. Write `docs/<name>-design.md` if the module involves nontrivial design decisions

Task naming: `just <name>::<verb>` (e.g. `just backup::run`, `just infra::doctor`).

---

## 4. Technology Stack

| Tool                                  | Role                         | Installation                                 |
| ------------------------------------- | ---------------------------- | -------------------------------------------- |
| [just](https://github.com/casey/just) | Task runner                  | `brew install just` / `apt install just`     |
| [restic](https://restic.net)          | Encrypted incremental backup | `brew install restic` / `apt install restic` |

Additional tools are module-specific and documented in each module's README.

---

## 5. Development Conventions

- **Language**: English (code, commit messages, non-draft docs)
- **Commits**: Conventional Commits — `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`
- **Draft files**: `*-draft.md` in `docs/` — Chinese allowed, never committed
- **AI agents**: pi agent + Claude Code; single-writer principle (only one agent writes to working
  tree at a time)

---

## 6. Future Considerations

- **CI**: GitHub Actions could run `backup check` on schedule or alert on stale snapshots
- **Notifications**: Long-running commands (backup, export, download) automatically send
  notifications via `modules/notify`. Wrapped by `modules/notify/scripts/run-and-notify.sh`. Disable
  globally with `NOTIFY_SILENT=true`.
- **Shared lib**: If scripts across modules share logic, extract to `scripts/lib/`
