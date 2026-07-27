#!/usr/bin/env bash
# convert.sh — Convert markdown files to PDF / EPUB / DOCX
#
# Environment:
#   MD_EXPORT_ROOT       — output directory (required)
#   MD_EXPORT_FORMATS    — comma-separated, default: pdf
#   MD_EXPORT_PDF_FONT   — PDF body font, default: Songti SC
#   MD_EXPORT_PDF_ENGINE — PDF engine, default: weasyprint
#
# Usage:
#   bash convert.sh [--toc] <file1.md> [file2.md ...]
#   bash discover.sh | bash convert.sh [--toc]
# ────────────────────────────────────────────────────────────

set -euo pipefail

# ── defaults ──────────────────────────────────────────────

: "${MD_EXPORT_ROOT:?✗ MD_EXPORT_ROOT is not set}"
: "${MD_EXPORT_FORMATS:=pdf}"
: "${MD_EXPORT_PDF_FONT:=Songti SC}"
: "${MD_EXPORT_PDF_ENGINE:=weasyprint}"

MODULE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE_DIR="$MODULE_DIR/templates"
TOC_FLAG=""

# ── parse args ──────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --toc) TOC_FLAG="--toc" ; shift ;;
    --)    shift ; break ;;
    -*)    echo "✗ Unknown option: $1" >&2 ; exit 1 ;;
    *)     break ;;
  esac
done

# If no files as args, read from stdin (pipe from discover.sh)
if [[ $# -eq 0 ]]; then
  if [[ -t 0 ]]; then
    echo "✗ No input files. Pass files as args or pipe from discover.sh" >&2
    exit 1
  fi
  # Read stdin lines into positional args (compatible with bash 3.2)
  stdin_files=()
  while IFS= read -r line; do
    stdin_files+=("$line")
  done
  set -- "${stdin_files[@]}"
fi

# ── git metadata ─────────────────────────────────────────

GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
GIT_DATE="$(git log -1 --format=%cI 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")"

# ── export function ──────────────────────────────────────

export_file() {
  local src="$1"
  local toc_flag="$2"

  # Resolve absolute path for pandoc
  local src_abs
  src_abs="$(cd "$(dirname "$src")" && pwd)/$(basename "$src")"

  # Compute relative path from project root
  local rel_path="${src#./}"
  local dir_part
  local stem
  stem="$(basename "$rel_path" .md)"

  # For absolute paths, flatten to just the filename (no directory mirroring)
  if [[ "$src" == /* ]]; then
    dir_part=""
    rel_path="$(basename "$src")"
  else
    dir_part="$(dirname "$rel_path")"
    # If dir_part is '.', use empty string (file is at root)
    if [[ "$dir_part" == "." ]]; then
      dir_part=""
    fi
  fi

  # Source path for metadata
  local source_path="$rel_path"

  IFS=',' read -ra formats <<< "$MD_EXPORT_FORMATS"
  for fmt in "${formats[@]}"; do
    fmt="$(echo "$fmt" | xargs)"  # trim whitespace

    # Determine output path (mirror directory structure)
    local out_dir="$MD_EXPORT_ROOT/$fmt"
    if [[ -n "$dir_part" ]]; then
      out_dir="$out_dir/$dir_part"
    fi
    local out_file="$out_dir/$stem"

    local pandoc_to
    case "$fmt" in
      pdf)  pandoc_to="pdf" ; out_file="${out_file}.pdf" ;;
      epub) pandoc_to="epub" ; out_file="${out_file}.epub" ;;
      docx) pandoc_to="docx" ; out_file="${out_file}.docx" ;;
      *)    echo "  ⚠ Unknown format: $fmt (skip)" >&2 ; continue ;;
    esac

    mkdir -p "$out_dir"

    # Handle existing files: append timestamp, then -N if still collides
    if [[ -f "$out_file" ]]; then
      local ts
      ts="$(date '+%Y%m%d-%H%M%S')"
      local base="${out_file%.*}"
      local ext="${out_file##*.}"
      out_file="${base}.${ts}.${ext}"
      # If timestamp still collides, try -1, -2, ...
      if [[ -f "$out_file" ]]; then
        local n=1
        while [[ -f "${base}.${ts}-${n}.${ext}" ]]; do
          n=$((n + 1))
        done
        out_file="${base}.${ts}-${n}.${ext}"
      fi
    fi

    echo "  → $out_file"

    # Build pandoc command
    local cmd=("pandoc" "$src_abs" "-f" "markdown" "-t" "$pandoc_to" "-o" "$out_file")
    cmd+=("--metadata=git-commit:$GIT_COMMIT")
    cmd+=("--metadata=git-date:$GIT_DATE")
    cmd+=("--metadata=git-branch:$GIT_BRANCH")
    cmd+=("--metadata=source-path:$source_path")

    if [[ -n "$toc_flag" ]]; then
      cmd+=("$toc_flag")
    fi

    if [[ "$fmt" == "pdf" ]]; then
      cmd+=("--pdf-engine=$MD_EXPORT_PDF_ENGINE")
      cmd+=("--template=$TEMPLATE_DIR/pdf.html")
    elif [[ "$fmt" == "epub" ]]; then
      if [[ -f "$TEMPLATE_DIR/epub.css" ]]; then
        cmd+=("--css=$TEMPLATE_DIR/epub.css")
      fi
    fi

    # Run
    if ! err_output=$("${cmd[@]}" 2>&1); then
      echo "  ✗ Failed: $rel_path → $fmt" >&2
      echo "    $err_output" >&2
    fi
  done
}

# ── main ─────────────────────────────────────────────────

count=0
for src in "$@"; do
  # Skip if not a .md file
  if [[ "$src" != *.md ]]; then
    echo "  ⚠ Skip (not .md): $src" >&2
    continue
  fi
  # Skip if file doesn't exist
  if [[ ! -f "$src" ]]; then
    echo "  ⚠ Skip (not found): $src" >&2
    continue
  fi
  count=$((count + 1))
  echo "[$count] $src"
  export_file "$src" "$TOC_FLAG"
done

echo ""
echo "✅ Done: $count file(s) exported to $MD_EXPORT_ROOT"
