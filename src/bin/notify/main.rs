// ─── notify — Personal notification CLI ──────────────────
//
// Subcommands:
//   send   — Send a notification (Telegram / stdout)
//   watch  — Run a command and notify on completion
//   test   — Dry-run test channels (no real API calls)
//   log    — Query the audit log
// ───────────────────────────────────────────────────────────

mod channel;
mod cli;
mod config;
mod log_store;
mod message;
mod watch;

use std::io::Read;

use clap::Parser;

use crate::channel::{Channel, SendResult};
use crate::cli::{Cli, Command, ListArgs, LogAction, SendArgs, TestArgs, WatchArgs};
use crate::config::Config;
use crate::log_store::LogStore;
use crate::message::{Level, Message};
use crate::watch::WatchRunner;

fn main() {
    config::init_env();
    let cli = Cli::parse();
    let config = Config::from_env();

    match cli.command {
        Command::Send(args) => cmd_send(&config, args),
        Command::Watch(args) => cmd_watch(&config, args),
        Command::Test(args) => cmd_test(&config, args),
        Command::Log(args) => cmd_log(&config, args),
        Command::Env => cmd_env(&config),
    }
}

// ── send ──────────────────────────────────────────────────

fn cmd_send(config: &Config, args: SendArgs) {
    let body = args.body.or(args.message).unwrap_or_default();
    let title = args.title.unwrap_or_default();
    let level = Level::from_str(&args.level).unwrap_or(Level::Info);

    let msg = Message::new(body)
        .with_title(title)
        .with_level(level);

    let channels = resolve_channels(config, args.channel.as_deref());
    let results = send_notification(config, &msg, &channels, args.proxy.as_deref());

    report_results(&results);
}

// ── watch ─────────────────────────────────────────────────

fn cmd_watch(config: &Config, args: WatchArgs) {
    // Ensure CWD is project root (resolve from binary path)
    ensure_project_root();

    // Resolve command: --command arg, or stdin
    let cmd = match &args.command {
        Some(c) => c.clone(),
        None => {
            let mut buf = String::new();
            if std::io::stdin().read_to_string(&mut buf).is_err() || buf.trim().is_empty()
            {
                eprintln!("✗ No command provided. Use --command or pipe to stdin.");
                return;
            }
            buf.trim().to_string()
        }
    };

    let tail = if args.tail == 0 { usize::MAX } else { args.tail };

    match WatchRunner::run(&cmd) {
        Ok(output) => {
            let wc = watch::WatchConfig {
                cmd: &cmd,
                title: args.title.as_deref(),
                tail_lines: tail,
                on_error: args.on_error,
            };
            match WatchRunner::to_message(&output, &wc) {
                Some(msg) => {
                    let channels = resolve_channels(config, args.channel.as_deref());
                    let results =
                        send_notification(config, &msg, &channels, args.proxy.as_deref());
                    report_results(&results);
                }
                None => {
                    // --on-error and command succeeded; no notification needed
                }
            }
        }
        Err(e) => {
            let msg = Message::new(format!("Failed to execute command: {}", e))
                .with_title("Command failed")
                .with_level(Level::Error);
            let channels = resolve_channels(config, args.channel.as_deref());
            let results =
                send_notification(config, &msg, &channels, args.proxy.as_deref());
            report_results(&results);
        }
    }
}

// ── test ──────────────────────────────────────────────────

