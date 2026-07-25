// ─── Stdout channel backend (test/debug) ──────────────────

use crate::channel::{Channel, ChannelError, SendResult};
use crate::message::Message;

/// Print notifications to stdout as formatted text (with emoji).
///
/// Used for testing / debugging without sending to a real service.
#[derive(Debug)]
pub struct StdoutBackend;

impl Channel for StdoutBackend {
    fn name(&self) -> &'static str {
        "stdout"
    }

    fn send(&self, msg: &Message) -> Result<SendResult, ChannelError> {
        let separator = "─".repeat(40);
        println!("{}", separator);
        println!("[stdout/{}]", msg.channel);
        print!("{}", msg.format());
        println!("{}", separator);
        Ok(SendResult {
            channel: "stdout".into(),
            success: true,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Level;

    #[test]
    fn test_stdout_backend_name() {
        let backend = StdoutBackend;
        assert_eq!(backend.name(), "stdout");
    }

    #[test]
    fn test_stdout_send_success() {
        let backend = StdoutBackend;
        let msg = Message::new("test").with_title("Test").with_level(Level::Info);
        let result = backend.send(&msg).unwrap();
        assert!(result.success);
        assert_eq!(result.channel, "stdout");
    }
}
