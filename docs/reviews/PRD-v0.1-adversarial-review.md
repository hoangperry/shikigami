# PRD v0.1 — Adversarial Review Log

> **Date**: 2026-04-22
> **Method**: `/octo:prd` Phase 2.5 — parallel multi-provider adversarial challenge
> **Reviewers**: Codex CLI (GPT-5), Gemini CLI
> **Target**: `docs/PRD.md` v0.1 draft
> **Outcome**: Radical Cut adopted → PRD v0.2

---

## 1. Review Dispatch Method

Both reviewers received the complete PRD v0.1 with an identical challenge prompt asking them to:

1. Identify wrong/untested assumptions
2. Surface missing user scenarios
3. Find contradicting requirements
4. Predict first real user complaint
5. Flag the biggest ignored risk
6. Technical challenges with the Tauri + Live2D + WebSocket stack
7. Realism check on OSS growth goals
8. (Gemini) Competitive landscape + Live2D OSS licensing specifics
9. (Gemini) Kaomoji-based emotion parsing concerns

Ran in parallel as background tasks; both exit code 0.

---

## 2. Gemini CLI Review (Verbatim)

### Assumptions Likely Wrong

- **"Always-on-Top" Welcome**: You assume developers want more visual noise. In a high-focus "deep work" state, a bouncing 2D character is likely to be perceived as "Clippy 2.0" — distracting rather than supportive.
- **Artist Influx**: You assume artists will learn a new `.shikigami` manifest. Unless there is an automated "VTube Studio to Shikigami" converter, your Community Library will stay empty.
- **Trigger Frequency**: You assume AI emits enough "emotive" events. Most CLI outputs are dry logs. Without synthetic personality injection, the character will spend 99% of its life in `idle`.

### User Scenarios Missing

- **Multi-Monitor Drift**: How does the overlay handle full-screen IDEs or monitor swapping?
- **Focus-Stealing Bug**: What happens when a user accidentally clicks the character while trying to hit a scrollbar or a terminal tab?
- **Toxic Loop**: If the AI enters a repetitive error loop, the character's "confused" or "warning" animations will become an annoying strobe light.

### Requirements Contradicting

- **Ultra-lightweight (<50MB) vs. Live2D/Tauri**: Tauri is lean, but the moment you initialize a Live2D Cubism core and a PixiJS WebGL context, you are blowing past 50MB. High-res textures for 2K displays will eat VRAM.
- **100% Local vs. URL Installs**: Supporting "Direct install from URL" (FR-035) introduces a massive security vector for a "privacy-first" app.

### First Real User Complaint

> *"She's blocking my Close button and I can't click through her without disabling her entirely. Also, why is she smiling while my production database is being wiped?"*

### Biggest Ignored Risk

**UX Fatigue.** The novelty factor of a desktop pet wears off in ~48 hours. Without utility (e.g., the pet actually *explaining* the error via a speech bubble), it becomes shelfware.

### Competitive Landscape

- **VTube Studio / Mascot.js**: These are specialized. Shikigami's only play is the **Adapter Bridge**. If you don't own the AI-to-Emotion mapping better than a generic OBS overlay, you lose.
- **VMC Protocol Threat**: Why build a renderer? If you just emitted VMC/OSC data, users could use their *existing* $5,000 professional VTuber models in VTube Studio. You are reinventing the wheel (the renderer) instead of the engine (the parser).

### Live2D OSS Licensing Reality

**This is your Red Flag.**

- The Live2D Cubism SDK is **proprietary**. You cannot legally redistribute the Core SDK source code.
- While you can use the Web SDK, your "Open Source" project will depend on a closed binary blob. This prevents Shikigami from ever being included in many Linux distros (Fedora/Debian) or truly FOSS repositories.
- **Blocker**: The "Small-scale" free license has revenue caps and strict redistribution terms. You need a legal ADR on this *today*.

### Emotion Parsing Concerns

