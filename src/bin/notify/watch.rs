// ─── WatchRunner — execute a command and capture output ──

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;


use crate::message::{Level, Message};

/// Result of running a watched command.
#[derive(Debug)]
pub struct WatchOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Configuration for building a watch notification message.
#[derive(Debug, Clone)]
pub struct WatchConfig<'a> {
    /// The command string (used in the notification body).
    pub cmd: &'a str,
    /// Optional title override; defaults to the command string.
    pub title: Option<&'a str>,
    /// Number of output lines to include (0 = all).
    pub tail_lines: usize,
    /// Only send when the command fails (exit != 0).
    pub on_error: bool,
}

impl<'a> WatchConfig<'a> {
    /// Create a new `WatchConfig` with defaults.
    pub fn new(cmd: &'a str) -> WatchConfig<'a> {
        WatchConfig {
            cmd,
            title: None,
            tail_lines: 30,
            on_error: false,
        }
    }

    /// Set the tail lines.
    pub fn with_tail(mut self, tail_lines: usize) -> Self {
        self.tail_lines = tail_lines;
        self
    }

    /// Enable on-error-only mode.
    // Public builder API; currently unused but kept for external consumers.
    #[allow(dead_code)]
    pub fn with_on_error(mut self, on_error: bool) -> Self {
        self.on_error = on_error;
        self
    }

    /// Set the title.
    // Public builder API; currently unused but kept for external consumers.
    #[allow(dead_code)]
    pub fn with_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

/// Execute a command and capture its output.
pub struct WatchRunner;

impl WatchRunner {
    /// Run a shell command, tee stdout/stderr to terminal, and capture output.
    ///
    /// The command is passed to `sh -c` for shell interpretation
    /// (supports pipes, redirects, etc.).
    pub fn run(cmd: &str) -> std::io::Result<WatchOutput> {
        let start = Instant::now();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout_handle = {
            let stdout = child.stdout.take().expect("failed to capture stdout");
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                let mut buf = String::new();
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            println!("{}", line);
                            buf.push_str(&line);
                            buf.push('\n');
                        }
                        Err(_) => break,
                    }
                }
                buf
            })
        };

        let stderr_handle = {
            let stderr = child.stderr.take().expect("failed to capture stderr");
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                let mut buf = String::new();
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            eprintln!("{}", line);
                            buf.push_str(&line);
                            buf.push('\n');
                        }
                        Err(_) => break,
                    }
                }
                buf
            })
        };

        let status = child.wait()?;
        let duration = start.elapsed().as_millis() as u64;
        let exit_code = status.code().unwrap_or(-1);

        let stdout = match stdout_handle.join() {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("⚠️ stdout reader thread panicked: {:?}", e);
                String::new()
            }
        };
        let stderr = match stderr_handle.join() {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("⚠️ stderr reader thread panicked: {:?}", e);
                String::new()
            }
        };

        Ok(WatchOutput {
            exit_code,
            stdout,
            stderr,
            duration_ms: duration,
        })
    }

    /// Build a notification `Message` from a `WatchOutput`.
    ///
    /// Returns `None` if `config.on_error` is set and the command succeeded.
    pub fn to_message(output: &WatchOutput, config: &WatchConfig) -> Option<Message> {
        let success = output.exit_code == 0;

        if config.on_error && success {
            return None;
        }

        let level = if success { Level::Success } else { Level::Error };
        let title = config.title.unwrap_or(config.cmd);
        let tail_lines = config.tail_lines;

        // Build body with command details
        let mut body = String::new();
        body.push_str(&format!("Command: {}\n", config.cmd));
        body.push_str(&format!("Duration: {}s\n", output.duration_ms / 1000));
        body.push_str(&format!("Exit code: {}", output.exit_code));
        body.push('\n');

        // Add output (success → stdout, failure → stderr)
        let output_text = if success {
            &output.stdout
        } else {
            // Prefer stderr for failures, fall back to stdout
            if output.stderr.is_empty() {
                &output.stdout
            } else {
                &output.stderr
            }
        };

        if !output_text.is_empty() {
            let lines: Vec<&str> = output_text.lines().collect();
            let total = lines.len();
            if tail_lines > 0 && tail_lines < total {
                let suffix = format!(
                    "\n[Truncated: showing last {}/{} lines]",
                    tail_lines, total
                );
                body.push_str(&format!(
                    "\n📋 Output (last {} lines):\n",
                    tail_lines
                ));
                for line in lines.iter().rev().take(tail_lines).rev() {
                    body.push_str(line);
                    body.push('\n');
                }
                body.push_str(&suffix);
                body.push('\n');
            } else {
                body.push_str("\n📋 Output:\n");
                for line in &lines {
                    body.push_str(line);
                    body.push('\n');
                }
            };

            // Truncate if too long for Telegram (4096 chars)
            let max_chars = if success { 2000usize } else { 3000usize };
            if body.len() > max_chars {
                let suffix = format!(
                    "\n[Truncated: showing {}/{} chars]",
                    max_chars,
                    body.len()
                );
                let keep = max_chars.saturating_sub(suffix.len());
                body.truncate(keep);
                body.push_str(&suffix);
            }
        }

        Some(
            Message::new(body)
                .with_title(title)
                .with_level(level)
                .with_channel(""),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_success() {
        let output = WatchRunner::run("echo hello").unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.trim(), "hello");
    }

    #[test]
    fn test_run_failure() {
        let output = WatchRunner::run("false").unwrap();
        assert_ne!(output.exit_code, 0);
    }

    #[test]
    fn test_to_message_success() {
        let output = WatchRunner::run("echo done").unwrap();
        let wc = WatchConfig::new("test-cmd").with_tail(10);
        let msg = WatchRunner::to_message(&output, &wc);
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.level, Level::Success);
        assert!(msg.body.contains("done"));
    }

    #[test]
    fn test_to_message_failure() {
        let output = WatchRunner::run("echo err >&2; false").unwrap();
        let wc = WatchConfig::new("fail-cmd").with_tail(10);
        let msg = WatchRunner::to_message(&output, &wc);
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert_eq!(msg.level, Level::Error);
    }

    #[test]
    fn test_on_error_suppresses_success() {
        let output = WatchRunner::run("true").unwrap();
        let wc = WatchConfig::new("ok").with_tail(10).with_on_error(true);
        let msg = WatchRunner::to_message(&output, &wc);
        assert!(msg.is_none());
    }

    #[test]
    fn test_on_error_shows_failure() {
        let output = WatchRunner::run("false").unwrap();
        let wc = WatchConfig::new("fail").with_tail(10).with_on_error(true);
        let msg = WatchRunner::to_message(&output, &wc);
        assert!(msg.is_some());
    }

    #[test]
    fn test_truncation() {
        let out = WatchOutput {
            exit_code: 0,
            stdout: "line\n".repeat(100),
            stderr: String::new(),
            duration_ms: 50,
        };
        let wc = WatchConfig::new("big-output").with_tail(5);
        let msg = WatchRunner::to_message(&out, &wc);
        assert!(msg.is_some());
        let body = msg.unwrap().body;
        assert!(body.contains("[Truncated: showing last 5/100 lines]"));
    }
}
