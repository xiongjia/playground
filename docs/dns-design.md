# DNS Module Design

> On-demand local DNS resolver for a single machine. Maps private names (`*.home.arpa`) to the best
> reachable address: **home LAN IP when on the home network, Tailscale IP when away** — without
> touching `/etc/hosts` on any machine.

---

## 1. Goal & Non-goals

**Goal**

- The user picks short private names (`vm001`) and edits `config/.env` to map each name to candidate
  addresses. On demand (`just dns::up`) a local CoreDNS serves those names for this machine;
  `just dns::down` stops everything. When stopped, normal DNS is untouched.

**Non-goals (v1)**

- No serving of other devices (no `0.0.0.0` bind, no split-horizon by query source).
- No docker, no scripts that modify `/etc`, no permanent daemons.
- No `/etc/hosts` modification on this or any other machine.

---

## 2. Architecture

```
config/.env (DNS_RECORDS — single source of truth)
      │  just dns::init-config   (init-config.sh)
      ▼
modules/dns/config/records      (parsed candidates: fqdn, LAN-token, TAIL-token, short alias)
      │  just dns::up            (start.sh → gen-corefile.sh + detect-net.sh facts)
      ▼
config/hosts.db + config/Corefile   (final A records, resolved for the current network)
      │  launch coredns (background, pidfile)
      ▼
CoreDNS @ 127.0.0.1:5363   (`just dns::run` keeps it attached to the terminal — logs on
screen, Ctrl-C stops it; `just dns::up` backgrounds it with a pidfile, `just dns::down` stops)
      ▲
      │  only *.home.arpa queries are routed here by /etc/resolver/home.arpa
      │  (one-time manual setup, see §5); everything else uses normal DNS
any process on this machine
```

Runtime files under `modules/dns/config/` are generated, gitignored, and never hand-edited.

---

## 3. DNS_RECORDS format

Space-separated `name=LAN[,TAIL]` entries (dotenv-friendly single var). Names without `.` get
`DNS_DOMAIN` (`home.arpa` default) appended; names with `.` are full names.

| token    | meaning                                                                                  |
| -------- | ---------------------------------------------------------------------------------------- |
| IP       | static address                                                                           |
| hostname | e.g. `vm001.tailXXXX.ts.net`; resolved to its current IP at generation (system resolver) |
| `auto`   | this machine's own address (LAN IP when at home, else tail IP)                           |
| `-`      | no candidate for that slot                                                               |

| example                                     | behaviour                                                           |
| ------------------------------------------- | ------------------------------------------------------------------- |
| `vm001=192.168.71.10,vm001.tailXXXX.ts.net` | home → LAN IP (direct); away → tail IP (fixed machine, recommended) |
| `vm001=192.168.71.10,100.65.66.22`          | same, tail IP written by hand                                       |
| `gh=vm001.tailXXXX.ts.net`                  | alias: always vm001's tail IP (single token = alias/static)         |
| `tmp=10.0.0.8`                              | static record, always served                                        |
| `me=auto`                                   | this machine itself                                                 |
| `router=192.168.1.1,-`                      | LAN-only (skipped when away)                                        |

---

## 4. Address selection

`gen-corefile.sh` consumes network facts from `detect-net.sh` and one host per record:

```
facts: PR_IP / PR_NET_INT+PR_MASK_INT (primary subnet), AT_HOME, TAIL_ACTIVE, TAIL_IP,
       PROBE_OK (DNS_LAN_PROBE)

dual candidate (LAN present):
  LAN usable when
    DNS_LAN_PROBE configured → probe reachable
    else token auto         → AT_HOME
    else token IP           → IP inside primary subnet (is_ip4 guarded)
  → A = LAN value
  else TAIL (needs TAIL_ACTIVE):
    auto → TAIL_IP | hostname → resolve now (dig) | IP → as-is
  else → skip record + warn   (never return a wrong address)

alias/static single token (LAN slot = '-'):
  auto   → PR_IP when AT_HOME else TAIL_IP
  hostname → resolve now (e.g. MagicDNS ts.net name → current 100.x)
  IP     → static, always served
```

