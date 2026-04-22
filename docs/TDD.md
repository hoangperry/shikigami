# Shikigami — Technical Design Document

> **Status**: v0.1 Draft · **Last Updated**: 2026-04-22 · **Owner**: @hoangperry
> **Scope**: Maps PRD v0.2 requirements to concrete modules, interfaces, data types, algorithms, and test strategy for v0.1 (macOS / sprite / Claude Code only).
> **Audience**: Implementers (human or AI), including Codex for the character-asset task.
> **Non-Goals**: This is not a tutorial. Reader is assumed to know Rust, TypeScript, React, and general game/UI patterns.

---

## 1. Top-Level Architecture

### 1.1 Process Model

Shikigami is a single Tauri 2 application with:

- **Rust backend** — event HTTP server, state machine, file watchers, window chrome, tray, IPC
- **WebView frontend** — React + PixiJS for character rendering, speech bubble, settings UI

One OS process. No background daemons. No sidecars in v0.1. (The LLM classifier from FR-F11 will be a sidecar when it ships, v2+.)

```
┌──────────────────────────────────────────────────────────────────┐
│  Tauri App Process (shikigami.app)                               │
│                                                                  │
│  ┌────────────────────────┐  ┌────────────────────────────────┐  │
│  │  Rust Backend          │  │  WebView Frontend              │  │
│  │  (tokio / axum)        │  │  (React + PixiJS)              │  │
│  │                        │  │                                │  │
│  │  • HTTP Event Server   │◄─┤  • Renderer canvas             │  │
│  │  • State Machine       │──►  • Speech bubble overlay       │  │
│  │  • Character FS loader │  │  • Settings modal              │  │
│  │  • Window manager      │  │  • Character library grid      │  │
│  │  • Tray menu           │  │                                │  │
│  └────────────────────────┘  └────────────────────────────────┘  │
│            ▲                                                     │
│            │ Tauri IPC (bidirectional)                           │
└────────────┼─────────────────────────────────────────────────────┘
             │
             │ HTTP POST /v1/events
             │ Authorization: Bearer <token>
             ▼
┌──────────────────────────────────────────────────────────────────┐
│  Claude Code                                                     │
│  ~/.claude/settings.json hooks → curl bridge script              │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 Module Boundaries

| Module | Language | Responsibility | Touches |
|--------|----------|---------------|---------|
| `event_server` | Rust | HTTP POST endpoint, auth, payload validation | → `state_machine` |
| `state_machine` | Rust | Dominant state + texture, dampening, severity scaling | → IPC emit to frontend |
| `transport_config` | Rust | Port discovery, token generation, recovery | → disk |
| `window_chrome` | Rust | Transparent overlay, always-on-top, click-through | → OS window APIs |
| `character_fs` | Rust | Install/uninstall, hot reload, zip unpack, manifest validate | → disk, IPC emit |
| `tray_menu` | Rust | System tray controls | → IPC emit |
| `renderer` | TS | Sprite animation, state transitions, texture composition | ← IPC state, → canvas |
| `speech_bubble` | TS | Contextual info overlay | ← IPC state |
| `settings_ui` | TS | Settings modal | ↔ IPC persist |
| `library_ui` | TS | Character grid, install drag-drop | ↔ IPC character_fs |
| `emotion_parser` | Rust | Text texture regex extraction | ← state_machine |

---

## 2. Folder Structure

```
shikigami/
├── Cargo.toml                          (workspace root — Rust)
├── package.json                        (frontend dependencies)
├── vite.config.ts
├── tsconfig.json
├── README.md
├── LICENSE                             (MIT)
│
├── src-tauri/                          (Rust backend)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs                     (app entry, plumb modules)
│       ├── lib.rs
│       ├── event/
│       │   ├── mod.rs
│       │   ├── server.rs               (axum HTTP server)
│       │   ├── schema.rs               (EventPayload + validation)
│       │   └── auth.rs                 (bearer token)
│       ├── state/
│       │   ├── mod.rs
│       │   ├── machine.rs              (event → state transitions)
│       │   ├── texture.rs              (texture regex extraction)
│       │   ├── dampen.rs               (2s dedup window)
│       │   └── canonical.rs            (state + texture enums)
│       ├── character/
│       │   ├── mod.rs
│       │   ├── loader.rs               (zip unpack, manifest parse)
│       │   ├── manifest.rs             (schema v1.0)
│       │   ├── watcher.rs              (hot reload)
│       │   └── atlas.rs                (optional sprite atlas build)
│       ├── window/
│       │   ├── mod.rs
│       │   ├── overlay.rs              (transparent + always-on-top)
│       │   ├── position.rs             (persistence + recovery)
│       │   └── screen_capture.rs       (OBS/Zoom detection)
│       ├── tray/
│       │   └── menu.rs
│       ├── ipc/
│       │   ├── mod.rs
│       │   └── commands.rs             (Tauri command handlers)
│       ├── config/
│       │   ├── mod.rs
│       │   ├── settings.rs
│       │   └── paths.rs                (OS-standard dirs)
│       └── errors.rs
│
├── src/                                (React + TS frontend)
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── Character.tsx               (PixiJS canvas)
│   │   ├── SpeechBubble.tsx
│   │   ├── SettingsModal.tsx
│   │   ├── CharacterLibrary.tsx
│   │   └── DebugOverlay.tsx
│   ├── renderer/
│   │   ├── SpriteRenderer.ts           (PixiJS implementation)
│   │   ├── AnimationLoader.ts          (frame sequence loading)
│   │   ├── TransitionEngine.ts         (crossfade / instant)
│   │   └── types.ts
│   ├── state/
│   │   ├── emotion.ts                  (Zustand store, mirrors backend)
│   │   ├── character.ts
│   │   └── settings.ts
│   ├── ipc/
│   │   ├── commands.ts                 (typed wrappers over Tauri invoke)
│   │   └── events.ts                   (Tauri event subscriptions)
│   └── styles/
│       └── global.css
│
├── hooks/                              (Claude Code integration)
│   ├── shikigami-hook.sh               (POSIX)
│   ├── shikigami-hook.ps1              (Windows, for v0.2)
│   └── README.md
│
├── cli/                                (separate Rust binary: shikigami CLI)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── commands/
│           ├── install.rs              (install character)
│           ├── pack.rs                 (pack .shikigami)
│           ├── validate.rs             (validate manifest)
│           ├── install_hook.rs         (modify ~/.claude/settings.json)
│           └── doctor.rs               (diagnostics)
│
├── schemas/                            (JSON Schemas — source of truth)
│   ├── manifest.v1.0.json
│   └── event.v1.0.json
│
├── characters/                         (shipped-with-app: "Linh" default)
│   └── linh/                           (dev source — packed at build time)
│       ├── manifest.json
│       ├── preview.webp
│       ├── LICENSE
│       └── assets/...
│
├── .github/workflows/
│   ├── ci.yml                          (lint, test, typecheck)
│   └── release.yml                     (macOS dmg build on tag)
│
└── docs/
    ├── PRD.md
    ├── TDD.md                          (this document)
    ├── adr/
    ├── debates/
    └── reviews/
