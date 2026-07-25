// ─── Channel trait and registry ──────────────────────────

pub mod stdout;
pub mod telegram;

use std::fmt::Debug;

use crate::message::Message;

/// Error returned when sending through a channel fails.
#[derive(Debug)]
pub struct ChannelError {
    pub message: String,
}

impl ChannelError {
    pub fn new(msg: impl Into<String>) -> Self {
        ChannelError { message: msg.into() }
    }
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ChannelError {}

/// Result of a single channel send.
#[derive(Debug)]
pub struct SendResult {
    pub channel: String,
    pub success: bool,
    pub error: Option<String>,
}

/// A notification channel that can send messages.
///
/// Implementations are synchronous. See `telegram::TelegramBackend`
/// and `stdout::StdoutBackend`.
pub trait Channel: Debug {
    /// Unique channel name (e.g. `"telegram"`, `"stdout"`).
    /// Currently unused in main.rs (backends are dispatched directly),
    /// but part of the public trait API for future consumers.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Send a message through this channel.
    fn send(&self, msg: &Message) -> Result<SendResult, ChannelError>;
}