fn cmd_test(config: &Config, args: TestArgs) {
    println!("═══ notify test ═══\n");

    // 1. Config check
    println!("── Config ──");
    println!("  NOTIFY_TELEGRAM_BOT_TOKEN = {}", mask_token(&config.telegram_bot_token));
    println!("  NOTIFY_TELEGRAM_CHAT_ID   = {}", config.telegram_chat_id.as_deref().unwrap_or("(not set)"));
    println!("  NOTIFY_TELEGRAM_PROXY     = {}", config.telegram_proxy.as_deref().unwrap_or("(not set)"));
    println!("  NOTIFY_DEFAULT_CHANNEL    = {}", config.default_channel);
    println!("  NOTIFY_LOG_FILE           = {}", config.log_store.path().display());
    println!("  NOTIFY_TAIL_LINES         = {}", config.tail_lines);
    println!();

    // 2. Build a test message and try each channel (dry-run)
    println!("── Channel test (dry-run, no real API call) ──");

    let test_msg = Message::new("This is a test notification.")
        .with_title("Test Notification")
        .with_level(Level::Success);

    let channels = if let Some(ch) = &args.channel {
        vec![ch.clone()]
    } else {
        // Test all available channels
        let mut names = Vec::new();
        if config.is_channel_available("telegram") {
            names.push("telegram".to_string());
        }
        names.push("stdout".to_string());
        names
    };

    for ch_name in &channels {
        match ch_name.as_str() {
            "telegram" => {
                if let (Some(token), Some(chat_id)) =
                    (&config.telegram_bot_token, &config.telegram_chat_id)
                {
                    println!("  [telegram] Token OK ({} chars), Chat ID OK", token.len());
                    println!("    Would POST to: https://api.telegram.org/bot{}/sendMessage", mask_token_end(token));
                    println!("    chat_id: {}", chat_id);

                    if args.verbose {
                        println!("    Message preview:");
                        for line in test_msg.format().lines() {
                            println!("      {}", line);
                        }
                    }
                } else {
                    println!("  [telegram] SKIP — token or chat_id not configured");
                }
            }
            "stdout" => {
                println!("  [stdout] ✓ (always available)");
                if args.verbose {
                    println!("    Message preview:");
                    for line in test_msg.format().lines() {
                        println!("      {}", line);
                    }
                }
            }
            _ => {
                println!("  [{}] SKIP — unknown channel", ch_name);
            }
        }
    }
    println!();

    // 3. Watch simulation
    if let Some(watch_cmd) = &args.watch {
        println!("── Watch simulation ──");
        println!("  Command: {}", watch_cmd);
        match WatchRunner::run(watch_cmd) {
            Ok(output) => {
                let wc = watch::WatchConfig::new(watch_cmd)
                    .with_tail(config.tail_lines);
                let msg = WatchRunner::to_message(&output, &wc);
                if let Some(msg) = msg {
                    println!("  → Level: {:?}", msg.level);
                    println!("  → Body length: {} chars", msg.body.len());
                    if args.verbose {
                        println!("  ── Full body ──");
                        println!("{}", msg.body);
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Failed to run: {}", e);
            }
        }
        println!();
    }

    println!("═══ Test complete ═══");
}

// ── log ───────────────────────────────────────────────────

fn cmd_log(config: &Config, args: cli::LogArgs) {
    let store = &config.log_store;

    match args.action {
        LogAction::List(list_args) => cmd_log_list(store, list_args),
        LogAction::Status => cmd_log_status(store),
    }
}

fn cmd_log_list(store: &LogStore, args: ListArgs) {
    let entries = match store.list(std::cmp::max(args.limit, 1000)) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("✗ Failed to read log: {}", e);
            return;
        }
    };

    let entries = if let Some(ch) = &args.channel {
        LogStore::filter_channel(entries, ch)
    } else {
        entries
    };

    let entries = match &args.status {
        Some(s) if s == "success" || s == "ok" => {
            LogStore::filter_status(entries, true)
        }
        Some(s) if s == "failed" || s == "fail" || s == "error" => {
            LogStore::filter_status(entries, false)
        }
        _ => entries,
    };

    let shown = entries.len().min(args.limit);
    for entry in &entries[..shown] {
        let status = if entry.success { "✓" } else { "✗" };
        println!(
            "{} [{}] {} {} {} — {}",
            status,
            &entry.timestamp[..19],
            entry.channel,
            entry.level,
            entry.title,
            entry.body.chars().take(60).collect::<String>(),
        );
    }
    println!("── {} entries shown ({} total matched) ──", shown, entries.len());
}

