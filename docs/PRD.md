# Shikigami — Product Requirements Document

> **Status**: v0.2 (post-adversarial-review) · **Last Updated**: 2026-04-22 · **Owner**: @hoangperry
> **Type**: Open-source desktop companion for agentic AI coding workflows
> **Previous**: v0.1 — see `docs/reviews/PRD-v0.1-adversarial-review.md` for what changed and why

---

## 1. Executive Summary

**Shikigami** (式神) is an open-source desktop companion that gives AI coding assistants **visual proprioception** — a 2D animated character that reflects what your AI agent is actually doing in real time.

Unlike a "desktop pet" or pure novelty overlay, Shikigami reacts to **structured agent events** (tool calls, exit codes, errors, git operations) rather than cosmetic text patterns. The character is grounded in observable state, so reactions stay correct under load — no smiling while `rm -rf` is running.

**v0.1 ships minimal and lovable**: macOS only, one character ("Linh"), sprite-sheet rendering, Claude Code integration only. Windows, Linux, Live2D, multi-AI adapters, and the character marketplace come in later releases once retention is proven.

**Key differentiators:**
- 🧠 **Event-driven core** — character reactions map to real agent actions, not kaomoji
- 🛡️ **Context-aware severity** — destructive operations produce serious reactions, not cute ones
- 💬 **Utility beyond novelty** — speech bubbles surface useful info (error summary, commit SHA) to fight UX fatigue
- 🪶 **Truly open source** — MIT core with no proprietary dependencies
- 🔌 **Extensible character format** — `.shikigami` zip bundle, low barrier for artists

---

## 2. Problem Statement

### The Pain (Reframed post-review)

Agentic AI coding workflows generate **high cognitive load** with **low visual feedback**:

