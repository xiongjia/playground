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

## Configuration

All variables go in `config/.env`:

| Variable            | Required | Default  | Description                                     |
| ------------------- | -------- | -------- | ----------------------------------------------- |
| `RESTIC_PASSWORD`   | Yes      | —        | Repository encryption password                  |
| `RESTIC_REPOSITORY` | Yes      | —        | Repo URL (`sftp:`, `s3:`, `b2:`, or local path) |
| `BACKUP_SOURCE_DIR` | Yes      | —        | Space-separated path(s) to back up              |
| `BACKUP_TAG`        | Yes      | —        | Snapshot tag for identification and retention   |
| `HOSTNAME`          | Yes      | —        | Machine identifier tag (shared global)          |
| `BACKUP_PARENT`     | No       | _(none)_ | `cd` here first so restic stores relative paths |

See `config/.env.example` for backend-specific examples (S3, B2, etc.).

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

## Notifications

`just backup::run` sends a Telegram notification on completion with runtime duration and
success/failure status.

| Result  | Example message                                     |
| ------- | --------------------------------------------------- |
| Success | `✅ backup::run succeeded (took: 2m15s)`            |
| Failure | `❌ backup::run failed (took: 0m45s, exit code: 2)` |

Disable globally: `NOTIFY_SILENT=true` in `config/.env`.

## Paths

`BACKUP_SOURCE_DIR` takes one or more absolute paths, space-separated:

```bash
BACKUP_SOURCE_DIR="/home/user/data /home/user/projects"
```
