# Shikigami — Event Processing Pipeline

> **Status**: v0.1 · **Last Updated**: 2026-04-22
> **Scope**: Narrates the single end-to-end data flow from a Claude Code hook firing to a sprite frame rendering. Describes each stage's responsibility, inputs, outputs, and boundaries.
> **Reads with**: `docs/PRD.md`, `docs/TDD.md` §4–§7, `docs/adr/002-signal-source.md`
> **Naming conventions** inspired by airi's `core-character` pipeline (see `docs/research/260422-airi-reusability-analysis.md` takeaway #6). Implementation is Rust-native and differs from airi's Vue+TS stack.

---

## 1. Why a Pipeline Doc?

The product's behaviour is defined by the contract *"what the character does when X happens"*. That contract spans five codebases: Claude Code hooks → Python bridge → Rust HTTP server → Rust state machine → React+PixiJS renderer. Each stage is legitimate abstraction, not arbitrary boilerplate.

This document makes the stages explicit so that:

- a bug report can name the stage at fault
- a contributor can replace one stage without touching adjacent ones
- new AI tool adapters (Cursor, Windsurf) only need to target the **ingest** stage

---

## 2. Pipeline at a Glance

```
╔══════════╗    ╔══════════╗    ╔══════════╗    ╔══════════╗    ╔══════════╗    ╔══════════╗
║  Hook    ║ ─▶ ║  Bridge  ║ ─▶ ║ Ingest   ║ ─▶ ║ Segment  ║ ─▶ ║ Resolve  ║ ─▶ ║  Emit    ║
║          ║    ║          ║    ║          ║    ║          ║    ║          ║    ║          ║
║ Claude   ║    ║ Python   ║    ║ axum +   ║    ║ Auth +   ║    ║ Hierar-  ║    ║ Tauri    ║
║ Code     ║    ║ trans-   ║    ║ schema + ║    ║ validate ║    ║ chical   ║    ║ event +  ║
║ Hook     ║    ║ former   ║    ║ rate-    ║    ║ + dampen ║    ║ Fusion   ║    ║ Frontend ║
║          ║    ║          ║    ║ limit    ║    ║ filter   ║    ║ Machine  ║    ║ listener ║
╚══════════╝    ╚══════════╝    ╚══════════╝    ╚══════════╝    ╚══════════╝    ╚══════════╝
    CC            Py                Rust             Rust             Rust            React

    ▲                                                                                   │
    └─────────────────────────────── Character reacts ◀────────────────────────────────┘
```

The process boundary is between **Bridge** and **Ingest**: HTTP POST over `127.0.0.1`. Everything to the left runs inside the Claude Code process; everything to the right is the Shikigami Tauri app.

---

## 3. Stages

### 3.1 Hook

**Where**: Claude Code internal, not our code.
**Input**: internal tool-lifecycle events (`PreToolUse`, `PostToolUse`, `Stop`, `UserPromptSubmit`, `SessionStart`).
**Output**: Claude Code hook JSON on stdin of our configured command.
**Contract**: whatever Claude Code ships. We do not control it; we observe it.

Out of scope. Documented so the pipeline has a visible origin.

---

### 3.2 Bridge

**Where**: `hooks/shikigami-hook.py`
**Input**: Claude Code hook JSON on stdin
**Output**: Shikigami `EventPayload v1.0` HTTP POST to `127.0.0.1:<port>/v1/events`
**Schema source of truth**: `schemas/event.v1.0.json`

Responsibilities:

- Read and parse Claude Code hook JSON
- Map `hook_event_name` → Shikigami `type` enum
- Detect destructive patterns on `Bash` commands (`rm -rf`, `DROP TABLE`, `git push --force`, `git reset --hard`) and upgrade severity to `critical`
- Read bearer token from `~/.shikigami/token`
- Read port from `~/.shikigami/config.json`
- POST with `Authorization: Bearer <token>` header
- **Never block** Claude Code — swallow all errors, return exit 0

Boundary: this is the **only** Claude-Code-specific code in the system. A Cursor adapter would reimplement exactly this stage with Cursor semantics; everything downstream is unchanged.

