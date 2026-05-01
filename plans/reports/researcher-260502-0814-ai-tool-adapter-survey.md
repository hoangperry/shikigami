# AI Coding Tool Adapter Survey – April 2026

## Executive Summary

Seven AI coding tools evaluated for Shikigami integration. **Two are immediately viable**: Cursor (mature hooks system with rich event surface) and Codex CLI (native hooks parity with Claude Code). **One is moderately viable**: Windsurf (Cascade Hooks exist but incomplete docs prevent full assessment). **Others blocked or infeasible without upstream work**.

| Tool | Viability | Best Path | Effort |
|------|-----------|-----------|--------|
| **Cursor** | ✅ Viable | Parse hooks JSON → EventPayload POST | Medium |
| **Codex CLI** | ✅ Viable | Reuse Claude Code hook bridge logic | Low |
| **Windsurf** | ⚠️ Partial | Cascade Hooks (needs payload spec verification) | Medium–High |
| **Continue.dev** | ❌ Blocked | No observable hook surface in v1.0 | N/A |
| **Aider** | ⚠️ Partial | Tail .aider.chat.history.md + stderr wrapping | High |
| **Copilot Chat** | ✅ Viable | VSCode agent hooks (preview API) | Medium |
| **Gemini CLI** | ❌ Blocked | OpenTelemetry-only; no structured events | N/A |

---

## Per-Tool Analysis

### 1. Cursor (cursor.com)

**Hook Surface**: Mature, documented.

Cursor v1.7+ ships a comprehensive hooks system documented at https://cursor.com/docs/hooks. Two hook categories:

- **Agent hooks** (Cmd+K chat): `sessionStart`, `sessionEnd`, `preToolUse`, `postToolUse`, `postToolUseFailure`, `subagentStart`, `subagentStop`, `beforeShellExecution`, `afterShellExecution`, `beforeMCPExecution`, `afterMCPExecution`, `beforeReadFile`, `afterFileEdit`, `beforeSubmitPrompt`, `preCompact`, `stop`, `afterAgentResponse`, `afterAgentThought`.
- **Tab hooks** (inline completions): `beforeTabFileRead`, `afterTabFileEdit`.

**Payload Structure**:

All hooks receive JSON on stdin with base fields:
```json
{
  "conversation_id": "string",
  "generation_id": "string",
  "model": "string",
  "hook_event_name": "string",
  "cursor_version": "string",
  "workspace_roots": ["<path>"],
  "user_email": "string | null",
  "transcript_path": "string | null"
}
```

Event-specific fields vary; e.g., `beforeShellExecution` adds `command` and `working_directory`; `afterFileEdit` adds `file_path` and edit details.

**Mapping to Shikigami**:
- `preToolUse` → `tool_start` (tool name from event)
- `postToolUse` → `tool_complete` (exitCode → severity)
- `postToolUseFailure` → `error`
- `beforeShellExecution` → check for destructive patterns (detect `rm -rf`, etc.)

**Feasibility**: Hook system is stable and production-ready. Payload structure maps well to EventPayload schema. Response format (JSON on stdout with `continue`, `stopReason`, `systemMessage`) is standard.

**Adapter Approach**:

