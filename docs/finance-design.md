# Finance Module Design

## Overview

The finance module provides a read-only toolchain for managing a
[Beancount](https://beancount.github.io/) double-entry ledger via
[Fava](https://beancount.github.io/fava/), a web-based UI.

```
┌──────────────────────────────────────────────────────────┐
│                     User (CLI / Browser)                 │
└──────────────┬───────────────────────────┬───────────────┘
               │ just finance::*           │ http://localhost:5500
               ▼                           ▼
┌──────────────────────────────┐ ┌──────────────────────────┐
│      uv run bean-*           │ │      fava (web server)   │
│   (check / query)            │ │   (browse / inspect)     │
└──────────┬───────────────────┘ └──────────┬───────────────┘
           │ read-only                      │ read-only
           ▼                                ▼
┌───────────────────────────────────────────────────────────┐
│              main.beancount (external file)               │
│         (NOT inside repository — FINANCE_BEANCOUNT_FILE)  │
└───────────────────────────────────────────────────────────┘
```

Key principles:

- **Ledger data never enters the repository** — `FINANCE_BEANCOUNT_FILE` points to an external path
- **All operations are read-only** — the module never writes to `.beancount` files
- **Python isolation** — beancount + fava run in a `uv`-managed virtual environment under
  `modules/finance/.venv/`

---

## Module Structure

```
modules/finance/
├── pyproject.toml     # Python dependencies (beancount, fava)
├── justfile           # 5 recipes: setup, check, query, serve, env
├── README.md          # User documentation
└── .venv/             # Virtual environment (gitignored)

docs/
├── finance-draft.md   # Planning notes (Chinese, never committed)
└── finance-design.md  # This document (committed)
```

---

## Command Specifications

### `just finance::setup`

| Property         | Value                                                      |
| ---------------- | ---------------------------------------------------------- |
| **Tool**         | `uv sync`                                                  |
| **Network**      | Yes (package download). Proxy via `FINANCE_PROXY`.         |
| **I/O**          | Writes `modules/finance/.venv/`, `modules/finance/uv.lock` |
| **Idempotent**   | Yes — safe to re-run                                       |
| **Exit on fail** | Non-zero if package resolution fails                       |

### `just finance::check`

| Property         | Value                                                                          |
| ---------------- | ------------------------------------------------------------------------------ |
| **Tool**         | `bean-check` (bundled with beancount)                                          |
| **Access**       | Read-only on `$FINANCE_BEANCOUNT_FILE`                                         |
| **I/O**          | stdout: error details on failure. Exit code 0 on success, non-zero on failure. |
| **Precondition** | `FINANCE_BEANCOUNT_FILE` must be set                                           |

### `just finance::query <expr>`

| Property         | Value                                                                         |
| ---------------- | ----------------------------------------------------------------------------- |
| **Tool**         | `bean-query` (bundled with beancount)                                         |
| **Format**       | CSV with header row                                                           |
| **Access**       | Read-only on `$FINANCE_BEANCOUNT_FILE`                                        |
| **I/O**          | stdout: CSV data. Pipe through `column -t -s ','` for human-friendly display. |
| **Precondition** | `FINANCE_BEANCOUNT_FILE` must be set                                          |

### `just finance::serve`

| Property         | Value                                               |
| ---------------- | --------------------------------------------------- |
| **Tool**         | `fava`                                              |
| **Interface**    | Web UI at `http://127.0.0.1:<port>` (default: 5500) |
| **Access**       | Read-only on `$FINANCE_BEANCOUNT_FILE`              |
| **Lifetime**     | Foreground process. Ctrl-C to stop.                 |
| **Precondition** | `FINANCE_BEANCOUNT_FILE` must be set                |

### `just finance::env`

| Property         | Value                                            |
| ---------------- | ------------------------------------------------ |
| **I/O**          | stdout: resolved env vars and ledger file status |
| **Side effects** | None — purely informational                      |

---

## Error Handling Strategy

| Scenario                         | Behavior                                                                         |
| -------------------------------- | -------------------------------------------------------------------------------- |
| `FINANCE_BEANCOUNT_FILE` not set | Recipe exits immediately with `✗ FINANCE_BEANCOUNT_FILE not set` and exit code 1 |
| Ledger file does not exist       | `bean-check` / `bean-query` / `fava` reports the error naturally                 |
| `uv sync` network failure        | `uv sync` exits with non-zero, error propagated to user                          |
| Query syntax error               | `bean-query` prints error details to stderr, exit code non-zero                  |
| Proxy unreachable                | `uv sync` times out or fails connection, error propagated                        |

All recipes use `&&` chaining — any failure in the chain stops execution immediately with the
failing command's exit code.

---

## Proxy Architecture

```
FINANCE_PROXY=http://127.0.0.1:7890
       │
       ▼
  just variable: _PROXY_INIT
       │
       ├──► uv sync --proxy $FINANCE_PROXY ...
       ├──► uv run  --proxy $FINANCE_PROXY ...
       └──► (no effect on env — no uv needed)
```

The proxy flag is computed once via the `[private]` just variable `_PROXY_INIT`, which sets a shell
variable `proxy_flag` at the start of each recipe. When `FINANCE_PROXY` is unset, `$proxy_flag`
expands to zero words (shell word splitting), keeping the command clean.

---

## Future Integration Points

### Notify Module

`modules/notify` can be used to send Telegram alerts when `bean-check` fails:

```bash
just notify::watch --command "just finance::check" --on-failure
```

This is handled entirely on the user/caller side — the finance module has no dependency on notify.

### Scheduled Price Updates

`bean-price` is not bundled with beancount 3. If added later, a `just finance::price` recipe could
be added following the same read-only pattern, potentially triggered via cron/launchd.

### Import Pipeline

`bean-identify` / `bean-extract` / `bean-file` were removed from beancount 3's default distribution.
If re-added via a separate package, a write-aware import flow could be designed, but this requires
significant safety considerations (the module currently enforces read-only).

---

## Configuration Reference

| Variable                 | Required | Default | Purpose                               |
| ------------------------ | -------- | ------- | ------------------------------------- |
| `FINANCE_BEANCOUNT_FILE` | Yes      | —       | Path to `main.beancount` (external)   |
| `FINANCE_FAVA_PORT`      | No       | `5500`  | Fava web UI port                      |
| `FINANCE_PROXY`          | No       | —       | HTTP proxy for `uv` package downloads |
