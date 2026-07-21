// ─── static-server — Static HTTP file server ──────────────
//
// Serves a directory over HTTP. Generic — usable by any module.
//
// Usage:
//   cargo run --bin static-server -- /path/to/serve
//   # → http://localhost:8080
//
// Environment:
//   SERVE_ROOT  — alternative to CLI argument
//   PORT        — server port (default: 8080)
// ─────────────────────────────────────────────────────────────

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use tiny_http::{Header, Response, StatusCode, Server};

fn main() {
    // ── resolve root directory ───────────────────────────────
    let root = env::args()
        .nth(1)
        .or_else(|| env::var("SERVE_ROOT").ok())
        .unwrap_or_else(|| {
            eprintln!("Usage: static-server <SERVE_ROOT>");
            eprintln!("       or set SERVE_ROOT environment variable");
            process::exit(1);
        });

    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        eprintln!("✗ Not a directory: {}", root);
        process::exit(1);
    }

    // Canonicalize root once for path traversal checks
    let root_canonical = root_path.canonicalize().unwrap_or_else(|e| {
        eprintln!("✗ Failed to canonicalize root: {}", e);
        process::exit(1);
    });

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("✗ Failed to bind {}: {}", addr, e);
        process::exit(1);
    });

    println!("🌐 Serving {} on http://localhost:{}", root, port);
    println!("   Press Ctrl+C to stop");

    // ── helpers ──────────────────────────────────────────────
    fn mime_type(ext: &str) -> &'static str {
        match ext {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript",
            "json" => "application/json",
            "mp4" => "video/mp4",
            "mkv" => "video/x-matroska",
            "webm" => "video/webm",
            "vtt" => "text/vtt; charset=utf-8",
            "srt" => "text/plain; charset=utf-8",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "woff2" => "font/woff2",
            "woff" => "font/woff",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            _ => "application/octet-stream",
        }
    }

    fn header_from_str(name: &str, value: &str) -> Header {
        Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
    }

    /// Parse a `Range: bytes=...` header.
    /// Returns `(start, end)` where end is inclusive (0-indexed).
    /// Supports: `0-1023`, `1024-` (to end), `-1024` (last N bytes).
    fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
        let rest = range_str.strip_prefix("bytes=")?;
        let (start_str, end_str) = rest.split_once('-')?;

        match (start_str.is_empty(), end_str.is_empty()) {
            (false, false) => {
                // bytes=0-1023
                let start: u64 = start_str.parse().ok()?;
                let end: u64 = end_str.parse().ok()?;
                if start > end || end >= file_size {
                    return None;
                }
                Some((start, end))
            }
            (false, true) => {
                // bytes=1024-
                let start: u64 = start_str.parse().ok()?;
                if start >= file_size {
                    return None;
                }
                Some((start, file_size - 1))
            }
            (true, false) => {
                // bytes=-1024 (last N bytes)
                let n: u64 = end_str.parse().ok()?;
                if n == 0 {
                    return None;
                }
                let start = file_size.saturating_sub(n);
                Some((start, file_size - 1))
            }
            (true, true) => None,
        }
    }

    // ── request loop ─────────────────────────────────────────
    for request in server.incoming_requests() {
        let url = percent_encoding::percent_decode_str(request.url())
            .decode_utf8_lossy()
            .to_string();

        // Strip query string (?...) from URL
        let path_only = url.split('?').next().unwrap_or(&url);
        // Resolve the full path and verify it's under root
        let rel_path = if path_only == "/" || path_only.is_empty() {
            "index.html".to_string()
        } else {
            path_only.trim_start_matches('/').to_string()
        };

        let full_path = root_path.join(&rel_path);

        // Security: canonicalize and verify the resolved path stays under root
        let canonical = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                let _ = request.respond(
                    Response::from_string("Not Found").with_status_code(404),
                );
                continue;
            }
        };

        if !canonical.starts_with(&root_canonical) {
            let _ = request.respond(
                Response::from_string("Forbidden").with_status_code(403),
            );
            continue;
        }

        if canonical.is_dir() {
            // Directory -> redirect to index.html in that dir
            let index_path = canonical.join("index.html");
            if index_path.is_file() {
                let redirect = format!("/{}/index.html", rel_path.trim_end_matches('/'));
                let _ = request.respond(
                    Response::from_string("")
                        .with_status_code(302)
                        .with_header(header_from_str("Location", &redirect)),
                );
            } else {
                let _ = request.respond(
                    Response::from_string("Directory listing not available")
                        .with_status_code(403),
                );
            }
            continue;
        }

        if !canonical.is_file() {
            let _ = request.respond(
                Response::from_string("Not Found").with_status_code(404),
            );
            continue;
        }

        // ── read and serve file ──────────────────────────────
        let file_data = match fs::read(&canonical) {
            Ok(d) => d,
            Err(e) => {
                let _ = request.respond(
                    Response::from_string(format!("Internal error: {}", e))
                        .with_status_code(500),
                );
                continue;
            }
        };

        let file_size = file_data.len() as u64;
        let ext = canonical
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let mime = mime_type(ext);

        // Check for Range header
        let range_header = request.headers().iter().find(|h| {
            h.field.as_str().as_str().eq_ignore_ascii_case("range")
        });

        if let Some(range_hdr) = range_header {
            if let Some((start, end)) = parse_range(range_hdr.value.as_str(), file_size) {
                let chunk = &file_data[start as usize..=end as usize];
                let content_range =
                    format!("bytes {}-{}/{}", start, end, file_size);

                let response = Response::from_data(chunk.to_vec())
                    .with_status_code(StatusCode(206))
                    .with_header(header_from_str("Content-Type", mime))
                    .with_header(header_from_str(
                        "Content-Range",
                        &content_range,
                    ))
                    .with_header(header_from_str(
                        "Content-Length",
                        &(end - start + 1).to_string(),
                    ))
                    .with_header(header_from_str(
                        "Accept-Ranges",
                        "bytes",
                    ))
                    .with_header(header_from_str(
                        "Access-Control-Allow-Origin",
                        "*",
                    ));

                let _ = request.respond(response);
                continue;
            } else {
                // Invalid range -> 416 Range Not Satisfiable
                let response = Response::from_string("Range Not Satisfiable")
                    .with_status_code(416)
                    .with_header(header_from_str(
                        "Content-Range",
                        &format!("bytes */{}", file_size),
                    ));
                let _ = request.respond(response);
                continue;
            }
        }

        // ── full file response ───────────────────────────────
        let response = Response::from_data(file_data)
            .with_header(header_from_str("Content-Type", mime))
            .with_header(header_from_str("Access-Control-Allow-Origin", "*"))
            .with_header(header_from_str(
                "Cache-Control",
                "public, max-age=3600",
            ))
            .with_header(header_from_str("Accept-Ranges", "bytes"));

        let _ = request.respond(response);
    }
}
