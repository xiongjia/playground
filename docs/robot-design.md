# Robot Module Design

> Design decisions for the desktop automation (anti-sleep, clicker) module.

---

## 1. Why pyautogui

- **Cross-platform** — macOS, Linux, Windows via a single API
- **Pure Python** — easy to install via `uv`, no native compilation
- **Well-maintained** — active development, comprehensive documentation
- **Fail-safe** — built-in `FAILSAFE` mode (flick mouse to top-left corner to abort)

Alternatives considered:

| Option                    | Why rejected                                                 |
| ------------------------- | ------------------------------------------------------------ |
| `applescript`/`osascript` | macOS-only, no Linux/Windows support                         |
| `xdotools`                | Linux-only, macOS requires XQuartz                           |
| `Selenium`                | Overkill for simple mouse/keyboard automation                |
| Rust `enigo` crate        | Would add a native binary to the project; higher maintenance |

---

## 2. Anti-sleep Strategy

### Problem

macOS and Linux will sleep/screensaver after a period of user inactivity. Some long-running
processes (downloads, backups, compilations) need the machine to stay awake.

### Solution

Nudge the mouse cursor by a tiny amount at random intervals. This is enough to reset the idle timer
without interfering with the user's work.

### Parameters

| Variable                          | Default | Description                                |
| --------------------------------- | ------- | ------------------------------------------ |
| `ROBOT_ANTI_SLEEP_INTERVAL`       | `30-90` | Random interval range in seconds           |
| `ROBOT_ANTI_SLEEP_MAX_MOVE`       | `5`     | Maximum pixels to move per nudge           |
| `ROBOT_ANTI_SLEEP_USER_THRESHOLD` | `20`    | Distance threshold to detect user activity |

### Algorithm

```
loop:
  sleep(random(INTERVAL_MIN, INTERVAL_MAX))
  current = mouse.position()
  if distance(current, last_known) < USER_THRESHOLD:
    # User is not actively using the mouse → safe to nudge
    dx = random(-MAX_MOVE, MAX_MOVE)
    dy = random(-MAX_MOVE, MAX_MOVE)
    pyautogui.moveRel(dx, dy)
    pyautogui.moveRel(-dx, -dy)  # move back to original position
  else:
    # User is active → skip this cycle, just update last_known
    pass
  last_known = current
```

### Why move back?

Without the return move, the cursor would drift over time. Moving back to the original position
keeps the cursor exactly where the user left it.

---

## 3. Permissions

### macOS

Controlling the mouse requires **Accessibility** permission:

```
System Settings → Privacy & Security → Accessibility
```

The terminal app (Terminal, iTerm2, VS Code, etc.) must be granted permission.

### Linux

Uses XTest extension, usually available by default on desktop environments (GNOME, KDE, etc.).
Wayland sessions may require additional configuration (`xdotool` or `ydotool`).

### Windows

No extra permissions needed. `pyautogui` uses Win32 API directly.

---

## 4. File Structure

```
modules/robot/
├── justfile                    # Recipe entry points
├── README.md                   # User documentation
├── pyproject.toml              # Python project config (uv)
└── scripts/
    └── anti_sleep.py           # Anti-sleep daemon
```

---

## 5. Python Dependencies (pyproject.toml)

```toml
[project]
dependencies = ["pyautogui>=0.9.54"]
```

Only `pyautogui` is required. It pulls in:

- `PyObjC` (macOS) — bridge to macOS accessibility APIs
- `python-xlib` (Linux) — bridge to X11
- `pymsgbox`, `pyscreeze`, `pytweening` — utilities

Dependencies are managed by `uv`:

```bash
uv sync --directory modules/robot
```

---

## 6. Recipe Design

```
robot/
├── setup        → uv sync (install dependencies)
├── anti-sleep   → python scripts/anti_sleep.py
└── env          → show resolved config
```

---

## 7. Safety

- **FAILSAFE**: `pyautogui.FAILSAFE = True` — moving the mouse to the top-left corner (0,0) raises
  `FailSafeException` and aborts the script
- **User detection**: If the mouse position has moved more than `USER_THRESHOLD` pixels since the
  last check, the script assumes the user is active and skips the nudge
- **Small moves**: Maximum 5 pixels per nudge — imperceptible to the user but enough to reset the
  idle timer

---

## 8. Limitations & Future Work

- **Wayland** — Linux Wayland sessions may not support XTest; `ydotool` or `wtype` could be used as
  alternatives
- **Clicker** — A mouse clicker/auto-clicker feature could be added with `pyautogui.click()`
- **Keystroke simulation** — `pyautogui.typewrite()` and `pyautogui.hotkey()` are available for
  keyboard automation
- **Window management** — `pygetwindow` could be added for window focus/minimize operations
- **Screenshots** — `pyautogui.screenshot()` could be used for visual monitoring
