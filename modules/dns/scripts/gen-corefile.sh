#!/usr/bin/env bash
# gen-corefile.sh — pick the best reachable IP per record and write config/hosts.db + config/Corefile.
#
# Per-record decision (see docs/dns-design.md for the full table):
#   dual candidate name=LAN,TAIL
#     LAN usable (DNS_LAN_PROBE ok, or IP is on the primary home subnet / auto when AT_HOME)
#       → A record = LAN IP                        (home: direct LAN access)
#     else TAIL usable (tailscale active; ip | auto → tail ip | hostname → resolve now)
#       → A record = tail IP                       (away: over tailscale)
#     else → record skipped with a warning
#   single token name=VAL  (LAN = '-', VAL lives in the tail slot)
#     auto  → this machine's own IP (primary LAN IP when AT_HOME, else tail IP)
#     ip    → static A record, always served
#     hostname → resolved at generation time (e.g. alias to a MagicDNS ts.net name)
#
# hostname tokens are resolved to IPv4 here (MagicDNS/ts.net names, through the
# system resolver) so hosts.db only ever contains plain A records. Short TTL in
# Corefile covers drift.

set -eu

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")/config"
RECORDS_FILE="$CONFIG_DIR/records"
BIND_IP=${DNS_BIND_IP:-127.0.0.1}
PORT=${DNS_PORT:-5363}
UPSTREAM=${DNS_UPSTREAM:-223.5.5.5}

if [ ! -f "$RECORDS_FILE" ]; then
  echo "✗ config/records missing — run 'just dns::init-config' first" >&2
  exit 1
fi

# ── load machine facts (detect-net.sh) ───────────────────────
OS=""; PR_IF=""; PR_IP=""; PR_IP_INT=""; PR_MASK_INT=""; PR_NET_INT=""
AT_HOME=0; TAIL_ACTIVE=0; TAIL_IP=""; PROBE_OK=""
while IFS='=' read -r k v; do
  [ -z "$k" ] && continue
  case "$k" in
    OS)          OS=$v          ;; PR_IF)   PR_IF=$v   ;; PR_IP)    PR_IP=$v ;;
    PR_IP_INT)   PR_IP_INT=$v   ;; PR_MASK_INT) PR_MASK_INT=$v ;; PR_NET_INT) PR_NET_INT=$v ;;
    AT_HOME)     AT_HOME=$v     ;; TAIL_ACTIVE) TAIL_ACTIVE=$v ;;
    TAIL_IP)     TAIL_IP=$v     ;; PROBE_OK) PROBE_OK=$v ;;
  esac
done < <(bash "$SCRIPT_DIR/detect-net.sh")

ip_int() { local a b c d; IFS=. read -r a b c d <<<"$1"; echo $(( (a<<24) + (b<<16) + (c<<8) + d )); }

is_ip4() { # strictly 4 octets, each 0–255
  case "$1" in
    ''|*[!0-9.]*) return 1 ;;
  esac
  local -a o
  IFS=. read -r -a o <<< "$1"
  [ "${#o[@]}" -eq 4 ] || return 1
  local octet
  for octet in "${o[@]}"; do
    [ -n "$octet" ] && [ "$octet" -ge 0 ] && [ "$octet" -le 255 ] || return 1
  done
  return 0
}

# resolve a hostname to its first IPv4 — through the system resolver first
# (so MagicDNS/ts.net names resolve the way the OS would), dig as a fallback.
# Always exits 0 (callers run under `set -e`); an empty result means "unresolved".
resolve_host() {
  local out=""
  if command -v dscacheutil >/dev/null 2>&1; then
    out=$(dscacheutil -q host -a name "$1" 2>/dev/null | awk '/^ip_address:/{print $2; exit}')
  fi
  if [ -z "$out" ] && command -v getent >/dev/null 2>&1; then
    out=$(getent ahostsv4 "$1" 2>/dev/null | awk '{print $1; exit}')
  fi
  if [ -z "$out" ] && command -v dig >/dev/null 2>&1; then
    out=$(dig +short +time=2 +tries=1 "$1" A 2>/dev/null | awk '/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$/{print; exit}')
  fi
  if [ -n "$out" ]; then
    echo "$out"
  fi
  return 0
}

