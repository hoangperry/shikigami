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
    # Forward identifiers so the backend can group / filter events by
    # session — lets the user pick which Claude Code tabs Hiyori reacts
    # to when several are running simultaneously.
    session_id = cc_event.get("session_id")
    cwd = cc_event.get("cwd")

    payload: dict = {"schemaVersion": "1.0", "source": "claude-code"}
    if isinstance(session_id, str) and session_id:
        payload["sessionId"] = session_id
    if isinstance(cwd, str) and cwd:
        payload["cwd"] = cwd

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
        with urllib.request.urlopen(req, timeout=2.0) as resp:
            log(f"POST {url} → {resp.status}")
    except urllib.error.URLError as e:
        log(f"POST failed: {e}")
    except Exception as e:  # never block Claude Code
        log(f"unexpected: {e}")


# Strip code blocks, tool-use blocks, and excessive whitespace so the TTS
# engine doesn't try to read raw JSON or markdown syntax aloud.
_CODE_BLOCK = re.compile(r"```[\s\S]*?```")
_INLINE_CODE = re.compile(r"`[^`]+`")
_MD_LINK = re.compile(r"\[([^\]]+)\]\([^)]+\)")
_MD_HEADING = re.compile(r"^#+\s*", re.MULTILINE)
_MD_BULLET = re.compile(r"^[\s]*[-*+]\s+", re.MULTILINE)
_WHITESPACE = re.compile(r"\s+")


def _flatten_content(content) -> str:
    """Claude Code transcript content can be str or list of typed blocks."""
    if isinstance(content, str):
        return content
    if not isinstance(content, list):
        return ""
    parts: list[str] = []
    for block in content:
        if isinstance(block, dict):
            # Skip tool_use / tool_result / thinking blocks — they are noise
            # for spoken output. Only keep prose text blocks.
            if block.get("type") in (None, "text"):
                t = block.get("text") or ""
                if isinstance(t, str):
                    parts.append(t)
    return "\n".join(parts)


def clean_for_speech(text: str, max_chars: int = 400) -> str:
    """Strip markdown / code so TTS reads natural prose. Truncate to keep the
    spoken segment short — long monologues are user-hostile."""
    if not text:
        return ""
    text = _CODE_BLOCK.sub(" (code) ", text)
    text = _INLINE_CODE.sub("", text)
    text = _MD_LINK.sub(r"\1", text)
    text = _MD_HEADING.sub("", text)
    text = _MD_BULLET.sub("", text)
    text = _WHITESPACE.sub(" ", text).strip()
    if len(text) > max_chars:
        # Cut at last sentence boundary inside the cap.
        cut = text[:max_chars]
        for end in (". ", "! ", "? ", "\n"):
            i = cut.rfind(end)
            if i >= max_chars * 0.6:
                return cut[: i + 1].strip()
        return cut.strip()
    return text


def extract_last_assistant_text(transcript_path: str) -> str | None:
    """Read the JSONL transcript and return the last assistant message text.
    Returns None on read error / no assistant turn / empty content."""
    try:
        p = pathlib.Path(transcript_path)
        if not p.is_file():
            return None
        # Transcripts can be large. Read all lines but only keep last 200.
        with p.open("r", encoding="utf-8") as f:
            lines = f.readlines()[-200:]
    except OSError as e:
        log(f"transcript read failed: {e}")
        return None

    for line in reversed(lines):
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        # Claude Code transcript schema: { "type": "assistant", "message": {...} }
        # or top-level role/content.
        msg = obj.get("message") if isinstance(obj.get("message"), dict) else obj
        role = obj.get("type") or msg.get("role") or msg.get("type")
        if role != "assistant":
            continue
        content = msg.get("content")
        text = _flatten_content(content).strip()
        if text:
            return clean_for_speech(text)
    return None


def post_say(text: str, token: str, base_url: str) -> None:
    """Trigger TTS via /v1/say. Same fire-and-forget contract as post()."""
    if not text or len(text) < 8:
        return
    say_url = base_url.replace("/v1/events", "/v1/say")
    body = json.dumps({"text": text}).encode("utf-8")
    req = urllib.request.Request(
        say_url,
        data=body,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        # Longer timeout — TTS may take ~1-2s for cloud providers.
        with urllib.request.urlopen(req, timeout=5.0) as resp:
            log(f"POST {say_url} → {resp.status}")
    except urllib.error.URLError as e:
        log(f"POST say failed: {e}")
    except Exception as e:
        log(f"unexpected say: {e}")


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

    # Stop hook = end of an assistant turn. Read the last assistant message
    # from the transcript and pipe it through TTS so Hiyori speaks the reply
    # aloud. Fire-and-forget — never blocks Claude Code.
    if cc_event.get("hook_event_name") == "Stop":
        transcript_path = cc_event.get("transcript_path")
        if isinstance(transcript_path, str) and transcript_path:
            last = extract_last_assistant_text(transcript_path)
            if last:
                post_say(last, token, url)
    return 0


if __name__ == "__main__":
    sys.exit(main())
