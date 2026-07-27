#!/usr/bin/env bash
# discover.sh — List markdown files eligible for export
#
# Environment:
#   MD_EXPORT_EXCLUDE — space-separated filename patterns to exclude (optional)
#                       Defaults to common non-document files if unset.
#
# Usage:  bash discover.sh
#         # => outputs relative paths like ./README.md
#         # => pipe to convert.sh for bulk export
# ────────────────────────────────────────────────────────────

set -euo pipefail

# ── exclude patterns (defaults) ──────────────────────────

DEFAULT_EXCLUDE="CHANGELOG.md LICENSE.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md"
EXCLUDE="${MD_EXPORT_EXCLUDE:-$DEFAULT_EXCLUDE}"

exclude_args=()
for pattern in $EXCLUDE; do
  exclude_args+=(-not -name "$pattern")
done

# ── discover ─────────────────────────────────────────────

find . -name '*.md' \
  -not -path './.git/*' \
  -not -path './.pi/*' \
  -not -path '*/node_modules/*' \
  -not -path '*/.venv/*' \
  -not -path './target/*' \
  -not -path '*/.claude/*' \
  -not -path '*/.cursor/*' \
  -not -name '*-draft.md' \
  "${exclude_args[@]}" \
  | sed 's|^\./||'  # strip leading ./ for clean paths
