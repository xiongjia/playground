# md-export Module Design

> Design decisions for the markdown-to-PDF/EPUB/DOCX export module.

---

## 1. Why pandoc + weasyprint

- **pandoc** — the de-facto standard document converter; supports Markdown → HTML → any format
- **weasyprint** — HTML/CSS → PDF renderer; pure Python, no LaTeX needed
- **No LaTeX dependency** — lowers the barrier for macOS users who don't have TeX installed

Alternative options considered and rejected:

| Option               | Why rejected                                                                    |
| -------------------- | ------------------------------------------------------------------------------- |
| `wkhtmltopdf`        | Unmaintained, QtWebKit-based, no longer packaged for Homebrew                   |
| `typst`              | Promising but early; pandoc support is via `typst` writer, not a drop-in engine |
| `pdflatex`/`xelatex` | Requires full TeX Live distribution (~2GB); overkill for Markdown docs          |

---

## 2. PDF Rendering Pipeline

Two approaches were tried; the second was chosen.

### Approach A: Two-step (abandoned)

```
pandoc -t html5 → body HTML → wrap in custom HTML → weasyprint → PDF
```

**Problem**: Chinese characters were lost in the pipeline. `pandoc -t html5` outputs valid UTF-8
HTML, but weasyprint couldn't resolve CJK fonts when the HTML lacked a proper `<head>` with charset.

### Approach B: pandoc --pdf-engine (chosen)

```
pandoc --pdf-engine=weasyprint --template=pdf.html → PDF
```

**Why it works**: pandoc manages the full HTML→PDF conversion internally, passing our custom
template to weasyprint. Font embedding and charset handling are correct.

---

## 3. Template Strategy

Three approaches were tried:

| Approach                      | Result                                                                         |
| ----------------------------- | ------------------------------------------------------------------------------ |
| **pandoc default template**   | CSS warnings (`overflow-x: auto`, `gap`, `@media` not supported by weasyprint) |
| **`--css` inject font CSS**   | `$pdf-font$` template variable not expanded in CSS files                       |
| **`-H` inject `<style>` tag** | Works but still uses pandoc default CSS with warnings                          |
| **`--template` custom HTML**  | **Chosen** — minimal CSS, no warnings, full control                            |

### pdf.html

A minimal HTML template that replaces pandoc's default. Contains only the CSS properties that
weasyprint supports:

```css
body { font-family: "Songti SC", "Hiragino Mincho ProN", serif; line-height: 1.8; ... }
```

