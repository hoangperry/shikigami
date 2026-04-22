# Shikigami — Product Requirements Document

> **Status**: Draft v0.1 · **Last Updated**: 2026-04-22 · **Owner**: @hoangperry
> **Type**: Open-source desktop companion for AI coding assistants

---

## 1. Executive Summary

**Shikigami** (式神) is an open-source, cross-platform desktop companion that makes AI coding assistants feel *alive*. By parsing AI output in real-time and rendering an emotionally-reactive 2D character as a Picture-in-Picture overlay, Shikigami transforms the cold, text-only experience of agentic coding into something playful, warm, and sustainable during long sessions.

**The vision**: A desktop full of spirits — each developer summons their own companion to sit beside them while they code. Characters are community-contributed, easy to create, and distributed as portable packages. The app starts with Claude Code integration and expands to any AI tool that can emit events (Cursor, Windsurf, ChatGPT, Gemini CLI, etc.).

**Key differentiators:**
- 🪶 Ultra-lightweight (Tauri-based, <50MB RAM idle)
- 🎨 Dual-renderer architecture (Live2D + Sprite Sheet) — low barrier for artists
- 🔌 Plug-and-play character packages (`.shikigami` format)
- 🔒 100% local by default — no telemetry, no cloud
- 🌏 Cross-platform from day one (Mac → Windows → Linux)

---

## 2. Problem Statement

### The Pain

Developers using AI coding assistants spend 4–8+ hours per day interacting with a text-only terminal or chat interface. Despite the AI providing substantial cognitive assistance, the experience is:

1. **Emotionally flat** — responses are walls of text with zero visual feedback
2. **Fatiguing over long sessions** — no "presence" companionship, isolation during deep work
3. **Hard to read at-a-glance** — tool success/failure/warning states require parsing text
4. **Lacking personality continuity** — different output styles feel disconnected

### Current Workarounds (Why They Fail)

| Workaround | Why it falls short |
|-----------|-------------------|
| VTube Studio + manual triggers | Requires constant manual input, not reactive |
| Music / ambient apps | Not tied to AI session state |
| Anime wallpaper engines | Static, no interaction with AI output |
| Custom Claude Code output styles (text only) | Still text, no visual element |

### Opportunity

No existing product bridges the gap between **agentic AI tools** and **desktop pet / VTuber** ecosystems. Shikigami fills this niche with a focused, open-source, extensible solution.

---

## 3. Goals & Success Metrics

### Goals (SMART)

| ID | Goal | Metric | Target (v1.0) | Priority |
|----|------|--------|---------------|----------|
| G1 | Ship cross-platform MVP | Platforms supported | Mac ✓ → Win → Linux | P0 |
| G2 | Low resource footprint | RAM idle / peak | <80MB / <200MB | P0 |
| G3 | Reactive emotion engine | Kaomoji → state mapping accuracy | ≥90% on test corpus | P0 |
| G4 | Character extensibility | Time to add new character | <30 min for artist | P0 |
| G5 | Open-source traction | GitHub stars | 1,000+ within 6 months | P1 |
| G6 | Community character library | Community-contributed characters | ≥10 within 3 months | P1 |
| G7 | Multi-AI tool support | Integration adapters | Claude Code + 2 others | P1 |

### Non-Success Metrics (explicitly not optimized)

- ❌ Enterprise adoption / B2B sales
- ❌ Monetization at launch (premium features deferred to v2+)
- ❌ Mobile / tablet support

---

## 4. Non-Goals

Explicitly out of scope for v1:

- ❌ **3D character support** — 2D only (VRM/VRoid rendered flat is future scope)
- ❌ **TTS / voice synthesis** — future feature, architecture will be ready but not shipped
- ❌ **Multiplayer / shared sessions** — single-user local experience
- ❌ **Character creation GUI** — contributors use external tools (Aseprite, Cubism) + CLI
- ❌ **Telemetry / analytics** — zero tracking, privacy-first
- ❌ **Cloud sync** — deferred to premium tier (v2+)
- ❌ **Mobile app** — desktop-only
- ❌ **AI-generated characters in-app** — contributors create assets externally

---

## 5. User Personas

### 5.1 Primary: **Linh — The Solo Indie Developer**

- **Profile**: 28, full-stack dev, uses Claude Code 6h/day, loves anime/game culture
- **Setup**: Mac M2, 2x external monitors, VSCode + iTerm
- **Pain**: "Long AI coding sessions feel lonely and exhausting. I miss having someone *present* while I code."
- **Goals**: Enjoyable workflow, showcase personality, fun over utility
- **Success for Linh**: "I smile when my character pouts at a failing test. My Discord friends ask what app I'm using."