```

---

## 3. Data Types & Schemas

### 3.1 Event Payload

**JSON Schema**: `schemas/event.v1.0.json`

```typescript
// TypeScript mirror (shared via codegen later)
type EventPayload = {
  schemaVersion: "1.0";
  source: "claude-code" | "cursor" | "windsurf" | "generic";  // v0.1 only "claude-code" accepted
  type: EventType;
  tool?: string;           // e.g., "Bash", "Edit", "Read"
  exitCode?: number;
  durationMs?: number;
  severity?: "info" | "warning" | "error" | "critical";  // default "info"
  text?: string;           // optional free text for texture extraction
  metadata?: Record<string, unknown>;
};

type EventType =
  | "session_start"
  | "session_end"
  | "session_idle_short"     // 30s no activity
  | "session_idle_long"      // 5min no activity
  | "user_prompt"
  | "assistant_message"
  | "tool_start"
  | "tool_complete"
  | "error"
  | "destructive_op_detected" // pre-exec inspection of rm -rf, DROP TABLE, force push
  | "git_commit"
  | "git_push";
```

Rust mirror in `src-tauri/src/event/schema.rs` using `serde`.

### 3.2 Canonical State + Texture

```rust
// src-tauri/src/state/canonical.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DominantState {
    Idle,
    Happy,
    Focused,
    Warning,
    Confused,
    Sleepy,
    Shy,
    Flirty,
    Overloaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Texture {
    Relieved,
    Playful,
    Exhausted,
    Alarmed,
    Cute,
    Smug,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedState {
    pub dominant: DominantState,
    pub texture: Option<Texture>,
    pub severity: Severity,
    pub duration_ms: u32,     // hint for renderer; overridable by animation.json
    pub trigger_event_id: String,
}
```

Animation key resolution: `dominant.snake_case()` + optional `"_" + texture.snake_case()`.
Examples: `idle`, `happy`, `happy_relieved`, `warning_critical`.

### 3.3 Character Manifest v1.0

**JSON Schema**: `schemas/manifest.v1.0.json`

```jsonc
{
  "schemaVersion": "1.0",
  "id": "linh-secretary",              // [a-z0-9-]{3,64}
  "name": "Linh (Secretary)",
  "description": "…",
  "author": "hoangperry",
  "version": "1.0.0",                   // semver
  "license": "CC-BY-SA-4.0",            // SPDX
  "tags": ["anime", "chibi"],
  "renderer": "sprite",                 // v0.1: only "sprite"
  "defaultState": "idle",
  "states": {
    "idle":    { "path": "assets/states/idle",    "fps": 12, "loop": true },
    "happy":   { "path": "assets/states/happy",   "fps": 15, "loop": false, "then": "idle", "durationMs": 1500,
                 "textures": {
                   "relieved": "assets/states/happy_relieved",
                   "playful":  "assets/states/happy_playful"
                 }
               },
    "focused": { "path": "assets/states/focused", "fps": 12, "loop": true },
    "warning": { "path": "assets/states/warning", "fps": 18, "loop": true }
  },
  "emotionOverrides": {                 // optional — persona-certified markers
    "(｡•̀ᴗ-)✧": { "texture": "playful" },
    "finally":   { "texture": "relieved" }
  }
}
```

---

## 4. Event Transport Module

### 4.1 HTTP Server (`src-tauri/src/event/server.rs`)

- **Framework**: `axum` (on `tokio` runtime)
- **Bind**: `127.0.0.1:<port>` only — never `0.0.0.0`
- **Endpoint**: `POST /v1/events`, `Content-Type: application/json`
- **Auth**: `Authorization: Bearer <token>` header required; missing/invalid → `401 Unauthorized`
- **Body**: must validate against `event.v1.0.json` schema; fail → `400 Bad Request` with `{error, details}` JSON
- **Success**: `202 Accepted` with empty body
- **Rate limit**: soft cap 50 events/second (drop with log warning), hard cap 500/sec (503)

### 4.2 Port Discovery (`src-tauri/src/event/server.rs` + `config/paths.rs`)

Startup sequence:

1. Read `~/.shikigami/config.json` → `port` (default `7796`)
2. Attempt `TcpListener::bind("127.0.0.1:<port>")`
3. On `AddrInUse`: scan `port+1 .. port+10`; first success wins
4. Write chosen port back to config
5. If all 10 fail: log error, tray notification "Port conflict — see settings", app still runs (UI works, no events received)

Hook scripts re-read `config.json` on each invocation to get current port.

### 4.3 Bearer Token (`src-tauri/src/event/auth.rs`)

- **Generation**: on first launch, create 32-byte random hex in `~/.shikigami/token` with `0600` permissions
- **Rotation**: manual via tray menu or `shikigami doctor --rotate-token`; rotating requires re-running `shikigami install-hook` (hook script references the token file directly)
- **Comparison**: constant-time comparison via `subtle` crate to prevent timing attacks

### 4.4 Hook Bridge Script

```bash
# hooks/shikigami-hook.sh
#!/bin/bash
set -euo pipefail
CONFIG="${HOME}/.shikigami/config.json"
TOKEN="${HOME}/.shikigami/token"
PORT="$(jq -r '.port // 7796' "$CONFIG" 2>/dev/null || echo 7796)"
URL="http://127.0.0.1:${PORT}/v1/events"

# Expects payload JSON on stdin from Claude Code hook system
BODY="$(cat)"

curl -s --max-time 1 -o /dev/null \
  -X POST "$URL" \
  -H "Authorization: Bearer $(cat "$TOKEN")" \
  -H "Content-Type: application/json" \
  -d "$BODY" || true   # never block Claude Code on shikigami unavailability
```

`shikigami install-hook` rewrites `~/.claude/settings.json` to register the bridge on `PreToolUse`, `PostToolUse`, `Stop`, `UserPromptSubmit`.

---

## 5. State Machine Module

### 5.1 Pipeline

```
EventPayload
    │
    ▼
┌─────────────────────────────────────┐
│ validate schema + severity default  │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Dampen::observe(event_type, sev)    │──► drop if within 2s window
└─────────────────────────────────────┘
    │  (passed)
    ▼
┌─────────────────────────────────────┐
│ Stage 1: map event → DominantState  │
│   (table-driven, see §5.3)          │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ severity == critical?               │──► yes: texture = None, lock
└─────────────────────────────────────┘
    │  (no)
    ▼
┌─────────────────────────────────────┐
│ Stage 2: texture = parse(event.text)│
│   (regex, see §5.4)                 │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Apply character compatibility:      │
│   does manifest support this        │
│   (dominant, texture)?              │
│   unsupported texture → None        │
└─────────────────────────────────────┘
    │
    ▼
ResolvedState → emit Tauri event "state_changed" to frontend
```

### 5.2 Dampening (`src-tauri/src/state/dampen.rs`)

Maintain a sliding window of last 20 `(event_type, severity, timestamp)` tuples. On new event:

- If identical `(event_type, severity)` seen within last 2000 ms → **drop silently**, log at trace level
- Exception: `severity == critical` always passes dampening

Prevents the "toxic loop strobe" Gemini flagged.

### 5.3 Event → Dominant Mapping Table

```rust
// src-tauri/src/state/machine.rs
fn map_event(event: &EventPayload) -> DominantState {
    use DominantState::*;
    match (event.event_type.as_str(), event.exit_code) {
        ("session_start", _)            => Idle,
        ("session_idle_long", _)        => Sleepy,
        ("session_idle_short", _)       => Idle,
        ("user_prompt", _)              => Focused,
        ("tool_start", _)               => Focused,
        ("tool_complete", Some(0))      => Happy,
        ("tool_complete", Some(_))      => Warning,
        ("error", _)                    => Warning,
        ("destructive_op_detected", _)  => Warning,   // severity typically critical
        ("git_commit", _)               => Happy,
        ("git_push", _)                 => Happy,
        ("assistant_message", _)        => Idle,       // default; may change later
        _                               => Idle,       // unknown event
    }
}
```

### 5.4 Texture Extraction (`src-tauri/src/state/texture.rs`)

```rust
// Pre-compiled regex set at startup
static TEXTURE_PATTERNS: Lazy<Vec<(Regex, Texture)>> = Lazy::new(|| vec![
    (Regex::new(r"\bfinally\b|phew|(´｡• ᵕ •｡`)").unwrap(), Texture::Relieved),
    (Regex::new(r"\(｡•̀ᴗ-\)✧|\bheh\b|~$").unwrap(),          Texture::Playful),
    (Regex::new(r"\bagain\b|still failing|\bugh\b").unwrap(), Texture::Exhausted),
    (Regex::new(r"⚠️|\bdangerous\b|\bcareful\b").unwrap(),    Texture::Alarmed),
    (Regex::new(r"\*[^*]+\*|♡").unwrap(),                     Texture::Cute),
    (Regex::new(r"told you|\( ?˶ˆᗜˆ˵ ?\)").unwrap(),          Texture::Smug),
]);

pub fn extract_texture(text: &str) -> Option<Texture> {
    // Return first match. Priority is the order above.
    // Multiple matches: first wins. Future v0.2 may composite.
    TEXTURE_PATTERNS.iter()
        .find(|(re, _)| re.is_match(text))
        .map(|(_, t)| *t)
}
```

Character-registered `emotionOverrides` are merged into this table at character load time (higher priority than defaults).

### 5.5 Severity Scaling

Final `duration_ms` hint passed to frontend:

```
info     → base duration from manifest (e.g., 1500ms for happy)
warning  → base × 1.5
error    → base × 2.0, minimum 2500ms
critical → base × 3.0, minimum 4000ms, texture suppressed
```

---

## 6. Character System

### 6.1 Install Flow (`src-tauri/src/character/loader.rs`)

```
User drags .shikigami onto app window
  │
  ▼
1. Copy to ~/.shikigami/characters/<uuid>.shikigami (staging)
2. Unzip into ~/.shikigami/characters/<id>/
3. Validate manifest.json against schema
4. Verify all state paths exist
5. Verify frame files are readable WebP/PNG
6. (Optional) Build runtime atlas: pack frames into N×M texture sheet, write atlas.json
7. Emit Tauri event "character_installed" with character id
8. Frontend reloads library grid
  │
  ▼
On error at any step:
  - Move to ~/.shikigami/characters/_broken/<id>/
  - Log diagnostic to ~/.shikigami/logs/install.log
  - Emit "character_broken" with {id, reason}
  - Library grid shows ⚠ card
```

### 6.2 Hot Reload (`src-tauri/src/character/watcher.rs`)

`notify` crate watches `~/.shikigami/characters/` with debouncing 500ms. On change:

- New directory → attempt install flow on the new item
- Removed directory → emit `character_uninstalled`
- Modified file in active character → reload that state's animation on next transition

### 6.3 Runtime Atlas Building (`src-tauri/src/character/atlas.rs`)

At install time (not runtime), pack frames per-state into a power-of-two texture sheet using a simple row-fit algorithm:

- Max atlas size: 2048×2048 (compatibility with lowest-common-denominator GPUs)
- If frames don't fit in one atlas → split into multiple atlases per state
- Output: `<state>/atlas_<n>.webp` + `<state>/atlas.json` (UV coords per frame)

Frontend prefers atlases when present, falls back to per-frame loading.

### 6.4 Minimum Viable Character

Character passes validation with only `idle` and `happy` states. Missing states at runtime:

- Request for missing dominant → fall back to `idle` + log
- Request for unsupported texture → fall back to base dominant + log

---

## 7. Renderer (Frontend)

### 7.1 PixiJS Setup (`src/renderer/SpriteRenderer.ts`)

- PixiJS v8 WebGL renderer on transparent canvas
- Single `Container` for character; `Sprite` swapped when frame advances
- Resolution: device pixel ratio honored; frames pre-scaled via `resolution` param
- `AnimationLoader` reads from Tauri IPC (backend provides base64 or file:// URL to atlases); PIXI Assets loader handles decode

### 7.2 Transition Engine (`src/renderer/TransitionEngine.ts`)

Two transition types:

- **Crossfade** (default, 200ms): two Sprites, one fading out while the other fades in. GPU alpha blend.
- **Instant** (warnings, severity `critical`): immediate swap, no blend.

Transition table (overridable by manifest):

```
idle   → happy    : crossfade 150ms
idle   → warning  : instant
any    → warning  + critical : instant (always)
happy  → idle     : crossfade 300ms
focused→ focused  (texture change) : crossfade 100ms
```

### 7.3 Frame Pacing

- Target 60fps canvas redraw (PixiJS default)
- Animation frames advance based on `fps` from manifest, decoupled from render loop
- `AnimationClock` uses `requestAnimationFrame` delta, rounds to nearest manifest-fps multiple
- Idle animation pauses (no redraw) when window not visible (saves battery)

### 7.4 Performance Budget Breakdown

| Cost Source | Budget | Enforcement |
|-------------|--------|-------------|
| Idle: no animation tick when occluded | 0% CPU | `requestVideoFrameCallback` pause |
| Animating: PixiJS sprite swap | <1ms/frame | perf.now() in dev mode |
| Memory: loaded atlases | <50MB per active character | assert at install |
| Memory: Tauri baseline | <30MB | Tauri default |
| Total idle target | <80MB RAM | CI check via `memstat` |
| Total peak during state change | <200MB | CI check |

---

## 8. Speech Bubble (`src/components/SpeechBubble.tsx`)

Purpose (FR-055): surface useful context info so Shikigami has *utility* value, not pure novelty.

### 8.1 Trigger Rules

| Condition | Bubble Content | TTL |
|-----------|---------------|-----|
| `destructive_op_detected` (critical) | "⚠️ About to run: `<tool args[:80]>`" + destructive-op name | until dismissed or tool_complete |
| `tool_complete` exitCode ≠ 0 | "❌ `<tool>` failed (exit `<code>`): `<first line of stderr>`" | 8s |
| `git_commit` | "✓ `<short SHA>` on `<branch>`: `<first line of msg>`" | 4s |
| `session_idle_long` | (none — character alone handles this) | — |
| `error` severity `critical` | full error summary with 2s delay before showing | until dismissed |

### 8.2 Visual Spec

- Floats near the character (offset computed from character anchor)
- Max 2 lines of text, 80 chars each line (truncate with ellipsis)
- Fade in 150ms, fade out 300ms
- Dark mode aware (respects macOS appearance)
- Non-interactive by default (click-through); severity `critical` bubbles require explicit close

---

## 9. Window Chrome & System Behavior

### 9.1 Overlay Window (`src-tauri/src/window/overlay.rs`)

Tauri `WindowBuilder` with:

- `decorations: false`
- `transparent: true`
- `always_on_top: true`
- `resizable: true`
- `fullscreen: false`
- `skip_taskbar: true`
- `visible_on_all_workspaces: true` (macOS)
- Initial size: 320×480, min: 160×240, max: 800×1200

### 9.2 Click-Through (`src-tauri/src/window/overlay.rs`)

macOS: toggle `NSWindow.ignoresMouseEvents` via `objc2`. IPC command `set_click_through(bool)`.

### 9.3 Screen Capture Detection (`src-tauri/src/window/screen_capture.rs`)

Poll once/sec for running apps matching:
- OBS Studio (`com.obsproject.obs-studio`)
- QuickTime screen record (check for active `.mov` recording via NSRunningApplication)
- Zoom / Teams (screen share via `ScreenCaptureKit` detection if available on macOS 12.3+)

When detected and setting `auto_hide_during_capture == true`: hide window; restore when detection clears.

### 9.4 Offscreen Recovery

On startup:
1. Load saved window frame from settings
2. Query current screen set
3. If saved frame is entirely outside any screen → reset to center of primary screen + log warning
4. Tray menu has `Reset Position` always available

---

## 10. IPC Contract (Rust ↔ Frontend)

### 10.1 Tauri Commands (TS → Rust)

```typescript
// src/ipc/commands.ts
interface ShikigamiCommands {
  // Character management
  list_characters(): Promise<CharacterSummary[]>;
  get_active_character(): Promise<CharacterSummary | null>;
  set_active_character(id: string): Promise<void>;
  install_character(path: string): Promise<InstallResult>;
  uninstall_character(id: string): Promise<void>;

  // Settings
  get_settings(): Promise<Settings>;
  update_settings(patch: Partial<Settings>): Promise<Settings>;

  // Window
  set_click_through(enabled: boolean): Promise<void>;
  reset_position(): Promise<void>;

  // Debug / doctor
  get_state_history(limit: number): Promise<ResolvedState[]>;
  rotate_token(): Promise<void>;
}
```

### 10.2 Tauri Events (Rust → TS)

```typescript
interface ShikigamiEvents {
  "state_changed":      { payload: ResolvedState };
  "character_installed":{ payload: CharacterSummary };
  "character_uninstalled": { payload: { id: string } };
  "character_broken":   { payload: { id: string; reason: string } };
  "port_conflict":      { payload: { attempted: number; chosen: number | null } };
  "window_position_reset": { payload: null };
}
```

---

## 11. Testing Strategy

### 11.1 Rust Unit Tests

- **State machine**: table-driven tests covering every `(event_type, exit_code, severity) → DominantState` mapping. Target ≥95% line coverage of `state/machine.rs`.
- **Texture extraction**: property-based tests (via `proptest`) for regex stability across random UTF-8 inputs; corpus-based test against `tests/fixtures/texture_corpus/*.txt` with expected-output annotations.
- **Dampening**: deterministic tests with injected clock.
- **Manifest validation**: golden-file tests — every file in `tests/fixtures/manifests/valid/*` passes, every file in `invalid/*` fails with expected error code.

### 11.2 Rust Integration Tests

- Spin up HTTP server on ephemeral port, POST curated event sequences, assert final state history.
- Simulate port conflict (bind a blocker on 7796 before startup) → assert server picks 7797.
- Full install flow: pack a fixture directory into a `.shikigami`, call loader, assert character is active.

### 11.3 Frontend Unit Tests

- **Vitest** for pure TS: animation frame resolver, transition table lookup, texture → animation-key composition.
- **Playwright Component Tests** for React components (SpeechBubble, SettingsModal, CharacterLibrary).

### 11.4 E2E Tests

- **Playwright driving the packaged app** (macOS only in v0.1):
  - Launch app, verify overlay appears
  - POST an event via curl, verify character state changes via debug overlay assertion
  - Drag-drop `.shikigami` → verify grid update
  - Toggle click-through, verify mouse pass-through

### 11.5 Test Fixtures

- `tests/fixtures/characters/minimal/` — 2-state (idle + happy) character for fast tests
- `tests/fixtures/characters/full/` — all 9 states + 6 textures for coverage
- `tests/fixtures/characters/broken/` — deliberately malformed for error-path tests
- `tests/fixtures/events/*.ndjson` — curated event sequences per scenario

### 11.6 Performance Gates (CI)

- Build `shikigami.dmg`, launch in headless macOS runner, measure RSS for 60s idle → fail build if >80MB sustained
- Benchmark event → state machine latency under load → fail if p99 >5ms
- Startup time budget: app visible in <1.5s on M1, <2.5s on Intel

---

## 12. Build & CI

### 12.1 Tauri Config (`src-tauri/tauri.conf.json`)

```jsonc
{
  "$schema": "https://schema.tauri.app/config/2.0.0",
  "productName": "Shikigami",
  "identifier": "dev.shikigami.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [],                      // we create windows programmatically
    "trayIcon": { "iconPath": "icons/tray.png" },
    "security": {
      "csp": "default-src 'self'; img-src 'self' data: file: asset:; style-src 'self' 'unsafe-inline';"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "macOS": {
      "minimumSystemVersion": "12.0",
      "entitlements": "entitlements.plist",
      "signingIdentity": null,           // v0.1 unsigned; v0.2 add identity
      "providerShortName": null,
      "hardenedRuntime": true
    }
  }
}
```

### 12.2 CI Pipeline (`.github/workflows/ci.yml`)

- Trigger: PR and push to main
- Steps: Rust `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, TS `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm build`
- macOS runner: build Tauri app, run E2E tests, enforce memory/latency gates

### 12.3 Release Pipeline (`.github/workflows/release.yml`)

- Trigger: `v*` tag push
- Build universal macOS binary (arm64 + x64)
- Create DMG, upload as GitHub Release asset
- Generate SHA256 checksums

---

## 13. Logging & Observability

### 13.1 Log File

- Location: `~/.shikigami/logs/shikigami.log`
- Rotation: `tracing-appender` daily rotation, keep last 7 days
- Level: `info` by default; `debug` or `trace` via env `SHIKIGAMI_LOG`

### 13.2 Debug Overlay (FR-045)

Enabled via `SHIKIGAMI_DEBUG=1` env or `--debug` flag. Shows:
- Current dominant state + texture
- Last 10 events (type, severity, timestamp, mapped state)
- Active character id, loaded states
- Port + token status
- FPS + RAM gauge

---

## 14. Character Asset Specification (for Codex / contributor task)

> **This section specifies exactly what the character artist task must produce for the default "Linh" sprite character.** Codex or a contributor can read this section standalone.

### 14.1 Deliverable

A single `.shikigami` package named `linh.shikigami` (zip archive) containing:

```
linh.shikigami (zip)
├── manifest.json              (spec below)
├── preview.webp               (512×512, still frame of idle pose)
├── LICENSE                    (MIT or CC-BY-SA-4.0 plaintext, SPDX identifier matches manifest.license)
├── README.md                  (character lore, 50-300 words, optional)
└── assets/states/
    ├── idle/
    │   ├── frame_00.webp
    │   ├── ...
    │   └── frame_11.webp      (12 frames, 12 fps = 1s loop)
    ├── happy/
    │   ├── frame_00.webp
    │   ├── ...
    │   └── frame_14.webp      (15 frames, 15 fps = 1s, non-looping)
    ├── happy_relieved/
    │   └── frame_00..19.webp  (20 frames, 12 fps = ~1.6s slow sigh)
    ├── focused/
    │   └── frame_00..11.webp  (12 frames, 12 fps loop)
    ├── warning/
    │   └── frame_00..17.webp  (18 frames, 18 fps = 1s loop)
    └── sleepy/
        └── frame_00..09.webp  (10 frames, 10 fps loop)
```

### 14.2 Visual Direction

- **Character identity**: anime/chibi young adult woman, secretary aesthetic
  - White blouse, pencil skirt, thin-frame glasses, shoulder-length hair
  - Warm expression, not cold/corporate
  - SFW, tasteful — gesture/expression conveys personality, not overt sexualization
- **Style**: clean line art, soft cel shading; cohesive palette (suggested: warm off-whites + muted blue accent)
- **Camera**: 3/4 view, character centered in 512×512 frame with ~80px breathing room top/bottom

### 14.3 State-by-State Animation Brief

| State | Frames | FPS | Loop | Description |
|-------|--------|-----|------|-------------|
| `idle` | 12 | 12 | yes | Breathing loop. Subtle chest rise, occasional blink (frames 3, 9). Neutral friendly expression. |
| `happy` | 15 | 15 | no | Quick smile bloom: eyes squint slightly, head tilt 5°, small hand wave once. Returns to neutral on last frame. |
| `happy_relieved` | 20 | 12 | no | Slow exhale: close eyes frame 4-8, shoulders drop and rise, half-smile opens. Slower, more weight. |
| `focused` | 12 | 12 | yes | Working pose: hands positioned as if typing (not necessarily visible). Eyes alert, slight forward lean. |
| `warning` | 18 | 18 | yes | Concerned expression: slight frown, one hand raised palm-up in "wait" gesture. Frame loops with small rocking motion. |
| `sleepy` | 10 | 10 | yes | Dozing: head slowly nods down frames 0-5, jerks up frame 6, resets. One eye closed throughout. |

### 14.4 Technical Requirements

- **Resolution per frame**: 1024×1024 source, 512×512 final in zip (downscale at export)
- **Format**: WebP lossless or lossy q=90; PNG acceptable but WebP preferred (smaller)
- **Transparency**: alpha channel required; no background color
- **Frame size consistency**: all frames within a state have identical pixel dimensions
- **File naming**: `frame_00.webp`, `frame_01.webp`, ..., `frame_NN.webp` zero-padded
- **Total package size**: target <20MB compressed; hard cap 30MB

### 14.5 Manifest to Ship

```jsonc
{
  "schemaVersion": "1.0",
  "id": "linh-secretary",
  "name": "Linh",
  "description": "Your calm, competent secretary. Watches code with quiet attention.",
  "author": "hoangperry",
  "version": "1.0.0",
  "license": "CC-BY-SA-4.0",
  "tags": ["anime", "chibi", "secretary", "sfw"],
  "renderer": "sprite",
  "defaultState": "idle",
  "states": {
    "idle":    { "path": "assets/states/idle",    "fps": 12, "loop": true },
    "happy":   { "path": "assets/states/happy",   "fps": 15, "loop": false, "then": "idle", "durationMs": 1000,
                 "textures": { "relieved": "assets/states/happy_relieved" }
               },
    "focused": { "path": "assets/states/focused", "fps": 12, "loop": true },
    "warning": { "path": "assets/states/warning", "fps": 18, "loop": true },
    "sleepy":  { "path": "assets/states/sleepy",  "fps": 10, "loop": true }
  }
}
```

### 14.6 Acceptance Criteria

- `shikigami validate linh.shikigami` passes without errors
- Package loads in dev build without missing-asset warnings
- Frame consistency: no pixel shift between frames when played back (character doesn't "wiggle")
- License file contains valid SPDX-matching content
- Preview.webp clearly readable at 128×128 thumbnail size
- All 5 states playable end-to-end without crashes
- Total package size <25MB

---

## 15. Open Technical Questions

- [ ] Which Rust HTTP framework: `axum` (preferred) vs `warp` vs `hyper+tower` — decide in Phase 1 kickoff
- [ ] Atlas generator algorithm: bin-packing quality vs build-time latency tradeoff
- [ ] Tauri 2 WebView transparency bugs on macOS 15 — test on Sequoia before finalizing Phase 0
- [ ] PixiJS v8 vs v7: v8 is newer but check for sprite-animation regressions
- [ ] Notarization strategy for v0.1: rely on "right-click open" workaround vs invest in Apple Developer account
- [ ] Minimum macOS version: set 12 (Monterey) or 13 (Ventura) based on ScreenCaptureKit dependency

---

## 16. Implementation Order (Phase 0 → 3)

```
Phase 0: Foundation
  └── Tauri scaffold + transparent window POC + CI
  └── Shared schema files (event + manifest)

Phase 1: Event → State → IPC (NO renderer yet)
  └── HTTP server + auth + port recovery
  └── State machine + dampening + texture extraction
  └── Frontend: receive state_changed event, render as text debug panel
  └── Claude Code hook script + `shikigami install-hook` CLI

Phase 2: Character Loading + Sprite Rendering
  └── .shikigami package loader + manifest validation
  └── PixiJS sprite renderer + transitions
  └── Linh fixture character (Codex task)
  └── Hot reload

Phase 3: UX Polish
  └── Speech bubble
  └── Settings modal
  └── Character library grid
  └── Screen capture detection
  └── Signed .dmg release
```

Each phase ends with a tagged pre-release (`v0.1.0-alpha.N`) for internal testing.

---

*"Design documents are love letters to future maintainers."*
