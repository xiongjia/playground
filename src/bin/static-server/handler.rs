// ─── Per-request handler for static-server ────────────────

use std::fs;
use std::path::Path;
use tiny_http::{Header, Request, Response, StatusCode};

use crate::mime;
use crate::range;

/// Build a `tiny_http::Header` from string parts (panics on invalid input).
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

/// Handle one incoming HTTP request.
///
/// Resolves the URL to a file under `root`, performs path traversal
/// checks, and serves the file with appropriate MIME type, CORS
/// headers, and Range request support.
pub fn handle_one(request: Request, root: &Path, root_canonical: &Path) {
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

    let full_path = root.join(&rel_path);

    // Security: canonicalize and verify the resolved path stays under root
    let canonical = match full_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            let _ = request.respond(
                Response::from_string("Not Found").with_status_code(404),
            );
            return;
        }
    };

    if !canonical.starts_with(root_canonical) {
        let _ = request.respond(
            Response::from_string("Forbidden").with_status_code(403),
        );
        return;
    }

    if canonical.is_dir() {
        // Directory -> redirect to index.html in that dir
        let index_path = canonical.join("index.html");
        if index_path.is_file() {
            let redirect = format!("/{}/index.html", rel_path.trim_end_matches('/'));
            let _ = request.respond(
                Response::from_string("")
                    .with_status_code(302)
                    .with_header(header("Location", &redirect)),
            );
        } else {
            let _ = request.respond(
                Response::from_string("Directory listing not available")
                    .with_status_code(403),
            );
        }
        return;
    }

    if !canonical.is_file() {
        let _ = request.respond(
            Response::from_string("Not Found").with_status_code(404),
        );
        return;
    }

    // ── read and serve file ──────────────────────────────
    let file_data = match fs::read(&canonical) {
        Ok(d) => d,
        Err(e) => {
            let _ = request.respond(
                Response::from_string(format!("Internal error: {}", e))
                    .with_status_code(500),
            );
            return;
        }
    };

    let file_size = file_data.len() as u64;
    let ext = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let mime_type = mime::from_ext(ext);

    // Check for Range header
    let range_header = request.headers().iter().find(|h| {
        h.field.as_str().as_str().eq_ignore_ascii_case("range")
    });

    if let Some(range_hdr) = range_header {
        if let Some((start, end)) = range::parse_range(range_hdr.value.as_str(), file_size) {
            let chunk = &file_data[start as usize..=end as usize];
            let content_range = format!("bytes {}-{}/{}", start, end, file_size);

            let response = Response::from_data(chunk.to_vec())
                .with_status_code(StatusCode(206))
                .with_header(header("Content-Type", mime_type))
                .with_header(header("Content-Range", &content_range))
                .with_header(header("Content-Length", &(end - start + 1).to_string()))
                .with_header(header("Accept-Ranges", "bytes"))
                .with_header(header("Access-Control-Allow-Origin", "*"));

            let _ = request.respond(response);
            return;
        } else {
            // Invalid range -> 416 Range Not Satisfiable
            let response = Response::from_string("Range Not Satisfiable")
                .with_status_code(416)
                .with_header(header("Content-Range", &format!("bytes */{}", file_size)));
            let _ = request.respond(response);
            return;
        }
    }

    // ── full file response ───────────────────────────────
    let response = Response::from_data(file_data)
        .with_header(header("Content-Type", mime_type))
        .with_header(header("Access-Control-Allow-Origin", "*"))
        .with_header(header("Cache-Control", "public, max-age=3600"))
        .with_header(header("Accept-Ranges", "bytes"));

    let _ = request.respond(response);
}