### 5.2 Secondary: **Kenji — The Open-Source Maintainer & Character Artist**

- **Profile**: 35, Live2D/Spine hobbyist, runs a small indie game studio
- **Pain**: "I want to share my characters, but VTube Studio's model format is cumbersome and closed."
- **Goals**: Contribute to open projects, portfolio visibility, low technical barrier to distribution
- **Success for Kenji**: "I packaged my OC as `.shikigami` in one afternoon and got 200 downloads in a week."

### 5.3 Tertiary: **Alex — The Curious AI Power User**

- **Profile**: 22, uses Cursor/Windsurf/ChatGPT, non-developer (designer)
- **Pain**: "I'd love this for my Cursor too, not just Claude Code."
- **Goals**: Plug into any AI workflow
- **Success for Alex**: "Shikigami works with my AI tool through an adapter I installed in 5 min."

---

## 6. Functional Requirements

### 6.1 Core Engine (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-001 | WebSocket server on localhost | Accepts POST events on configurable port (default `:7796`) |
| FR-002 | Emotion parser module | Detects kaomoji, action text, keywords → emits `EmotionState` |
| FR-003 | State transition machine | Smooth transitions between states with configurable timing |
| FR-004 | Transparent always-on-top window | Frameless, click-through optional, resizable, repositionable |
| FR-005 | System tray integration | Show/hide, switch character, open settings, quit |
| FR-006 | Settings persistence | JSON config in OS-standard location |
| FR-007 | Hot reload of characters | Detect filesystem changes, reload without app restart |

### 6.2 Character System (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-010 | Character Package format (`.shikigami`) | Standardized zip bundle with manifest schema v1.0 |
| FR-011 | Manifest validation | Schema validation on load, graceful error messages |
| FR-012 | Sprite Sheet renderer | Supports PNG sequences + JSON animation descriptors |
| FR-013 | Live2D renderer | Cubism 4+ `.model3.json` support via official Web SDK |
| FR-014 | Renderer plugin interface | Third-party renderers can register via plugin manifest |
| FR-015 | Character library browser | Grid view of installed characters with preview |
| FR-016 | Multi-character swap | Hot-swap active character without session interruption |
| FR-017 | State mapping per character | Character-specific emotion overrides via manifest |

### 6.3 Integration Adapters (P0 — Claude Code; P1 — others)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-020 | Claude Code hook bridge | Shell/PowerShell script, installs via single command |
| FR-021 | Generic HTTP adapter | Any tool can POST JSON events → triggers state |
| FR-022 | Cursor adapter | P1 — supports Cursor's extension API or log tailing |
| FR-023 | Windsurf adapter | P1 — same as Cursor |
| FR-024 | ChatGPT/Claude web adapter | P2 — browser extension that emits events |

### 6.4 Installation Methods (P0)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-030 | App installer (Mac `.dmg`) | Universal binary, code-signed (best-effort for v1) |
| FR-031 | App installer (Win `.msi`) | NSIS-based, Windows 10+ |
| FR-032 | App installer (Linux) | AppImage + `.deb` + `.rpm` |
| FR-033 | Character install — drag & drop | Drop `.shikigami` on app window → installed |
| FR-034 | Character install — CLI | `shikigami install <path-or-url>` |
| FR-035 | Character install — URL | Direct install from GitHub release URL |
| FR-036 | Character install — registry (P1) | `shikigami install <name>` from public registry |

### 6.5 Settings & UX (P1)

| ID | Requirement | Acceptance Criteria |
|----|-------------|---------------------|
| FR-040 | Window position/size memory | Restores on restart |
| FR-041 | Character size slider | 50%–200% scale |
| FR-042 | Opacity control | 20%–100% |
| FR-043 | Click-through toggle | Allows clicks to pass through to apps behind |
| FR-044 | Auto-start on login | Opt-in checkbox |
| FR-045 | Debug overlay | Dev mode shows current state + incoming events |

### 6.6 Future Scope (Backlog — NOT in v1)

> Architecture must keep these doors open but v1 does **NOT** ship them.

