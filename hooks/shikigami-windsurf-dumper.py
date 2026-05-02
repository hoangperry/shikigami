#!/usr/bin/env python3
"""
shikigami-windsurf-dumper — capture Windsurf Cascade Hook payloads for
schema reverse-engineering.

Background: Windsurf ships Cascade Hooks (https://docs.windsurf.com/docs/agent/cascade-hooks)
but the public docs don't fully specify the JSON payload shape per
event. We can't write the bridge transform without knowing the actual
keys. This dumper is a no-op shim: register it as Windsurf's hook
command, run a few normal sessions, then upload the captured JSONL to
GitHub issue #34 so a maintainer can write the transform.

Usage (per Windsurf docs — the exact registration command differs by
Windsurf version; consult their docs for the current syntax):

    # Pseudocode — adjust per the version installed:
    # In Windsurf settings, register hooks:
    #   PreToolUse, PostToolUse, UserPromptSubmit, Stop, SessionStart
    # All pointing at: python3 /path/to/shikigami-windsurf-dumper.py

Output: each invocation appends ONE line of JSONL to:
    ~/.shikigami/windsurf-payloads.jsonl

After collecting ~10-20 invocations across normal use, upload the file
to issue #34 via:

    gh issue comment 34 \\
      --body "Captured Windsurf payloads attached" \\
      --body-file ~/.shikigami/windsurf-payloads.jsonl

(or paste the contents into a GitHub comment).

Privacy: this script captures the raw JSON Windsurf sends to its
hooks. That can include shell commands, file paths, and prompt text.
**Review the file before uploading** — strip secrets, redact paths
that include personal data. The dumper has zero network activity by
design; it only writes locally.

This script is part of GitHub issue #34. It is intentionally NOT
wired into the main bridge (hooks/shikigami-hook.py) because we
can't write a transform until we have the schema. After enough
samples are collected and the transform is implemented, this
dumper will be deleted.
"""
from __future__ import annotations

import json
import os
import pathlib
import sys
from datetime import datetime, timezone

OUT_DIR = pathlib.Path(
    os.environ.get("SHIKIGAMI_HOME", pathlib.Path.home() / ".shikigami")
)
OUT_FILE = OUT_DIR / "windsurf-payloads.jsonl"


def main() -> int:
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            return 0
        # Validate it parses, but persist raw text too — Windsurf may
        # add fields we want to inspect literally.
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            parsed = None  # capture anyway under a flag

        OUT_DIR.mkdir(parents=True, exist_ok=True)
        record = {
            "captured_at": datetime.now(timezone.utc).isoformat(),
            "argv": sys.argv[1:],
            "raw_text_len": len(raw),
            "parsed_ok": parsed is not None,
            "raw_text": raw if parsed is None else None,
            "parsed": parsed,
        }
        with OUT_FILE.open("a", encoding="utf-8") as f:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")
    except Exception:
        # Never block Windsurf — even if our capture fails the host
        # tool should keep running normally.
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