fn cmd_log_status(store: &LogStore) {
    match store.status() {
        Ok(status) => {
            println!("Sent: {} | Success: {} | Failed: {}", status.total, status.success, status.failed);
            let mut channels: Vec<_> = status.by_channel.into_iter().collect();
            channels.sort_by_key(|(_, count)| *count);
            for (ch, count) in channels.iter().rev() {
                println!("  {}: {}", ch, count);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to read log: {}", e);
        }
    }
}

// ── helpers ───────────────────────────────────────────────

/// Resolve channel names from CLI `--channel` or fall back to config default.
fn resolve_channels(config: &Config, channel_opt: Option<&str>) -> Vec<String> {
    let spec = channel_opt.unwrap_or(&config.default_channel);
    spec.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Send a notification through the given channels, always logging to audit.
///
/// Each channel receives the message independently. Errors from one channel
/// do not affect others.
fn send_notification(
    config: &Config,
    msg: &Message,
    channels: &[String],
    proxy: Option<&str>,
) -> Vec<SendResult> {
    let mut results = Vec::new();
    let log_store = &config.log_store;

    for ch_name in channels {
        let start = std::time::Instant::now();
        let (result, error) = match ch_name.as_str() {
            "telegram" => {
                if let (Some(token), Some(chat_id)) =
                    (&config.telegram_bot_token, &config.telegram_chat_id)
                {
                    let proxy = proxy
                        .map(|s| s.to_string())
                        .or_else(|| config.telegram_proxy.clone());
                    let backend =
                        channel::telegram::TelegramBackend::new(
                            token.clone(),
                            chat_id.clone(),
                            proxy,
                        );
                    match backend.send(msg) {
                        Ok(r) => (r, None),
                        Err(e) => {
                            let err_msg = e.message.clone();
                            (SendResult {
                                channel: "telegram".into(),
                                success: false,
                                error: Some(err_msg.clone()),
                            }, Some(err_msg))
                        }
                    }
                } else {
                    let err = "Telegram not configured (set NOTIFY_TELEGRAM_BOT_TOKEN \
                        and NOTIFY_TELEGRAM_CHAT_ID)".to_string();
                    (SendResult {
                        channel: "telegram".into(),
                        success: false,
                        error: Some(err.clone()),
                    }, Some(err))
                }
            }
            "stdout" => {
                let backend = channel::stdout::StdoutBackend;
                match backend.send(msg) {
                    Ok(r) => (r, None),
                    Err(e) => {
                        let err_msg = e.message.clone();
                        (SendResult {
                            channel: "stdout".into(),
                            success: false,
                            error: Some(err_msg.clone()),
                        }, Some(err_msg))
                    }
                }
            }
            _ => {
                (SendResult {
                    channel: ch_name.clone(),
                    success: false,
                    error: Some(format!("Unknown channel: {}", ch_name)),
                }, None)
            }
        };

        let duration = start.elapsed().as_millis() as u64;

        // Always log to audit trail
        if let Err(e) = log_store.write(msg, ch_name, result.success, error, duration) {
            eprintln!("⚠️ Failed to write audit log: {}", e);
        }

        results.push(result);
    }

    results
}

/// Print send results to stderr.
fn report_results(results: &[SendResult]) {
    for r in results {
        if r.success {
            eprintln!("✓ [{}] sent", r.channel);
        } else {
            eprintln!(
                "✗ [{}] failed: {}",
                r.channel,
                r.error.as_deref().unwrap_or("unknown error")
            );
        }
    }
}

/// Show resolved environment configuration.
fn cmd_env(config: &Config) {
    println!("═══ notify environment ═══");
    println!();
    println!("── Raw env vars ──");
    for key in [
        "NOTIFY_TELEGRAM_BOT_TOKEN",
        "NOTIFY_TELEGRAM_CHAT_ID",
        "NOTIFY_TELEGRAM_PROXY",
        "NOTIFY_DEFAULT_CHANNEL",
        "NOTIFY_LOG_FILE",
        "NOTIFY_TAIL_LINES",
    ] {
        let val = std::env::var(key).ok();
        let display = if key.contains("TOKEN") || key.contains("PASSWORD") {
            mask_token(&val)
        } else {
            val.unwrap_or_else(|| "(not set)".to_string())
        };
        println!("  {} = {}", key, display);
    }
    println!();
    println!("── Resolved config ──");
    println!("  Notify log file  = {}", config.log_store.path().display());
    println!("  Tail lines       = {}", config.tail_lines);
    println!("  Default channel  = {}", config.default_channel);
    println!("  Telegram token   = {}", mask_token(&config.telegram_bot_token));
    println!("  Telegram chat_id = {}", config.telegram_chat_id.as_deref().unwrap_or("(not set)"));
    println!("  Telegram proxy   = {}", config.telegram_proxy.as_deref().unwrap_or("(not set)"));
    println!();
    println!("── Source ──");
    println!("  config/.env (base) + config/.env.dev.local (overrides)");
}

/// Try to chdir to the project root.
/// Resolved from the binary path; silently does nothing if resolution fails.
fn ensure_project_root() {
    if let Some(root) = config::find_project_root() {
        let _ = std::env::set_current_dir(root);
    }
}

/// Mask a token for display: show last 4 chars only.
fn mask_token(token: &Option<String>) -> String {
    match token {
        Some(t) if t.len() > 4 => {
            format!("****{}", &t[t.len() - 4..])
        }
        Some(_) => "****".to_string(),
        None => "(not set)".to_string(),
    }
}

/// Show the end of a token for identification.
fn mask_token_end(token: &str) -> String {
    if token.len() > 8 {
        format!("...{}", &token[token.len() - 8..])
    } else {
        "****".to_string()
    }
}
