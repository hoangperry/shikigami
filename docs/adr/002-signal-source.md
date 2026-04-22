# ADR-002: Signal Source — Hierarchical Fusion (Event-Gated, Text-Textured)

**Status**: ✅ Accepted (revised after Steelman Tournament debate)
**Date**: 2026-04-22 (v1 accepted); revised same day after 4-way debate
**Deciders**: @hoangperry
**Informed by**: Codex + Gemini adversarial reviews (kaomoji fragility); 4-way Steelman Tournament debate `docs/debates/2026-04-22-signal-source/`

## Context

PRD v0.1 specified an **emotion parser** that detects kaomoji (`(｡•̀ᴗ-)✧`), *action text* (`*em nghiêng đầu*`), and keywords as the primary signal source for character state.

Both adversarial reviewers flagged text-parse-primary as the weakest foundation of the product. The initial revision (v1 of this ADR) pivoted to **Event-Driven Primary** with text-parse as an opt-in fallback only.

A subsequent 4-way Steelman Tournament (Claude Opus, Sonnet, Gemini CLI, Codex proxy) debated four positions:
- **A** Event-Driven Primary (v1 of this ADR)
- **B** Text-Parse Primary (v0.1 original)
- **C** Equal Hybrid (fusion classifier)
- **D** LLM Classifier (small local quantized model)

The debate revealed that pure Position A was **reductive** — it collapsed emotional expression to the event vocabulary and produced "a character-skinned status bar." Position A's own pre-emptive defense ("events gate, text textures") was *already* a hierarchical hybrid. Honest synthesis required renaming the architecture.

Position D was correct long-term but budget-infeasible for v0.1. Position B was fatal on the 95% corpus reality (most users do not run themed output styles). Position C's orthogonality insight — events as skeleton, text as flesh — became the keystone of the synthesis.

## Decision

Adopt **Hierarchical Fusion** as the canonical architecture.

### Pipeline

```
┌──────────────────────────────────────────────────────────┐
│ Event arrives (tool_start, tool_complete, error, ...)   │
└──────────────────────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ STAGE 1 — Event + Severity → DOMINANT STATE (canonical)  │
│                                                          │
│ Maps to one of 9 canonical states:                       │
│   idle / happy / focused / warning / confused /          │
│   sleepy / shy / flirty / overloaded                     │
│                                                          │
│ Critical severity LOCKS the state (skip Stage 2).        │
└──────────────────────────────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────────────┐
│ STAGE 2 — Text Texture (from event.text if present)      │
│                                                          │
│ Runs in parallel regex on text field.                    │
│ Produces zero or more TEXTURE MODIFIERS:                 │
│   relieved / playful / exhausted / alarmed / cute / ...  │
│                                                          │
│ Composed into final animation key:                       │
│   <dominant>[_<texture>]                                 │
└──────────────────────────────────────────────────────────┘
               │
               ▼
  Final state: happy_relieved / warning_critical /
               focused_alarmed / sleepy_cute / idle
```

### Concrete Examples

| Event | Text field | Dominant | Texture | Final State |
|-------|-----------|----------|---------|-------------|
| `git_commit` | "fix critical bug, finally" | `happy` | `relieved` (from "finally") | `happy_relieved` — sigh + smile |
| `git_commit` | "minor typo" | `happy` | none | `happy` — default brief smile |
| `destructive_op_detected rm -rf` | *anything* | `warning` (critical) | ignored | `warning_critical` — alarm, no softening |
| `tool_start Bash` | "⚠️ this is destructive" | `focused` | `alarmed` (from ⚠️) | `focused_alarmed` |
| `session_idle_long` 5min | "*em cắn nhẹ đầu bút*" | `sleepy` | `cute` (action text) | `sleepy_cute` |
| `tool_complete` exitCode 0 | (no text) | `happy` | none | `happy` (default) |

### Core Rules

1. **Events always fire first.** The dominant state is decided before any text is parsed.
2. **Severity is supreme.** `critical` severity locks the dominant state and skips texture modifiers. This prevents "smiling while the database is wiped."
3. **Texture is optional.** Most events produce a dominant state with no texture. Missing `event.text` is a valid common case — the animation key falls back to `<dominant>` alone.
4. **Texture vocabulary is small and canonical.** v0.1 ships ~6 textures: `relieved`, `playful`, `exhausted`, `alarmed`, `cute`, `smug`. Characters may ignore textures they don't implement — graceful fallback to dominant.
5. **Persona-certified styles** may register a marker vocabulary (e.g., `nekomata-engineer.manifest.json`) declaring emoji/action-text → texture mappings. Community-maintained.

### Character Compatibility

Character packages declare which textures they support per state:

```json
"states": {
  "happy": {
    "path": "assets/states/happy",
    "fps": 15,
    "textures": {
      "relieved": "assets/states/happy_relieved",
      "playful":  "assets/states/happy_playful"
    }
  }
}
```

Unknown or unsupported textures fall back to the dominant state's base animation with a debug log entry. Contributors are never forced to ship textures — minimum viable character still = `idle + happy`.

## Consequences

### Positive

- **Context-correctness preserved** — events decide dominant state; destructive ops cannot be overridden by benign text (the Gemini "DB wipe while smiling" attack is eliminated).
- **Emotional nuance unlocked** — characters express complex states (`happy_relieved`, `focused_alarmed`) that neither pure channel can produce alone.
- **Scales O(tools)** for adapters — new AI tool = new event adapter, not new regex library.
- **Works for 95% corpus** — users on default Claude Code output see correct states even with zero text parsing.
- **Delights persona power users** — themed output styles (`nekomata-engineer`, `sexy-secretary`) light up texture modifiers their authors designed for.
- **No deadlock / oscillation** — hierarchy breaks all ties. Unlike Position C's equal-weight fusion, this architecture has no magic-number failure mode.
- **Future-proof for LLM classifier** — when `shikigami-mind` (FR-F11) ships in v2+, the sidecar emits the same event schema; nothing in the core changes.

### Negative

- Adds modest complexity over pure event-driven. Two-stage pipeline instead of one.
- Texture regex is still rule-based; persona authors who emit unconventional markers won't activate textures. Acceptable — they still get correct dominant states.
- Character authors now have an optional texture layer to think about. Documented as optional; minimum viable character does not need it.

### Neutral

- Texture vocabulary will evolve. v0.1 ships six textures; new ones require community + maintainer approval to enter canon. Unknown textures fall back gracefully, so additions are non-breaking.

## Alternatives Considered (via Steelman Tournament)

1. **Pure Event-Driven (Position A, v1 of this ADR)**
   - ❌ Rejected: too reductive. Debate revealed pre-emptive defenses were already hybrids.

2. **Pure Text-Parse (Position B, v0.1 original)**
   - ❌ Rejected: fatal on 95% corpus reality + destructive-op timing attack.

3. **Equal-Weight Fusion (Position C)**
   - ❌ Rejected: magic-number fusion weights create deadlock/oscillation risk. Orthogonality insight absorbed via hierarchical structure instead.

4. **Local LLM Classifier (Position D)**
   - 🗓️ Deferred to v2+ as optional `shikigami-mind` sidecar (FR-F11). Budget and ship-date make it out-of-scope for v0.1.

## References

- `docs/debates/2026-04-22-signal-source/05-synthesis-verdict.md` — full debate synthesis
- `docs/reviews/PRD-v0.1-adversarial-review.md` — original adversarial challenges
- Position write-ups: `01-position-A-*.md` through `04-position-D-*.md`
