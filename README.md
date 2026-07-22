# Personal automation toolkit

## Modules

| Module    | Description                  | Commands                                                                              |
| --------- | ---------------------------- | ------------------------------------------------------------------------------------- |
| `backup`  | encrypted incremental backup | `just backup::run`, `just backup::list`, ...                                          |
| `videodl` | yt-dlp video & subtitle mgr  | `just videodl::dl <url>`, `just videodl::serve`, `just videodl::dl-cookie <url>`, ... |

See `modules/<name>/README.md` for module-specific usage.

## Prepare

### Dependencies

| Tool            | Version                | Source                                     |
| --------------- | ---------------------- | ------------------------------------------ |
| `just`          | pinned in `Cargo.toml` | `cargo install just`                       |
| `dprint`        | pinned in `Cargo.toml` | `cargo install dprint`                     |
| `restic`        | latest                 | [restic.net](https://restic.net)           |
| `yt-dlp`        | latest                 | [yt-dlp](https://github.com/yt-dlp/yt-dlp) |
| `static-server` | in `src/bin/`          | `cargo run --bin static-server`            |

## Quick Start

```bash
cp config/.env.example config/.env
# edit config/.env — fill in your paths and secrets
just backup::init          # one-time setup
just backup::run           # daily backup

# videodl
just videodl::init
just videodl::cookies
just videodl::dl "https://..."
just videodl::gen-index
just videodl::serve        # http://localhost:8080
```

## Development

```bash
just fmt        # format all files
just fmt-check  # check formatting (CI gate)
```