# LAN candidate usable? (machine-level context from facts)
lan_ok() { # $1 = LAN token (ip or 'auto')
  if [ -n "${DNS_LAN_PROBE:-}" ]; then
    # probe is authoritative: 1 = home router reachable (token must still be a real candidate)
    [ "${PROBE_OK:-0}" = "1" ] && { [ "$1" = "auto" ] || is_ip4 "$1"; }
  elif [ "$1" = "auto" ]; then
    # "this machine" — only meaningful when we are actually on a home LAN
    [ "${AT_HOME:-0}" = "1" ]
  elif is_ip4 "$1"; then
    # explicit LAN IP: usable when it sits on the primary home subnet
    if [ -n "${PR_NET_INT:-}" ] && [ -n "${PR_MASK_INT:-}" ]; then
      [ $(( $(ip_int "$1") & PR_MASK_INT )) -eq "$PR_NET_INT" ]
    else
      false
    fi
  else
    # anything else in the LAN slot is unusable (init-config already rejects it)
    false
  fi
}

hosts_tmp="$CONFIG_DIR/hosts.db.tmp"
core_tmp="$CONFIG_DIR/Corefile.tmp"
: > "$hosts_tmp"

skipped=0
count=0

while IFS=$'\t' read -r fqdn lan tail_val alias; do
  if [ -z "$fqdn" ]; then
    continue
  fi
  lan=${lan:--}
  tail_val=${tail_val:--}
  alias=${alias:--}
  chosen=""
  where=""

  if [ "$lan" != "-" ]; then
    # dual-candidate mode: home → LAN
    if lan_ok "$lan"; then
      if [ "$lan" = "auto" ]; then chosen="${PR_IP:-}"; else chosen="$lan"; fi
      if [ -n "$chosen" ]; then
        where="lan"
      fi
    fi
    if [ -z "$chosen" ] && [ "$tail_val" != "-" ] && [ "${TAIL_ACTIVE:-0}" = "1" ]; then
      # home check failed → fall back to tail candidate (away from home)
      case "$tail_val" in
        auto) chosen="${TAIL_IP:-}" ;;
        *.*)  chosen=$(resolve_host "$tail_val") ;;
        *)    if is_ip4 "$tail_val"; then chosen="$tail_val"; else chosen=$(resolve_host "$tail_val"); fi ;;
      esac
      if [ -n "$chosen" ]; then
        where="tail"
      fi
    fi
  elif [ "$tail_val" != "-" ]; then
    # alias / static single-token mode (no home gating)
    case "$tail_val" in
      auto)
        if [ "${AT_HOME:-0}" = "1" ] && [ -n "${PR_IP:-}" ]; then
          chosen="${PR_IP}"; where="lan"
        elif [ "${TAIL_ACTIVE:-0}" = "1" ] && [ -n "${TAIL_IP:-}" ]; then
          chosen="${TAIL_IP}"; where="tail"
        fi ;;
      *)
        if is_ip4 "$tail_val"; then
          chosen="$tail_val"; where="static"
        else
          chosen=$(resolve_host "$tail_val")
          if [ -n "$chosen" ]; then
            where="tail"
          fi
        fi ;;
    esac
  fi

  if [ -n "$chosen" ]; then
    names="$fqdn"
    # bare short name alias (vm001) so single-label lookups hit the same record
    if [ "$alias" != "-" ] && [ "$alias" != "$fqdn" ]; then
      names="$fqdn $alias"
    fi
    echo "$chosen $names" >> "$hosts_tmp"
    count=$((count+1))
    echo "✓ $fqdn → $chosen ($where)"
  else
    skipped=$((skipped+1))
    echo "⚠ $fqdn → skipped (no reachable candidate now)" >&2
  fi
done < "$RECORDS_FILE"

if ! mv "$hosts_tmp" "$CONFIG_DIR/hosts.db"; then
  echo "✗ cannot write $CONFIG_DIR/hosts.db" >&2
  exit 1
fi

# ── write Corefile ───────────────────────────────────────────
{
  echo ".:${PORT} {"
  echo "    bind ${BIND_IP}"
  echo "    log"
  echo "    errors"
  echo "    hosts {"
  awk '{ print "        " $0 }' "$CONFIG_DIR/hosts.db"
  echo "        fallthrough"
  echo "    }"
  echo "    forward . ${UPSTREAM} {"
  echo "        max_concurrent 1000"
  echo "    }"
  echo "    cache 30"
  echo "}"
} > "$core_tmp"
if ! mv "$core_tmp" "$CONFIG_DIR/Corefile"; then
  echo "✗ cannot write $CONFIG_DIR/Corefile" >&2
  exit 1
fi

echo "✓ wrote ${count} A record(s) → $CONFIG_DIR/hosts.db, Corefile → $CONFIG_DIR/Corefile"
if [ "$skipped" -gt 0 ]; then
  echo "⚠ ${skipped} record(s) skipped (see warnings above)" >&2
fi
