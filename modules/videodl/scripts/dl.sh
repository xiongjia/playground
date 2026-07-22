#!/usr/bin/env bash
# ─── yt-dlp wrapper ────────────────────────────────────────────
# Handles directory structure, naming, and error handling.
# Called by justfile, not meant to be invoked directly.
#
# Usage:
#   dl.sh <url>                         # default quality (≤1080p)
#   dl.sh --best <url>                  # best quality
#   dl.sh --audio <url>                 # audio only
#   dl.sh --sub-only <url>              # subtitles only
#   dl.sh --format <format> <url>       # custom format
# ────────────────────────────────────────────────────────────────
set -euo pipefail

# Debug mode: enable with VIDEO_DL_DEBUG=1
if [[ -n "${VIDEO_DL_DEBUG:-}" ]]; then
    set -x
    PS4='+ [${BASH_SOURCE##*/}:${LINENO}] '
fi

VIDEO_DL_ROOT="${VIDEO_DL_ROOT:?VIDEO_DL_ROOT not set}"
SUBTITLE_LANGS="${SUBTITLE_LANGS:-en,zh-Hans}"
ARCHIVE_FILE="${VIDEO_DL_ROOT}/archive.txt"
TEMP_DIR="${VIDEO_DL_ROOT}/temp"

# ── parse mode ────────────────────────────────────────────────
MODE="default"
CUSTOM_FORMAT=""
URL=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --best)      MODE="best"; shift ;;
        --audio)     MODE="audio"; shift ;;
        --sub-only)  MODE="sub-only"; shift ;;
        --format)    MODE="custom"; CUSTOM_FORMAT="$2"; shift 2 ;;
        --list)      MODE="list"; URL="$2"; shift 2 ;;
        --info)      MODE="info"; URL="$2"; shift 2 ;;
        --help|-h)   MODE="help"; shift ;;
        *)           URL="$1"; shift ;;
    esac
done

# ── URL validation ────────────────────────────────────────────
if [[ "$MODE" =~ ^(list|info)$ && -z "$URL" ]]; then
    echo "Error: --list/--info requires a URL" >&2
    exit 1
fi

if [[ "$MODE" == "help" || -z "$URL" && "$MODE" == "default" ]]; then
    echo "Usage: dl.sh [options] <url>"
    echo ""
    echo "Options:"
    echo "  (none)      Download video + subtitles (default, ≤1080p)"
    echo "  --best      Download best quality (unlimited)"
    echo "  --audio     Audio only (m4a)"
    echo "  --sub-only  Download subtitles only"
    echo "  --format F  Custom format (e.g. 'bestvideo[height<=720]+bestaudio')"
    echo "  --list <url>   List available formats and subtitles"
    echo "  --info <url>   Show video metadata"
    exit 0
fi

# ── ensure directories exist ──────────────────────────────────
mkdir -p "$TEMP_DIR"
mkdir -p "$(dirname "$ARCHIVE_FILE")"

# ── build yt-dlp base args ────────────────────────────────────
BASE_ARGS=(
    --write-subs
    --sub-langs "$SUBTITLE_LANGS"
    --write-auto-subs
    --embed-metadata
    --write-thumbnail
    --convert-thumbnails jpg
    --write-info-json
    --download-archive "$ARCHIVE_FILE"
    --no-overwrites
    --continue
    --ignore-errors
    --socket-timeout 15
    --retry-sleep "exp=1:20"
)

# Dedicated proxy env var (avoids interfering with other programs)
if [[ -n "${VIDEO_DL_PROXY:-}" ]]; then
    BASE_ARGS+=(--proxy "$VIDEO_DL_PROXY")
fi

# Skip SSL verification when YT_DLP_NO_VERIFY_SSL is set (proxy SSL inspection)
if [[ -n "${YT_DLP_NO_VERIFY_SSL:-}" ]]; then
    BASE_ARGS+=(--no-check-certificates)
fi

# Cookies: priority: COOKIES_FILE > COOKIES_FROM > .cookies file
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COOKIES_LOCAL="${SCRIPT_DIR}/../.cookies"
COOKIE_ARG=()

if [[ -n "${COOKIES_FILE:-}" ]]; then
    COOKIE_ARG=(--cookies "$COOKIES_FILE")
elif [[ -n "${COOKIES_FROM:-}" ]]; then
    COOKIE_ARG=(--cookies-from-browser "$COOKIES_FROM")
elif [[ -f "$COOKIES_LOCAL" ]]; then
    COOKIE_ARG=(--cookies "$COOKIES_LOCAL")
fi

BASE_ARGS+=("${COOKIE_ARG[@]}")