Rationale:

- **LAN first when at home** — direct traffic, no dependence on the tunnel.
- **Tail fallback when away** — ts.net/MagicDNS names keep working even when the tail IP changes;
  hostname tokens are resolved to a plain A record at generation time so `hosts.db` stays trivial
  and offline-debuggable.
- **Skip, don't guess** — an unreachable candidate yields a warning, not a wrong answer.
- `tailscale status` (not `tailscale ip`) decides activity: `tailscale ip` has been observed
  printing a cached 100.x even when the daemon reports "stopped".

---

## 5. macOS integration (manual, one-time)

CoreDNS runs on a high port **5363** so the daily start/stop needs no root. A per-domain resolver
file tells macOS to send only `home.arpa` queries there:

```
# /etc/resolver/home.arpa  (created once by the user per README — not scripted)
domain home.arpa
nameserver 127.0.0.1
port 5363
```

Then flush: `dscacheutil -flushcache; killall -HUP mDNSResponder`.

- Only `*.home.arpa` is routed to CoreDNS; all other domains keep the normal resolver chain, so
  daily DNS is never slower or affected.
- Bare short names (`vm001`) resolve too: `hosts.db` carries them as aliases on each A record
  **and** macOS appends `home.arpa` as a search domain (one-time manual step, see module README:
  `networksetup -setsearchdomains …`). Without the search domain only `vm001.home.arpa` resolves.
- When CoreDNS is down, `home.arpa` lookups fail fast (loopback connection refused → NXDOMAIN; RFC
  8375 domain, no public collision). No silent wrong answers.
- Revert = remove the file + flush. The module never writes `/etc`.
- Linux variant (README note): same Corefile; point a stub at it via
  `/etc/resolv.conf`/`systemd-resolved` per `DNS_DOMAIN`.

---

## 6. Security

- `DNS_BIND_IP=127.0.0.1` (default). CoreDNS is a recursive forwarder: binding everything without
  restrictions would create an open resolver. Serving other devices is explicitly out of scope for
  v1 (split-horizon layout is future work).
- Never map public domains (`github.com`, …) — private TLD only.
- `DNS_ENABLED` defaults to `false`; `dns::up` refuses to run without the explicit switch.
- No secrets in `DNS_RECORDS`; `modules/dns/config/` is gitignored.

---

## 7. Verification matrix (run as part of acceptance)

| scenario                                       | expected                                                     |
| ---------------------------------------------- | ------------------------------------------------------------ |
| `brew install coredns`                         | nothing listens on any port (no auto-start)                  |
| `dns::init-config` after editing `DNS_RECORDS` | `config/records` updated                                     |
| home, dual candidate                           | LAN IP chosen (`dig vm001.home.arpa @127.0.0.1 -p 5363`)     |
| away (tailscale on), dual candidate            | tail IP chosen (hostname resolved via MagicDNS)              |
| tailscale stopped, no LAN match                | record skipped + warning, others still served                |
| `dns::down`                                    | pid gone; `dig ...home.arpa` → NXDOMAIN; other domains fine  |
| `dns::status`                                  | process + records + live probe                               |
| `dns::doctor`                                  | read-only report of binary / resolver file / records / facts |

Live end-to-end (`dns::up`/`status`) requires the one-time `brew install coredns` + the
`/etc/resolver` file, both done manually by the user per the module README.

---

## 8. Limitations & future work

- Addresses are pinned at generation time (hosts plugin, no CNAME). If a tail IP changes mid-run,
  re-run `dns::up`/`restart`. Short CoreDNS TTL limits client caching.
- Multi-subnet homes: a LAN candidate on a different subnet than the machine's own is conservatively
  skipped unless `DNS_LAN_PROBE` (home router) is configured.
- `auto` assumes the machine itself is the target.
- Future: notify wrapper on start/stop failure, split-horizon serving for LAN/tailnet devices (two
  instances or CoreDNS `view`), DNS-01 TXT via zone file.