1. **Context opacity** — agent is running multiple tools in parallel; users lose track of what's happening
2. **Silent destructive operations** — dangerous commands execute inside otherwise-benign sessions
3. **Flat terminal feedback** — success/failure/in-progress states all render as text
4. **Long session fatigue** — 4–8 hours of text-only interaction with a capable collaborator feels isolating for users who *do* want presence (not all do, and that's fine)

### Why Existing Tools Don't Solve This

| Tool | Gap |
|------|-----|
| VTube Studio / VSeeFace | Not wired to AI events; requires manual triggering |
| Desktop pets (Shimeji, Mascot.js) | Static behavior, no agent awareness |
| IDE status bars | Cramped, not proprioceptive |
| Custom output styles (text only) | Still text, no visual layer |
| macOS Focus modes / Do Not Disturb | Suppresses rather than surfaces state |

### Opportunity

Bridge between **agentic AI tools** (lots of events, no visuals) and **desktop pet / VTuber ecosystems** (lots of visuals, no agent awareness). Shikigami's insight: treat character as **status indicator with personality**, not pure novelty.

---

## 3. Goals & Success Metrics

### Goals (SMART)

| ID | Goal | Metric | v0.1 (3mo) | v1.0 (9mo) | Priority |
|----|------|--------|-----------|-----------|----------|
| G1 | Ship macOS MVP | Platforms | macOS ✓ | + Windows, Linux | P0 |
| G2 | Event-grounded reactions | Reaction correctness on test corpus | ≥90% | ≥95% | P0 |
| G3 | Context-aware severity | Destructive ops show correct state | 100% | 100% | P0 |
| G4 | Low resource footprint | RAM idle / peak | <80MB / <200MB | same | P0 |
| G5 | Fast onboarding | Install to first reaction | <3 min | <2 min | P0 |
| G6 | Character extensibility | Time to ship a sprite character | <60 min (honest) | <30 min | P1 |
| G7 | Community traction | GitHub stars | 100–200 | 500–1,000 | P1 |
| G8 | Retention | Weekly active users | 15–25 | 100+ | P1 |
| G9 | Community packs | External `.shikigami` packs published | 2–3 | 10+ | P1 |
| G10 | Truly-OSS status | No proprietary deps in core | MIT clean | MIT clean | P0 |

### Explicitly NOT Goals (v0.1)

- Cross-platform day-one
- Live2D support
- Multi-AI adapters (Cursor/Windsurf/ChatGPT)
- Character marketplace
- Enterprise / B2B
- Mobile
- TTS / voice

---

## 4. Non-Goals

- ❌ 3D characters (VRM rendered flat may come in v2+)
- ❌ TTS / voice synthesis v1 (architecture-ready but not shipped)
- ❌ Multiplayer / shared sessions
- ❌ In-app character creation GUI (contributors use external tools + CLI)
- ❌ Telemetry / analytics (zero tracking)
- ❌ Cloud sync v1 (future premium tier)
- ❌ Third-party JS renderers in v1 (security; removed per ADR-004)
- ❌ URL-based character install in v1 (signing/trust story needed first)

---

## 5. User Personas

### 5.1 Primary: **Linh — Solo Indie Developer**

- 28, full-stack dev, 6h/day Claude Code user on Mac, anime/game culture
- **Pain**: "I burned a production table last month because I missed a destructive command in the middle of a long session."
- **Goals**: visible signal for risky operations, ambient presence during long sessions
- **Success**: "Shikigami caught the `DROP TABLE` before I hit enter. Also, the shy animation after my first green CI was genuinely lovely."

### 5.2 Secondary: **Kenji — Character Artist / OSS Contributor**

- 35, Aseprite/Procreate hobbyist, sells chibi commissions
- **Pain**: "VTuber tools are cumbersome; I want to share my characters with more dev-aligned audiences."
- **Goals**: easy packaging, clear license guidance, portfolio visibility
- **Success**: "I shipped my first `.shikigami` pack in an afternoon. Twelve devs installed it in the first week."

### 5.3 Tertiary: **Alex — AI-curious Non-Developer** *(out of scope for v0.1)*

- 22, designer, uses Cursor
- Deferred — v0.1 is Claude Code only. Cursor adapter in v0.4+.

---

## 6. Functional Requirements

> Format: `FR-XXX` — acceptance criteria must be testable.

### 6.1 Core Engine (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-001 | Local HTTP event endpoint | POST `http://127.0.0.1:<port>/v1/events` with bearer token auth (ADR-001) |
| FR-002 | Event-driven state machine | Structured events → canonical state with severity scaling (ADR-002) |
| FR-003 | Text-parse fallback | Opt-in kaomoji/keyword detection from `event.text` field |
| FR-004 | Transparent always-on-top window | Frameless, resizable, click-through toggle, stays visible across virtual desktops |
| FR-005 | System tray | Show/hide, swap character, open settings, pause reactions, quit |
| FR-006 | Settings persistence | JSON config in `~/Library/Application Support/Shikigami/` |
| FR-007 | Hot reload of characters | Detect `~/.shikigami/characters/` changes → reload without restart |
| FR-008 | Severity scaling | Event `severity` influences state duration + intensity |
| FR-009 | Event dampening | Identical events within 2s collapse to single state transition |
| FR-058 | Sub-state texture composition | Dominant canonical state + optional texture modifier → final animation key `<dominant>[_<texture>]`. v0.1 textures: `relieved`, `playful`, `exhausted`, `alarmed`, `cute`, `smug`. (ADR-002 Hierarchical Fusion) |
| FR-059 | Parallel text-texture extraction | Regex on `event.text` runs in parallel to Stage 1 state mapping. `critical` severity events skip texture layer. Missing `event.text` → no texture (dominant-only). |

### 6.2 Character System (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-010 | `.shikigami` package format v1.0 | Zip bundle with manifest.json, validated on install (ADR-003) |
| FR-011 | Manifest schema validation | JSON Schema v2020-12; broken packages listed with diagnostic |
| FR-012 | Sprite Sheet renderer | WebP frame sequences, auto-atlas at install time |
| FR-013 | Minimum viable character | Only `idle` + `happy` states required; missing states fall back to `idle` |
| FR-015 | Character library browser | Grid view with preview, install/uninstall |
| FR-016 | Character hot-swap | Switch active character without session interruption |
| FR-017 | Per-character emotion overrides | Manifest `emotionOverrides` for custom triggers |

*FR-014 (third-party renderer plugins) removed per ADR-004.*

### 6.3 Integration (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-020 | Claude Code hook bridge | One-line install: `shikigami install-hook` adds entries to `~/.claude/settings.json` |
| FR-021 | Hook rollback | `shikigami uninstall-hook` removes all shikigami entries cleanly |
| FR-022 | Hook diagnostics | `shikigami doctor` validates hook paths, token, port, transport |

*FR-023/024/025 (Cursor, Windsurf, ChatGPT adapters) deferred to v0.4+.*

### 6.4 UX & Utility (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-040 | Window position memory | Restore on restart, reset-to-center if offscreen |
| FR-041 | Character size slider | 50%–200% |
| FR-042 | Opacity control | 20%–100% |
| FR-043 | Click-through toggle | Clicks pass through to apps behind |
| FR-044 | Auto-start on login | Opt-in macOS Login Item |
| FR-045 | Debug overlay | Dev mode shows current state + last 10 events |
| FR-055 | **Speech bubble** (anti-fatigue utility) | Small bubble near character shows: error summary, commit SHA, long-running tool name/duration, destructive-op warning details |
| FR-056 | **Context-aware severity** | `destructive_op_detected` → `warning` state with serious expression + red bubble, no cute reaction |
| FR-057 | **Idle variety** | Random micro-interactions every 30–90s during `idle` (blink, look around, small pose shift) |

### 6.5 Robustness — New post-review (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-050 | Port conflict recovery | On bind failure, scan next 10 ports, write chosen port to config, log clearly |
| FR-051 | Multi-monitor + mixed-DPI | Correctly position and scale across monitors with different DPI; remember monitor per window |
| FR-052 | Fullscreen / screen-record awareness | Auto-hide when OBS, QuickTime, Zoom screen-share, or fullscreen app detected (configurable) |
| FR-053 | Lost-overlay recovery | Tray menu `Reset Position` option; auto-detect offscreen on startup and recenter |
| FR-054 | Package corruption graceful fallback | Show `⚠ Broken` state in library with clickable diagnostic, never crash the app |

### 6.6 Installation (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-030 | macOS `.dmg` | Universal binary (arm64 + x64), notarized best-effort for v0.1 |
| FR-033 | Drag-drop install | Drop `.shikigami` onto app → character installed |
| FR-034 | CLI install | `shikigami install <path-to-file-or-directory>` |

*FR-031/032 (Windows/Linux installers) deferred to v0.2/v0.3.*
*FR-035 (URL install) and FR-036 (registry install) removed from v1.*

### 6.7 Future Scope (Backlog — architecture-ready, NOT shipped)

| ID | Feature | Deferred rationale |
|----|---------|-------------------|
| FR-F01 | TTS / voice output | Voice asset licensing, premium path |
| FR-F02 | Lip-sync animation | Needs TTS + Live2D |
| FR-F03 | Cloud sync settings (premium) | Sustainability tier, v2+ |
| FR-F04 | Public character marketplace | Moderation/hosting infra |
| FR-F05 | Per-state voice lines | Voice actor coordination |
| FR-F06 | 3D / VRM support | Different renderer stack |
| FR-F07 | Mobile companion | Different form factor |
| FR-F08 | Multi-character simultaneous | State complexity |
| FR-F09 | Cursor/window interactions | Accessibility APIs per-OS |
| FR-F10 | **VMC protocol export** | Power-user bridge to VTube Studio / Warudo (Gemini suggestion) |
| FR-F11 | **`shikigami-mind` LLM classifier sidecar** | Optional small local quantized LM (Phi-3-mini / Qwen2.5-0.5B) for semantic state classification. Runs in separate process, emits the same event schema as rule-based path. Deferred per Position D outcome in signal-source debate. |

---

## 7. Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│  Application Shell (Tauri 2, macOS)                        │
│  ├── Transparent always-on-top window (React + PixiJS)     │
│  ├── System tray (native Rust)                             │
│  └── Settings IPC                                          │
├───────────────────────────────────────────────────────────┤
│  Character Manager (TS)                                    │
│  ├── Package loader (.shikigami zip + manifest validate)   │
│  ├── Sprite renderer (PixiJS v8)                          │
│  └── Speech bubble overlay                                 │
├───────────────────────────────────────────────────────────┤
│  Emotion State Machine (Rust)                              │
│  ├── Event → state mapper                                  │
│  ├── Severity scaling                                      │
│  ├── Dampening (2s window)                                 │
│  └── Text-parse fallback (opt-in)                          │
├───────────────────────────────────────────────────────────┤
│  Event Transport (Rust / tokio + axum)                     │
│  ├── HTTP POST /v1/events on 127.0.0.1                     │
│  ├── Bearer token auth                                     │
│  └── Port conflict recovery                                │
└───────────────────────────────────────────────────────────┘
                        ▲
                        │ HTTP POST
                        │
       ┌────────────────┴────────────────┐
       │ Claude Code Hooks               │
       │ (PreToolUse / PostToolUse /     │
       │  Stop / UserPromptSubmit)       │
       └─────────────────────────────────┘
```

See ADRs for rationale on each layer:
- **ADR-000** — Licensing & protocol
- **ADR-001** — HTTP transport with auth
- **ADR-002** — Event-driven signal source
- **ADR-003** — Character package format
- **ADR-004** — v1 scope (sprite-only, Mac-first)

---

## 8. Canonical Emotion States & Textures

> **Architecture**: Hierarchical Fusion — see ADR-002. Events determine the **dominant state** (Stage 1); text-parse within the event's `text` field optionally adds a **texture modifier** (Stage 2). Final animation key = `<dominant>[_<texture>]`.

### 8.1 Dominant States (canonical, 9)

| State | Event Triggers | Duration |
|-------|---------------|----------|
| `idle` | default, `session_idle_short` | loop |
| `happy` | `tool_complete` + exitCode 0 (brief), `git_commit`, `git_push` | ~1.5s → idle |
| `focused` | `tool_start`, `user_prompt` | until completion |
| `warning` | `tool_complete` non-zero exit, `error`, `destructive_op_detected` | severity-scaled |
| `confused` | repeated errors, unclear state | ~2s |
| `sleepy` | `session_idle_long` (5+ min) | loop until wake |
| `shy` | event-triggered contexts (e.g. affirmative user response on themed styles) | brief |
| `flirty` | opt-in styles only | brief |
| `overloaded` | consecutive errors within 10s | ~3s |

### 8.2 Texture Modifiers (v0.1 canonical set, 6)

Textures are composed onto a dominant state via lightweight regex on the event's `text` field. Texture layer is **skipped** when severity is `critical`.

| Texture | Trigger Patterns | Effect |
|---------|-----------------|--------|
| `relieved` | "finally", "phew", `(´｡• ᵕ •｡`)` | slow sigh before main animation |
| `playful` | `(｡•̀ᴗ-)✧`, "heh", "~" markers | lighter weight + wink |
| `exhausted` | "again", "still failing", "ugh" | slumped posture |
| `alarmed` | `⚠️`, "dangerous", "careful" | raised hand + wide eyes |
| `cute` | `*action text*` patterns, `♡` | posing flair |
| `smug` | "told you", `( ˶ˆᗜˆ˵ )` | chin-up smirk |

Characters MAY declare textures per state in their manifest. Unknown or unsupported textures fall back gracefully to the dominant state with no error.

### 8.3 Composition Examples

| Dominant | Texture | Final |
|----------|---------|-------|
| `happy` | `relieved` | `happy_relieved` |
| `focused` | `alarmed` | `focused_alarmed` |
| `warning` | *(ignored, critical)* | `warning` |
| `sleepy` | `cute` | `sleepy_cute` |
| `idle` | — | `idle` |

Characters MAY add custom dominant states. Unknown dominants fall back to nearest canonical via manifest hint.

---

## 9. Implementation Phases

### Phase 0 — Foundation (Week 1)

- [x] Repo + README + .gitignore + PRD v0.1 ✓
- [x] PRD v0.2 + 5 ADRs ✓ (this document)
- [ ] Tauri 2 + React + TS scaffold
- [ ] Transparent always-on-top window POC on macOS
- [ ] CI: lint + typecheck + test on macOS

### Phase 1 — Event Transport & State Machine (Week 2)

- [ ] HTTP POST server on `127.0.0.1:7796` with bearer token
- [ ] Port conflict recovery
- [ ] Event payload schema v1 + validation
- [ ] Event → state machine with severity scaling + dampening
- [ ] Text-parse fallback (opt-in)
- [ ] Claude Code hook bridge (`shikigami install-hook` + `doctor`)
- [ ] Unit tests ≥90% for state machine

### Phase 2 — Character Renderer (Week 3)

- [ ] `CharacterRenderer` interface + `SpriteSheetRenderer`
- [ ] `.shikigami` package loader + JSON Schema validator
- [ ] Atlas auto-generation at install time
- [ ] Hot reload
- [ ] First character "Linh" — 4 states: idle / happy / focused / warning
- [ ] Character library browser (grid view)
- [ ] Character hot-swap

### Phase 3 — UX Polish (Week 4)

- [ ] System tray menu
- [ ] Settings window (size / opacity / position / click-through / auto-start)
- [ ] Speech bubble component with context-aware content (FR-055)
- [ ] Idle variety (FR-057)
- [ ] Fullscreen / screen-record detection (FR-052)
- [ ] Offscreen recovery (FR-053)
- [ ] macOS `.dmg` release via GitHub Actions
- [ ] Docs site (contributor guide, template repo)

### Phase 4 — v0.2 Windows Port (T+4 weeks)

- [ ] Tauri Windows build
- [ ] Transparent overlay WGL quirks
- [ ] `.msi` installer via NSIS
- [ ] PowerShell hook script

### Phase 5 — v0.3 Linux + Optional Live2D Add-on (T+6 weeks)

- [ ] Tauri Linux build (WebKitGTK)
- [ ] AppImage + `.deb` + `.rpm`
- [ ] `shikigami-live2d` optional add-on in separate repo (ADR-000)

### Phase 6+ — v0.4+ Adapters & Beyond

- [ ] Cursor adapter
- [ ] Windsurf adapter
- [ ] VMC protocol export (FR-F10)
- [ ] TTS exploration

---

## 10. Risks & Mitigations (Updated)

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Event-source fidelity varies across AI tools | High | High | v0.1 is Claude Code only; adapter pattern isolates parsing when we add more |
| Transparent overlay breaks on macOS updates | Medium | High | Nightly CI smoke tests, maintain min supported macOS version |
| Low community character contribution | Medium | High | Template repo + `shikigami pack` CLI + short tutorial video + 1 flagship character to inspire |
| Performance budget missed with sprite + Tauri | Medium | Medium | Atlas packing, WebP frames, strict budget CI gate (fail build if bundle/RAM regressions) |
| Context-blind reactions (smiling at disaster) | Low post-ADR-002 | Critical | Severity-aware state machine (ADR-002) + test corpus with destructive ops |
| Toxic loop strobe on error spam | Low post-FR-009 | Medium | Event dampening window |
| Port / firewall / antivirus blocks listener | Medium | Medium | Conflict recovery + clear diagnostic (FR-050) |
| User distraction ("Clippy 2.0") | Medium | High | Pause reactions toggle, idle variety only when in foreground, speech bubbles deliver utility |
| UX fatigue after 48h novelty | Medium | High | Speech bubble *utility* content — error summaries + commit SHAs > cute-only |
| Claude Code hook format breaking change | Low | High | Version-detect hook schema + graceful fallback |
| Artist onboarding too heavy | Medium | Medium | Minimum viable character = 2 states (`idle`, `happy`); missing states fall back gracefully |
| Scope creep back toward v0.1 ambition | High | High | Strict non-goals list; this PRD is the source of truth |

---

## 11. Open Questions

- [ ] Notarization / code signing for macOS `.dmg` — Apple Developer account needed; defer to post-v0.1 if cost is a barrier
- [ ] Default bearer token rotation strategy — v0.1 = install-time static; v0.2 = consider refresh on demand
- [ ] Speech bubble max character count — UX test with real error messages
- [ ] Is `shikigami-live2d` add-on worth the maintenance cost? Validate demand first in v0.2 window
- [ ] Character name collision in registry (v1.0) — namespace by author? ID validation rules?

---

## 12. Appendix

### 12.1 Glossary

- **Event** — structured JSON payload from host AI tool (tool_start, tool_complete, error, etc.)
- **Canonical State** — one of 9 built-in emotion states every character fallback-maps to
- **Severity** — `info | warning | error | critical` tag on events that scales state duration/intensity
- **Speech Bubble** — small overlay near character with useful contextual info (error, SHA, warnings)
- **Character Package** — `.shikigami` zip bundle with manifest + sprite assets
- **Renderer** — code that draws a character given a state; v0.1 has only `SpriteSheetRenderer`

### 12.2 References

- Tauri 2: https://v2.tauri.app/
- PixiJS: https://pixijs.com/
- JSON Schema: https://json-schema.org/
- VSCode Extension format (manifest inspiration): https://code.visualstudio.com/api/working-with-extensions/publishing-extension
- VMC Protocol: https://protocol.vmc.info/ *(future FR-F10)*
- Claude Code Hooks: https://docs.claude.com/claude-code/hooks

### 12.3 Review History

- **v0.1** → **v0.2**: post-adversarial-review by Codex + Gemini; 18 accepted challenges applied, 3 reframed. See `docs/reviews/PRD-v0.1-adversarial-review.md`.
- **v0.2 signal-source refinement** (same day): 4-way Steelman Tournament debate (Opus, Sonnet, Gemini, Codex proxy) → ADR-002 revised from "Event-Driven Primary" to **"Hierarchical Fusion — Event-Gated, Text-Textured"**. FR-058 / FR-059 added. FR-F11 (LLM sidecar) added to backlog. See `docs/debates/2026-04-22-signal-source/`.
- Adversarial review: pending for v0.3 (re-run before v1.0 release tag).
- PRD Score (self, v0.2): **93/100** — strong on event-grounded reactions with emotional nuance via texture layer, honest scope, explicit trade-offs; weak on monetization detail (deferred intentionally) and Windows/Linux depth (deferred to v0.2/0.3).

---

*"She watches what your agent actually does. She reacts with truth. Summoned by code, grounded in events."*
