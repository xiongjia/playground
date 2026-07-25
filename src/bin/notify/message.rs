// ─── Message types for the notify system ──────────────────

use chrono::{DateTime, Local};

/// Notification severity level.
#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Level {
    /// General information
    #[default]
    Info,
    /// Successful operation
    Success,
    /// Warning / notable event
    Warning,
    /// Error / failure
    Error,
}

impl Level {
    /// Return the emoji prefix for this level.
    pub fn emoji(&self) -> &'static str {
        match self {
            Level::Info => "🔔",
            Level::Success => "✅",
            Level::Warning => "⚠️",
            Level::Error => "❌",
        }
    }

    /// Parse from a CLI argument string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "info" => Some(Level::Info),
            "success" => Some(Level::Success),
            "warning" | "warn" => Some(Level::Warning),
            "error" | "err" => Some(Level::Error),
            _ => None,
        }
    }
}

/// A notification message ready to be sent through a channel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    /// Short title (shown in bold on Telegram).
    #[serde(default)]
    pub title: String,
    /// Body text.
    #[serde(default)]
    pub body: String,
    /// Severity level.
    #[serde(default)]
    pub level: Level,
    /// ISO-8601 timestamp when the message was created.
    pub timestamp: DateTime<Local>,
    /// Target channel name (set before sending).
    #[serde(default)]
    pub channel: String,
}

impl Message {
    /// Create a new message with the given body, defaulting to Info level.
    pub fn new(body: impl Into<String>) -> Self {
        Message {
            title: String::new(),
            body: body.into(),
            level: Level::Info,
            timestamp: Local::now(),
            channel: String::new(),
        }
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the level.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the channel name.
    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = channel.into();
        self
    }

    /// Format the message as a human-readable string for Telegram / stdout.
    ///
    /// Format:
    /// ```text
    /// {emoji} {title}
    /// {body}
    /// ```
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str(self.level.emoji());
        if !self.title.is_empty() {
            out.push(' ');
            out.push_str(&self.title);
        }
        out.push('\n');
        if !self.body.is_empty() {
            out.push_str(&self.body);
            out.push('\n');
        }
        out
    }

    /// Truncate the body to at most `max_chars` characters.
    ///
    /// If truncation occurs, appends `[Truncated: showing N/M chars]`.
    /// Returns `true` if truncation happened.
    // Currently unused; kept as a public API for external message builders.
    #[allow(dead_code)]
    pub fn truncate_body(&mut self, max_chars: usize) -> bool {
        if self.body.len() <= max_chars {
            return false;
        }
        let suffix = format!(
            "\n[Truncated: showing {}/{} chars]",
            max_chars,
            self.body.len()
        );
        let keep = max_chars.saturating_sub(suffix.len());
        self.body.truncate(keep);
        self.body.push_str(&suffix);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_emoji() {
        assert_eq!(Level::Info.emoji(), "🔔");
        assert_eq!(Level::Success.emoji(), "✅");
        assert_eq!(Level::Warning.emoji(), "⚠️");
        assert_eq!(Level::Error.emoji(), "❌");
    }

    #[test]
    fn test_level_parse() {
        assert_eq!(Level::from_str("info"), Some(Level::Info));
        assert_eq!(Level::from_str("SUCCESS"), Some(Level::Success));
        assert_eq!(Level::from_str("warn"), Some(Level::Warning));
        assert_eq!(Level::from_str("error"), Some(Level::Error));
        assert_eq!(Level::from_str("unknown"), None);
    }

    #[test]
    fn test_message_format() {
        let msg = Message::new("Hello").with_title("Test").with_level(Level::Info);
        let text = msg.format();
        assert!(text.contains("🔔"));
        assert!(text.contains("Test"));
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_truncation() {
        let mut msg = Message::new("a".repeat(100));
        assert!(msg.truncate_body(50));
        assert!(msg.body.len() <= 50);
        assert!(msg.body.contains("[Truncated"));
    }

    #[test]
    fn test_no_truncation_when_under_limit() {
        let mut msg = Message::new("short");
        assert!(!msg.truncate_body(100));
        assert_eq!(msg.body, "short");
    }
}
