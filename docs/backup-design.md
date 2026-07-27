# Backup Module Design

> Design decisions for the restic-based backup module.

---

## 1. Why restic

- **Encrypted, incremental, deduplicated** — safe for remote/cloud storage
- **Multiple backends** — local, SFTP, S3, B2, REST server; switch by changing `$RESTIC_REPOSITORY`
- **Self-contained binary** — no daemon, no DB; snapshots are plain files
- **Battle-tested** — mature CLI, active community

---

## 2. Tag Strategy

Every snapshot is tagged with three dimensions:

| Tag               | Source                         | Example            | Purpose                                                                   |
| ----------------- | ------------------------------ | ------------------ | ------------------------------------------------------------------------- |
| `$BACKUP_TAG`     | `config/.env`                  | `my-backup`        | Isolate this project's snapshots from others in the same repo             |
| `host:<hostname>` | `$HOSTNAME` from `config/.env` | `host:macbook-pro` | Identify the source machine                                               |
| `path:<basename>` | `basename $BACKUP_SOURCE_DIR`  | `path:data`        | Identify the data source (supports multiple backup targets in the future) |

**Why tags matter:**

- `restic forget --tag "$BACKUP_TAG"` — only clean up this project's snapshots, never touch
  unrelated ones
- `restic snapshots --tag host:my-mac` — filter by machine
- Tags can be combined: `restic forget --tag "$BACKUP_TAG" --keep-daily 7` — precise retention per
  project

---

## 3. Retention Policy

```
--keep-daily   7     last 7 daily snapshots
--keep-weekly  4     last 4 weekly snapshots
--keep-monthly 6     last 6 monthly snapshots
```

**Rationale:**

| Policy    | Coverage                      | Reasoning                                              |
| --------- | ----------------------------- | ------------------------------------------------------ |
| 7 daily   | One week of granular recovery | Most common "oops I deleted this yesterday" scenario   |
| 4 weekly  | ~1 month of weekly points     | Catch issues noticed days later                        |
| 6 monthly | Half-year safety net          | Defend against long-unnoticed corruption or ransomware |

**Applied after every `just backup::run`**, so daily backup never accumulates.

Preview with `just backup::forget-dry-run` before actual cleanup.

---

## 4. Backup Workflow

```
just backup::run
  │
  ├── 1. restic backup $BACKUP_SOURCE_DIR
  │       ├── --tag "$BACKUP_TAG"
  │       ├── --tag host:<hostname>
  │       ├── --tag path:<basename>
  │       └── --exclude-file modules/backup/.restic_exclude
  │
  └── 2. restic forget (on success)
          ├── --keep-daily 7
          ├── --keep-weekly 4
          ├── --keep-monthly 6
          ├── --prune
          └── --tag "$BACKUP_TAG"
```

**Why forget+prune inline:** restic prune re-reads all pack files. Running it every time is safe (no
data loss) and keeps the repository lean. For large repos, separate them: `just backup::run-only`
daily and `just backup::forget` weekly via cron.

---

## 5. Exclude Strategy

Exclude patterns live in `modules/backup/.restic_exclude` (version controlled). They cover:

| Category        | Examples                                     |
| --------------- | -------------------------------------------- |
| Temp / swap     | `*.tmp`, `*~`, `.DS_Store`                   |
| Build artifacts | `node_modules/`, `target/`, `dist/`, `*.pyc` |
| Logs            | `*.log`, `logs/`                             |
| IDE             | `.vscode/`, `.idea/`                         |
| OS metadata     | `.Spotlight-V100`, `.fseventsd`              |

Machine-local or path-specific exclusions can be added via `$BACKUP_SOURCE_DIR/.resticignore`
(per-directory, not committed if added to `.gitignore`).

---

## 6. Security Model

| Concern        | Measure                                                                |
| -------------- | ---------------------------------------------------------------------- |
| Encryption     | restic repository password in `config/.env` (gitignored)               |
| Remote access  | SFTP key or S3 credentials via environment variables                   |
| Exfil risk     | restic uses TLS for remote backends; no plaintext data stored remotely |
| Key management | Password manager for `RESTIC_PASSWORD`; `.env` is local-only           |

No secrets are embedded in justfiles or scripts.

---

## 7. Recovery Procedure

```bash
# List available snapshots
just backup::list

# Restore latest to a temporary directory
just backup::restore /tmp/restore-test

# Restore a specific snapshot by ID
restic restore <snapshot-id> --target /tmp/restore-test

# Mount snapshots as a FUSE filesystem (interactive browsing)
restic mount /mnt/restic
```

For critical recovery, always restore to a different location than the original, then diff before
overwriting.

---

## 8. Multi-backend Support

Switching backends requires only changing `$RESTIC_REPOSITORY` in `config/.env`:

```bash
# Local
RESTIC_REPOSITORY=/Volumes/BackupDrive/restic-repo

# SFTP
RESTIC_REPOSITORY=sftp:user@nas:/volume1/backups/restic

# S3 (compatible)
RESTIC_REPOSITORY=s3:my-bucket/restic
AWS_ACCESS_KEY_ID=xxx
AWS_SECRET_ACCESS_KEY=xxx

# Backblaze B2
RESTIC_REPOSITORY=b2:bucket-name:/path
B2_ACCOUNT_ID=xxx
B2_ACCOUNT_KEY=xxx
```

Each backend needs `restic init` once. For simultaneous multi-destination, use restic's
`--copy-chunker-params` or run separate repos.

---

## 9. Notifications

`just backup::run` automatically sends a notification on completion via `modules/notify`.

**Notification behavior:**

| Trigger        | Channel           | Message                                                                         |
| -------------- | ----------------- | ------------------------------------------------------------------------------- |
| Backup success | Telegram / Stdout | `✅ backup::run succeeded (took: 2m15s · restic forget: removed 3 snapshots)`   |
| Backup failure | Telegram / Stdout | `❌ backup::run failed (took: 0m45s, exit code: 2 · ERROR: connection refused)` |

**Control:**

- `NOTIFY_SILENT=true` in `config/.env` — disable all notifications globally
- Use `just notify::test` to verify notification channels work

**Implementation:**

The backup command is wrapped by `modules/notify/scripts/run-and-notify.sh`, which measures runtime
(via `date +%s`), captures the exit code, captures the last line of stdout as a result summary, and
calls `notify send --level success/error` with the result. The original exit code is preserved so
`&&` chains (backup → forget) work correctly.

---

## 10. Automation (Future)

- `launchd` / `cron` daily trigger: `just backup::run`
- `Healthchecks.io` ping after successful backup for alerting on failure
- `restic check --read-data-subset=5%` scheduled weekly for data integrity verification
