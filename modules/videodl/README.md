# videodl — yt-dlp Video & Subtitle Manager

Download management module built on [yt-dlp](https://github.com/yt-dlp/yt-dlp). Downloads videos and
subtitles, organizes them into a clean directory structure, and generates a static HTML index with
native HTML5 video player for browsing.

## Features

- **Download** — Video + subtitles (English priority), playlist, channel
- **Auto-classify** — Organized into `channel/`, `playlist/`, `single/`
- **Dedup** — Skips already downloaded via `archive.txt`
- **Proxy** — Dedicated `VIDEO_DL_PROXY` (won't interfere with other programs)
- **Cookies** — Auto-loads `.cookies` after `just videodl::cookies`
- **Browser UI** — Static HTML with native HTML5 video, Chrome-compatible via built-in HTTP server
- **Subtitles** — WebVTT with centering and adjustable font size
- **Offline** — All assets served locally via Rust HTTP server

## Directory Structure

```
$VIDEO_DL_ROOT/
├── archive.txt              # Download archive (dedup)
├── index.html               # Browse page (generated)
├── player.html              # Player page (generated)
├── channel/<Name>/<Date>-<Title>/
├── playlist/<Name>/<Date>-<Title>/
└── single/<Date>-<Title>/
```

## Prerequisites

- `yt-dlp` + `ffmpeg` — `brew install yt-dlp ffmpeg`
- Node.js 18+ — for `gen-index.ts`
- Rust toolchain — for `static-server` HTTP server

## Setup

```bash
cp config/.env.example config/.env        # set VIDEO_DL_ROOT
just videodl::init                         # create dirs

# Pre-export cookies (no password prompt on subsequent downloads)
just videodl::cookies
just videodl::dl "https://youtube.com/watch?v=..."

# Or live browser cookies (more reliable but prompts for password every time)
just videodl::dl-cookie "https://youtube.com/watch?v=..."

just videodl::gen-index
just videodl::serve                        # http://localhost:8080
```

## Configuration

All variables go in `config/.env`:

| Variable         | Required | Default      | Description                                                |
| ---------------- | -------- | ------------ | ---------------------------------------------------------- |
| `VIDEO_DL_ROOT`  | Yes      | —            | Download root directory (videos, subtitles, index)         |
| `SUBTITLE_LANGS` | No       | `en,zh-Hans` | Subtitle language priority, comma-separated                |
| `VIDEO_DL_PROXY` | No       | —            | Dedicated proxy (won't interfere with other programs)      |
| `COOKIES_FROM`   | No       | —            | Browser name for cookie export (`chrome`, `firefox`, etc.) |
| `COOKIES_FILE`   | No       | —            | Explicit path to cookies.txt file                          |

## Notifications

Download commands send a Telegram notification on completion (success or failure) with runtime
duration.

**Commands with notifications:**

| Command                          |     Notification     |
| -------------------------------- | :------------------: |
| `dl <url>`                       |          ✅          |
| `dl-best <url>`                  |          ✅          |
| `dl-audio <url>`                 |          ✅          |
| `dl-cookie <url>`                |          ✅          |
| `dl-with-format <f> <url>`       |          ✅          |
| `list`, `info`, `archive`, `env` | — (instant commands) |

Disable globally: `NOTIFY_SILENT=true` in `config/.env`.

## Build

`static-server` (Rust HTTP server) is built automatically by `just videodl::serve`. To build
explicitly:

```bash
# Build all Rust binaries (notify + static-server)
just build

# Build only static-server
just build-static-server
just videodl::build          # module-specific build
```

## Commands

| Command                    | Description                                   |
| -------------------------- | --------------------------------------------- |
| `dl <url>`                 | Download with pre-exported cookies            |
| `dl-best <url>`            | Best quality                                  |
| `dl-audio <url>`           | Audio only                                    |
| `dl-sub-only <url>`        | Subtitles only                                |
| `dl-with-format <f> <url>` | Custom format                                 |
| `dl-cookie <url>`          | Download with live browser cookies            |
| `list <url>`               | List formats/subtitles                        |
| `info <url>`               | Show metadata                                 |
| `env`                      | Show environment, paths, tool versions        |
| `archive`                  | View download archive                         |
| `cookies [browser]`        | Export cookies from browser to .cookies       |
| `gen-index`                | Generate index.html + player.html             |
| `serve`                    | Start HTTP server (auto-builds static-server) |
| `build`                    | Build all Rust deps                           |
| `browse`                   | Open in Finder                                |
| `play <keyword>`           | Search and play                               |
| `update`                   | Update yt-dlp                                 |
