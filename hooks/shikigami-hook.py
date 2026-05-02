#!/usr/bin/env python3
"""
shikigami-hook — AI coding tool hook → Shikigami event bridge.

Reads a hook payload from stdin (Claude Code or Codex CLI — both ship
near-identical schemas), transforms it into a Shikigami EventPayload
(schemas/event.v1.0.json), and POSTs to the local HTTP event server.
Never blocks the AI tool; all failures are swallowed silently (opt-in
trace via SHIKIGAMI_DEBUG=1).

Usage:
    echo "$HOOK_JSON" | shikigami-hook.py                    # default: claude-code
    echo "$HOOK_JSON" | shikigami-hook.py --source codex     # OpenAI Codex CLI

The `--source` flag controls only the EventPayload `source` field —
both tools deliver the same event names (PreToolUse / PostToolUse /
UserPromptSubmit / Stop / SessionStart) on the same JSON structure,
so the transform logic is shared. Codex adds `PermissionRequest`
which maps to a warning-severity event so Hiyori reacts to approval
prompts; Claude Code never emits this name.

Claude Code `settings.json` wires this in per hook event; see
`scripts/install-hook.py` for automated installation. For Codex, see
the README install snippet (TOML config in ~/.codex/config.toml).
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


# Cursor renames a handful of fields and uses camelCase event names
# while sticking to the same payload structure overall. Normalising at
# the edge means the rest of transform() doesn't need to know the
# source — single code path, single set of tests. Map both directions
# only for the 5-event minimal scope agreed in the v0.4 debate
# (sessionStart, preToolUse, postToolUse, postToolUseFailure, stop).
# The 13+ Cursor-specific events (afterMCPExecution, preCompact, etc.)
# fall through to the unknown-event branch and are silently skipped
# until real Cursor users report which ones matter.
_CURSOR_EVENT_RENAME = {
    "sessionStart": "SessionStart",
    "preToolUse": "PreToolUse",
    "postToolUse": "PostToolUse",
    "postToolUseFailure": "PostToolUse",  # treat failure as PostToolUse w/ exit_code !=0
    "stop": "Stop",
}

# Windsurf (Codeium) Cascade Hooks split tool calls by TYPE rather than
# a single PreToolUse + tool_name pair: pre_run_command for shell, pre_
# read_code for Read, pre_write_code for Edit, pre_mcp_tool_use for MCP
# tools. Per https://docs.windsurf.com/windsurf/cascade/hooks, payloads
# wrap event-specific data in a `tool_info` object and use snake_case
# event names. We map the 11 documented events that fit Shikigami's
# existing taxonomy. `agent_action_name` and `model_name` arrive on
# every payload but are unused at this layer.
#
# Each entry is (claude_shape_event_name, synthetic_tool_name). The
# tool_name stays None for non-tool events (UserPromptSubmit, Stop,
# SessionStart) — the existing transform() branches don't need it.
_WINDSURF_EVENT_MAP: dict[str, tuple[str, str | None]] = {
    "pre_user_prompt": ("UserPromptSubmit", None),
    "pre_run_command": ("PreToolUse", "Bash"),
    "post_run_command": ("PostToolUse", "Bash"),
    "pre_read_code": ("PreToolUse", "Read"),
    "post_read_code": ("PostToolUse", "Read"),
    "pre_write_code": ("PreToolUse", "Edit"),
    "post_write_code": ("PostToolUse", "Edit"),
    "pre_mcp_tool_use": ("PreToolUse", "MCP"),
    "post_mcp_tool_use": ("PostToolUse", "MCP"),
    "post_cascade_response": ("Stop", None),
    "post_cascade_response_with_transcript": ("Stop", None),
    "post_setup_worktree": ("SessionStart", None),
}


def normalize_cursor(cursor_event: dict) -> dict:
    """Rewrite a Cursor hook payload onto the Claude-Code key shape so
    transform() doesn't need source-specific branches everywhere.

    Cursor differences (per https://cursor.com/docs/hooks):
      - hook_event_name uses camelCase → map to PascalCase
      - conversation_id replaces session_id
      - workspace_roots[0] replaces cwd
      - tool_name + tool_input live at the same path; usable as-is
      - postToolUseFailure → synthesise an exit_code=1 tool_response so
        the existing PostToolUse branch flags it as warning severity
    """
    normalized = dict(cursor_event)  # shallow copy — never mutate caller's dict
    raw_name = cursor_event.get("hook_event_name") or ""
    normalized["hook_event_name"] = _CURSOR_EVENT_RENAME.get(raw_name, raw_name)

    if "conversation_id" in cursor_event and "session_id" not in cursor_event:
        normalized["session_id"] = cursor_event["conversation_id"]

    if "cwd" not in cursor_event:
        roots = cursor_event.get("workspace_roots")
        if isinstance(roots, list) and roots and isinstance(roots[0], str):
            normalized["cwd"] = roots[0]

    if raw_name == "postToolUseFailure":
        existing = cursor_event.get("tool_response") or {}
        if not isinstance(existing, dict):
            existing = {}
        merged = {**existing, "exit_code": existing.get("exit_code", 1)}
        normalized["tool_response"] = merged

    return normalized


def normalize_windsurf(ws_event: dict) -> dict:
    """Rewrite a Windsurf Cascade Hooks payload onto the Claude-Code key
    shape so transform() doesn't need source-specific branches.

    Windsurf differences (per https://docs.windsurf.com/windsurf/cascade/hooks):
      - hook_event_name uses snake_case + tool-typed names
        (pre_run_command vs PreToolUse + tool_name="Bash")
      - trajectory_id replaces session_id
      - tool_info nests the per-event data; we flatten the relevant
        fields into the Claude shape (command_line → tool_input.command,
        cwd → cwd, transcript_path → transcript_path, user_prompt →
        prompt)
      - response field on post_cascade_response is the assistant's reply
        text — stash on `last_assistant_text` so the Stop branch can
        forward it to TTS without needing the transcript file
    """
    raw_name = ws_event.get("hook_event_name") or ""
    mapping = _WINDSURF_EVENT_MAP.get(raw_name)
    if mapping is None:
        # Unknown event — pass through unchanged so transform() will
        # log + skip via the "unknown hook_event_name" branch.
        return ws_event

    claude_name, synthetic_tool = mapping
    normalized: dict = {"hook_event_name": claude_name}

    if synthetic_tool is not None:
        normalized["tool_name"] = synthetic_tool

    if "trajectory_id" in ws_event:
        normalized["session_id"] = ws_event["trajectory_id"]

    info = ws_event.get("tool_info") or {}
    if not isinstance(info, dict):
        info = {}

    # Lift event-specific fields out of tool_info into the flat shape
    # the Claude-style transform expects.
    if synthetic_tool == "Bash":
        cmd = info.get("command_line", "")
        normalized["tool_input"] = {"command": cmd}
        if "cwd" in info:
            normalized["cwd"] = info["cwd"]
    elif synthetic_tool in ("Read", "Edit"):
        normalized["tool_input"] = {"file_path": info.get("file_path", "")}
    elif synthetic_tool == "MCP":
        # MCP tools don't carry shell commands, so destructive detection
        # never fires for them; tool_input shape is informational only.
        normalized["tool_input"] = {
            "server": info.get("mcp_server_name"),
            "tool": info.get("mcp_tool_name"),
        }

    # post_run_command, post_read_code, post_write_code, post_mcp_tool_use
    # all benefit from a synthesised tool_response so the existing
    # PostToolUse branch can flag exit codes. Windsurf doesn't expose
    # exit_code directly per docs — assume success unless mcp_result
    # indicates otherwise. Tolerant: real users will report mismatches.
    if claude_name == "PostToolUse":
        normalized["tool_response"] = {
            "exit_code": 0,
            "stdout": str(info.get("mcp_result", ""))[:1024],
        }

    if raw_name == "pre_user_prompt":
        normalized["prompt"] = info.get("user_prompt", "")

    if raw_name == "post_cascade_response_with_transcript":
        if "transcript_path" in info:
            normalized["transcript_path"] = info["transcript_path"]

    return normalized


def transform(cc_event: dict, source: str = "claude-code") -> dict | None:
    """Transform an upstream hook payload into Shikigami EventPayload v1.0.

    The same transform handles Claude Code, Codex CLI, Cursor, and
    Windsurf. Claude Code + Codex converged on identical hook event
    names + JSON structure (see https://developers.openai.com/codex/hooks);
    Cursor uses camelCase event names + slightly different field names
    which we rewrite to the Claude shape via normalize_cursor() before
    dispatch; Windsurf uses snake_case event names + tool-typed events
    + a nested tool_info object which normalize_windsurf() flattens.
    The `source` arg becomes the EventPayload `source` field.

    Returns None when the event should be skipped (e.g., unknown hook
    name — including the 13+ Cursor-specific and 1+ Windsurf-specific
    events outside the minimal scope agreed in the v0.4 debate).
    """
    if source == "cursor":
        cc_event = normalize_cursor(cc_event)
    elif source == "windsurf":
        cc_event = normalize_windsurf(cc_event)

    hook_name = cc_event.get("hook_event_name") or cc_event.get("hook") or ""
    tool_name = cc_event.get("tool_name")
    tool_input = cc_event.get("tool_input") or {}
    tool_response = cc_event.get("tool_response") or {}
    # Forward identifiers so the backend can group / filter events by
    # session — lets the user pick which AI-tool tabs Hiyori reacts to
    # when several are running simultaneously.
    session_id = cc_event.get("session_id")
    cwd = cc_event.get("cwd")

    payload: dict = {"schemaVersion": "1.0", "source": source}
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

    elif hook_name == "PermissionRequest":
        # Codex-only: user is being asked to approve a tool action. Map
        # to a warning state so Hiyori signals "agent is waiting on you"
        # rather than letting it look idle while permission is pending.
        # Claude Code never emits this name; the branch is unreachable
        # for source="claude-code" and that's fine.
        payload.update({
            "type": "permission_request",
            "tool": tool_name,
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


def parse_source(argv: list[str]) -> str:
    """Tiny argparse stand-in — keeps the script stdlib-only with no
    argparse import overhead on the hot path. Accepts `--source <name>`
    or `--source=<name>`. Defaults to claude-code for backwards compat.
    """
    for i, a in enumerate(argv):
        if a == "--source" and i + 1 < len(argv):
            return argv[i + 1]
        if a.startswith("--source="):
            return a.split("=", 1)[1]
    return "claude-code"


def main() -> int:
    source = parse_source(sys.argv[1:])
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            log("empty stdin")
            return 0
        cc_event = json.loads(raw)
    except json.JSONDecodeError as e:
        log(f"bad JSON stdin: {e}")
        return 0

    payload = transform(cc_event, source=source)
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
