#!/usr/bin/env bash
# ─── run-and-notify.sh ───────────────────────────────────────
# Universal command wrapper: run a command, measure runtime, and
# send a notification on completion via `notify send`.
#
# Usage:  run-and-notify.sh <label> [--] <command...>
#
# The wrapped command's stdout passes through in real time and its
# last non-empty line is included in the notification as a summary
# (e.g. "14 file(s) exported").  stderr passes through separately
# and is never captured, so progress bars/yT-dlp logs don't pollute
# the notification message.
#
# Environment:
#   NOTIFY_SILENT=true   — skip notification, just run the command
#   NOTIFY_BIN=path      — explicit notify binary path
#
# The original exit code is preserved so && / || chains work.
#
# notify binary lookup order:
#   1. $NOTIFY_BIN (explicit override)
#   2. `notify` in PATH
#   3. cargo run --manifest-path <root>/Cargo.toml --release --bin notify
# ────────────────────────────────────────────────────────────
# NOTE: No set -e here — we must capture the exit code of "$@".

label="$1"
shift

# Skip optional -- separator (improves readability at call sites)
if [ $# -ge 1 ] && [ "$1" = "--" ]; then
    shift
fi

start=$(date +%s)

# Capture stdout for summary while showing everything in real time.
# stderr passes through directly (not captured), so progress output
# from yt-dlp / restic doesn't pollute the notification message.
# Derive project root from script location (not CWD)
# Script is at: <root>/modules/notify/scripts/run-and-notify.sh
_script_dir="$(cd "$(dirname "$0")" && pwd)"
_project_root="$(cd "$_script_dir/../../.." && pwd)"
mkdir -p "$_project_root/.tmp"
output=$(mktemp -p "$_project_root/.tmp" notify-XXXXXX)
trap 'rm -f "$output"' EXIT
"$@" 2>&1 > >(tee "$output")
exit_code=$?

end=$(date +%s)
duration=$((end - start))

# Extract last non-empty line as result summary
summary=$(awk 'NF {last=$0} END {print last}' "$output")

# Format human-readable duration
if [ $duration -ge 3600 ]; then
    formatted="$((duration / 3600))h $(( (duration % 3600) / 60 ))m $((duration % 60))s"
elif [ $duration -ge 60 ]; then
    formatted="$((duration / 60))m $((duration % 60))s"
elif [ $duration -eq 0 ]; then
    formatted="<1s"
else
    formatted="${duration}s"
fi

# Resolve notify binary
if [ -n "${NOTIFY_BIN:-}" ]; then
    notify_cmd=("$NOTIFY_BIN")
elif command -v notify &>/dev/null; then
    notify_cmd=(notify)
else
    notify_cmd=(cargo run -q --manifest-path "$_project_root/Cargo.toml" --release --bin notify --)
fi

# Build message with optional summary line
if [ -n "$summary" ]; then
    msg_suffix="${formatted} · ${summary}"
else
    msg_suffix="${formatted}"
fi

# Notify unless silenced
if [ "${NOTIFY_SILENT:-false}" != "true" ]; then
    if [ $exit_code -eq 0 ]; then
        "${notify_cmd[@]}" send --level success "${label} succeeded (took: ${msg_suffix})"
    else
        "${notify_cmd[@]}" send --level error "${label} failed (took: ${msg_suffix}, exit code: ${exit_code})"
    fi
fi

exit $exit_code