- No `overflow-x: auto` (not supported by weasyprint)
- No `@media` queries (weasyprint doesn't evaluate them)
- No `gap` in flex/grid contexts (not supported)
- Uses cm/in for margins instead of viewport-relative units

---

## 4. Font Handling

### Embedded CID fonts

PDFs use CID (Character ID) font embedding. All fonts are **subsetted and embedded** in the PDF:

```
EIATWD+PingFang-SC                   CID Type 0C (OT)  emb sub uni
OKAPEP+Songti-SC-Bold                CID TrueType      emb sub uni
```

- `emb=yes` — font data is embedded in the PDF
- `sub=yes` — only used glyphs are subsetted (smaller file size)
- `uni=yes` — ToUnicode CMap included (text selection/search works)

### Font stack

| Platform | Body              | Code                  |
| -------- | ----------------- | --------------------- |
| macOS    | Songti SC         | SF Mono / Menlo       |
| Linux    | Noto Serif CJK SC | Noto Sans Mono CJK SC |
| Windows  | Microsoft YaHei   | Consolas              |

Fallback chain in template:

```css
font-family: "Songti SC", "Hiragino Mincho ProN", "Noto Serif CJK SC", serif;
```

### Why not PingFang SC?

First attempt used `PingFang SC` but some PDF viewers (Preview on macOS) failed to render it.
`Songti SC` is a serif font that works reliably across all viewers.

---

## 5. File Structure

```
modules/md-export/
├── justfile                    # Recipe entry points
├── README.md                   # User documentation
├── templates/
│   ├── pdf.html                # Pandoc HTML template for PDF
│   └── epub.css                # EPUB styling
└── scripts/
    ├── discover.sh             # Find eligible .md files
    └── export.sh               # Core export logic
```

---

## 6. Export Logic

### File discovery (discover.sh)

- Finds all `*.md` files under the project root
- Excludes `.git/`, `.pi/`, `node_modules/`, `.venv/`, `target/`, `*-draft.md`
- Applies `MD_EXPORT_EXCLUDE` patterns (default: CHANGELOG, LICENSE, CONTRIBUTING, SECURITY,
  CODE_OF_CONDUCT)
- Outputs relative paths, one per line

### Core export (export.sh)

For each input file × each format:

1. **Determine output path**:
   - Relative path → mirror directory structure: `docs/arch.md` → `pdf/docs/arch.pdf`
   - Absolute path → flatten to basename: `/path/to/README.md` → `pdf/README.pdf`

2. **Conflict handling**:
   - First write → `README.pdf`
   - Second write → `README.20260727-143022.pdf` (timestamp suffix)
   - Third write → `README.20260727-143022-1.pdf` (timestamp + counter)

3. **Git metadata injection**:
   - `--metadata=git-commit:$(git rev-parse --short HEAD)`
   - `--metadata=git-date:$(git log -1 --format=%cI)`
   - `--metadata=git-branch:$(git rev-parse --abbrev-ref HEAD)`
   - `--metadata=source-path:<relative path>`

4. **Pandoc invocation**:
   - PDF: `pandoc --pdf-engine=weasyprint --template=pdf.html`
   - EPUB: `pandoc --css=epub.css`
   - DOCX: `pandoc` (uses reference.docx if present)

---

## 7. Recipe Design

```
md-export/
├── convert <file>...          → PDF only (default)
├── convert-toc <file>...      → PDF + TOC
├── convert-docx <file>...     → DOCX
├── convert-docx-toc <file>... → DOCX + TOC
├── convert-epub <file>...     → EPUB
├── convert-epub-toc <file>... → EPUB + TOC
├── convert-all                → batch PDF
├── convert-all-toc            → batch PDF + TOC
├── convert-all-docx           → batch DOCX
├── convert-all-epub           → batch EPUB
├── list                       → discoverable .md files
├── list-out                   → already exported files
├── env                        → show config & tool versions
├── clean                      → remove all exports
└── browse                     → open in Finder
```

Format-specific commands use `export MD_EXPORT_FORMATS=<fmt>` to propagate the env var through bash
pipes (inline `VAR=val cmd1 | cmd2` only affects cmd1).

---

## 8. Configuration

| Variable               | Default      | Purpose                            |
| ---------------------- | ------------ | ---------------------------------- |
| `MD_EXPORT_ROOT`       | _(required)_ | Output directory                   |
| `MD_EXPORT_FORMATS`    | `pdf`        | Comma-separated output formats     |
| `MD_EXPORT_PDF_FONT`   | `Songti SC`  | PDF body font name                 |
| `MD_EXPORT_PDF_ENGINE` | `weasyprint` | PDF rendering engine               |
| `MD_EXPORT_EXCLUDE`    | _(built-in)_ | Filename patterns to skip in batch |

---

## 10. Notifications

Batch export commands automatically send a notification on completion via `modules/notify`.

**Affected commands:**

- `md-export::convert-all`
- `md-export::convert-all-toc`
- `md-export::convert-all-docx`
- `md-export::convert-all-epub`

Single-file `convert <file>` does not send notification (typically fast).

**Notification behavior:**

| Trigger        | Channel           | Message                                                                            |
| -------------- | ----------------- | ---------------------------------------------------------------------------------- |
| Export success | Telegram / Stdout | `✅ md-export::convert-all succeeded (took: 1m23s · ✅ Done: 14 file(s) exported)` |
| Export failure | Telegram / Stdout | `❌ md-export::convert-all failed (took: 0m45s, exit code: 2)`                     |

**Control:**

- `NOTIFY_SILENT=true` in `config/.env` — disable all notifications globally

---

## 11. Limitations & Future Work

- **Chinese font rendering** — relies on macOS system fonts; Linux users must install
  `fonts-noto-cjk`
- **Image paths** — relative image paths in markdown may break if the source file is outside the
  project root (not an issue for this repo, which has no markdown images)
- **Internal anchor links** — anchors like `#getting-started--create-a-map` produce "No anchor"
  warnings from pandoc; these are cosmetic and don't affect rendering
- **No PDF/A compliance** — generated PDFs are standard 1.7, not PDF/A
