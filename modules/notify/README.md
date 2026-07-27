# notify — CLI notification tool

Send notifications via Telegram Bot, watch command execution, and audit all notifications to a local
log.

## Prerequisites

- [Rust](https://rustup.rs/) toolchain

## Setup

```bash
# 1. Configure
cp config/.env.example config/.env
# edit config/.env — set NOTIFY_TELEGRAM_BOT_TOKEN, NOTIFY_TELEGRAM_CHAT_ID
```

## Telegram Bot Setup

> If you already have a Telegram bot, skip to [Step 1b](#step-1b-get-token-from-existing-bot).

### Step 1: Create a new bot and get the token

1. Open Telegram and search for [@BotFather](https://t.me/BotFather)
2. Send `/newbot` and follow the prompts:
   - Choose a display name (e.g. `My Notifier`)
   - Choose a username ending in `bot` (e.g. `my_notifier_bot`)
3. On success, BotFather returns a **token** like:
   ```
   123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
   ```
4. Copy this token and set it in `config/.env`:
   ```
   NOTIFY_TELEGRAM_BOT_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
   ```

### Step 1b: Get token from an existing bot

If you already have a bot (e.g. created earlier for another project):

1. Open Telegram and search for [@BotFather](https://t.me/BotFather)
2. Send `/mybots` — BotFather shows a list of your bots
3. Tap the bot you want to use for notifications
4. Tap **API Token** to reveal or copy the token
5. Paste it into `config/.env`:
   ```
   NOTIFY_TELEGRAM_BOT_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
   ```

### Step 2: Get your Chat ID

Your Chat ID is the identifier that tells the bot where to send messages. There are two ways:

**Option A — Use @userinfobot (recommended):**

1. Search for [@userinfobot](https://t.me/userinfobot) on Telegram
2. Send `/start` to the bot
3. It replies with your **Chat ID** (a number, e.g. `123456789`)
4. Copy this number to `config/.env`:
   ```
   NOTIFY_TELEGRAM_CHAT_ID=123456789
   ```

**Option B — Use a test message to your own bot:**

1. Open your new bot and click **Start**
2. Send any message to the bot
3. Visit: `https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates` (replace `<YOUR_TOKEN>` with your
   actual token)
4. Look for `"chat":{"id":123456789,...}` in the JSON response
5. Copy the `id` number to `config/.env`

### Step 3: Test the connection

```bash
# Dry-run test (no real API call)
just notify::test

# Send a real test message
just notify::send "Hello from notify!" --level success

# If behind a GFW, configure proxy:
# NOTIFY_TELEGRAM_PROXY=http://127.0.0.1:7890
# Or pass inline:
# just notify::send "test" --proxy http://127.0.0.1:7890
```

## Build

```bash
# Build all Rust binaries (notify + static-server, release)
just build

# Build only notify
just build-notify
just notify::build         # module-specific build

# Debug build (no --release)
just build-debug
just build-debug-notify
```

## Usage

```bash
# Send a notification
just notify::send "Hello world"
just notify::send "Backup done" --level success
just notify::send "Error!" --level error --channel telegram,stdout

# Watch a command and get notified on completion
just notify::watch "echo hello"                               # basic
just notify::watch "long-task.sh" --on-error                  # only on failure
just notify::watch "echo hello" --channel stdout              # output to stdout

# Test channels (dry-run, no real API calls)
just notify::test
just notify::test --channel stdout --verbose
just notify::test --watch "echo hello"

# Query audit log
just notify::log list
just notify::log list --status failed
just notify::log status

# Show resolved environment
just notify::env
```

## Configuration (`config/.env`)

| Variable                    | Required | Default                     | Description                                                     |
| --------------------------- | -------- | --------------------------- | --------------------------------------------------------------- |
| `NOTIFY_TELEGRAM_BOT_TOKEN` | Yes*     | —                           | Telegram bot token from @BotFather                              |
| `NOTIFY_TELEGRAM_CHAT_ID`   | Yes*     | —                           | Recipient chat ID                                               |
| `NOTIFY_TELEGRAM_PROXY`     | No       | —                           | Proxy URL (e.g., `http://127.0.0.1:7890`)                       |
| `NOTIFY_DEFAULT_CHANNEL`    | No       | `telegram`                  | Default channel(s), comma-separated                             |
| `NOTIFY_LOG_FILE`           | No       | `modules/notify/notify.log` | Audit log path                                                  |
| `NOTIFY_TAIL_LINES`         | No       | `30`                        | Default output tail lines for `watch`                           |
| `NOTIFY_SILENT`             | No       | `false`                     | Global switch: `true` disables all notifications (wrap & watch) |

*Required only if using Telegram channel.

## Channels

| Channel  | Name       | Description                                 |
| -------- | ---------- | ------------------------------------------- |
| Telegram | `telegram` | Sends via Telegram Bot API (supports proxy) |
| Stdout   | `stdout`   | Prints to stdout (testing / debugging)      |
| Log      | `log`      | Always writes to audit log (implicit)       |

## Proxy Priority

1. CLI `--proxy` flag
2. `NOTIFY_TELEGRAM_PROXY` env var
3. `HTTPS_PROXY` / `HTTP_PROXY` env var
4. Direct connection (no proxy)
