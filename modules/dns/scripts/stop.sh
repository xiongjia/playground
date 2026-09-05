#!/usr/bin/env bash
# stop.sh — stop the background coredns started by start.sh. Idempotent.

set -u

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")/config"
PIDFILE="$CONFIG_DIR/coredns.pid"

if [ ! -f "$PIDFILE" ]; then
  echo "• coredns is not running (no pidfile)"
  exit 0
fi

pid=$(cat "$PIDFILE" 2>/dev/null || true)
if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
  echo "• coredns not running (stale pidfile removed)"
  rm -f "$PIDFILE"
  exit 0
fi

kill "$pid" 2>/dev/null || true
for _ in 1 2 3 4 5; do
  kill -0 "$pid" 2>/dev/null || break
  sleep 0.2
done
if kill -0 "$pid" 2>/dev/null; then
  kill -9 "$pid" 2>/dev/null || true
fi
rm -f "$PIDFILE"
echo "✓ coredns stopped (was pid $pid)"
