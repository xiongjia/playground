#!/usr/bin/env bash
# detect-net.sh — read-only network probe for the dns module.
#
# Prints KEY=VALUE facts consumed by gen-corefile.sh / status.sh / doctor.sh.
# No side effects; safe to run anytime (also from `just dns::env`).
#
# Facts:
#   OS            macos | linux | other
#   PR_IF         primary (default-route) interface
#   PR_IP         primary IPv4
#   PR_IP_INT     primary IPv4 as integer
#   PR_MASK_INT   subnet mask as integer
#   PR_NET_INT    subnet address as integer
#   AT_HOME       1 when the machine looks like it is on its home LAN
#                 (DNS_LAN_PROBE reachable, or primary addr is non-public & not a tunnel iface)
#   TAIL_ACTIVE   1 when `tailscale status` succeeds (NOT just `tailscale ip`)
#   TAIL_IP       tailscale IPv4 (100.x) when active
#   PROBE_OK      1 when DNS_LAN_PROBE is set and reachable, 0 when set and down, empty when unset

set -u

case "$(uname -s)" in
  Darwin) OS="macos" ;;
  Linux)  OS="linux" ;;
  *)      OS="other" ;;
esac

# ── primary interface ────────────────────────────────────────
PR_IF=""
if [ "$OS" = "macos" ]; then
  PR_IF=$(route -n get default 2>/dev/null | awk '/interface:/{print $2; exit}')
else
  PR_IF=$(ip route show default 2>/dev/null | awk '/^default/{print $5; exit}')
fi

# ── primary ip + mask ────────────────────────────────────────
PR_IP=""; PR_MASK_INT=""
if [ -n "$PR_IF" ]; then
  if [ "$OS" = "macos" ]; then
    PR_IP=$(ipconfig getifaddr "$PR_IF" 2>/dev/null || true)
    mask_hex=$(ifconfig "$PR_IF" 2>/dev/null | awk '/inet /{print $4; exit}')
    if [ -n "$mask_hex" ]; then
      hex=${mask_hex#0x}
      PR_MASK_INT=$(( 16#$hex ))
    fi
  else
    line=$(ip -4 -o addr show dev "$PR_IF" 2>/dev/null | awk '{print $4; exit}')
    PR_IP=${line%%/*}
    cidr=${line##*/}
    if [ -n "$cidr" ] && [ "$cidr" != "$PR_IP" ]; then
      PR_MASK_INT=$(( (0xFFFFFFFF << (32 - cidr)) & 0xFFFFFFFF ))
    fi
  fi
fi

ip_to_int() { local a b c d; IFS=. read -r a b c d <<<"$1"; echo $(( (a<<24) + (b<<16) + (c<<8) + d )); }
int_to_ip() { local n=$1; echo "$(( (n>>24)&255 )).$(( (n>>16)&255 )).$(( (n>>8)&255 )).$(( n&255 ))"; }

PR_IP_INT=""; PR_NET_INT=""
if [ -n "$PR_IP" ]; then
  PR_IP_INT=$(ip_to_int "$PR_IP")
  [ -n "$PR_MASK_INT" ] && PR_NET_INT=$(( PR_IP_INT & PR_MASK_INT ))
fi

# True for private/loopback/CGNAT-ish ranges — i.e. "not a public unicast address".
# CGNAT 100.64/10 (used by tailscale) is included on purpose; AT_HOME additionally
# excludes tunnel interfaces, so it never counts as "home LAN".
is_non_public() {
  local n=$1 a b
  a=$(( n>>24 )); b=$(( (n>>16)&255 ))
  { [ "$a" -eq 10 ] \
    || { [ "$a" -eq 172 ] && [ "$b" -ge 16 ] && [ "$b" -le 31 ]; } \
    || { [ "$a" -eq 192 ] && [ "$b" -eq 168 ]; } \
    || { [ "$a" -eq 100 ] && [ "$b" -ge 64 ] && [ "$b" -le 127 ]; } \
    || [ "$a" -eq 127 ]; } 2>/dev/null
}

# ── optional home-LAN probe ──────────────────────────────────
PROBE_OK=""   # empty = not configured
if [ -n "${DNS_LAN_PROBE:-}" ]; then
  if [ "$OS" = "macos" ]; then
    if ping -c 1 -W 1500 "$DNS_LAN_PROBE" >/dev/null 2>&1; then PROBE_OK=1; else PROBE_OK=0; fi
  else
    if ping -c 1 -W 2 "$DNS_LAN_PROBE" >/dev/null 2>&1; then PROBE_OK=1; else PROBE_OK=0; fi
  fi
fi

# ── AT_HOME: probe if configured, else non-public-primary heuristic ──
AT_HOME=0
if [ -n "${DNS_LAN_PROBE:-}" ]; then
  [ "$PROBE_OK" = "1" ] && AT_HOME=1
else
  case "$PR_IF" in
    utun*|tun*|tailscale*) : ;;                       # tunnel iface → not home LAN
    *)
      if [ -n "$PR_IP_INT" ] && is_non_public "$PR_IP_INT"; then AT_HOME=1; fi ;;
  esac
fi

# ── tailscale state (time-capped; the CLI can hang) ──────────
# Note: `tailscale ip` alone is unreliable (prints cached 100.x even when stopped),
# so activity is judged by `tailscale status` exit code only.
# macOS has no coreutils `timeout`, so a portable 2s cap is emulated (tailscale status hangs when the daemon is stopped).

tailscale_status() {
  local rc
  if command -v timeout >/dev/null 2>&1; then
    timeout 2 tailscale status >/dev/null 2>&1
    rc=$?
  else
    tailscale status >/dev/null 2>&1 &
    local p=$!
    { sleep 2; kill "$p" 2>/dev/null; } &
    local w=$!
    wait "$p" 2>/dev/null
    rc=$?
    kill "$w" 2>/dev/null
  fi
  return "$rc"
}

tailscale_ip() {
  local out="" tmp rc
  tmp=$(mktemp) || return 1
  if command -v timeout >/dev/null 2>&1; then
    timeout 2 tailscale ip -4 >"$tmp" 2>/dev/null
    rc=$?
  else
    tailscale ip -4 >"$tmp" 2>/dev/null &
    local p=$!
    { sleep 2; kill "$p" 2>/dev/null; } &
    local w=$!
    wait "$p" 2>/dev/null
    rc=$?
    kill "$w" 2>/dev/null
  fi
  out=$(head -1 "$tmp" 2>/dev/null || true)
  rm -f "$tmp"
  [ "$rc" -eq 0 ] && [ -n "$out" ] && echo "$out"
}

TAIL_ACTIVE=0; TAIL_IP=""
if command -v tailscale >/dev/null 2>&1 && tailscale_status; then
  TAIL_ACTIVE=1
  TAIL_IP=$(tailscale_ip || true)
fi

# ── output ───────────────────────────────────────────────────
echo "OS=$OS"
echo "PR_IF=${PR_IF:-}"
echo "PR_IP=${PR_IP:-}"
echo "PR_IP_INT=${PR_IP_INT:-}"
echo "PR_MASK_INT=${PR_MASK_INT:-}"
echo "PR_NET_INT=${PR_NET_INT:-}"
echo "AT_HOME=${AT_HOME}"
echo "TAIL_ACTIVE=${TAIL_ACTIVE}"
echo "TAIL_IP=${TAIL_IP:-}"
echo "PROBE_OK=${PROBE_OK:-}"
