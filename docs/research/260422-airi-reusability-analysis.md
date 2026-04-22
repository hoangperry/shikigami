# Research — airi (moeru-ai) Reusability Analysis for Shikigami

> **Date**: 2026-04-22 · **Author**: Claude Opus (moderator) + Gemini CLI + direct GitHub API
> **Repo under review**: https://github.com/moeru-ai/airi (MIT, TypeScript)
> **Target**: Shikigami (Tauri 2 + React 19 + PixiJS + Claude Code hooks)
> **Method**: `gh api` primary source + Gemini deep-research dispatch + direct source peeks. Codex dispatch timed out and was killed.

---

## 1. Executive Summary

airi is a **massive MIT-licensed monorepo** (38.5k ⭐, 38 packages + 6 apps) building a Neuro-sama-style VTuber companion. It is strategically aligned with Shikigami on several axes (Live2D/VRM rendering, emotion pipelines, character plugin systems, desktop overlay) but **architecturally divergent**: airi is Vue + Electron, Shikigami is React + Tauri.

**Verdict**: code cannot be straight-copied without heavy porting, but **patterns, protocols, and a few isolated packages are high-value**. The most direct reuse is:

1. **`@proj-airi/plugin-protocol`** — k8s-style plugin/module identity + JSON Schema config contracts. Port to Rust+TS or use directly from npm.
2. **The interleaved `[EMOTION:x]` prompt-tag pattern** (concept, not code) — adapt to Shikigami's text-texture layer.
3. **`unspeech`** (sibling repo) — run as external TTS proxy if/when FR-F01 voice lands, instead of writing our own.

**Do NOT adopt**: Electron window code (we have Tauri), Vue-coupled `Velin` prompting engine, the DuckDB memory layer (scope creep for v0.1).

---

## 2. Key Themes

### 2.1 airi is an AI-VTuber Platform, Not a Coding Companion

airi's north star is *"Re-creating Neuro-sama, a soul container of AI waifu"*. It games (Minecraft, Factorio, KSP), streams, handles Discord voice, maintains vector memory, and ships iOS/Android apps. Its stage is the *entire* user experience.

Shikigami is narrower: **visual proprioception for agentic workflows**. Our character reacts to Claude Code hook events. We share rendering + emotion overlap, but not streaming, gaming, or voice chat (v0.1).