| ID | Feature | Why deferred |
|----|---------|--------------|
| FR-F01 | TTS / voice output | Needs character voice assets, licensing complexity |
| FR-F02 | Lip-sync animation | Depends on TTS + Live2D params |
| FR-F03 | Cloud sync settings (premium) | Requires auth/backend infra — premium sustainability model |
| FR-F04 | Character marketplace (web) | Requires moderation, hosting, payments infrastructure |
| FR-F05 | Voice lines per state | Needs voice actor coordination |
| FR-F06 | 3D character support (VRM) | Different renderer stack, future exploration |
| FR-F07 | Mobile companion app | Different form factor, later phase |
| FR-F08 | Multi-character on screen simultaneously | Complex state management |
| FR-F09 | Character interactions with cursor/windows | Requires accessibility APIs per OS |

---

## 7. Character System Architecture (Deep Dive)

> **This is the core extensibility foundation. Designed for longevity.**

### 7.1 Design Principles

1. **Renderer-agnostic core** — emotion state machine knows nothing about rendering
2. **Single package format** — `.shikigami` is the universal distribution unit
3. **Low barrier for artists** — someone who knows Photoshop + PNG export should be able to ship a character
4. **Forward compatibility** — schema versioning lets future formats coexist
5. **Dependency explicit** — manifest declares renderer, version, requirements

### 7.2 Layered Architecture

```
┌─────────────────────────────────────────────┐
│       Application UI (React)                │
├─────────────────────────────────────────────┤
│       Character Manager                     │
│   (load, validate, lifecycle)               │
├─────────────────────────────────────────────┤
│       Renderer Plugin Interface             │
│                                             │
│   ┌──────────┐ ┌─────────┐ ┌────────────┐  │
│   │ Sprite   │ │ Live2D  │ │ Future...  │  │
│   │ Renderer │ │ Renderer│ │ Spine/Lottie│ │
│   └──────────┘ └─────────┘ └────────────┘  │
├─────────────────────────────────────────────┤
│       Emotion State Machine (Rust)          │
│   (transitions, timing, blending hints)     │
├─────────────────────────────────────────────┤
│       Emotion Parser (Rust)                 │
│   (regex + ML-lite classifier future)       │
├─────────────────────────────────────────────┤
│       WebSocket Server (Rust/tokio)         │
└─────────────────────────────────────────────┘
```

### 7.3 Character Package Format — `.shikigami`

A **zip bundle** (just like `.vsix`, `.jar`, `.docx`). Renamed extension for discoverability.

```
linh-secretary.shikigami
├── manifest.json           ← required, schema-validated
├── preview.webp            ← required, 512x512 thumbnail
├── README.md               ← optional, character lore
├── LICENSE                 ← required (CC-BY / CC-BY-SA / custom)
├── assets/
│   ├── states/
│   │   ├── idle/
│   │   │   ├── frame_00.webp
│   │   │   ├── frame_01.webp
│   │   │   └── ...
│   │   ├── happy/
│   │   ├── shy/
│   │   └── focused/
│   └── audio/              ← future (FR-F01)
│       └── voice_lines/
└── config/
    ├── emotion_overrides.json  ← custom triggers for this character
    └── anchor_points.json      ← render position hints
```

### 7.4 Manifest Schema (v1.0)

```json
{
  "$schema": "https://shikigami.dev/schema/manifest/v1.0.json",
  "schemaVersion": "1.0",
  "id": "linh-secretary",
  "name": "Linh (Secretary)",
  "description": "Your charming executive assistant who watches over your code.",
  "author": {
    "name": "hoangperry",
    "url": "https://github.com/hoangperry"
  },
  "version": "1.0.0",
  "license": "CC-BY-SA-4.0",
  "tags": ["anime", "chibi", "secretary", "SFW"],
  "renderer": {
    "type": "sprite",
    "version": "1.0"
  },
  "defaultState": "idle",
  "states": {
    "idle": {
      "path": "assets/states/idle",
      "frameRate": 12,
      "loop": true,
      "blendable": true
    },
    "happy": {
      "path": "assets/states/happy",
      "frameRate": 15,
      "loop": false,
      "then": "idle",
      "duration": 2000
    },
    "shy": {
      "path": "assets/states/shy",
      "frameRate": 10,
      "loop": false,
      "then": "idle"
    }
    // ... other states
  },
  "transitions": {
    "default": { "duration": 200, "type": "crossfade" },
    "idle->happy": { "duration": 150, "type": "crossfade" },
    "any->warning": { "duration": 50, "type": "instant" }
  },
  "emotionOverrides": {
    "(｡•̀ᴗ-)✧": "flirty",
    "wink": "flirty"
  },
  "futureFeatures": {
    "audio": false,
    "lipSync": false
  }
}
```

