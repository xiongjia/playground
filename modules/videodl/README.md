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
# 1. Configure
cp config/.env.example config/.env        # set VIDEO_DL_ROOT

# 2. Init directory structure
just videodl::init

# 3. Export cookies from Chrome (required for YouTube)
just videodl::cookies                     # saves to .cookies, auto-loaded

# 4. Download a video
just videodl::dl "https://youtube.com/watch?v=..."

# 5. Generate HTML index and serve
just videodl::gen-index
just videodl::serve                        # http://localhost:8080
```

> `just videodl::cookies` is a one-time step. It exports YouTube cookies from your browser to
> `modules/videodl/.cookies` (gitignored). All subsequent downloads use this file automatically.

## Commands

| Command                    | Description                                   |
| -------------------------- | --------------------------------------------- |
| `dl <url>`                 | Download (≤1080p)                             |
| `dl-best <url>`            | Best quality                                  |
| `dl-audio <url>`           | Audio only                                    |
| `dl-sub-only <url>`        | Subtitles only                                |
| `dl-with-format <f> <url>` | Custom format                                 |
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
