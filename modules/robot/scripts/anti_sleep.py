"""anti_sleep.py — Prevent system sleep by micro-moving the mouse."""

import os
import random
import sys
import time

import pyautogui

# ── Config ──────────────────────────────────────────────────
INTERVAL_RANGE = os.getenv("ROBOT_ANTI_SLEEP_INTERVAL", "30-90")
MAX_MOVE = int(os.getenv("ROBOT_ANTI_SLEEP_MAX_MOVE", "5"))
USER_THRESHOLD = int(os.getenv("ROBOT_ANTI_SLEEP_USER_THRESHOLD", "20"))

try:
    parts = INTERVAL_RANGE.split("-")
    if len(parts) != 2:
        raise ValueError
    INTERVAL_MIN = int(parts[0])
    INTERVAL_MAX = int(parts[1])
except ValueError:
    INTERVAL_MIN, INTERVAL_MAX = 30, 90

# If mouse is at (0,0) when moveRel() starts, FAILSAFE raises FailSafeException.
# This gives the user a kill-switch: just flick the mouse to top-left.
pyautogui.FAILSAFE = True


def main():
    print(
        f"🛌 Anti-sleep started  (interval={INTERVAL_MIN}-{INTERVAL_MAX}s, "
        f"max_move={MAX_MOVE}px, threshold={USER_THRESHOLD}px)"
    )
    print("   Move mouse to top-left corner (0,0) or press Ctrl-C to stop.")

    last_x, last_y = pyautogui.position()

    try:
        while True:
            time.sleep(random.uniform(INTERVAL_MIN, INTERVAL_MAX))

            cur_x, cur_y = pyautogui.position()

            # If mouse moved significantly, user is active — skip this cycle
            if abs(cur_x - last_x) > USER_THRESHOLD or abs(cur_y - last_y) > USER_THRESHOLD:
                last_x, last_y = cur_x, cur_y
                continue

            # Random nudge direction
            dx = random.choice([-1, 1]) * random.randint(1, MAX_MOVE)
            dy = random.choice([-1, 1]) * random.randint(1, MAX_MOVE)

            # Nudge away then back. If the user moves the mouse during the
            # 0.5s gap, the return nudge drifts by ~1-5px — negligible.
            pyautogui.moveRel(dx, dy, duration=0.3)
            time.sleep(0.5)
            pyautogui.moveRel(-dx, -dy, duration=0.3)

            last_x, last_y = pyautogui.position()
    except (KeyboardInterrupt, pyautogui.FailSafeException):
        print("\n✓ Anti-sleep stopped.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
