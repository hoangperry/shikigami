#!/usr/bin/env python3
"""
install-hook — configure Claude Code to send events to Shikigami.

Idempotently merges Shikigami hook entries into ~/.claude/settings.json
without clobbering existing hooks. Can also uninstall.

Usage:
    install-hook.py install
    install-hook.py uninstall
    install-hook.py status
    install-hook.py doctor

Scope: registers shikigami-hook.py on PreToolUse, PostToolUse,
UserPromptSubmit, Stop, and SessionStart events.
"""
from __future__ import annotations

import json
import os
import pathlib
import sys
import shutil
from typing import Any

SETTINGS = pathlib.Path.home() / ".claude" / "settings.json"
HOOK_SCRIPT = (
    pathlib.Path(__file__).resolve().parent.parent / "hooks" / "shikigami-hook.py"
)
MARKER = "shikigami-hook"

HOOK_EVENTS = [
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "Stop",
    "SessionStart",
]


def load_settings() -> dict:
    if not SETTINGS.exists():
        return {}
    try:
        return json.loads(SETTINGS.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        die(f"{SETTINGS} is not valid JSON: {e}")


def save_settings(cfg: dict) -> None:
    SETTINGS.parent.mkdir(parents=True, exist_ok=True)
    if SETTINGS.exists():
        backup = SETTINGS.with_suffix(".json.bak")
        shutil.copy2(SETTINGS, backup)
        print(f"  backed up existing settings → {backup}")
    SETTINGS.write_text(json.dumps(cfg, indent=2) + "\n", encoding="utf-8")


def die(msg: str) -> None:
    print(f"✗ {msg}", file=sys.stderr)
    sys.exit(1)


def is_shikigami_hook(item: dict) -> bool:
    """Return True for any hook entry that looks like ours."""
    if not isinstance(item, dict):
        return False
    inner = item.get("hooks")
    if not isinstance(inner, list):
        return False
    for h in inner:
        if isinstance(h, dict):
            cmd = h.get("command", "")
            if MARKER in str(cmd):
                return True
    return False


def ensure_hook_event(cfg: dict, event: str, command: str) -> bool:
    """Ensure `command` is registered for `event`. Returns True if modified."""
    hooks = cfg.setdefault("hooks", {})
    per_event = hooks.setdefault(event, [])
    if not isinstance(per_event, list):
        die(f"hooks.{event} is not a list in settings.json")

    # Already installed?
    for entry in per_event:
        if is_shikigami_hook(entry):
            return False

    per_event.append(
        {
            "matcher": "",
            "hooks": [
                {
                    "type": "command",
                    "command": command,
                    "description": f"Shikigami — {event}",
                }
            ],
        }
    )
    return True


def remove_hook_event(cfg: dict, event: str) -> bool:
    """Remove shikigami entries from `event`. Returns True if any removed."""
    hooks = cfg.get("hooks", {})
    per_event = hooks.get(event)
    if not isinstance(per_event, list):
        return False
    before = len(per_event)
    per_event[:] = [e for e in per_event if not is_shikigami_hook(e)]
    return len(per_event) != before


def cmd_install() -> None:
    if not HOOK_SCRIPT.exists():
        die(f"hook script missing: {HOOK_SCRIPT}")
    # Windows ships `python` (not `python3`) on PATH; macOS / Linux ship
    # `python3`. Use the right symbol for the host so Claude Code can
    # actually exec the hook command after install. Quote the path so a
    # space in the user's home directory (common on Windows) doesn't
    # split the command at shell-execution time.
    py = "python" if sys.platform == "win32" else "python3"
    command = f'{py} "{HOOK_SCRIPT}"'

    cfg = load_settings()
    changed = False
    for evt in HOOK_EVENTS:
        if ensure_hook_event(cfg, evt, command):
            print(f"  + registered on {evt}")
            changed = True
        else:
            print(f"  · already installed on {evt}")

    if changed:
        save_settings(cfg)
        print(f"✓ installed into {SETTINGS}")
    else:
        print("✓ already up-to-date, nothing to change")


def cmd_uninstall() -> None:
    cfg = load_settings()
    removed_any = False
    for evt in HOOK_EVENTS:
        if remove_hook_event(cfg, evt):
            print(f"  - removed from {evt}")
            removed_any = True

    if removed_any:
        save_settings(cfg)
        print(f"✓ uninstalled from {SETTINGS}")
    else:
        print("✓ no Shikigami entries found, nothing to remove")


def cmd_status() -> None:
    cfg = load_settings()
    any_installed = False
    for evt in HOOK_EVENTS:
        per_event = cfg.get("hooks", {}).get(evt, [])
        has = any(is_shikigami_hook(e) for e in per_event if isinstance(e, dict))
        marker = "✓" if has else "·"
        if has:
            any_installed = True
        print(f"  {marker} {evt}")
    if not any_installed:
        print("  (no Shikigami hooks installed)")


def cmd_doctor() -> None:
    ok = True
    def check(label: str, cond: bool, detail: str = "") -> None:
        nonlocal ok
        mark = "✓" if cond else "✗"
        print(f"  {mark} {label}" + (f" — {detail}" if detail else ""))
        if not cond:
            ok = False

    check("hook script exists", HOOK_SCRIPT.exists(), str(HOOK_SCRIPT))
    check("hook script executable", HOOK_SCRIPT.exists() and os.access(HOOK_SCRIPT, os.X_OK))
    shikigami_home = pathlib.Path.home() / ".shikigami"
    check("~/.shikigami exists", shikigami_home.is_dir(), str(shikigami_home))
    token = shikigami_home / "token"
    check("token file present", token.is_file(), f"{token}")
    if token.is_file():
        try:
            t = token.read_text().strip()
            check("token is 64 hex chars", len(t) == 64)
        except OSError:
            check("token is readable", False)
    check("settings.json exists", SETTINGS.is_file(), str(SETTINGS))
    if SETTINGS.is_file():
        cmd_status()

    if ok:
        print("\n✓ all checks passed")
    else:
        print("\n✗ some checks failed")
        sys.exit(1)


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 1
    verb = sys.argv[1]
    if verb == "install":
        cmd_install()
    elif verb == "uninstall":
        cmd_uninstall()
    elif verb == "status":
        cmd_status()
    elif verb == "doctor":
        cmd_doctor()
    else:
        print(__doc__)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
