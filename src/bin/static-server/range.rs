// ─── HTTP Range header parsing ────────────────────────────

/// Parse a `Range: bytes=...` header.
///
/// Supports three forms:
/// - `bytes=0-1023`   — specific byte range (inclusive)
/// - `bytes=1024-`    — from offset to end
/// - `bytes=-1024`    — last N bytes
///
/// Returns `(start, end)` where both are 0-indexed and inclusive,
/// or `None` if the range is invalid or out of bounds.
pub fn parse_range(range_str: &str, file_size: u64) -> Option<(u64, u64)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_range() {
        assert_eq!(parse_range("bytes=0-1023", 10000), Some((0, 1023)));
    }

    #[test]
    fn test_open_ended_range() {
        assert_eq!(parse_range("bytes=500-", 1000), Some((500, 999)));
    }

    #[test]
    fn test_suffix_range() {
        assert_eq!(parse_range("bytes=-100", 1000), Some((900, 999)));
    }

    #[test]
    fn test_start_exceeds_size() {
        assert_eq!(parse_range("bytes=1000-", 500), None);
    }

    #[test]
    fn test_start_after_end() {
        assert_eq!(parse_range("bytes=100-50", 1000), None);
    }

    #[test]
    fn test_end_exceeds_size() {
        assert_eq!(parse_range("bytes=0-1000", 500), None);
    }

    #[test]
    fn test_empty_range() {
        assert_eq!(parse_range("bytes=-0", 1000), None);
    }

    #[test]
    fn test_no_prefix() {
        assert_eq!(parse_range("0-100", 1000), None);
    }

    #[test]
    fn test_end_of_file() {
        // Last single byte
        assert_eq!(parse_range("bytes=999-999", 1000), Some((999, 999)));
    }

    #[test]
    fn test_zero_suffix() {
        assert_eq!(parse_range("bytes=-0", 1000), None);
    }
}
