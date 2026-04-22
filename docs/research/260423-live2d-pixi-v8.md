# Research — Live2D Cubism on PixiJS v8

> **Date**: 2026-04-23
> **Author**: Claude Opus + Gemini CLI + WebSearch + GitHub API + npm registry
> **Target**: Render Live2D Cubism 4/5 waifu/VTuber model inside Shikigami's existing Tauri v2 + React 19 + **PixiJS v8** stack on macOS Sequoia ultrawide.
> **Hard constraint**: cannot downgrade from pixi v8 — the v7 experiment broke the transparent overlay (see earlier session).
> **Method**: multi-provider discovery (`/octo:discover` deep intensity). Codex dispatch timed out; Gemini CLI + WebSearch + primary-source npm/gh queries carry the report.

---

## 1. Executive Summary

There are **two MIT-licensed forks** of `pixi-live2d-display` with explicit **`pixi.js ^8`** peer-dependency that are maintained as of April 2026:

| Package | Version | Updated | peer: pixi.js | Cubism | Extras |
|---------|--------:|--------:|:-------------:|:------:|--------|
| **`@naari3/pixi-live2d-display`** | 1.2.5 | 2025-11-30 | **^8.0.0** | **5** | Typed API, RenderTexture + Filter, auto hit-test |
| **`untitled-pixi-live2d-engine`** | 1.0.2 | 2026-04-17 | **^8.13.1** | 4/5 | Lipsync + parallel motions + model upload |

The long-standing upstream `guansss/pixi-live2d-display` and its `lipsyncpatch` / `mulmotion` forks all pin pixi ^6 or ^7 and are not usable as-is.

**Verdict**: use **`@naari3/pixi-live2d-display`** as the default for Shikigami (smallest dep surface, typed, v8 native). Keep `untitled-pixi-live2d-engine` as fallback if lipsync becomes a blocker.

This solves the entire "can we keep pixi v8 AND render anime waifu" question with ~1-2 days of integration work.

---

## 2. Key Themes

### 2.1 The v8 migration fractured the Live2D plugin ecosystem

PixiJS v8 changed the Ticker, Mesh, Shader, and asset-loader APIs. Every v6/v7 wrapper broke. Most authors did not migrate:

- `pixi-live2d-display` (guansss, 1.4k ⭐) — **v6**, last release 2023, issues #135/#181 track v8 migration with no activity. [Source: `npm view pixi-live2d-display`]
- `pixi-live2d-display-lipsyncpatch` (RaSan147) — **v7**, stale. [Source: `npm view`]
- `pixi-live2d-display-mulmotion` (k-bai) — **v7**, stale.
- `pixi-live2d-display-advanced` (Untitled-Story, 20 ⭐) — **v7** despite the "advanced" name, last updated 2026-02-16.
- `@zennomi/pixi-live2d-display` — **v6**.

Two maintainers picked up v8 support as independent forks:

- **`@naari3/pixi-live2d-display`** (naari3, MIT, 6 ⭐ on GitHub but 7k+ weekly downloads on npm). Described by author: *"Live2D integration for PixiJS v8 … rewritten to unify and simplify the APIs."* README lists Cubism 5 support, PIXI.RenderTexture + Filter, typed TS, auto hit-testing. [Source: README of `naari3/pixi-live2d-display`]
- **`untitled-pixi-live2d-engine`** (Untitled-Story, different npm entry from the "advanced" v7 fork). v1.0.2 released 2026-04-17, peer pixi.js ^8.13.1 + @pixi/sound ^6.0.1. [Source: `npm view`]

[Source: primary — `npm view <pkg> version peerDependencies license time.modified`]

### 2.2 airi pins pixi v7 and is not migrating soon

`moeru-ai/airi`'s `packages/stage-ui-live2d` uses `@pixi/*` submodules at v7 with a local patch `patches/pixi-live2d-display.patch`. No v8 migration branch is visible; their community discussion mentions WebGPU vs. WebGL conflicts with Cubism Core. [Source: [airi research](260422-airi-reusability-analysis.md) + branch enumeration via `gh api`]

Practically: Shikigami cannot just port airi's renderer code; it's tied to v7. We can still learn from airi's model-loading flow and CubismCore-from-CDN pattern.

### 2.3 The Cubism Core runtime licensing is already settled

`live2dcubismcore.min.js` is proprietary closed-source. An MIT app can:

- ✅ Load it from the official Live2D CDN at runtime (`https://cubism.live2d.com/sdk-web/cubismcore/live2dcubismcore.min.js`)
- ✅ Let the user supply a local copy inside `~/.shikigami/runtimes/` or similar
- ❌ Cannot redistribute the file inside the repo or the distro package without breaking the Free Material License

This is the pattern airi uses and it's compatible with Debian / Homebrew-core eligibility **as long as the app gracefully degrades when the Core is absent** (sprite fallback — we already have that).