---

### 3.3 Ingest

**Where**: `src-tauri/src/event/server.rs` → `ingest_event` handler (axum)
**Input**: HTTP POST at `/v1/events`
**Output**: a validated `EventPayload` ready for the segment stage
**Status codes**:

| Status | Meaning |
|--------|---------|
| `401` | Missing or invalid bearer token |
| `400` | JSON parse failure or schema-version mismatch |
| `202` | Accepted (may still be dropped by dampener downstream) |
| `503` | Rate-limit hard cap reached |

Responsibilities:

- Bind to `127.0.0.1` only; never external
- Port conflict recovery: scan `preferred..preferred+10`, persist chosen port to settings
- Constant-time bearer token comparison (`subtle::ConstantTimeEq`)
- `serde(deny_unknown_fields)` deserialization against `EventPayload`
- `validate_version()` ensures `schemaVersion == "1.0"`
- Log + emit telemetry (tracing spans)

Boundary: at this point the event is an in-memory `EventPayload` struct with types checked. Nothing downstream sees raw JSON.

---

### 3.4 Segment

**Where**: `src-tauri/src/state/dampen.rs` + early portion of `server::ingest_event`
**Input**: `EventPayload`
**Output**: the same `EventPayload`, or a silent drop

Responsibilities:

- **Dampening**: 2 s sliding window dedup on `(event_type, severity)`. Duplicates within window drop silently (log at `trace`). `Severity::Critical` always bypasses.
- **Rate limiting**: soft cap 50 events/s (warn + drop), hard cap 500/s (503 to caller)

Boundary: this is the layer that prevents the "toxic loop strobe" failure mode. It does **not** understand emotion semantics — it only filters by shape.

Named after airi's `segmentation` stage, though our segmentation is temporal (dedup window), not semantic (sentence splitting).

---

### 3.5 Resolve

**Where**: `src-tauri/src/state/machine.rs::resolve` + `src-tauri/src/state/texture.rs::extract`
**Input**: `EventPayload`
**Output**: `ResolvedState { dominant, texture, severity, duration_ms, event_id }`

This is the **Hierarchical Fusion** stage defined in ADR-002. Two sub-steps:

**Resolve Stage 1 — Dominant** (`machine.rs::map_event`)
A table-driven mapping from `(EventType, exit_code)` to `DominantState`. For example:

```
(ToolComplete,       Some(0))  → Happy
(ToolComplete,       Some(_))  → Warning
(DestructiveOpDetected, _)     → Warning
(SessionIdleLong,    _)        → Sleepy
(UserPrompt,         _)        → Focused
```

**Resolve Stage 2 — Texture** (`texture.rs::extract`)
If `severity == Critical`, **skipped** (preserves the "no smiling during `rm -rf`" invariant). Otherwise, `event.text` is matched against a pre-compiled regex table with first-match-wins ordering:

```
finally | phew             → Relieved
⚠ | dangerous | careful    → Alarmed
*action text*              → Cute
told you | ( ˶ˆᗜˆ˵ )       → Smug
```

**Post-resolve** — `severity_scale(base, sev)` applies duration multipliers (`1×/1.5×/2×/3×`) with minimum floors for `Error`/`Critical`.

Output animation key: `<dominant>[_<texture>]`, e.g. `happy_relieved`, `warning`, `focused_alarmed`.

Boundary: the first stage that knows anything about emotion. Replacing it is how you'd swap the signal strategy (e.g., future FR-F11 LLM sidecar sidecar would swap this module's implementation, not the pipeline around it).

---

### 3.6 Emit

**Where**: `src-tauri/src/lib.rs` (Tauri `Emitter`) → `src/App.tsx` (React listener)
**Input**: `ResolvedState`
**Output**: `state_changed` Tauri event in the WebView; React state update

Responsibilities:

- Backend: `app.emit("state_changed", &resolved)` — typed serialization via serde+Tauri
- Frontend: `listen<ResolvedState>("state_changed", ...)` in a React effect; Zustand store mirrors the latest value
- Frontend: Phase 2 will translate `animation_key` into a sprite animation on PixiJS canvas

