#!/usr/bin/env bash
# init-config.sh — sync DNS_RECORDS (config/.env) → modules/dns/config/records
#
# records format (tab separated):  <fqdn> <LAN-token> <TAIL-token> <alias>
#   tokens:  IP | hostname (e.g. vm001.tailXXXX.ts.net) | auto | '-' (none)
#   LAN slot accepts only: ip | auto | '-'
#   TAIL slot accepts:      ip | hostname | auto | '-'
#   a short name gets $DNS_DOMAIN appended (alias = the short name); a name
#   containing '.' is used as-is and has no short alias.
#
# Idempotent. `--check` validates + previews without writing (used by doctor).
# DNS_RECORDS is the single source of truth; this script is the only writer of
# config/records (config/ itself is gitignored).

set -eu

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONFIG_DIR="$(dirname "$SCRIPT_DIR")/config"
DOMAIN=${DNS_DOMAIN:-home.arpa}
RECORDS=${DNS_RECORDS:-}

CHECK=0
[ "${1:-}" = "--check" ] && CHECK=1

if [ -z "$RECORDS" ]; then
  echo "✗ DNS_RECORDS is empty — add hostname=LAN[,TAIL] entries to config/.env (see README)" >&2
  exit 1
fi

mkdir -p "$CONFIG_DIR"
tmp="$CONFIG_DIR/records.tmp"
: > "$tmp"

# tail-slot grammar: '-', 'auto', IPv4-ish, or hostname chars
valid_token() {
  case "$1" in
    -|auto|"") return 0 ;;
    *[!A-Za-z0-9._-]*) return 1 ;;
    *) return 0 ;;
  esac
}

is_ip4() { # strictly 4 dot-separated octets, each 0–255
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

fail() { echo "$1" >&2; rm -f "$tmp"; exit 1; }

n=0
set -- $RECORDS
for tok in "$@"; do
  n=$((n+1))
  case "$tok" in
    *=*) ;;
    *) fail "✗ entry #$n '$tok': expected name=LAN[,TAIL]" ;;
  esac
  name=${tok%%=*}
  vals=${tok#*=}
  case "$name" in
    ''|*[!A-Za-z0-9._-]*) fail "✗ entry #$n: bad name '$name' (letters/digits/._- only)" ;;
  esac
  IFS=',' read -r -a parts <<< "$vals"
  if [ "${#parts[@]}" -gt 2 ]; then
    fail "✗ entry #$n '$name': too many tokens (max: name=LAN[,TAIL])"
  fi
  lan=${parts[0]:-}
  tail_val=${parts[1]:-}
  # a single token is an alias/static value (lives in the TAIL slot, no LAN candidate)
  if [ "${#parts[@]}" -le 1 ]; then
    tail_val="$lan"
    lan="-"
  fi
  [ -z "$tail_val" ] && tail_val="-"
  # LAN slot is only ever an address candidate: ip | auto | '-'
  if [ "$lan" != "-" ] && [ "$lan" != "auto" ] && ! is_ip4 "$lan"; then
    fail "✗ entry #$n '$name': bad LAN token '$lan' (expect IP | auto | -)"
  fi
  valid_token "$tail_val" || fail "✗ entry #$n '$name': bad TAIL token '$tail_val'"
  case "$name" in
    *.*) fqdn="$name"; alias="-" ;;
    *)   fqdn="$name.$DOMAIN"; alias="$name" ;;
  esac
  printf '%s\t%s\t%s\t%s\n' "$fqdn" "$lan" "$tail_val" "$alias" >> "$tmp"
done

if [ "$CHECK" = 1 ]; then
  echo "✓ DNS_RECORDS OK — $(wc -l < "$tmp" | tr -d ' ') record(s), preview:"
  awk -F '\t' '{ printf "   %-34s lan=%-20s tail=%-22s alias=%s\n", $1, $2, $3, $4 }' "$tmp"
  rm -f "$tmp"
  exit 0
fi

if ! mv "$tmp" "$CONFIG_DIR/records"; then
  echo "✗ cannot write $CONFIG_DIR/records" >&2
  exit 1
fi
echo "✓ synced $n record(s) from DNS_RECORDS → $CONFIG_DIR/records"
awk -F '\t' '{ printf "   %-34s lan=%-20s tail=%-22s alias=%s\n", $1, $2, $3, $4 }' "$CONFIG_DIR/records"