- **Kaomoji is a Niche**: Modern LLMs (Claude/GPT) don't use `(｡•̀ᴗ-)✧` unless explicitly system-prompted.
- **False Positives**: Parsing "Error" as warning is easy. Parsing "I'm sorry, I can't do that" (Refusal) vs "I'm sorry about the bug" (Apology) requires NLP, not Regex.
- **Multilingual**: Kaomoji/Emoji are universal, but action text (*sighs*, *pouts*) is language-dependent. v1 will likely be English-only despite the Global goal.

---

## 3. Codex CLI Review (Verbatim)

### Assumptions Likely Wrong

- You are overestimating the size of the problem. Most AI-coding users do not feel "lonely"; they feel overloaded. A companion overlay may read as distraction, not relief.
- You are assuming anime-adjacent aesthetics generalize. This is a niche on top of a niche: AI power users who also want a desktop pet beside serious work.
- You are assuming stable integration surfaces for Claude/Cursor/Windsurf before proving them. Those products change fast and often do not expose clean event APIs.
- You are assuming "<30 min for an artist" while requiring manifests, licensing, previews, state vocab, transitions, packaging, and validation.
- You are assuming `<80MB idle / <200MB peak` with Tauri + React + Pixi + Live2D + transparent overlay + hot reload across 3 OSes. That is optimistic bordering on fantasy.

### Missing User Scenarios

- Port `7796` already in use, corporate firewall rules, antivirus blocking localhost listeners.
- Hook install fails after CLI/tool update; user wants rollback, diagnostics, and recovery.
- Multi-monitor, mixed DPI, sleep/wake, fullscreen apps, screen sharing, recording, and OBS capture behavior.
- Accessibility and failure recovery: overlay becomes unclickable, stuck always-on-top, or disappears off-screen.
- Package corruption, bad manifests, partial installs, uninstall cleanup, schema migrations, and upgrading old character packs.
- Contributor pain: testing on Windows/Linux, validating licenses for Live2D assets, debugging renderer-specific bugs without a reproducible harness.

### Requirements Contradicting

- "Ultra-lightweight" vs dual renderer, hot reload, React UI, plugin system, Live2D, and cross-platform transparency.
- "Simple for artists" vs hybrid per-state renderers, manifest tuning, performance budgets, and CLI-based packaging.
- "Cross-platform from day one" vs platform support arriving in later phases.
- "100% local" vs URL installs, public registry, and no package signing.
- "No arbitrary code execution" vs loading third-party renderers from `~/.shikigami/renderers/*.js`.

### First Real User Complaint

> *"It's cute for five minutes, then it becomes random, noisy, and wrong. Why is it reacting to text patterns instead of what my agent is actually doing?"*

### Biggest Ignored Risk

This is a delight product without a proven core habit. If reactions are wrong even 10-20% of the time, trust collapses. Delight tools get uninstalled fast when they add friction to a workflow that users already tolerate.

### Technical Issues with Tauri + Live2D Web SDK + WebSocket

- **FR-001 is internally wrong**: a WebSocket server does not "accept POST events". You have not chosen a clean event transport.
- Transparent always-on-top WebGL windows are notoriously inconsistent across macOS/Windows/Linux.
- Live2D in a Tauri webview adds GPU, texture, and lifecycle complexity that fights your memory targets.
- Localhost WebSocket without auth is a local attack surface; "generic HTTP adapter with auth" is deferred, but the risk exists in v1.
- Mixed-renderer states create timing, transition, and asset-loading edge cases that will be hard to make feel smooth.

### OSS / Community Realism

- `1000` stars in 6 months is possible only with exceptional art, a viral demo, and relentless distribution. It is not a sensible baseline metric for this niche.
- A more credible target: `100–300` stars, `20–50` active users, `3–5` quality external character packs, and one integration that works flawlessly.
- **Constructive reset**: ship sprite-only, one tool integration, one great character, no third-party renderers, and prove retention before expanding scope.

---

## 4. Triage & Action Matrix

Severity legend: **🔴 Critical** (blocks v1) / **🟡 High** (must handle v1) / **🟢 Medium** (v2+)

### 4.1 Accepted (Apply to v0.2)

