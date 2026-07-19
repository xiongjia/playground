# Personal automation toolkit

## Modules

| Module   | Description                  | Commands                                     |
| -------- | ---------------------------- | -------------------------------------------- |
| `backup` | encrypted incremental backup | `just backup::run`, `just backup::list`, ... |

See `modules/<name>/README.md` for module-specific usage.

## Prepare

### Dependencies

| Tool     | Version                | Source                           |
| -------- | ---------------------- | -------------------------------- |
| `just`   | pinned in `Cargo.toml` | `cargo install just`             |
| `dprint` | pinned in `Cargo.toml` | `cargo install dprint`           |
| `restic` | latest                 | [restic.net](https://restic.net) |

## Quick Start

```bash
cp config/.env.example config/.env
# edit config/.env — fill in your paths and secrets
just backup::init          # one-time setup
just backup::run           # daily backup
```

## Development

```bash
just fmt        # format all files
just fmt-check  # check formatting (CI gate)
```
