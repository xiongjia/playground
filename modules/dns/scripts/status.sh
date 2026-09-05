#!/usr/bin/env bash
# status.sh — runtime status: process, A records, live query probe. Read-only.

set -u

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")/config"
BIND_IP=${DNS_BIND_IP:-127.0.0.1}
PORT=${DNS_PORT:-5363}
PIDFILE="$CONFIG_DIR/coredns.pid"

running=0
if [ -f "$PIDFILE" ]; then
  pid=$(cat "$PIDFILE" 2>/dev/null || true)
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    running=1
  fi
fi

if [ "$running" = 1 ]; then
  echo "● coredns: running (pid $pid, ${BIND_IP}:${PORT})"
else
  echo "○ coredns: not running"
  exit 0
fi

if [ -f "$CONFIG_DIR/hosts.db" ]; then
  total=$(wc -l < "$CONFIG_DIR/hosts.db" | tr -d ' ')
  echo "A records in hosts.db: $total"
  awk '{ printf "   %-34s %s\n", $2, $1 }' "$CONFIG_DIR/hosts.db"
else
  echo "hosts.db: missing"
  exit 0
fi

first=$(awk 'NR==1{print $2; exit}' "$CONFIG_DIR/hosts.db")
if [ -n "$first" ] && command -v dig >/dev/null 2>&1; then
  ans=$(dig +short +time=2 +tries=1 "$first" @"$BIND_IP" -p "$PORT" 2>/dev/null | tr '\n' ' ')
  echo "live probe: dig $first @${BIND_IP} -p ${PORT} → ${ans:-<no answer>}"
fi