| # | Challenge | Source | Sev | Action |
|---|-----------|--------|-----|--------|
| C1 | Kaomoji parsing is fragile — LLMs don't emit these | Both | 🔴 | **Pivot to event-driven signals** (ADR-002) |
| C2 | Live2D SDK proprietary — blocks OSS purity | Gemini | 🔴 | **Sprite-only v1, Live2D becomes optional add-on** (ADR-000, ADR-004) |
| C3 | FR-001 WebSocket "accepts POST" is wrong | Codex | 🔴 | **Redesign transport** (ADR-001) |
| C4 | 1000 stars unrealistic | Codex | 🔴 | Recalibrate goals: 200 stars, 20 retention users, 3 community packs |
| C5 | Third-party renderer contradicts no-arbitrary-exec | Codex | 🔴 | **Remove FR-014 from v1** |
| C6 | URL install contradicts privacy-first | Both | 🔴 | **Remove FR-035 from v1** |
| C7 | UX fatigue in 48h | Gemini | 🔴 | **Add utility — speech bubbles with context info** (new FR-055) |
| C8 | Context-blind state ("smiling during DB wipe") | Gemini | 🔴 | **Severity-aware states** (new FR-056) |
| C9 | Scope too big for 4-week MVP | Both | 🔴 | **Mac-only v0.1, Windows v0.2, Linux v0.3** |
| C10 | Port conflict / firewall / antivirus | Codex | 🟡 | New FR-050 — conflict recovery |
| C11 | Multi-monitor + mixed DPI + fullscreen + OBS | Both | 🟡 | New FR-051, FR-052 |
| C12 | Lost overlay recovery | Codex | 🟡 | New FR-053 |
| C13 | Package corruption / bad manifest | Codex | 🟡 | New FR-054 — graceful fallback |
| C14 | Localhost transport without auth | Codex | 🟡 | Add auth token in ADR-001 |
| C15 | Toxic loop strobe (repeated warning) | Gemini | 🟡 | State transition dampening |
| C16 | Artist onboarding pain | Both | 🟡 | Ship `shikigami pack` CLI + template repo + tutorial video |
| C17 | Idle variety needed | Gemini | 🟢 | New FR-057 — random micro-interactions |
| C18 | Multilingual action text | Gemini | 🟢 | Defer to v2, English-only v1 (honest scoping) |

### 4.2 Rejected / Reframed

| # | Challenge | Source | Reason |
|---|-----------|--------|--------|
| R1 | "Developers feel overloaded, not lonely" | Codex | **Reframed, not rejected**: Shikigami repositioned as *"visual proprioception for agentic workflows"* — status indicator, not emotional companion. Still valuable. |
| R2 | "Pivot to VMC protocol bridge — don't build renderer" | Gemini | **Partial**: Added as v3+ backlog (FR-F10). VMC export is strategic but requires users install VTube Studio separately — removes plug-and-play out-of-the-box experience that we want for v1. |
| R3 | "Anime aesthetic is niche-on-niche" | Codex | **Partial**: True, but it's the founder's intended audience. Honest positioning ≠ broadening. |

---

## 5. Key Decisions Made

- **ADR-000**: Live2D excluded from v1 core. Optional add-on installed separately when user wants it.
- **ADR-001**: Transport = **local HTTP POST with auth token on `127.0.0.1`**, not WebSocket server. Simpler, matches event semantics, bearer token mitigates local attack surface.
- **ADR-002**: Signal source = **event-driven primary** (tool calls, exit codes, git ops) with text-parse as secondary fallback.
- **ADR-003**: `.shikigami` package format retained, sprite-focused schema v1.0.
- **ADR-004**: v1 scope = **Mac + sprite + Claude Code + 1 character**. No third-party renderers, no URL install, no public registry, no multi-AI adapters.

---

## 6. PRD v0.2 Summary

**Tagline shift**: "Your AI agent's emotional companion" → **"Visual proprioception for agentic workflows"**

**Scope cut**: ~40% of v0.1 features moved to backlog.

**New goals**: 200 stars / 20 retention users / 3 community packs / 95% correct state reactions in 3 months.

**Reviewers' verdict on v0.2 plan**: pending — will re-run before v1.0 release tag.

---

## 7. Review Artifacts

- Gemini output: background task `bzf4njfx6`
- Codex output: background task `bbz8ru71i`
- Both completed successfully with exit code 0
