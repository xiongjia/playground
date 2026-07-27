# Robot Module

Desktop automation tools — mouse/keyboard/window control via Python (`pyautogui`).

## Prerequisites

- **[uv](https://docs.astral.sh/uv/getting-started/installation/)** >= 0.4 — Python package manager
  - macOS: `brew install uv`
  - Other: `pipx install uv` or the
    [official installer](https://docs.astral.sh/uv/getting-started/installation/)
- Python >= 3.12 (managed automatically by `uv`)

## OS Permissions

Controlling the mouse requires accessibility permissions:

- **macOS**: System Settings → Privacy & Security → Accessibility → grant permission to your
  terminal app (e.g. Terminal, iTerm2, VS Code).
- **Linux**: Typically no extra setup (XTest extension is usually available).
- **Windows**: No extra setup.

## Configuration

Set the following in `config/.env` (see `config/.env.example`):

```env
# Anti-sleep interval range (seconds), randomly chosen each cycle
# ROBOT_ANTI_SLEEP_INTERVAL=30-90

# Max pixels to move per nudge
# ROBOT_ANTI_SLEEP_MAX_MOVE=5

# Distance threshold to detect user activity (pixels)
# ROBOT_ANTI_SLEEP_USER_THRESHOLD=20
```

> **Troubleshooting**: If `just robot::setup` fails to download packages (network restrictions), set
> `HTTP_PROXY=http://127.0.0.1:1095` before the command. uv respects standard proxy env vars.

## Commands

| Command                  | Description                                                           |
| ------------------------ | --------------------------------------------------------------------- |
| `just robot::setup`      | Install / update Python dependencies                                  |
| `just robot::anti-sleep` | Start anti-sleep daemon — randomly nudges mouse every 30-90s (Ctrl-C) |
| `just robot::env`        | Show resolved environment config                                      |

### Examples

```bash
# First use: install dependencies
just robot::setup

# Start anti-sleep — randomly nudges mouse every 30-90s to prevent sleep
just robot::anti-sleep

# Override interval inline
ROBOT_ANTI_SLEEP_INTERVAL=10-30 just robot::anti-sleep
```