[Source: airi repo description + homepage https://airi.moeru.ai/docs/]

### 2.2 Monorepo Is Mature and Modular

Primary data from `gh api repos/moeru-ai/airi`:

- **6 apps** — `stage-web` (Vue SPA), `stage-tamagotchi` (Electron desktop), `stage-pocket` (Capacitor mobile), `server`, `ui-server-auth`, `component-calling`
- **38 packages** — including `core-character`, `core-agent`, `plugin-protocol`, `plugin-sdk`, `stage-ui-live2d`, `stage-ui-three`, `model-driver-lipsync`, `model-driver-mediapipe`, `electron-eventa`, `electron-screen-capture`, `server-runtime`, `stream-kit`, `memory-pgvector`
- **pnpm workspaces + Turbo** build system

Each package has focused scope — some as small as 1–3 dependencies (e.g., `model-driver-lipsync` wraps `wlipsync`, a single wasm library).

[Source: `gh api repos/moeru-ai/airi/contents/packages` and `/apps`]

### 2.3 Tech-Stack Divergence Is the Blocker

| Area | airi | Shikigami |
|------|------|-----------|
| UI framework | Vue 3 / Nuxt | React 19 |
| Desktop shell | Electron | Tauri 2 (Rust backend) |
| Styling | UnoCSS | inline / CSS vars |
| Renderer | PixiJS + Three.js | PixiJS v8 (sprite-only v0.1) |
| Build | Vite + Turbo | Vite only |
| LLM transport | `xsai` (airi-owned) | direct hooks |
| State | Vue composables | Zustand |

Code-level reuse requires Vue→React porting and Electron→Tauri porting. **Framework-agnostic packages** (pure TS, no Vue imports) are the exception worth targeting.

### 2.4 The Plugin Protocol Is the Hidden Gem

`@proj-airi/plugin-protocol` defines **`PluginIdentity`, `ModuleIdentity`, `ModuleConfigSchema`** with k8s-style labels. This mirrors a problem Shikigami will face in v0.4+ when Cursor / Windsurf adapters arrive.

Concrete excerpt from `packages/plugin-protocol/src/types/events.ts`:

```ts
export interface PluginIdentity {
  id: string            // "telegram-bot", "stage-tamagotchi"
  version?: string
  labels?: Record<string, string>
}

export interface ModuleIdentity {
  id: string
  kind: 'plugin'
  plugin: PluginIdentity
  labels?: Record<string, string>  // k8s-style selectors
}
```

This gives a clean way to express "Shikigami adapter for Claude Code v1.2" without baking that string into the protocol. [Source: direct fetch of `packages/plugin-protocol/src/types/events.ts`]

### 2.5 Emotion Handling: airi Uses Prompt-Tags, Shikigami Uses Hook Events

airi uses its `Velin` engine to inject system prompts that force the model to emit `[EMOTION:joy]` tags. The frontend parses these from the stream.

Shikigami rejected text-primary signals (ADR-002 Hierarchical Fusion) — we use **structured events first, text textures second**. But airi's pattern **does** apply to our text-texture layer: when a persona-certified output style (`nekomata-engineer`) emits `(｡•̀ᴗ-)✧`, we're essentially doing interleaved-tag parsing.

[Source: Gemini analysis, §3 Problem-Solution Overlap]

---

## 3. Candidate Evaluation

### 3.1 Package-Level Adoption

| Package | License | Shape | Tech Alignment | Reuse Mode | Priority |
|---------|---------|-------|----------------|------------|----------|
| `@proj-airi/plugin-protocol` | MIT | Pure TS types + `@moeru/eventa` + `@xsai/shared-chat` | ✅ High — framework-agnostic | **(a) Install or port to schemas/** | 🟢 ADOPT — port to `schemas/plugin.v1.0.json` for v0.4+ multi-adapter work |
| `@proj-airi/stage-ui-live2d` | MIT | 28 deps (`@pixi/*`) | 🟡 Medium — PixiJS aligned, but Vue components wrap them | **(b) Port to React hooks** | 🟡 DEFER — pick up when Live2D add-on ships |
| `@proj-airi/model-driver-lipsync` | MIT | 1 dep (`wlipsync`) | ✅ High — framework-agnostic wasm wrapper | **(a) Install direct** | 🟢 FUTURE — valuable when TTS lands (FR-F01) |
| `@proj-airi/core-character` | MIT | `segmentation, emotion, delay, optional TTS` orchestrator; 3 deps | 🟡 Medium — pipeline concept, code body compiled | **(d) Inspire-only** | 🟡 STEAL — borrow the pipeline stage model |
| `@proj-airi/electron-eventa` | MIT | Electron IPC contracts | ❌ Low — Electron-only | **(d) Ignore** | ⚪ SKIP |
| `@proj-airi/stage-tamagotchi` (app) | MIT | 98 deps, Electron Main+Renderer | ❌ Low — Electron architecture | **(d) Inspire-only** | ⚪ SKIP for code; study for UX patterns |
| `@proj-airi/electron-screen-capture` | MIT | Electron-specific screen capture | ❌ Low | **(d) Ignore** | ⚪ SKIP — we'd write Tauri ScreenCaptureKit equivalent |
| `xsai` (sibling repo) | MIT | LLM provider router | ✅ High | **(a) Install** | 🟢 ADOPT — if/when Shikigami talks to LLMs directly |
| `unspeech` (sibling repo) | MIT | TTS/STT proxy service | ✅ High (runs as sidecar) | **(c) External service** | 🟢 FUTURE — when FR-F01 voice lands |

### 3.2 Pattern-Level Adoption (Ideas, Not Code)

1. **K8s-style labels on plugin identity** — `{ env: "prod", app: "claude-code", version: "1.2" }` → enables policy-driven routing later.
2. **Pipeline stage model** — `segmentation → emotion → delay → TTS`. Maps cleanly onto Shikigami's `event → dominant → texture → emit`.
3. **Interleaved emotion tags with streaming** — use for Shikigami's text-texture parsing when persona outputs are stream-parsed in v2+.
4. **Separation of "Soul" and "Stage"** — logic/LLM layer distinct from renderer. Already matches ADR-002 split; use airi as precedent if team ever debates collapsing the layers.
5. **Audio-visual sync via metadata** — attach emotion metadata to TTS request so the character animates *before* audio starts. File under FR-F02 lip-sync when relevant.

### 3.3 Things to Refuse

1. **Electron — any Electron code.** We have Tauri. Copying Electron overlay logic is negative value: bigger binary, worse resource footprint, more native quirks.
2. **Vue-coupled engines (`Velin`).** Introduces Vue compiler into an otherwise pure React stack. Use the idea, not the code.
3. **DuckDB / PGLite memory layers.** Scope creep. Shikigami's v0.1 character is not a persistent companion — it's a reactive indicator. Re-evaluate in v2 if long-running sessions need cross-session state.
4. **Game-playing agent crates.** Irrelevant to our scope.

---

## 4. Legal / Licensing Summary

- **airi core**: MIT ([Source: `gh api repos/moeru-ai/airi/license`](https://github.com/moeru-ai/airi/blob/main/LICENSE))
- **Compatibility with Shikigami**: MIT → MIT is trivially compatible
- **Attribution**: when adopting a specific file/pattern, credit in the adapted file's top comment per MIT convention
- **Third-party deps inside airi packages**: audit transitively before importing. Example flags:
  - `@pixi/*` are MIT (fine)
  - `wlipsync` — [Inference] likely MIT/Apache-2.0 but unverified
  - Live2D Cubism in `stage-ui-live2d` — [⚠️] depends on proprietary Live2D SDK; this is exactly the reason we deferred Live2D to an add-on (ADR-000)

[Inference]: adopting `stage-ui-live2d` is **not cleaner** than our current Live2D strategy because it ultimately still depends on the proprietary Cubism Core runtime. airi does not solve our licensing problem — they just ship the add-on inside the same repo.

---

## 5. Key Takeaways (Actionable)

1. **Port `plugin-protocol` types** into `schemas/plugin.v1.0.json` during Phase 4+ (multi-AI adapters) to get a mature k8s-style identity model instead of inventing one. Effort: ~1 day.

2. **Steal the `[EMOTION:x]` prompt-tag pattern** as a first-class texture trigger in `emotionOverrides` manifest section. When a persona ships, mandate a canonical tag vocabulary. Effort: ~half day + documentation.

3. **Keep Shikigami truly FOSS** by rejecting `stage-ui-live2d` as our Live2D entry point. ADR-000 (optional add-on in a separate repo) remains the right path; airi does not change that calculus.

4. **Consider `unspeech` as our TTS proxy** when FR-F01 voice support is scheduled. It solves the provider-abstraction problem we would otherwise spend weeks on. Deploy as sidecar process, not dependency.

5. **Do not adopt any Electron code.** Tauri has native advantages on macOS (transparency, resources); Electron patterns translate poorly.

6. **Adopt the "pipeline stage model"** as a documentation frame — add `docs/PIPELINE.md` describing `event → dominant → texture → emit` as discrete stages, inspired by airi's `core-character` naming. Effort: ~2h doc work.

7. **Star the repo and follow releases** — airi moves fast (commit activity high, 38.5k ⭐). Watch specifically for `plugin-protocol` and `stream-kit` evolutions.

---

## 6. Sources & Attribution

| Claim | Source | Type |
|-------|--------|------|
| airi is MIT, 38.5k stars, TypeScript | `gh api repos/moeru-ai/airi` | ✅ Verified |
| Monorepo structure (apps + packages + integrations) | `gh api repos/moeru-ai/airi/contents` | ✅ Verified |
| Package names in `packages/` and `apps/` | `gh api repos/moeru-ai/airi/contents/packages` + `/apps` | ✅ Verified |
| `plugin-protocol` PluginIdentity shape | Direct read of `packages/plugin-protocol/src/types/events.ts` | ✅ Verified |
| `core-character` description ("segmentation, emotion, delay, optional TTS") | `packages/core-character/package.json` description field | ✅ Verified |
| `model-driver-lipsync` wraps `wlipsync` | `packages/model-driver-lipsync/package.json` deps | ✅ Verified |
| `stage-tamagotchi` has 98 deps | `apps/stage-tamagotchi/package.json` dep count | ✅ Verified |
| `stage-ui-live2d` uses PixiJS | `packages/stage-ui-live2d/package.json` deps | ✅ Verified |
| `@proj-airi/realtime-audio` description | WebFetch of README.md | [Inference] — not verified directly |
| `xsai` is the LLM provider abstraction | Gemini analysis + README mention | [Inference] |
| `unspeech` is TTS proxy | Gemini analysis | [Opinion: Gemini] |
| `Velin` is Vue-SFC prompt orchestration | Gemini analysis | [Opinion: Gemini] — worth verifying before adopting |
| "Electron overlay has bigger binary than Tauri" | PRD §2 + common knowledge | [Source: Shikigami PRD / Tauri docs] |
| `stage-ui-live2d` still depends on proprietary Cubism Core | [Inference] from package description + ADR-000 | [Inference] |

---

## 7. Methodology

**Providers used:**
- 🟦 **`gh` CLI primary-source**: authoritative metadata and file contents from GitHub API
- 🟨 **Gemini CLI**: reusability analysis and architectural patterns (completed in ~50s, 780 words)
- 🔴 **Codex CLI**: dispatched for integration-risk analysis; **timed out and was killed** after >10min without output. Not represented in this report.
- 🟣 **WebFetch**: README extraction for high-level project summary

**Exploration paths:**
- `gh api repos/moeru-ai/airi` for metadata
- `gh api repos/moeru-ai/airi/contents/{packages,apps}` for monorepo tree
- Direct fetches of `package.json` for: `core-character`, `plugin-protocol`, `stage-ui-live2d`, `model-driver-lipsync`, `stage-tamagotchi`, `electron-eventa`
- Direct source fetch: `packages/plugin-protocol/src/types/events.ts`
- Gemini asked about license compatibility, reuse modes, patterns-vs-code, warnings

**Gaps / Not covered:**
- `xsai` and `unspeech` live in sibling repos and were not inspected directly. Treated as Gemini opinion, not verified.
- `core-character` source is effectively empty at `src/index.ts` (`export {}`) — the real implementation is compiled or split; not inspected.
- No runtime benchmarking comparing airi's electron binary size vs expected Tauri build. Deferred.
- Transitive dep audit (license + size) for packages marked "adopt" not performed.

**Cross-references:**
- Shikigami ADR-000 (licensing strategy) informed the Live2D refusal
- Shikigami ADR-002 (signal source) informed the text-tag pattern evaluation
- Shikigami PRD G10 (truly-OSS status) informed the `stage-ui-live2d` refusal

---

## 8. Open Questions

1. Does `@proj-airi/plugin-protocol` ship as a standalone npm package, or must it be consumed from the monorepo? If bundled, vendoring a single file is cleanest.
2. What is `wlipsync`'s license? If MIT/Apache-2.0, `model-driver-lipsync` becomes a direct adoption candidate for Phase 6+ lip-sync work.
3. Is `unspeech` actively maintained separately from airi? Self-hosted viability.
4. Would airi accept a PR porting part of `stage-ui-live2d` to framework-agnostic? Upstream alignment may be cheaper than fork.
5. Is there a Tauri integration effort anywhere in airi (`tauri` in package names)? Quick check `gh search code --repo moeru-ai/airi tauri` before concluding.