Boundary: this is the process/language boundary between Rust and the WebView. Everything downstream is pure frontend.

---

### 3.7 Render *(Phase 2, not shipped in v0.1 alpha)*

**Where**: `src/renderer/SpriteRenderer.ts` (planned)
**Input**: `animation_key` from the emitted state
**Output**: PixiJS canvas paint

Deferred per TDD §7 + issue #16. Listed here so the pipeline is complete.

---

## 4. Invariants

These hold across the pipeline regardless of stage implementation:

1. **Events are opinionless**. The pipeline has no knowledge of which AI tool produced an event once past the Bridge stage. Cursor and Claude Code events are indistinguishable to Resolve.
2. **Critical severity is sticky within a single event**. Once classified Critical, texture is suppressed, dampening is bypassed, and the resolved state always wins against softer concurrent events until its duration elapses.
3. **No backpressure on the hook**. The Bridge returns immediately; the Tauri app pulls at its own pace. Events that cannot be accepted in time are dropped, never queued indefinitely.
4. **Idempotency of dampening**. Running the same event twice within the window has the same observable effect as running it once.
5. **Graceful degradation per stage**. Each stage defines its failure mode (Bridge fails silent; Ingest returns HTTP status; Resolve defaults to `Idle` on unknown event types; Emit logs on Tauri failure).

---

## 5. Extension Points

| Want to add... | Modify only |
|----------------|-------------|
| Cursor / Windsurf support | **Bridge** — write a new `hooks/cursor-hook.py` that speaks the same `EventPayload v1.0` |
| A new emotion state | **Resolve** (`state/canonical.rs` + `machine.rs` mapping table) + character manifest schema |
| A new texture modifier | **Resolve** (`state/texture.rs` regex + `canonical.rs` enum) + character asset optional |
| Replace regex with LLM classifier (FR-F11) | **Resolve** — swap `texture::extract` or wrap with a sidecar call; upstream and downstream stages unchanged |
| Different event transport (e.g., Unix socket) | **Ingest** + **Bridge** — add a second endpoint, keep HTTP POST as default |
| Event logging / metrics export | Add a tap after **Segment** or in **Emit**; no core-stage changes |

This is why the pipeline is five stages instead of one fat function.

---

## 6. Performance Budgets

Per-event budget as a hard ceiling:

| Stage | Budget | Enforced by |
|-------|-------:|-------------|
| Bridge (Python transform + curl) | 50 ms wall-clock | `curl --max-time 1` (fail fast) |
| Ingest (axum parse + auth) | 2 ms CPU (p99) | integration tests future |
| Segment (dampen lookup) | 50 µs CPU (p99) | unit test + criterion bench future |
| Resolve (mapping + regex) | 500 µs CPU (p99) | criterion bench future |
| Emit (Tauri IPC serialize) | 1 ms wall-clock | Tauri overhead, not ours to optimize |
| **Total CPU** | **≤ 5 ms per event** | CI perf gate (TBD) |

Idle RAM when no events arrive: <80 MB (Tauri + WebView baseline + Zustand store + regex caches).

---

## 7. Observability

Each stage emits structured tracing at defined levels:

- `Bridge` — `SHIKIGAMI_DEBUG=1` → stderr
- `Ingest` — `info` for 202s, `warn` for 4xx/5xx
- `Segment` — `trace` for dampened drops, `warn` for rate-limit drops
- `Resolve` — `info` for each resolved state (with animation_key)
- `Emit` — `warn` on Tauri emit failure

Logs land in `~/.shikigami/logs/shikigami.log` (planned daily rotation). Env override: `SHIKIGAMI_LOG=debug`.

---

## 8. References

- `docs/adr/001-event-transport.md` — Ingest design
- `docs/adr/002-signal-source.md` — Resolve design (Hierarchical Fusion)
- `docs/TDD.md` §4–§7 — Implementation specifications
- `schemas/event.v1.0.json` — Bridge/Ingest contract
- `docs/research/260422-airi-reusability-analysis.md` — Pipeline naming inspiration

---

*"Stages have single responsibilities. Boundaries carry contracts. Replace a stage, keep the pipeline."*
