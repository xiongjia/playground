// ─── CLI argument parsing via clap derive ─────────────────

use clap::{Parser, Subcommand};

/// Personal notification tool — send messages, watch commands, and audit logs.
#[derive(Parser, Debug)]
#[command(name = "notify", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Send a notification message
    Send(SendArgs),
    /// Run a command and send notification on completion
    Watch(WatchArgs),
    /// Test notification channels (dry-run, no real send)
    Test(TestArgs),
    /// Query notification audit log
    Log(LogArgs),
    /// Show resolved environment configuration
    Env,
}

// ── send ──────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct SendArgs {
    /// Message body (positional shorthand)
    pub message: Option<String>,

    /// Message title
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Message body (explicit, overrides positional)
    #[arg(short = 'b', long)]
    pub body: Option<String>,

    /// Severity level: info | success | warning | error
    #[arg(short = 'l', long, default_value = "info")]
    pub level: String,

    /// Target channels (comma-separated)
    #[arg(short = 'c', long)]
    pub channel: Option<String>,

    /// HTTP proxy URL for Telegram API
    #[arg(long)]
    pub proxy: Option<String>,
}

// ── watch ─────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct WatchArgs {
    /// Command to run (use -- to pass complex commands)
    #[arg(short = 'c', long)]
    pub command: Option<String>,

    /// Output tail lines (default: 30, 0 = all)
    #[arg(short = 'n', long, default_value = "30")]
    pub tail: usize,

    /// Custom notification title (default: command string)
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Only notify on error (silent on success)
    #[arg(long)]
    pub on_error: bool,

    /// Target channels (comma-separated)
    #[arg(short = 'C', long)]
    pub channel: Option<String>,

    /// HTTP proxy URL for Telegram API
    #[arg(long)]
    pub proxy: Option<String>,
}

// ── test ──────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct TestArgs {
    /// Channel to test (default: all configured)
    #[arg(short = 'c', long)]
    pub channel: Option<String>,

    /// Simulate a watch command to test truncation
    #[arg(long)]
    pub watch: Option<String>,

    /// Show detailed debug output
    #[arg(long)]
    pub verbose: bool,
}

// ── log ───────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct LogArgs {
    #[command(subcommand)]
    pub action: LogAction,
}

#[derive(Subcommand, Debug)]
pub enum LogAction {
    /// List recent log entries
    List(ListArgs),
    /// Show summary statistics
    Status,
}

#[derive(clap::Args, Debug)]
pub struct ListArgs {
    /// Max entries to show (default: 20)
    #[arg(short = 'n', long, default_value = "20")]
    pub limit: usize,

    /// Filter by channel name
    #[arg(short = 'c', long)]
    pub channel: Option<String>,

    /// Filter by status: success | failed
    #[arg(short = 's', long)]
    pub status: Option<String>,
}
