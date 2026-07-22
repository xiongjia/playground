#!/usr/bin/env bash
# ─── Export cookies from browser to .cookies file ────────────
# Usage:  cookies.sh [browser] <output_file>
#   browser: chrome (default), firefox, safari, brave
# ────────────────────────────────────────────────────────────────
set -euo pipefail

BROWSER="${1:-chrome}"
OUTPUT="${2:?Usage: cookies.sh [browser] <output_file>}"

echo "🍪 Exporting cookies from $BROWSER ..."

yt-dlp --cookies-from-browser "$BROWSER" \
    --cookies "$OUTPUT" \
    --skip-download \
    "https://www.youtube.com/watch?v=dZyqFwwPKnY" \
    2>/dev/null || true

if [ -f "$OUTPUT" ] && [ -s "$OUTPUT" ]; then
    COUNT=$(wc -l < "$OUTPUT" | tr -d ' ')
    echo "✓ Saved to $OUTPUT ($COUNT lines)"
else
    echo "✗ Failed to export cookies from $BROWSER"
    exit 1
fi