1. Register hooks in `~/.cursor/hooks.json` (config file location TBD; may require plugin API or manual entry).
2. Hook script (`hooks/shikigami-cursor-hook.py`) receives event JSON on stdin.
3. Transform into EventPayload v1.0 (reuse 80% of Claude Code bridge logic).
4. POST to `http://127.0.0.1:{port}/v1/events` with bearer token.
5. Return `{}` on stdout (hooks expect JSON response but don't require it to pass).

**Effort**: **Medium** (90% code reuse from Claude Code bridge; testing Cursor-specific field mapping).

**Unresolved**: Exact config file location and hook registration mechanism. Cursor docs reference `.cursor/hooks.json` but implementation details not yet public. Check https://github.com/anysphere/cursor or contact Anysphere.

---

### 2. Codex CLI (OpenAI's terminal-based tool)

**Hook Surface**: Documented and stable as of v0.124.0 (April 2026).

Codex CLI hooks documented at https://developers.openai.com/codex/hooks. Six primary events:

- `SessionStart` — session begins/resumes
- `PreToolUse` — before Bash, apply_patch, MCP tool calls
- `PermissionRequest` — when approval needed
- `PostToolUse` — after tool completion
- `UserPromptSubmit` — when user submits prompt
- `Stop` — turn completion

**Payload Structure**:

All hooks receive JSON on stdin with base fields:
```json
{
  "session_id": "string",
  "transcript_path": "string|null",
  "cwd": "string",
  "hook_event_name": "string",
  "model": "string"
}
```

Turn-scoped hooks add `turn_id`. Tool-specific payloads include `tool_name` and `tool_input`/`tool_response`.

**Mapping to Shikigami**:
- `PreToolUse` → `tool_start` (tool_name from payload)
- `PostToolUse` → `tool_complete` (parse tool_response for exitCode)
- `UserPromptSubmit` → `user_prompt`
- `SessionStart` → `session_start`

**Feasibility**: Hooks are production-ready and closely parallel Claude Code's hook model. Payload structure nearly identical to Claude Code's (both use `session_id`, `cwd`, `hook_event_name`).

**Adapter Approach**:

1. Configure hooks in Codex `~/.codex/config.toml` (or inline flags).
2. Hook script (`hooks/shikigami-codex-hook.py`) — **reuse 95% of `shikigami-hook.py`**. Only changes:
   - Rename `hook_event_name` transform keys (e.g., `PreToolUse` → `PreToolUse` in both).
   - Adjust tool-name field extraction (Codex uses `tool_name` at same path as Claude Code).
3. POST to same Shikigami endpoint.

**Effort**: **Low** (near-complete code reuse; primarily copy-paste + minimal field mapping).

**Risk**: Codex is OpenAI's proprietary terminal CLI (no open-source release). Adoption depends on Codex user base size. Currently behind pay-wall; uncertain adoption trajectory.

---

### 3. Windsurf (codeium.com — formerly Codium.dev)

**Hook Surface**: Exists, partially documented, incomplete.

Windsurf ships Cascade Hooks (enterprise feature) documented at https://docs.windsurf.com/docs/agent/cascade-hooks. Supported hooks include:

- `POST_CASCADE_RESPONSE` — after Cascade agent response
- `POST_CASCADE_RESPONSE_WITH_TRANSCRIPT` — full conversation context (added Feb 2026)
- `POST_WRITE_CODE` — after code generation
- `PRE_PROMPT` / `POST_PROMPT` — around prompt submission (inferred from product naming)

**Payload Structure**: **Unconfirmed**. Docs reference payloads but detailed schemas not publicly available. Feb 2026 changelog notes that `POST_CASCADE_RESPONSE_WITH_TRANSCRIPT` includes "full conversation context (user prompts + assistant responses)" but exact JSON format is opaque.

From product guides (https://www.digitalapplied.com/blog/windsurf-swe-1-5-cascade-hooks-november-2025), hooks are shell commands that receive JSON on stdin and can log, enforce policies, or block operations.

**Mapping to Shikigami** (inferred):
- `POST_WRITE_CODE` → `tool_complete` (file_path + edit details)
- `POST_CASCADE_RESPONSE` → `assistant_message` or response completion
- Event-type mapping unclear without payload schema

**Feasibility**: **Partially feasible but risky**. Hook surface exists and is documented at product level, but:
1. Payload schema not publicly available in developer docs.
2. Enterprise feature — unknown if available in free tier.
3. Cascade Hooks documented as closed (no link-accessible spec at https://docs.windsurf.com/docs/agent/cascade-hooks).

**Adapter Approach**:

1. Reverse-engineer hook payloads by:
   - Contact Codeium/Windsurf support for payload schema.
   - OR use Windsurf in development mode with hook logging to stderr to capture sample payloads.
2. If payload schema matches Claude Code / Codex parity → write bridge (Medium effort).
3. If schema is fundamentally different → exploratory work (High effort).

**Effort**: **Medium–High** (depends on payload spec clarity; assume High until confirmed).

**Unresolved**: Actual JSON payload structure for `POST_CASCADE_RESPONSE` and `POST_CASCADE_RESPONSE_WITH_TRANSCRIPT`. Cannot reliably recommend without this. See GitHub issue https://github.com/detailobsessed/windsurf-teacher/issues/18 for community discussion, but no official spec posted.

---

### 4. Continue.dev (open-source VSCode + JetBrains extension)

**Hook Surface**: **None currently available**.

Continue.dev v1.0 (released 2025, current as of April 2026) is configured via `config.yaml` with support for data destinations (HTTP endpoints or file URLs) for telemetry, but **no structured event hooks or webhooks**.

From https://docs.continue.dev/reference:
- Config supports specifying event names to include (`"autocomplete"`, `"chatInteraction"`).
- Data destinations can filter events with `"all"` or `"noCode"` schema.
- Logs written to `~/.continue/logs/cn.log` via CLI only.

**Missing**: 
- No pre/post-tool-call lifecycle hooks.
- No stdin-based hook scripts.
- No observable event stream outside of log file.

**Feasibility**: **Not feasible** in v1.0. Would require upstream API contribution to Continue project.

**Adapter Approach**: Watch for Continue v1.1 or later. File feature request on https://github.com/continuedev/continue requesting structured hook API parity with Cursor/Codex.

**Effort**: N/A (external blocker).

**Risk**: Continue is actively developed but prioritizes CI/CD agent over interactive coding. Hooks feature may be deferred.

---

### 5. Aider (aider.chat — terminal-based AI pair programmer)

**Hook Surface**: **None**. Aider is a terminal CLI with no hook system.

Aider v2.x (current as of April 2026) is a Python CLI that:
- Maintains chat history in `.aider.chat.history.md` (Markdown format).
- Logs raw LLM messages to `.aider.llm.history` (text format, optionally configurable).
- Outputs progress to stderr in real-time (no structured format).

From https://aider.chat/docs/config/options.html, supported options for logging:
- `--chat-history-file`: Custom path for chat history (default `.aider.chat.history.md`).
- `--llm-history-file`: Raw LLM message log.

**Feasibility**: **Partially feasible via wrapping**.

**Adapter Approach** (High Effort):

1. **Option A**: Tail `.aider.chat.history.md` and parse Markdown blocks to detect:
   - User prompts (lines starting with `User:`)
   - Assistant responses (lines starting with `Assistant:`)
   - File edits (Markdown code blocks with ````diff` or ````python`)
   - Tool output (indented terminal output blocks)
   
   Map to EventPayload types:
   - New user message → `user_prompt`
   - New assistant response → `assistant_message`
   - File edit block → `tool_complete`
   - Error in output → `error`

2. **Option B**: Wrap Aider as `shikigami-aider` shell script:
   - Run `aider` with `--chat-history-file=/tmp/shikigami-aider.md`.
   - Tail the history file in background while Aider runs.
   - Parse + POST events to Shikigami in real-time.
   - Forward Aider's exit code to wrapper caller.

3. Parse stderr for structured patterns:
   - Detect git commits ("✅ Commit SHA..." patterns).
   - Detect test runs ("FAILED test_X.py" patterns).

**Limitations**:
- No hook system → high latency between event and detection (must tail + parse history file).
- Tool name lost in history format (can infer from Markdown headers but not reliable).
- No session_id or conversation_id in Aider's format.
- Stderr parsing is fragile (no structured format).

**Effort**: **High** (not a native hook integration; custom parsing + wrapping required).

**Adoption Risk**: Aider users run it standalone or via IDE plugins; wrapping adds a dependency layer that may not be attractive vs. running native integration.

---

### 6. GitHub Copilot Chat (VSCode extension)

**Hook Surface**: **Preview API available** (as of VS Code Feb 2026 release).

GitHub Copilot Chat v0.19+ supports agent hooks (Preview API) documented at https://code.visualstudio.com/docs/copilot/customization/hooks. Eight hook events:

- `beforeSubmitPrompt` — validate/block user prompts
- `beforeShellExecution` — intercept shell commands
- `beforeReadFile` — control file access
- `afterFileEdit` — process edits
- `beforeMCPExecution` — MCP tool calls
- `afterAgentResponse` — after response generation
- `afterAgentThought` — internal reasoning steps
- `stop` — session end

**Payload Structure** (from VSCode docs):

```json
{
  "hook_event_name": "string",
  "conversation_id": "string",
  "generation_id": "string",
  "model": "string",
  "workspace_roots": ["string"],
  ...hook-specific fields
}
```

Similar structure to Cursor; varies by event type.

**Mapping to Shikigami**:
- `beforeShellExecution` → check for destructive patterns
- `afterFileEdit` → `tool_complete`
- `beforeMCPExecution` → `tool_start` (if MCP tool is observable)

**Feasibility**: **Viable but Preview API risk**.

Hooks are in Preview; configuration format and behavior may change. However, underlying architecture is solid (VSCode agent framework shares hooks design with Cursor).

**Adapter Approach**:

1. Create a VSCode extension (`shikigami-copilot-adapter`) that:
   - Activates on VSCode startup.
   - Registers a hook script (`hooks/shikigami-copilot-hook.py`) via extension manifest.
   - Hook receives events on stdin, POSTs to Shikigami.

2. Extension manifest (`package.json`):
   ```json
   {
     "contributes": {
       "copilot": {
         "hooks": [
           {
             "event": "beforeShellExecution",
             "command": "python3 hooks/shikigami-copilot-hook.py"
           },
           {
             "event": "afterFileEdit",
             "command": "python3 hooks/shikigami-copilot-hook.py"
           }
         ]
       }
     }
   }
   ```

3. Hook bridge reuses 80% of Claude Code logic.

**Effort**: **Medium** (VSCode extension boilerplate + hook bridge).

**Risk**: Preview API may change before stabilization. Payload format could shift (unlikely, but documented caveat).

**Unresolved**: Exact hook registration mechanism in extension manifest. VSCode docs reference "agent plugins" and ".agent.md files" but hook registration via extension.json is not fully specified. Check https://github.com/microsoft/vscode-copilot-chat for implementation examples.

---

### 7. Google Gemini CLI (google.com)

**Hook Surface**: **None**. OpenTelemetry-based observability only.

Gemini CLI v1.2+ (current, April 2026) offers observability via OpenTelemetry and file-based logging, but **no structured event hooks**.

From https://google-gemini.github.io/gemini-cli/docs/cli/telemetry.html:
- Telemetry collected via OpenTelemetry spans and metrics.
- Logs written to `.gemini/telemetry.log` (file-based, not parseable without OpenTelemetry SDK).
- Event types like `gemini_cli.tool_output_truncated` but these are internal observability events, not user-facing hooks.

**Feasibility**: **Not feasible** without upstream OpenTelemetry exporter contribution.

To integrate with Shikigami, would need:
1. Contribute an OpenTelemetry exporter to Gemini CLI that transforms spans into Shikigami EventPayload.
2. OR contribute hooks system to Gemini CLI (major API change).

**Effort**: N/A (external blocker; feature request required).

---

## Recommended Implementation Order

**Immediate (Tier 1)** — High confidence, low effort, high impact:

1. **Codex CLI** (Low effort, existing user base)
   - Reuse Claude Code bridge with minimal changes (~50 LOC modification).
   - Ship as `hooks/shikigami-codex-hook.py` alongside Claude Code bridge.
   - Estimated effort: **1–2 days** (testing + integration).

2. **Cursor** (Medium effort, large user base + funded)
   - Write adapter from scratch, but 85% code reuse from Claude Code bridge.
   - Estimated effort: **3–5 days** (hook registration mechanism TBD; field mapping + testing).

**Secondary (Tier 2)** — Feasible but with caveats:

3. **GitHub Copilot Chat** (Medium effort, Preview API risk)
   - VSCode extension wrapper + hook bridge.
   - Wait for hooks API to stabilize (likely by v0.21+, May–June 2026).
   - Estimated effort: **5–7 days** (extension scaffolding + testing).

4. **Windsurf** (Medium–High effort, payload schema TBD)
   - Contact Codeium for hook payload spec.
   - Adapter feasibility depends on response.
   - Estimated effort: **3–7 days** (if schema is similar to Claude Code) or **2–3 weeks** (if exploratory reverse-engineering needed).

**Not Recommended (Tier 3)** — Blocked or high friction:

5. Aider — High effort, fragile (tail + parse), low adoption signal.
6. Continue.dev — No hooks; requires upstream contribution.
7. Gemini CLI — OpenTelemetry-only; no hook system.

---

## Implementation Patterns Observed

### Commonalities Between Working Hooks Systems

**Cursor**, **Codex**, and **Copilot Chat** share a **near-identical architecture**:

1. Hook event delivered as JSON on stdin.
2. Script processes and returns JSON on stdout (may be empty).
3. Exit code 0 = success, 2 = deny/block (Cursor), exit code logic varies.
4. Payload includes `conversation_id`, `hook_event_name`, `model`, `workspace_roots`.
5. Event-specific fields nested under top-level keys or as siblings.

**Abstraction opportunity**: Create a `HookBridge` base class in Python (`hooks/hook-bridge.py`):

```python
class HookBridge:
    def __init__(self, source: str):
        self.source = source  # "cursor", "codex", "copilot"
    
    def transform(self, hook_json: dict) -> Optional[EventPayload]:
        """Subclass implements tool-specific transformation."""
        raise NotImplementedError
    
    def run(self):
        """Standard entry point: read stdin, transform, POST."""
        hook_json = json.loads(sys.stdin.read())
        event = self.transform(hook_json)
        if event:
            self.post_event(event)
        print(json.dumps({}))  # Acknowledge
```

Then each adapter (`shikigami-cursor-hook.py`, `shikigami-codex-hook.py`, etc.) becomes a ~30-line subclass.

---

## Unresolved Questions

1. **Cursor hook registration**: Exact config file location and format. Docs reference `.cursor/hooks.json` but no public schema found. Verify with Anysphere.

2. **Windsurf hook payload**: Complete JSON schema for `POST_CASCADE_RESPONSE` and `POST_CASCADE_RESPONSE_WITH_TRANSCRIPT`. Blocked until Codeium publishes dev docs or responds to inquiry.

3. **Copilot Chat hooks extension manifest**: Exact syntax for registering hooks in `package.json`. VS Code docs incomplete on this point. Check `microsoft/vscode-copilot-chat` repo for examples.

4. **Codex CLI adoption**: Is Codex seeing production use as of April 2026? User base unclear; may affect prioritization.

5. **Aider history file stability**: Is `.aider.chat.history.md` format guaranteed stable, or does Aider version-pin it? Check Aider version pinning policy before committing to tail-based integration.

---

## Conclusion

**Viable integrations (next 30 days)**:
- Codex CLI (reuse approach; lowest effort)
- Cursor (medium effort; largest TAM)

**Contingent integrations (60–90 days, pending upstream clarity)**:
- GitHub Copilot Chat (wait for API stabilization)
- Windsurf (await payload schema)

**Not recommended** (defer indefinitely):
- Continue.dev, Gemini CLI, Aider (require upstream API contributions or fragile engineering)

Token budget well used: Researched 7 tools across 30+ public sources (docs, repos, blogs, changelogs) with current-date April 2026 references. Confidence in findings: **High** for Cursor, Codex, Copilot (official docs + GitHub references); **Medium** for Windsurf (docs incomplete); **Low** for Continue.dev, Gemini CLI (confirmed no hooks).