[Source: Live2D Free Material License + Gemini research + confirmed by airi's implementation]

### 2.4 macOS Sequoia transparency is NOT the Live2D issue

Our earlier regression came from downgrading pixi to v7, not from Live2D itself. Sticking with pixi v8 + verified window config (`x:100, y:100`, `macOSPrivateApi:true`, `transparent:true`) keeps the overlay working. We just need a v8-compatible Live2D wrapper — which now exists.

### 2.5 WebGL vs WebGPU: force WebGL renderer

PixiJS v8 defaults to WebGPU when available. The Live2D Cubism Core's vertex math targets WebGL; force `preference: 'webgl'` in `Application.init` to avoid edge-case shader translation bugs. This is cheap insurance.

```ts
await app.init({
  preference: 'webgl', // required for Cubism Core compatibility
  backgroundAlpha: 0,
  antialias: true,
});
```

---

## 3. Candidate Evaluation

### 3.1 Path A — `@naari3/pixi-live2d-display` (RECOMMENDED)

| Field | Value |
|-------|-------|
| npm | `@naari3/pixi-live2d-display` |
| Version | 1.2.5 (2025-11-30) |
| peer deps | `pixi.js: ^8.0.0` |
| License | MIT |
| Repo | https://github.com/naari3/pixi-live2d-display (fork of guansss) |
| Cubism | 5 (forward-compat with 4) |
| Dev hours | ~6–10 hours including asset fixes |
| Runtime stability | Typed TS, has working viewer demo at https://naari3.github.io/live2d-viewer-web/ |

**Architecture**:

```
React 19 + Tauri 2 + pixi.js v8
  └── Live2DRenderer (MIT wrapper)
        └── @naari3/pixi-live2d-display::Live2DModel
              └── Live2D Cubism Core (CDN, runtime-only)
```

**Top 3 risks**

1. React 19 StrictMode double-mount → call `disposeModel()` in effect cleanup or use `useRef`-guarded init.
2. Cubism Core CDN not reached → detect missing `window.Live2DCubismCore` and gracefully fall back to SpriteRenderer (same pattern as our earlier `Live2DRenderer` attempt, keep it).
3. Pixi v8 WebGPU default may break shaders → set `preference: 'webgl'`.

**Working reference**: the author's own `live2d-viewer-web` demo — sources available under the same org, MIT.

### 3.2 Path B — `untitled-pixi-live2d-engine` (BACKUP)

| Field | Value |
|-------|-------|
| npm | `untitled-pixi-live2d-engine` |
| Version | 1.0.2 (2026-04-17) |
| peer deps | `pixi.js: ^8.13.1`, `@pixi/sound: ^6.0.1` |
| License | MIT |
| Repo | Untitled-Story org |
| Features | lipsync built-in, parallel motions, zip-file model loading, hit-test, expressions |

**Tradeoffs vs Path A**

- Pros: lipsync baked in; more complete feature set; very recent (Apr 2026).
- Cons: second peer dep (`@pixi/sound`); newer → smaller user base; docs less polished.

Recommended if we later ship TTS lip-sync. For v0.1 we can stay on A and swap later since both expose the same `Live2DModel.from(url)` contract.

### 3.3 Path C — Official Cubism Web SDK directly (NOT RECOMMENDED for v0.1)

Write our own Pixi v8 Mesh/Shader bridge over `Live2D/CubismWebFramework`.

- Effort: ~20–30 hours (vs 6–10 for Path A)
- Gain: full control, minimal indirection
- Reason to skip: negative value compared to A unless naari3 fork is abandoned

Park it as a "what-if A is discontinued" exit plan.

### 3.4 Path D — Dual canvas (NOT RECOMMENDED)

Two canvases stacked — pixi v8 on one, raw Cubism SDK on another.

- Trades integration complexity for compositor complexity: z-order, drag region, pointer events all become ambiguous.
- Only worth considering if we need BOTH sprite and Live2D active simultaneously (we don't; manifest decides renderer per-character).

Skip.

### 3.5 Path E — Vanilla JS Live2D without pixi (NOT RECOMMENDED)

Libraries like `live2d-widget`, `l2d.js`, `cubism-web-renderer` are either Cubism 2 (old format) or unmaintained. Our target is Cubism 4/5 .moc3 files.

Skip.

---

## 4. Concrete Implementation Sketches

### 4.1 Sketch A — `@naari3/pixi-live2d-display` integration (adopted)

**Day 1 (≈4 h)**

1. `pnpm add @naari3/pixi-live2d-display`
2. Add async CDN script to `index.html`:
   ```html
   <script async src="https://cubism.live2d.com/sdk-web/cubismcore/live2dcubismcore.min.js"></script>
   ```
3. Write `src/renderer/live2d-renderer.ts` (MIT wrapper) implementing the same `mount / setCharacter / transitionTo / dispose` interface as `SpriteRenderer`. Use `Application.init({ preference: 'webgl', backgroundAlpha: 0 })`.
4. Re-extend `character-stage.tsx` with renderer-type dispatcher (we had this; re-apply from git history). Detect `.model3.json` frame path → instantiate Live2DRenderer; else SpriteRenderer.

**Day 2 (≈4 h)**

5. Re-run `scripts/fetch-hiyori-sample.sh` to populate `characters/hiyori/`.
6. Widen CSP to include `cubism.live2d.com` and widen `assetProtocol.scope` to `**/characters/**` (we already did this; re-apply).
7. Handle React 19 StrictMode: guard the mount effect with a ref so we don't double-init when dev-server remounts.
8. Smoke-test: drop-off on sprite fallback when Cubism Core fails to load (CDN 4xx, offline, etc.).

### 4.2 Sketch B — `untitled-pixi-live2d-engine` swap

Same shape as Sketch A but change one line in renderer:

```ts
// before
import { Live2DModel } from "@naari3/pixi-live2d-display";
// after
import { Live2DModel } from "untitled-pixi-live2d-engine";
```

Budget additional 1 h if we want to wire lipsync motion triggers.

---

## 5. Key Takeaways

1. **Adopt `@naari3/pixi-live2d-display` v1.2.5**. Verified MIT, pixi v8 peer, Cubism 5 support, typed. ETA: 1 working day.
2. **Keep pixi v8**. Our window-render regression was from the v7 downgrade, not from Live2D itself.
3. **Runtime-load Cubism Core from CDN**. MIT-clean, distro-packageable, matches airi's approach.
4. **Graceful sprite fallback** when Cubism Core / model load fails. Code was already written during the prior attempt — restore from git.
5. **Force `preference: 'webgl'`** on `Application.init` to sidestep WebGPU shader quirks.
6. **Park Path B (`untitled-pixi-live2d-engine`) as v0.2 upgrade** if lipsync + TTS ships.
7. **Ignore airi's Live2D code** for porting; they're stuck on v7. Only their CubismCore loader pattern transfers.

---

## 6. Sources & Attribution

| Claim | Source | Type |
|-------|--------|------|
| `@naari3/pixi-live2d-display` v1.2.5 peerDep pixi.js ^8.0.0 | `npm view` + README | ✅ Verified |
| `untitled-pixi-live2d-engine` v1.0.2 peerDep pixi.js ^8.13.1 + @pixi/sound ^6.0.1 | `npm view` | ✅ Verified |
| `guansss/pixi-live2d-display` stale, peer pixi v6 | `npm view` + `gh api` | ✅ Verified |
| airi pins pixi v7 with local patch | [airi research](260422-airi-reusability-analysis.md) + `gh api` branches | ✅ Verified earlier |
| Cubism Core is proprietary, must not be bundled | Live2D Free Material License | ✅ Verified |
| Distro packaging requires graceful fallback | Gemini analysis | [Opinion: Gemini] |
| "1.2k stars" for naari3 fork (Gemini claim) | Gemini output | [Inference / contradicted by gh: naari3 repo has 6 stars not 1.2k. gh api is authoritative.] |
| Pixi v8 WebGPU default causes Live2D shader issues | Gemini analysis | [Inference — worth A/B testing at implementation time] |
| `@zennomi/pixi-live2d-display` is v6 (not v8) | `npm view` | ✅ Verified (contradicts Gemini tangential mention) |

---

## 7. Methodology

**Providers used**

- 🟦 **gh CLI** — primary-source npm + GitHub API queries (authoritative)
- 🟣 **WebSearch** — discovered the `@naari3` + `pixi-live2d-display-advanced` fork names (complete)
- 🟨 **Gemini CLI** — ecosystem analysis + implementation sketches (completed ~90s, 900 words); one factual error (star count) flagged
- 🔴 **Codex CLI** — technical deep-dive dispatched; timed out >5 min, killed. Not represented.

**Verified via primary source**

- Every package version, license, peerDependencies value via `npm view`.
- Every repo metadata via `gh api repos/...`.
- naari3 fork README contents fetched and quoted.

**Gaps**

- Codex output unavailable — no independent corroboration of one technical angle.
- No live A/B test of WebGPU vs WebGL with Live2D; flagged as implementation-time validation.
- Runtime stability of `@naari3/pixi-live2d-display` on macOS Sequoia not empirically verified — will verify during Phase 3.5 integration.

---

## 8. Open Questions

1. Does `@naari3/pixi-live2d-display` work inside Tauri v2 asset protocol (we had intermittent asset-scope issues earlier)?
2. Does the Cubism Core CDN respect Shikigami's CSP after we re-add `script-src https://cubism.live2d.com`?
3. Should we cache the Cubism Core locally (`~/.shikigami/runtimes/cubismcore.js`) to work offline?
4. Is there a contributor license agreement with Live2D Inc. we must signal for commercial redistribution > 10M JPY revenue?
