#!/usr/bin/env bash
# ─── run-backup.sh ───────────────────────────────────────────
# Internal: run restic backup + forget + prune.
# Called by `just backup::run` (with notification wrapper) and
# `just backup::run-only` (directly, no notification).
# ────────────────────────────────────────────────────────────
set -euo pipefail

if [ -z "$BACKUP_SOURCE_DIR" ]; then
    echo "✗ BACKUP_SOURCE_DIR not set"
    exit 1
fi

FIRST=$(echo "$BACKUP_SOURCE_DIR" | cut -d' ' -f1)
BASENAME=$(basename "$FIRST")

cd "${BACKUP_PARENT:-.}"

restic backup $BACKUP_SOURCE_DIR \
    --tag "$BACKUP_TAG" \
    --tag "host:$HOSTNAME" \
    --tag "path:$BASENAME" \
    --exclude-file "$1"

echo "✓ Backup done. Running forget + prune …"

restic forget \
    --keep-daily 7 \
    --keep-weekly 4 \
    --keep-monthly 6 \
    --prune \
    --tag "$BACKUP_TAG"
