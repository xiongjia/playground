# dns — on-demand local DNS (CoreDNS)

Temporarily resolves your **own private names** (`*.home.arpa`) for **this machine only**, with the
address picked by the current network:

| You are…         | `vm001.home.arpa` resolves to       |
| ---------------- | ----------------------------------- |
| on the home LAN  | `192.168.71.10` (direct, no tunnel) |
| away (tailscale) | vm001's tailscale IP (`100.x`)      |

Nothing runs by default: `just dns::up` starts a local CoreDNS on `127.0.0.1:5363`, `just dns::down`
stops it. When stopped, only your private TLD is affected (NXDOMAIN); normal browsing never touches
this server and is unaffected.

Both the full name (`vm001.home.arpa`) **and the bare short name** (`vm001`) resolve — the short
name works because the server also serves `vm001` as an alias, and macOS appends the `home.arpa`
search domain to single-label lookups (one-time optional step below).

## Prerequisites (one-time, manual — no scripts)

```bash
brew install coredns
```

> `brew install coredns` does **not** auto-start anything (no launchd service, no port listener). It
> only installs the binary.

## One-time setup (manual, sudo once)

Route only the private domain to the local CoreDNS via a per-domain resolver file (this does not
touch system-wide DNS settings):

```bash
sudo mkdir -p /etc/resolver
sudo tee /etc/resolver/home.arpa >/dev/null <<'EOF'
domain home.arpa
nameserver 127.0.0.1
port 5363
EOF
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

(If you changed `DNS_DOMAIN` in `.env`, use that name instead of `home.arpa`.)

Optional — use bare short names (`ping vm001` instead of `ping vm001.home.arpa`). Add `home.arpa` as
a search domain on your primary network (GUI: System Settings → Wi-Fi/Network → Details → DNS →
Search Domains, add `home.arpa`; or CLI, once per interface):

```bash
sudo networksetup -listallnetworkservices   # find your service name, e.g. Wi-Fi
sudo networksetup -setsearchdomains Wi-Fi home.arpa
```

To revert later:

```bash
sudo rm /etc/resolver/home.arpa
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

Only queries for `*.home.arpa` are routed to CoreDNS; every other domain keeps using your normal
DNS. When CoreDNS is down, `*.home.arpa` lookups fail fast (NXDOMAIN) — they do **not** fall back to
the public internet (`.home.arpa` is reserved by RFC 8375 and does not exist publicly), and nothing
else slows down.

## Configuration

All settings live in `config/.env` (see `config/.env.example`). `DNS_RECORDS` is the single source
of truth for the hostname → address mapping:

```bash
DNS_ENABLED=true                          # safety switch; dns::up refuses unless true
DNS_RECORDS="vm001=192.168.71.10,vm001.tail520e20.ts.net"
```

Format: space-separated `name=LAN[,TAIL]` entries; a name without `.` gets `DNS_DOMAIN` appended
(`vm001` → `vm001.home.arpa`), a name with `.` is used as-is. Each token is one of:

| token    | meaning                                                                                             |
| -------- | --------------------------------------------------------------------------------------------------- |
| IP       | static address                                                                                      |
| hostname | e.g. `vm001.tailXXXX.ts.net` (MagicDNS); resolved to the current `100.x` when the file is generated |
| `auto`   | this machine's own address (LAN IP when at home, else tail IP)                                      |
| `-`      | no candidate for that slot                                                                          |

| example                                       | behaviour                                                         |
| --------------------------------------------- | ----------------------------------------------------------------- |
| `vm001=192.168.71.10,vm001.tail520e20.ts.net` | home → LAN IP; away → tail IP (recommended for a fixed machine)   |
| `vm001=192.168.71.10,100.65.66.22`            | same, tail IP written by hand                                     |
| `router=192.168.1.1,-`                        | LAN-only record                                                   |
| `me=auto`                                     | this machine itself                                               |
| `gh=vm001.tail520e20.ts.net`                  | alias — always the tail IP of vm001 (pure alias, no LAN shortcut) |

`-,VAL` (explicit dash) behaves exactly like a single `VAL` — alias/static, served whenever
resolvable; it is not gated on tailscale activity. Only the two-token form `IP,-` (LAN-only) gets
skipped when away.

Optional variables: `DNS_DOMAIN` (default `home.arpa`), `DNS_PORT` (`5363` — high port, no root;
5353 is macOS/App mDNS and often already taken), `DNS_BIND_IP` (`127.0.0.1`), `DNS_UPSTREAM` —
non-matching names are forwarded there; you may give **several upstreams space-separated** (e.g.
`DNS_UPSTREAM="223.5.5.5 119.29.29.29"`); CoreDNS health-checks them and load-balances
automatically. `DNS_GOMAXPROCS` caps the Go runtime's CPU usage (e.g. `2` for a tiny resolver;
default is all cores). `DNS_BIN` overrides the coredns executable (default `coredns` on PATH).

> Linux: the same Corefile works; point a stub resolver at it for `$DNS_DOMAIN` via
> `/etc/resolv.conf` or `systemd-resolved` (the module's macOS-specific helper scripts and the
> `/etc/resolver` step do not apply).

## Usage

```bash
just dns::init-config     # validate DNS_RECORDS and sync → config/records
just dns::up              # sync + pick IPs per current network + start CoreDNS
just dns::run             # same, but foreground — Ctrl-C stops it (logs on screen + coredns.log)
just dns::status          # process / A records / live dig probe
just dns::list            # show the current mapping
just dns::env             # resolved config + network facts
just dns::doctor          # read-only check of prerequisites
just dns::down            # stop CoreDNS (idempotent)
just dns::restart
just dns::log             # tail CoreDNS logs
```

After `dns::up`:

```bash
dig vm001.home.arpa @127.0.0.1 -p 5363     # → 192.168.71.10 (home) or 100.x (away)
dig vm001 @127.0.0.1 -p 5363                # bare short name resolves too
ping vm001                                  # works once the home.arpa search domain is set
```

Runtime files (`config/records`, `hosts.db`, `Corefile`, pid, log) live in `modules/dns/config/`,
which is gitignored and generated — never edit them by hand.

## Troubleshooting

- **`dns::run`/`dns::up` still binds the old port** — `DNS_PORT` may be overridden in `config/.env`
  or `config/.env.dev.local` (local overrides win over the code default). Keep it in sync with the
  `port` line in `/etc/resolver/home.arpa`. The default is `5363`; `5353` is macOS mDNS and often
  held by other apps.

## How the address is picked

On `dns::up` the script probes the network, then per record:

1. **LAN candidate** is used when the machine looks like it is at home: `DNS_LAN_PROBE` reachable if
   configured, otherwise the LAN IP falls inside this machine's current subnet.
2. Otherwise the **TAIL candidate** is used when tailscale is active (`tailscale status` — not
   `tailscale ip`, which can print stale values). A ts.net name is resolved to the current `100.x`
   at generation time, through the system resolver so MagicDNS names match the OS view.
3. No reachable candidate → that record is skipped with a warning (never a wrong IP).

## Safety notes

- `DNS_BIND_IP` defaults to `127.0.0.1` — CoreDNS can act as a recursive forwarder, so binding
  `0.0.0.0` without restrictions would turn this machine into an open resolver.
- Never put real public domains (`github.com`, …) in `DNS_RECORDS`; keep them under the private TLD.
- The one-time `/etc/resolver` change is manual and reversible (commands above); the module never
  writes to `/etc`.