### 7.5 Renderer Plugin Interface (TypeScript)

```typescript
interface CharacterRenderer {
  readonly type: string;
  readonly supportedSchemaVersion: string;

  canHandle(manifest: CharacterManifest): boolean;
  mount(container: HTMLElement, character: Character): Promise<void>;
  transitionTo(state: EmotionState, options?: TransitionOptions): Promise<void>;
  tick(deltaMs: number): void;
  dispose(): void;
}
```

Built-in renderers register via `RendererRegistry.register(new SpriteRenderer())`. Third-party renderers can be loaded from `~/.shikigami/renderers/*.js` (sandboxed).

### 7.6 Emotion State Vocabulary (Canonical)

Core states every character must support (fallback to `idle` if missing):

| State | Description |
|-------|-------------|
| `idle` | Default, breathing loop |
| `happy` | Positive completion, success |
| `focused` | Working, processing |
| `shy` | Compliment received, blushing moment |
| `confused` | Unclear input, question |
| `flirty` | Playful, teasing |
| `warning` | Danger detected, caution |
| `overloaded` | Logic conflict, overwhelmed |
| `sleepy` | Idle too long |

Characters MAY add custom states (e.g., `coffee_break`, `victory_pose`). Unknown states received from engine → fall back to nearest canonical via manifest hint.

### 7.7 Adding a New Character (Contributor Flow)

**Target: <30 min for an artist familiar with PNG export.**

1. Clone template: `git clone shikigami/character-template`
2. Replace PNG frames in `assets/states/*/`
3. Edit `manifest.json` (name, id, preview)
4. Run `shikigami pack .` → outputs `my-character.shikigami`
5. Test: `shikigami install ./my-character.shikigami`
6. Publish: GitHub release or submit to registry

---

## 8. Animation Source Strategy (Deep Dive)

### 8.1 Tier 1: Sprite Sheet (Frame-Based) — Default & Community-First

**Why**: Lowest barrier, most inclusive format.

- **Source tools**: Aseprite, Photoshop, Procreate, Clip Studio, Krita — anything that exports PNG/WebP
- **Frame rate**: 8–24 fps recommended (not 60 — save bandwidth)
- **Format**: WebP preferred (smaller than PNG, supports alpha)
- **Resolution**: 512×512 or 1024×1024 @ 2x — downscaled at runtime
- **Atlas optimization**: CLI tool (`shikigami pack`) auto-generates sprite atlas for runtime perf

### 8.2 Tier 2: Live2D Cubism — Flagship Quality

**Why**: Industry standard for anime VTuber feel, smooth deformation.

- **Format**: Cubism 4.x `.model3.json` + `.moc3` + textures
- **SDK**: Official Cubism Web SDK (MIT-compatible for open source)
- **State mapping**: Manifest maps emotion states → Cubism parameters + motion files
- **License note**: Free Cubism models (CC) work fine; commercial models require Cubism license which users handle themselves

### 8.3 Tier 3 (Future): Spine, Lottie, VRM

- **Spine 2D**: `.skel` — premium rigging, paid editor, target indie game artists
- **Lottie**: `.json` — After Effects export, great for UI animations but limited for character
- **VRM**: 3D model rendered flat — bridge to VTuber ecosystem

### 8.4 Hybrid Characters

**Important**: A single character package can **mix renderers per state**!

```json
"states": {
  "idle": { "renderer": "live2d", "motion": "idle.motion3.json" },
  "warning": { "renderer": "sprite", "path": "assets/states/warning" }
}
```

Use case: Live2D for idle breathing (smooth), sprite frames for expressive warning animation.

### 8.5 Animation Performance Budget

Per-frame targets for always-on overlay:
- **GPU**: <2ms per frame (60fps budget)
- **CPU**: <5% of one core at idle
- **Memory**: <50MB per character loaded
- **Disk**: <30MB per `.shikigami` package (compressed)

Enforced via `shikigami validate <package>` CLI.

---

## 9. Implementation Phases

### Phase 0 — Foundation (Week 1)

- [x] Repo setup, README, .gitignore ✓
- [ ] Tauri 2 scaffold with React + TS
- [ ] Transparent always-on-top window POC
- [ ] ADR-001: Tauri vs Electron decision doc
- [ ] ADR-002: Character Package format decision

### Phase 1 — Core Engine (Week 2)

