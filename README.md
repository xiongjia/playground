# Personal automation toolkit

## Modules

| Module      | Description                                 | Commands                                                                              |
| ----------- | ------------------------------------------- | ------------------------------------------------------------------------------------- |
| `backup`    | encrypted incremental backup                | `just backup::run`, `just backup::list`, ...                                          |
| `videodl`   | yt-dlp video & subtitle mgr                 | `just videodl::dl <url>`, `just videodl::serve`, `just videodl::dl-cookie <url>`, ... |
| `notify`    | notification CLI                            | `just notify::send "msg"`, `just notify::watch "cmd"`, `just notify::log list`, ...   |
| `finance`   | beancount + fava ledger toolkit (read-only) | `just finance::check`, `just finance::query <expr>`, `just finance::serve`, ...       |
| `robot`     | desktop automation (anti-sleep, clicker)    | `just robot::setup`, `just robot::anti-sleep`, ...                                    |
| `md-export` | markdown → PDF/EPUB/DOCX exporter           | `just md-export::convert README.md`, `just md-export::convert-all`, ...               |

See `modules/<name>/README.md` for module-specific usage.

## Prepare

### Dependencies

| Tool            | Version                | Source                                                                       |
| --------------- | ---------------------- | ---------------------------------------------------------------------------- |
| `just`          | pinned in `Cargo.toml` | `cargo install just`                                                         |
| `dprint`        | pinned in `Cargo.toml` | `cargo install dprint`                                                       |
| `uv`            | >= 0.4                 | [docs.astral.sh/uv](https://docs.astral.sh/uv/getting-started/installation/) |
| `restic`        | latest                 | [restic.net](https://restic.net)                                             |
| `yt-dlp`        | latest                 | [yt-dlp](https://github.com/yt-dlp/yt-dlp)                                   |
| `pandoc`        | latest                 | [pandoc.org](https://pandoc.org)                                             |
| `weasyprint`    | latest                 | [weasyprint.org](https://weasyprint.org)                                     |
| `notify`        | in `src/bin/`          | `just build-notify` / `just notify::build`                                   |
| `static-server` | in `src/bin/`          | `just build-static-server` / `just videodl::build`                           |

## Quick Start

```bash
cp config/.env.example config/.env
# edit config/.env — fill in your paths and secrets
just backup::init          # one-time setup
just backup::run           # daily backup

# videodl
just videodl::init
just videodl::cookies                         # pre-export cookies (no password prompt)
just videodl::dl "https://..."                 # download with exported cookies
just videodl::dl-cookie "https://..."          # download with live browser cookies
just videodl::gen-index
just videodl::serve                            # http://localhost:8080
just videodl::list "https://..."               # list available formats/subtitles

# notify
just notify::send "Backup done" --level success
just notify::watch "long-task.sh"
just notify::log status

# finance
just finance::setup           # install dependencies (first use)
just finance::check           # validate ledger
just finance::query "SELECT account, sum(position)"
just finance::serve           # http://127.0.0.1:5500

# robot
just robot::setup             # install dependencies (first use)
just robot::anti-sleep        # randomly nudges mouse every 30-90s to prevent sleep (Ctrl-C)

# md-export
just md-export::convert README.md                # single file to PDF
just md-export::convert-all                      # batch convert all .md files
just md-export::convert-all-toc                  # batch convert with table of contents
just md-export::convert-docx README.md           # to DOCX
just md-export::convert-epub README.md           # to EPUB
just md-export::browse                          # open exports in Finder
```

## Development

```bash
just fmt        # format all files
just fmt-check  # check formatting (CI gate)
just build      # build all Rust binaries (release)
just build-debug   # build all Rust binaries (debug)
just test       # run all Rust tests
```
