#!/usr/bin/env bash
# start.sh — sync env → pick IPs → launch coredns.
#
#   (no arg)          background + pidfile          → just dns::up
#   --foreground      attached to the terminal; exits on Ctrl-C → just dns::run
#
# Refuses to run unless DNS_ENABLED=true (safety switch).
# One-time manual setup (brew install coredns, /etc/resolver/<domain>) is done by the
# user per README — never automated here.

set -eu

MODE="bg"
[ "${1:-}" = "--foreground" ] && MODE="fg"

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")/config"
DOMAIN=${DNS_DOMAIN:-home.arpa}
PORT=${DNS_PORT:-5363}
BIND_IP=${DNS_BIND_IP:-127.0.0.1}
BIN=${DNS_BIN:-coredns}
PIDFILE="$CONFIG_DIR/coredns.pid"

# cap the Go runtime's CPU usage (default: all cores → set DNS_GOMAXPROCS=2 for a tiny resolver)
if [ -n "${DNS_GOMAXPROCS:-}" ]; then
  export GOMAXPROCS="$DNS_GOMAXPROCS"
fi

if [ "${DNS_ENABLED:-false}" != "true" ]; then
  echo "✗ DNS_ENABLED is not 'true' — set DNS_ENABLED=true in config/.env (safety switch)" >&2
  exit 1
fi

if ! command -v "$BIN" >/dev/null 2>&1; then
  echo "✗ '$BIN' not found on PATH — one-time manual install: brew install coredns (see README)" >&2
  exit 1
fi

# sync env → records, then records → hosts.db + Corefile
bash "$SCRIPT_DIR/init-config.sh"
bash "$SCRIPT_DIR/gen-corefile.sh" || { echo "✗ gen-corefile failed" >&2; exit 1; }

# refuse to double-run on the same bind/port
if [ -f "$PIDFILE" ]; then
  pid=$(cat "$PIDFILE" 2>/dev/null || true)
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    echo "✗ coredns already running (pid $pid) — 'just dns::down' first" >&2
    exit 1
  fi
  rm -f "$PIDFILE"   # stale pidfile
fi

if [ "$MODE" = "fg" ]; then
  # run attached to the terminal (Ctrl-C stops it); logs go to BOTH the terminal
  # and coredns.log. Process substitution (not a pipe) keeps $! = the real
  # coredns pid, so `just dns::down` from another terminal can stop the server.
  "$BIN" -conf "$CONFIG_DIR/Corefile" > >(tee "$CONFIG_DIR/coredns.log") 2>&1 &
  cpid=$!
  echo "$cpid" > "$PIDFILE"
  trap 'rm -f "$PIDFILE"' EXIT INT TERM
  echo "✓ coredns foreground (pid $cpid) — listening on ${BIND_IP}:${PORT}"
  echo "  Ctrl-C to stop (logs also in $CONFIG_DIR/coredns.log)"
  wait "$cpid"
else
  nohup "$BIN" -conf "$CONFIG_DIR/Corefile" > "$CONFIG_DIR/coredns.log" 2>&1 &
  pid=$!
  echo "$pid" > "$PIDFILE"

  sleep 0.5
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "✗ coredns exited on startup — log tail:" >&2
    tail -n 15 "$CONFIG_DIR/coredns.log" >&2 || true
    rm -f "$PIDFILE"
    exit 1
  fi

  echo "✓ coredns up (pid $pid) — listening on ${BIND_IP}:${PORT}"
  echo "  test: dig <host>.${DOMAIN} @${BIND_IP} -p ${PORT}"
  echo "  stop: just dns::down"
fi
