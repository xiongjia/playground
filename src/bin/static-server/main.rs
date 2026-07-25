// ─── static-server — Static HTTP file server ──────────────
//
// Serves a directory over HTTP. Generic — usable by any module.
//
// Usage:
//   cargo run --bin static-server -- /path/to/serve
//   cargo run --bin static-server          # uses SERVE_ROOT env var
//
// Environment:
//   SERVE_ROOT  — root directory to serve
//   PORT        — server port (default: 8080)
// ─────────────────────────────────────────────────────────────

mod cli;
mod handler;
mod mime;
mod range;

use tiny_http::Server;

fn main() {
    let config = cli::parse();

    let addr = format!("0.0.0.0:{}", config.port);
    let server = Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("✗ Failed to bind {}: {}", addr, e);
        std::process::exit(1);
    });

    println!(
        "🌐 Serving {} on http://localhost:{}",
        config.root.display(),
        config.port
    );
    println!("   Press Ctrl+C to stop");

    for request in server.incoming_requests() {
        handler::handle_one(request, &config.root, &config.root_canonical);
    }
}
