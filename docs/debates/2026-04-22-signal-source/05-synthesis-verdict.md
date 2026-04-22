# Synthesis & Verdict

**Moderator**: 🔵 Claude Opus
**Date**: 2026-04-22
**Topic**: Signal source for Shikigami — Event-driven vs Text-parse vs Equal Hybrid vs LLM Classifier

---

## 1. Strongest Argument from Each Defender

| Position | Defender | Strongest Argument |
|----------|----------|-------------------|
| **A — Event-Driven Primary** | Claude Opus | 95% corpus reality: most users run default Claude Code, not themed styles. Text-parse returns `idle` for them. Events work universally. |
| **B — Text-Parse Primary** | Sonnet | Persona packs are *structured contracts* — `nekomata-engineer` guarantees kaomoji at state transitions. That's not brittle pattern matching, that's schema parsing on a soft schema. |
| **C — Equal Hybrid** | Gemini | Events and text are *orthogonal, not redundant*. Skeleton vs flesh. Only a fusion architecture can express "relieved exhaustion" (commit succeeded + message contained "finally"). |
| **D — LLM Classifier** | Codex (proxy) | All rival positions decay the same way — as AI tools and personas fragment, rule-based adapters multiply O(tools × personas × locales). The LLM is the only O(1) long-term solution. |

## 2. Attacks That Landed

### On Position B (Text-Parse Primary) — Fatal
- **95% corpus fatality**: Sonnet's defense relies on users running themed output styles. But Shikigami targets *every Claude Code user who installs the hook*. For the default user, text-parse primary returns `idle` always. This kills Position B as *primary* signal — it remains valid as a *texture layer*.
- Additional: Position A's destructive-op attack ("smiling while DB is wiped") is unanswerable from pure text-parse — the `rm -rf` event fires *before* any warning text arrives.

### On Position D (LLM Classifier) — Scope, not Vision
- **Budget violation**: 500MB–2.2GB model blows v0.2's <80MB idle / <200MB peak targets by 10×. Codex's sidecar defense is plausible but out-of-scope for v0.1's 4-week MVP.
- **Ship-date reality**: Prompt-only classifier + llama.cpp integration + macOS notarization of an LLM add-on is a 4–6 week side quest. Not realistic now.
- Verdict: Position D is the correct *v2+ roadmap*, not the v0.1 architecture.

### On Position C (Equal Hybrid) — Partial
- **Magic-number defense is weak in pure form**: Gemini's rebuttal ("calibration parameters, not magic numbers") is clever but unfalsifiable. Without a real calibration dataset, tuning 50/50 vs 60/40 becomes vibes-based debugging.
- **But**: the orthogonality insight is correct and *survives the attack*. Fusion with **hierarchical precedence** (not equal weight) preserves the orthogonality benefit while eliminating deadlock.

### On Position A (Event-Driven Primary) — Reductive Framing
- **"Character-skinned status bar" critique lands**: pure event-driven collapses emotional expression to the event vocabulary. Position A's pre-emptive defense ("events gate, text textures") is *actually not pure Position A — it's a hierarchical hybrid*. This admission reveals that pure Position A loses on differentiation.

## 3. Verdict — Hierarchical Fusion (Event-Gated, Text-Textured)

**None of the four positions wins in pure form.** The debate forces a synthesis that takes the best of A and C while noting B's persona-contract insight and shelving D for v2+.

### The Winning Architecture

```
Event arrives → Severity + Event Type → DOMINANT STATE (canonical)
                                              │
                                              ▼
           Text within event.text field → SUB-STATE / TEXTURE
                                              │
                                              ▼
                            Final state: <dominant>_<texture?>
                            Duration + intensity scaled by severity
```

**Concrete examples:**

| Event | Text field | Dominant State | Sub-state texture | Final |
|-------|------------|----------------|-------------------|-------|
| `tool_complete` exitCode 0 (git_commit) | "fix critical bug, finally" | `happy` | `relieved` (from "finally") | `happy_relieved` — sigh + smile |
| `tool_complete` exitCode 0 (git_commit) | "minor typo" | `happy` | — | `happy` — default brief smile |
| `destructive_op_detected` rm -rf | *anything* | `warning` (critical) | — ignored | `warning_critical` — alarm, no texture override |
| `tool_start` Bash | "⚠️ this is destructive" | `focused` | `alarmed` (from ⚠️) | `focused_alarmed` |
| `session_idle_long` 5min | "*em cắn nhẹ đầu bút*" | `sleepy` | `cute` (from action text) | `sleepy_cute` |

### Why This Wins

1. **Position A's concerns are preserved**: events fire first and set the dominant state. Destructive-op signals cannot be overridden by benign text. Scales O(tools) for adapters.
2. **Position B's insight is absorbed**: persona-certified packs emit guaranteed markers that map to sub-state textures. Themed power users get their kaomoji blushes.
3. **Position C's orthogonality is honored**: events and text are treated as orthogonal input streams, not redundant. Fusion produces complex states neither channel alone can express.
4. **Position D's lesson is scheduled**: when rule-based adapter maintenance becomes real pain (v2+, new AI tools), `shikigami-mind` sidecar ships as optional LLM classifier add-on. Architecture doesn't have to change — the classifier emits the same event schema.

### Required Updates

- **ADR-002 revision**: rename from "Event-Driven Primary" to **"Hierarchical Fusion — Event-Gated, Text-Textured"**. The substance is only lightly changed; the naming honestly reflects what the architecture actually does.
- **New FR-058**: Sub-state texture layer. Each canonical state MAY have one or more texture modifiers that compose into a final animation key.
- **New FR-059**: Texture extraction via lightweight regex on `event.text` field, running in parallel to state mapping, with texture priority below severity (critical events never accept softening texture).
- **Backlog FR-F11**: `shikigami-mind` optional LLM classifier sidecar (v2+).
- **Persona certification criteria**: themed output styles that want first-class sub-state texture support can register their marker vocabulary (e.g., `nekomata-engineer.manifest.json` declares emoji → texture mappings). Community-maintained library.

## 4. Participation Quality Notes

- 🟣 **Sonnet**: Strongest single argument of the tournament ("persona packs as structured contracts"). Lost the debate on 95% corpus but the persona-contract insight is directly folded into the winning architecture.
- 🟡 **Gemini**: Best framing of the tournament ("skeleton vs flesh"). The orthogonality insight is the keystone of the synthesis. The magic-number defense was the weakest part.
- 🔴 **Codex (proxy)**: Cleanest long-term vision. The budget attack was well-defended but unrealistic for v0.1 ship date. Correctly identified as v2+ roadmap, not losing.
- 🔵 **Opus (self)**: My pre-emptive defense of Position A implicitly conceded Position C by describing a pipeline rather than pure events. Honest synthesis required admitting that.

## 5. Cost & Time

- Gemini CLI: ~40s wall clock, free tier
- Codex CLI: timed out / killed after 15+ min without output (proxy written by Opus covering the argument)
- Sonnet Agent: ~30s wall clock
- Total moderator time: ~10 min drafting + synthesis

## 6. Outcome

**Ship v0.1 with Hierarchical Fusion architecture. Revise ADR-002. Add FR-058/059. Position D parked for v2+.**

No other position could produce the full synthesis alone — the tournament was load-bearing for the decision.
