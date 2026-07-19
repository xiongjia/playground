# backup — restic backup module

Daily encrypted incremental backup for a specific directory via [restic](https://restic.net).

## Prerequisites

- [restic](https://restic.readthedocs.io/en/stable/020_installation.html) (`brew install restic` /
  `apt install restic`)

## Setup

```bash
# 1. Configure
cp config/.env.example config/.env
# edit config/.env — set RESTIC_PASSWORD, RESTIC_REPOSITORY, BACKUP_SOURCE_DIR
#   BACKUP_SOURCE_DIR supports space-separated multiple paths:
#   BACKUP_SOURCE_DIR="/path/to/a /path/to/b /path/to/c"

# 2. Init repository (one time only)
just backup::init

# 3. Run daily backup
just backup::run
```

## Usage

```bash
just backup::run             # daily: backup → forget → prune
just backup::run-only        # backup without cleanup
just backup::list            # list snapshots
just backup::status          # latest backup stats
just backup::restore ./out        # restore latest snapshot to ./out
just backup::restore ./out abc123   # restore specific snapshot by ID
just backup::list                    # find snapshot IDs
just backup::check           # verify repo integrity
```

## Tagging

Every snapshot is tagged with:

| Tag               | Example         | Purpose                |
| ----------------- | --------------- | ---------------------- |
| `$BACKUP_TAG`     | (configurable)  | project identifier     |
| `host:<hostname>` | `host:my-mac`   | machine identifier     |
| `path:<basename>` | `path:projects` | data source identifier |

## Retention

| Policy  | Keep   |
| ------- | ------ |
| Daily   | last 7 |
| Weekly  | last 4 |
| Monthly | last 6 |

`just backup::run` applies this automatically after each backup. Use `just backup::forget-dry-run`
to preview before actual cleanup.

## Paths

`BACKUP_SOURCE_DIR` takes one or more absolute paths, space-separated:

```bash
BACKUP_SOURCE_DIR="/home/user/data /home/user/projects"
```
