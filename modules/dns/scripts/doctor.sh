#!/usr/bin/env bash
# doctor.sh — read-only prerequisite + config check for the dns module.
# Exit 1 when something critical is missing (binary, resolver file, or records).

set -u

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
MOD_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG_DIR="$MOD_DIR/config"
DOMAIN=${DNS_DOMAIN:-home.arpa}
BIN=${DNS_BIN:-coredns}
PORT=${DNS_PORT:-5363}
BIND_IP=${DNS_BIND_IP:-127.0.0.1}

rc=0
ok()   { echo "✓ $1"; }
warn() { echo "⚠ $1"; rc=1; }

# ── binary ────────────────────────────────────────────────────
if command -v "$BIN" >/dev/null 2>&1; then
  ok "$BIN found: $(command -v "$BIN")"
else
  warn "$BIN not found — one-time manual install: brew install coredns"
fi

# ── /etc/resolver (one-time manual setup per README) ─────────
res_file="/etc/resolver/$DOMAIN"
res_fix() { # actionable hint when the resolver file disagrees with the module config
  warn "  fix: sudo sed -i '' \"s/^port .*/port $PORT/\" $res_file && sudo dscacheutil -flushcache"
}
if [ -f "$res_file" ]; then
  ok "/etc/resolver/$DOMAIN exists:"
  sed 's/^/    /' "$res_file"
  if grep -q "port $PORT" "$res_file" 2>/dev/null; then
    ok "resolver points at port $PORT"
  else
    warn "resolver file does not mention port $PORT — check contents"
    res_fix
  fi
else
  warn "/etc/resolver/$DOMAIN missing — follow the README 'One-time setup' (manual, sudo once):"
  warn "  domain $DOMAIN / nameserver $BIND_IP / port $PORT"
fi

# ── DNS_RECORDS parse ─────────────────────────────────────────
if bash "$SCRIPT_DIR/init-config.sh" --check >/dev/null 2>&1; then
  ok "DNS_RECORDS parses (config/.env)"
else
  warn "DNS_RECORDS invalid or empty (run 'just dns::init-config' to see the error)"
fi

# ── network facts (read-only) ─────────────────────────────────
facts=$(bash "$SCRIPT_DIR/detect-net.sh")
echo "─ network facts ─"
printf '%s\n' "$facts" | sed 's/^/  /'
home=$(printf '%s\n' "$facts" | sed -n 's/^AT_HOME=//p')
if [ "$home" = "1" ]; then
  echo "  → looks like you are on the home LAN"
else
  echo "  → not detected on the home LAN"
fi

exit "$rc"
