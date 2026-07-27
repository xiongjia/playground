# md-export — Markdown Document Exporter

Export markdown files to **PDF**, **EPUB**, and **DOCX** (MS Word) using
[pandoc](https://pandoc.org) and [weasyprint](https://weasyprint.org). No LaTeX required.

## Features

- **Three formats** — PDF (default), EPUB, DOCX via dedicated commands
- **Batch export** — Scan the whole repo and export all eligible `.md` files at once
- **Selective export** — Export specific files by path, support multiple files
- **Table of contents** — Optional `--toc` variant for every command
- **Chinese support** — Uses Songti SC (macOS) / Noto Serif CJK (Linux) with CID font embedding, no
  extra font installation needed on macOS/Windows
- **Git metadata** — Commit hash, date, branch automatically injected into documents
- **Exclusion rules** — Built-in defaults for non-document files (CHANGELOG, LICENSE…),
  user-overridable
- **Clean output** — Mirror directory structure under `$MD_EXPORT_ROOT`, zero naming conflicts

## Prerequisites

| Tool                                 | Purpose                  | Install                   |
| ------------------------------------ | ------------------------ | ------------------------- |
| [pandoc](https://pandoc.org) 3+      | Core conversion engine   | `brew install pandoc`     |
| [weasyprint](https://weasyprint.org) | PDF rendering (no LaTeX) | `brew install weasyprint` |

### Fonts

Fonts are **embedded in the PDF** (CID embedding), no extra installation needed on the viewer side.

| Platform    | Body font         | Monospace             | Install                      |
| ----------- | ----------------- | --------------------- | ---------------------------- |
| **macOS**   | Songti SC         | SF Mono / Menlo       | Built-in, zero setup         |
| **Linux**   | Noto Serif CJK SC | Noto Sans Mono CJK SC | `apt install fonts-noto-cjk` |
| **Windows** | Microsoft YaHei   | Consolas              | Built-in, zero setup         |

> Body font can be overridden via `MD_EXPORT_PDF_FONT` env var (default: Songti SC). PDF engine can
> be switched via `MD_EXPORT_PDF_ENGINE` (default: weasyprint, options: xelatex/typst).

## Directory Structure

```
$MD_EXPORT_ROOT/
├── pdf/
│   ├── README.pdf
│   ├── docs/
│   │   └── arch.pdf
│   └── modules/
│       ├── backup/README.pdf
│       ├── videodl/README.pdf
│       └── notify/README.pdf
├── epub/
│   └── ...
└── docx/
    └── ...
```

## Setup

```bash
cp config/.env.example config/.env
# Edit config/.env — set MD_EXPORT_ROOT
```

## Commands

| Command                      | Description                                  |
| ---------------------------- | -------------------------------------------- |
| `env`                        | Show environment variables and tool versions |
| `list`                       | List all discoverable `.md` files            |
| `list-out`                   | List already exported files                  |
| `convert <file>...`          | Convert to PDF (default), no TOC             |
| `convert-toc <file>...`      | Convert to PDF with TOC                      |
| `convert-docx <file>...`     | Convert to DOCX, no TOC                      |
| `convert-docx-toc <file>...` | Convert to DOCX with TOC                     |
| `convert-epub <file>...`     | Convert to EPUB, no TOC                      |
| `convert-epub-toc <file>...` | Convert to EPUB with TOC                     |
| `convert-all`                | Convert all to PDF, no TOC                   |
| `convert-all-toc`            | Convert all to PDF with TOC                  |
| `convert-all-docx`           | Convert all to DOCX                          |
| `convert-all-epub`           | Convert all to EPUB                          |
| `clean`                      | Remove all exported files                    |
| `browse`                     | Open export directory in Finder              |

> **Notifications**: `convert-all` / `convert-all-toc` / `convert-all-docx` / `convert-all-epub`
> send a Telegram notification on completion with runtime duration. Disable globally:
> `NOTIFY_SILENT=true` in `config/.env`.

## Examples

```bash
# Export a single file to PDF
just md-export::convert README.md

# Export multiple files to PDF
just md-export::convert docs/arch.md modules/backup/README.md

# Export with table of contents (PDF)
just md-export::convert-toc README.md

# Export to DOCX
just md-export::convert-docx README.md

# Export to EPUB
just md-export::convert-epub README.md

# Export everything to PDF
just md-export::convert-all

# Export everything to DOCX
just md-export::convert-all-docx

# Export with absolute path (flattened to filename)
just md-export::convert /some/other/path/doc.md

# Custom font
MD_EXPORT_PDF_FONT=STKaiti just md-export::convert README.md

# Custom PDF engine (if LaTeX installed)
MD_EXPORT_PDF_ENGINE=xelatex just md-export::convert README.md
```

## Configuration

All variables go in `config/.env`:

| Variable               | Default      | Description                               |
| ---------------------- | ------------ | ----------------------------------------- |
| `MD_EXPORT_ROOT`       | _(required)_ | Export output directory                   |
| `MD_EXPORT_FORMATS`    | `pdf`        | Comma-separated output formats            |
| `MD_EXPORT_PDF_FONT`   | `Songti SC`  | Body font for PDF                         |
| `MD_EXPORT_PDF_ENGINE` | `weasyprint` | PDF rendering engine                      |
| `MD_EXPORT_EXCLUDE`    | _(built-in)_ | Space-separated filename patterns to skip |

### Default excluded files

When `MD_EXPORT_EXCLUDE` is unset, the following files are skipped during `export-all`:

- `CHANGELOG.md`
- `LICENSE.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`

Set `MD_EXPORT_EXCLUDE` in `config/.env` to override completely.
