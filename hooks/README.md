# Claude Code → Shikigami Hook Bridge

The scripts in this directory wire Claude Code into Shikigami so the character reacts to your agent's activity in real time.

## Quick Start

```bash
# One-time install (modifies ~/.claude/settings.json)
python3 scripts/install-hook.py install

# Verify
python3 scripts/install-hook.py doctor

# Uninstall
python3 scripts/install-hook.py uninstall
```

The installer is idempotent — running `install` twice does not duplicate entries. It also creates a `.bak` backup of your existing `~/.claude/settings.json` on first write.

## What Gets Installed

Five entries added to `~/.claude/settings.json` under `hooks`:

| Claude Code event  | Shikigami event(s)                              |
|--------------------|-------------------------------------------------|
| `PreToolUse`       | `tool_start` · or `destructive_op_detected` (critical) |
| `PostToolUse`      | `tool_complete` with exit code                  |
| `UserPromptSubmit` | `user_prompt`                                   |
| `Stop`             | `session_end`                                   |
| `SessionStart`     | `session_start`                                 |

## Destructive Operation Detection

Before `Bash` tool calls, `shikigami-hook.py` inspects the command for patterns like:

- `rm -rf /path`
- `DROP TABLE` / `DROP DATABASE` / `TRUNCATE TABLE` (case-insensitive)
- `git push --force`
- `git reset --hard`

If any matches, it fires a **`critical`** severity event which locks Shikigami into a prominent warning state, bypassing all texture modifiers and dampening.

## Data Flow

```
Claude Code tool event
    ↓ stdin (JSON from Claude Code)
hooks/shikigami-hook.py
    ↓ transforms to Shikigami EventPayload v1.0
    ↓ reads bearer token from ~/.shikigami/token
    ↓ reads port from ~/.shikigami/config.json
    ↓ POST http://127.0.0.1:<port>/v1/events
Shikigami app event server
    ↓ validate schema + auth + dampen
    ↓ state machine resolves ResolvedState
    ↓ Tauri 'state_changed' event
Frontend / renderer
```

## Environment Overrides

Useful for dev / testing:

| Variable          | Effect                                                    |
|-------------------|-----------------------------------------------------------|
| `SHIKIGAMI_HOME`  | Override `~/.shikigami` (token + config location)         |
| `SHIKIGAMI_URL`   | Full event URL — skips port lookup, useful for tunneling  |
| `SHIKIGAMI_DEBUG` | `=1` writes transform + POST errors to stderr             |

## Failure Modes

The hook is designed to **never block Claude Code**:

- Shikigami not running → silent skip
- Token missing → silent skip
- Network error → silent skip (exit 0)
- Malformed Claude Code payload → silent skip

Enable `SHIKIGAMI_DEBUG=1` to surface the reasons in stderr.

## Why Python?

We chose Python 3 over POSIX shell + jq for this script because:

- macOS ships Python 3 by default; no extra install required
- stdin JSON parsing + urllib POST are stdlib-only
- Destructive-pattern detection uses regex cleanly
- Easier to maintain for community contributors

A Rust CLI replacement ships with the `shikigami` binary once the CLI crate is in place (tracked in issue #12).

## Manual Hook Invocation (testing)

```bash
# Simulate a tool_start
echo '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}' \
  | python3 hooks/shikigami-hook.py

# Simulate a destructive op (should fire critical)
echo '{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /tmp/foo"}}' \
  | SHIKIGAMI_DEBUG=1 python3 hooks/shikigami-hook.py

# Simulate a failed tool
echo '{"hook_event_name":"PostToolUse","tool_name":"Bash","tool_response":{"exit_code":1,"stderr":"boom"}}' \
  | python3 hooks/shikigami-hook.py
```

With Shikigami running and hooks installed, you should see the character animate accordingly.
