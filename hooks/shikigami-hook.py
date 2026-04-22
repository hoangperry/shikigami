#!/usr/bin/env python3
"""
shikigami-hook — Claude Code hook → Shikigami event bridge.

Reads the Claude Code hook payload from stdin, transforms it into a
Shikigami EventPayload (schemas/event.v1.0.json), and POSTs to the local
HTTP event server. Never blocks Claude Code; all failures are swallowed
silently (opt-in trace via SHIKIGAMI_DEBUG=1).

Usage:
    echo "$CLAUDE_CODE_HOOK_JSON" | shikigami-hook.py

Claude Code `settings.json` wires this in per hook event; see
`scripts/install-hook.py` for automated installation.
"""
from __future__ import annotations

import json
import os
import pathlib
import re
import sys
import urllib.error
import urllib.request

HOME = pathlib.Path(os.environ.get("SHIKIGAMI_HOME", pathlib.Path.home() / ".shikigami"))
TOKEN_FILE = HOME / "token"
CONFIG_FILE = HOME / "config.json"
DEBUG = os.environ.get("SHIKIGAMI_DEBUG", "0") == "1"
DESTRUCTIVE_PATTERNS = [
    re.compile(r"\brm\s+-[rf]+[^\s]*\s"),
    re.compile(r"\brm\s+-[rf]+\s+/"),
    re.compile(r"\bDROP\s+(?:TABLE|DATABASE|SCHEMA)\b", re.IGNORECASE),
    re.compile(r"\bTRUNCATE\s+TABLE\b", re.IGNORECASE),
    re.compile(r"\bgit\s+push\b.*--force"),
    re.compile(r"\bgit\s+reset\s+--hard\b"),
]


def log(msg: str) -> None:
    if DEBUG:
        print(f"[shikigami-hook] {msg}", file=sys.stderr)


def read_token() -> str | None:
    try:
        t = TOKEN_FILE.read_text(encoding="utf-8").strip()
        return t if len(t) == 64 else None
    except OSError as e:
        log(f"token read failed: {e}")
        return None


def read_port() -> int:
    if "SHIKIGAMI_URL" in os.environ:
        return 0  # signaled by caller
    try:
        cfg = json.loads(CONFIG_FILE.read_text(encoding="utf-8"))
        return int(cfg.get("port", 7796))
    except (OSError, json.JSONDecodeError, ValueError):
        return 7796


def is_destructive(command: str) -> bool:
    return any(p.search(command) for p in DESTRUCTIVE_PATTERNS)


def transform(cc_event: dict) -> dict | None:
    """Transform a Claude Code hook payload into Shikigami EventPayload v1.0.
    Returns None when the event should be skipped (e.g., unknown hook name).
    """
    hook_name = cc_event.get("hook_event_name") or cc_event.get("hook") or ""
    tool_name = cc_event.get("tool_name")
    tool_input = cc_event.get("tool_input") or {}
    tool_response = cc_event.get("tool_response") or {}

    payload: dict = {"schemaVersion": "1.0", "source": "claude-code"}

    if hook_name == "PreToolUse":
        command = (tool_input.get("command") if isinstance(tool_input, dict) else "") or ""
        if tool_name == "Bash" and is_destructive(command):
            payload.update({
                "type": "destructive_op_detected",
                "tool": tool_name,
                "severity": "critical",
                "text": command[:200],
                "metadata": {"hook": hook_name},
            })
        else:
            payload.update({
                "type": "tool_start",
                "tool": tool_name,
                "severity": "info",
                "metadata": {"hook": hook_name},
            })

    elif hook_name == "PostToolUse":
        exit_code = None
        if isinstance(tool_response, dict):
            exit_code = tool_response.get("exit_code") or tool_response.get("exitCode")
        text = ""
        if isinstance(tool_response, dict):
            text = tool_response.get("stderr") or tool_response.get("stdout") or ""
        severity = "info" if (exit_code in (0, None)) else "warning"
        payload.update({
            "type": "tool_complete",
            "tool": tool_name,
            "exitCode": exit_code if isinstance(exit_code, int) else 0,
            "severity": severity,
            "text": text[:1024] if isinstance(text, str) else "",
            "metadata": {"hook": hook_name},
        })

    elif hook_name == "UserPromptSubmit":
        prompt = cc_event.get("prompt") or ""
        payload.update({
            "type": "user_prompt",
            "severity": "info",
            "text": prompt[:256] if isinstance(prompt, str) else "",
            "metadata": {"hook": hook_name},
        })

    elif hook_name == "Stop":
        payload.update({
            "type": "session_end",
            "severity": "info",
            "metadata": {"hook": hook_name},
        })

    elif hook_name == "SessionStart":
        payload.update({
            "type": "session_start",
            "severity": "info",
            "metadata": {"hook": hook_name},
        })

    elif hook_name == "Notification":
        payload.update({
            "type": "error",
            "severity": "warning",
            "text": str(cc_event.get("message", ""))[:512],
            "metadata": {"hook": hook_name},
        })

    else:
        log(f"unknown hook_event_name: {hook_name}")
        return None

    return payload


def post(payload: dict, token: str, url: str) -> None:
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=1.0) as resp:
            log(f"POST {url} → {resp.status}")
    except urllib.error.URLError as e:
        log(f"POST failed: {e}")
    except Exception as e:  # never block Claude Code
        log(f"unexpected: {e}")


def main() -> int:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            log("empty stdin")
            return 0
        cc_event = json.loads(raw)
    except json.JSONDecodeError as e:
        log(f"bad JSON stdin: {e}")
        return 0

    payload = transform(cc_event)
    if payload is None:
        return 0

    token = read_token()
    if not token:
        log("no token — is shikigami running?")
        return 0

    url = os.environ.get("SHIKIGAMI_URL") or f"http://127.0.0.1:{read_port()}/v1/events"
    post(payload, token, url)
    return 0


if __name__ == "__main__":
    sys.exit(main())