- [ ] Emotion Parser (Rust) with kaomoji regex table
- [ ] Emotion State Machine with transitions
- [ ] WebSocket server on `localhost:7796`
- [ ] Claude Code hook bridge script (`hooks/shikigami-hook.sh`)
- [ ] Unit tests: emotion parser ≥90% coverage

### Phase 2 — Renderer System (Week 3)

- [ ] `CharacterRenderer` interface
- [ ] `SpriteSheetRenderer` via PixiJS
- [ ] Manifest schema v1.0 + JSON Schema validator
- [ ] Character loader + hot reload
- [ ] First character: "Linh" (3 states: idle/happy/shy)

### Phase 3 — UX & Polish (Week 4)

- [ ] System tray menu
- [ ] Settings window (size/opacity/position)
- [ ] Character library browser
- [ ] Drag-and-drop `.shikigami` install
- [ ] macOS `.dmg` signed release

### Phase 4 — Live2D + Cross-Platform (Week 5-6)

- [ ] `Live2DRenderer` integration
- [ ] Windows `.msi` build + CI
- [ ] Linux AppImage build + CI
- [ ] GitHub Actions release pipeline

### Phase 5 — Community Launch (Week 7-8)

- [ ] Public registry prototype (static GitHub-hosted)
- [ ] `shikigami` CLI (`install`, `pack`, `validate`)
- [ ] Character template repo
- [ ] Landing page + demo video
- [ ] Submit to Hacker News / r/ChatGPTCoding / Twitter dev community

### Future (Post-v1) — Multi-AI Integration

- [ ] Cursor adapter
- [ ] Windsurf adapter
- [ ] Generic HTTP/webhook adapter with auth
- [ ] TTS integration (premium?)
- [ ] Cloud sync (premium tier)

---

## 10. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Live2D SDK licensing ambiguity for OSS | Medium | High | Start sprite-only; Live2D opt-in add-on with clear licensing docs |
| Tauri WebView inconsistencies across OS | Medium | Medium | Test WebGL matrix early; fallback to Canvas2D when needed |
| Low community character contribution | Medium | High | Invest in template repo + tutorial video + `shikigami pack` CLI ergonomics |
| Emotion parser accuracy on non-English text | High | Medium | Multi-language regex tables + future ML classifier option |
| Performance regression at scale (multi-character) | Low | Medium | Single-character-on-screen rule for v1; performance budget enforced in CI |
| Claude Code hook format changes | Low | High | Adapter pattern isolates parsing; version-detect hook version |
| Character pack supply chain risk (malicious package) | Medium | High | Manifest-only, no arbitrary code execution; sandbox third-party renderers |
| Burnout / scope creep | High | High | Strict non-goals list; "done is better than perfect"; community help |
| Rendering CPU/GPU hog on older machines | Medium | Medium | Enforce performance budget; adaptive framerate downshift |

---

## 11. Open Questions

- [ ] Default listen port — `7796` or let user configure? → **Decision: 7796 default, configurable**
- [ ] Character package signing/verification? → **v1: no signing; v2: optional signing via public keys**
- [ ] Do we ship with any characters or empty? → **Ship 1 default (Linh, sprite-based)**
- [ ] License choice: MIT vs Apache 2.0 vs GPL? → **Lean MIT for max adoption; characters CC-BY-SA**
- [ ] Name collision risk with existing "shikigami" projects? → **Check npm, pip, cargo, GitHub before v0.1**

---

## 12. Appendix

### 12.1 Glossary

- **Emotion State** — canonical token representing character mood (e.g., `happy`, `shy`)
- **Renderer** — plugin that draws a character given a state (Sprite, Live2D, ...)
- **Character Package** — `.shikigami` bundle (zip) containing manifest + assets
- **Manifest** — `manifest.json` describing character metadata + state → asset mapping
- **Hook Bridge** — script that subscribes to Claude Code events and POSTs to WS server

### 12.2 References

- Tauri 2: https://v2.tauri.app/
- Live2D Cubism SDK: https://www.live2d.com/en/sdk/
- PixiJS: https://pixijs.com/
- VSCode Extension format (inspiration): https://code.visualstudio.com/api/working-with-extensions/publishing-extension

---

**Adversarial review**: pending (Phase 2.5 skipped for draft; will run before v1.0 release)
**PRD Score (self)**: 88/100 — strong on extensibility & clarity; weak on monetization + multi-AI adapter detail

---

*"She watches. She listens. She reflects. Summoned by code, animated by soul."*
