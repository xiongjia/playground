// ─── Telegram Bot channel backend ─────────────────────────

use ureq::Agent;

use crate::channel::{Channel, ChannelError, SendResult};
use crate::message::Message;

/// Send notifications via Telegram Bot API.
#[derive(Debug)]
pub struct TelegramBackend {
    bot_token: String,
    chat_id: String,
    agent: Agent,
}

impl TelegramBackend {
    /// Create a new Telegram backend.
    ///
    /// `proxy_url` is optional — supports `NOTIFY_TELEGRAM_PROXY`,
    /// `HTTPS_PROXY`, or `HTTP_PROXY` env vars via `resolve_proxy`.
    pub fn new(
        bot_token: String,
        chat_id: String,
        proxy_url: Option<String>,
    ) -> Self {
        let agent = build_agent(proxy_url);
        TelegramBackend {
            bot_token,
            chat_id,
            agent,
        }
    }

    /// Execute a single Telegram API request.
    fn send_request(&self, url: &str, json: &str) -> Result<SendResult, ChannelError> {
        match self
            .agent
            .post(url)
            .header("Content-Type", "application/json")
            .send(json)
        {
            Ok(resp) => {
                let status = resp.status();
                if status == 200 {
                    Ok(SendResult {
                        channel: "telegram".into(),
                        success: true,
                        error: None,
                    })
                } else {
                    Err(ChannelError::new(format!(
                        "Telegram API returned {} (check token, chat_id, and message format)",
                        status
                    )))
                }
            }
            Err(e) => Err(ChannelError::new(format!("ureq error: {}", e)))
        }
    }
}

impl Channel for TelegramBackend {
    fn name(&self) -> &'static str {
        "telegram"
    }

    fn send(&self, msg: &Message) -> Result<SendResult, ChannelError> {
        let text = msg.format();
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );

        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": text,
            "disable_web_page_preview": true,
        });
        let json_str = serde_json::to_string(&body)
            .map_err(|e| ChannelError::new(format!("JSON serialization error: {}", e)))?;

        // Retry up to 3 times with exponential backoff (1s, 2s, 4s)
        let mut last_error = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                let delay = std::time::Duration::from_secs(1 << attempt);
                std::thread::sleep(delay);
            }

            match self.send_request(&url, &json_str) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e.message;
                    if !last_error.contains("ureq error") {
                        break;
                    }
                }
            }
        }

        Err(ChannelError::new(format!(
            "Telegram API failed after 3 retries: {}", last_error
        )))
    }
}

/// Build a `ureq::Agent` with optional proxy support.
///
/// Proxy resolution priority (high → low):
///   1. Explicit `proxy_url` argument
///   2. `NOTIFY_TELEGRAM_PROXY` env var
///   3. `HTTPS_PROXY` env var
///   4. `HTTP_PROXY` env var
///   5. Direct connection (no proxy)
pub fn build_agent(explicit_proxy: Option<String>) -> Agent {
    let proxy = explicit_proxy.or_else(|| {
        std::env::var("NOTIFY_TELEGRAM_PROXY")
            .ok()
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
    });

    let proxy = proxy.and_then(|url| ureq::Proxy::new(&url).ok());

    let config = Agent::config_builder()
        .proxy(proxy)
        .build();

    Agent::new_with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Level;

    /// Test that the TelegramBackend dry-run (no real API call) constructs
    /// correctly and reports its name.
    #[test]
    fn test_telegram_backend_name() {
        let backend = TelegramBackend::new(
            "123:abc".into(),
            "456".into(),
            None,
        );
        assert_eq!(backend.name(), "telegram");
    }

    /// Test that a message formats correctly for Telegram.
    #[test]
    fn test_message_format_for_telegram() {
        let msg = Message::new("Hello world")
            .with_title("Test")
            .with_level(Level::Success);
        let text = msg.format();
        assert!(text.contains("✅"));
        assert!(text.contains("Test"));
        assert!(text.contains("Hello world"));
    }
}
