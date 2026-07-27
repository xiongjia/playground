# Finance Module

Beancount + Fava toolchain for personal finance management.

## Prerequisites

- **[uv](https://docs.astral.sh/uv/getting-started/installation/)** >= 0.4 — Python package manager
  - macOS: `brew install uv`
  - Other: `pipx install uv` or the
    [official installer](https://docs.astral.sh/uv/getting-started/installation/)
- Python >= 3.12 (managed automatically by `uv`)

## Configuration

All variables go in `config/.env`:

| Variable                 | Required | Default | Description                                              |
| ------------------------ | -------- | ------- | -------------------------------------------------------- |
| `FINANCE_BEANCOUNT_FILE` | Yes      | —       | Path to `main.beancount` (must be **outside** this repo) |
| `FINANCE_FAVA_PORT`      | No       | `5500`  | Fava web UI port                                         |
| `FINANCE_PROXY`          | No       | —       | Proxy for uv package downloads                           |

> ⚠️ **Security**: Ledger files contain sensitive financial data. **Do not store them inside this
> repository.** All `just finance::*` commands are **read-only** — they never modify your ledger.

## Commands

| Command                      | Description                               |
| ---------------------------- | ----------------------------------------- |
| `just finance::setup`        | Install / update Python dependencies      |
| `just finance::check`        | Validate ledger syntax and balance        |
| `just finance::query <expr>` | Run a Beancount Query Language query      |
| `just finance::serve`        | Start fava web UI (http://127.0.0.1:5500) |
| `just finance::env`          | Show resolved environment configuration   |

### Examples

```bash
# First use: install dependencies
just finance::setup

# Validate ledger
just finance::check

# Query account balances (output is CSV)
just finance::query "SELECT account, sum(position) WHERE currency = 'CNY'"

# Query with human-friendly formatting
just finance::query "SELECT account, sum(position)" | column -t -s ','

# Start fava web interface
just finance::serve
```

Note: `just finance::query` outputs raw CSV (header row + data). Pipe through `column -t -s ','` for
readable table output.
