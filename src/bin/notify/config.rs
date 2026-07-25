// ─── Config — merge environment variables and CLI overrides ─

use std::path::PathBuf;
use std::fs;

use crate::log_store::LogStore;

/// Resolve the project root from the binary path.
///
/// Binary is at `<root>/target/<profile>/notify`, walks up 3 levels.
/// Returns `None` if resolution fails or the root has no `justfile`.
pub fn find_project_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let root = exe.parent()?             // target/<profile>
        .parent()?                         // target
        .parent()?;                        // project root
    root.join("justfile").exists().then_some(root.to_path_buf())
}

/// Load `config/.env` then `config/.env.dev.local` into the process environment.
///
/// Tries multiple paths in order:
/// 1. CWD-relative `config/.env` (normal case when run via `just`)
/// 2. Parent directory of the binary (standalone from anywhere)
///
/// This runs before `Config::from_env()` so the binary works both when called
/// via `just` (which also loads these files via `dotenv-command`) and when run
/// standalone.
pub fn init_env() {
    let base_dirs = [
        PathBuf::from("."),                      // CWD (via just / cargo run)
        std::env::current_exe().ok()                  // binary dir
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default(),
    ];
    for base in &base_dirs {
        if base.join("config/.env").exists() {
            // Order: .env first, then .env.dev.local overrides
            load_dotenv_file(&base.join("config/.env"));
            load_dotenv_file(&base.join("config/.env.dev.local"));
            return;
        }
    }
}

/// Parse a single `.env` file and set each `KEY=VALUE` as an env var.
/// Silently skips if the file doesn't exist.
fn load_dotenv_file(path: &std::path::Path) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,  // file not found — ok
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !key.is_empty() {
                // SAFETY: setting env vars from a trusted .env file is safe.
                // This is the same pattern used by every dotenv crate.
                unsafe { std::env::set_var(key, value); }
            }
        }
    }
}

/// Runtime configuration for the notify tool.
///
/// Built from environment variables (see `Config::from_env`) and can
/// be overridden by CLI args (see `cli.rs`).
#[derive(Debug)]
pub struct Config {
    /// Telegram Bot token.
    pub telegram_bot_token: Option<String>,
    /// Telegram Chat ID.
    pub telegram_chat_id: Option<String>,
    /// Explicit Telegram API proxy URL.
    pub telegram_proxy: Option<String>,
    /// Default channels (comma-separated).
    pub default_channel: String,
    /// Audit log store (lazily initialized on first use).
    pub log_store: LogStore,
    /// Default tail lines for `watch`.
    pub tail_lines: usize,
}

impl Config {
    /// Build config from environment variables.
    pub fn from_env() -> Self {
        let log_file = std::env::var("NOTIFY_LOG_FILE")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(LogStore::default_path);

        Config {
            telegram_bot_token: std::env::var("NOTIFY_TELEGRAM_BOT_TOKEN").ok(),
            telegram_chat_id: std::env::var("NOTIFY_TELEGRAM_CHAT_ID").ok(),
            telegram_proxy: std::env::var("NOTIFY_TELEGRAM_PROXY").ok(),
            default_channel: std::env::var("NOTIFY_DEFAULT_CHANNEL")
                .unwrap_or_else(|_| "telegram".into()),
            log_store: LogStore::new(log_file),
            tail_lines: std::env::var("NOTIFY_TAIL_LINES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }

    /// Check if a channel name is available in the default config.
    pub fn is_channel_available(&self, name: &str) -> bool {
        match name {
            "telegram" => {
                self.telegram_bot_token.is_some() && self.telegram_chat_id.is_some()
            }
            "stdout" | "log" => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // SAFETY: clearing env vars in test to ensure a clean default state.
        // Single-threaded test context — no risk of concurrent access.
        unsafe {
            std::env::remove_var("NOTIFY_TELEGRAM_BOT_TOKEN");
            std::env::remove_var("NOTIFY_TELEGRAM_CHAT_ID");
            std::env::remove_var("NOTIFY_TELEGRAM_PROXY");
            std::env::remove_var("NOTIFY_DEFAULT_CHANNEL");
            std::env::remove_var("NOTIFY_LOG_FILE");
            std::env::remove_var("NOTIFY_TAIL_LINES");
        }
        // With no env set, should have sensible defaults
        let config = Config::from_env();
        assert_eq!(config.default_channel, "telegram");
        assert_eq!(config.tail_lines, 30);
        assert!(config.telegram_bot_token.is_none());
    }
}
