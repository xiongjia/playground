// ─── CLI argument / env parsing for static-server ─────────

use std::env;
use std::path::PathBuf;
use std::process;

/// Parsed server configuration.
pub struct Config {
    /// The root directory to serve (non-canonicalized, as provided).
    pub root: PathBuf,
    /// Canonicalized root directory, used for path traversal checks.
    pub root_canonical: PathBuf,
    /// TCP port to listen on.
    pub port: u16,
}

/// Parse configuration from CLI arguments and environment variables.
///
/// Priority:
///   1. CLI argument `static-server /path/to/serve`
///   2. `SERVE_ROOT` environment variable
///   3. Prints usage and exits if neither is set
pub fn parse() -> Config {
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

    let root_canonical = root_path.canonicalize().unwrap_or_else(|e| {
        eprintln!("✗ Failed to canonicalize root: {}", e);
        process::exit(1);
    });

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    Config {
        root: root_path,
        root_canonical,
        port,
    }
}