# ── quality / format / info ───────────────────────────────────
case "$MODE" in
    best)
        FORMAT_ARGS=(-f "bestvideo+bestaudio/best")
        ;;
    audio)
        FORMAT_ARGS=(-f "bestaudio" --extract-audio --audio-format m4a)
        ;;
    sub-only)
        FORMAT_ARGS=(--skip-download --write-subs --write-auto-subs)
        ;;
    custom)
        FORMAT_ARGS=(-f "$CUSTOM_FORMAT")
        ;;
    list)
        exec yt-dlp -F "${COOKIE_ARG[@]}" "$URL"
        ;;
    info)
        exec yt-dlp -J "${COOKIE_ARG[@]}" "$URL"
        ;;
    default)
        FORMAT_ARGS=(-f "bestvideo[height<=1080]+bestaudio/best[height<=1080]")
        ;;
esac

# ── classify URL to determine output directory ────────────────
# Uses yt-dlp --print to extract fields (no JSON parsing needed)
echo "🔍 Analyzing URL ..."
CLASSIFY_CMD=(yt-dlp --print playlist_title --print channel --print uploader \
    --flat-playlist --socket-timeout 10 "${COOKIE_ARG[@]}" "$URL")
# Print classify command with cookie path masked for privacy
CLASSIFY_CMD_SAFE=("${CLASSIFY_CMD[@]}")
for i in "${!CLASSIFY_CMD_SAFE[@]}"; do
    if [[ "${CLASSIFY_CMD_SAFE[$i]}" == --cookies && $((i+1)) -lt "${#CLASSIFY_CMD_SAFE[@]}" ]]; then
        CLASSIFY_CMD_SAFE[$((i+1))]='***'
    fi
done
echo "  └─ ${CLASSIFY_CMD_SAFE[*]}"

# Use timeout to prevent hanging on network issues (10s hard limit)
CLASSIFY=$(timeout 10 "${CLASSIFY_CMD[@]}" 2>/dev/null) || true

if [[ -n "$CLASSIFY" ]]; then
    echo "  ✓ classify result:"
    echo "    playlist_title = $(echo "$CLASSIFY" | sed -n '1p')"
    echo "    channel        = $(echo "$CLASSIFY" | sed -n '2p')"
    echo "    uploader       = $(echo "$CLASSIFY" | sed -n '3p')"
    PLAYLIST_TITLE=$(echo "$CLASSIFY" | sed -n '1p')
    CHANNEL=$(echo "$CLASSIFY" | sed -n '2p')
    UPLOADER=$(echo "$CLASSIFY" | sed -n '3p')
fi

if [[ -n "${PLAYLIST_TITLE:-}" && "${PLAYLIST_TITLE:-}" != "NA" ]]; then
    CATEGORY="playlist"
    CATEGORY_NAME="$PLAYLIST_TITLE"
elif [[ -n "${CHANNEL:-}" && "${CHANNEL:-}" != "NA" ]]; then
    CATEGORY="channel"
    CATEGORY_NAME="$CHANNEL"
elif [[ -n "${UPLOADER:-}" && "${UPLOADER:-}" != "NA" ]]; then
    CATEGORY="channel"
    CATEGORY_NAME="$UPLOADER"
else
    CATEGORY="single"
    CATEGORY_NAME=""
    echo "  (classify unavailable, saved to single/)"
fi

# Sanitize name for directory use
sanitize() {
    echo "$1" | sed 's/[\/:*?"<>|]//g' | sed 's/  */ /g' | sed 's/^ *//;s/ *$//'
}

CATEGORY_NAME=$(sanitize "$CATEGORY_NAME")

if [[ -n "$CATEGORY_NAME" ]]; then
    OUTPUT_DIR="${VIDEO_DL_ROOT}/${CATEGORY}/${CATEGORY_NAME}"
else
    OUTPUT_DIR="${VIDEO_DL_ROOT}/single"
fi

echo "📂 Target: $OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# ── output template ───────────────────────────────────────────
# Structure: <CategoryDir>/<UploadDate>-<Title>/<Title>.ext
OUTPUT_TMPL="${OUTPUT_DIR}/%(upload_date>%Y%m%d)s-%(title)s/%(title)s.%(ext)s"

echo "⬇️  Downloading ..."
echo "  └─ yt-dlp ${BASE_ARGS[*]} ${FORMAT_ARGS[*]} -o $OUTPUT_TMPL $URL"

yt-dlp "${BASE_ARGS[@]}" "${FORMAT_ARGS[@]}" \
    -o "$OUTPUT_TMPL" \
    "$URL"

echo "✅ Done"
