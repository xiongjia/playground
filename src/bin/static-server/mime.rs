// ─── MIME type lookup by file extension ───────────────────

/// Map a file extension to a MIME type string.
///
/// Returns `application/octet-stream` for unknown extensions.
pub fn from_ext(ext: &str) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_types() {
        assert_eq!(from_ext("html"), "text/html; charset=utf-8");
        assert_eq!(from_ext("mp4"), "video/mp4");
        assert_eq!(from_ext("vtt"), "text/vtt; charset=utf-8");
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(from_ext("zzz"), "application/octet-stream");
    }

    #[test]
    fn test_empty_extension() {
        assert_eq!(from_ext(""), "application/octet-stream");
    }
}
