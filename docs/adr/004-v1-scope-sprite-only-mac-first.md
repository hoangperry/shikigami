# ADR-004: v1 Scope — Sprite-only, Mac-first, Claude Code-only

**Status**: ✅ Accepted
**Date**: 2026-04-22
**Deciders**: @hoangperry
**Informed by**: Both adversarial reviews ("scope too big" convergent verdict)

## Context

PRD v0.1 was ambitious:
- Cross-platform day one (Mac + Windows + Linux)
- Dual renderer (Sprite + Live2D)
- Multiple AI integrations (Claude Code + Cursor + Windsurf + ChatGPT)
- Third-party renderer plugin system
- URL install + public registry
- 1000 GitHub stars in 6 months

Both reviewers independently flagged this as **impossible to deliver in 4–6 weeks** while keeping quality. Codex proposed a "constructive reset": **sprite-only, one tool integration, one great character, no third-party renderers, prove retention before expanding scope.**

## Decision

v0.1 ships the **Minimum Lovable Product**:

### In Scope (v0.1.0 — target: 4 weeks)

- ✅ **Platform**: macOS only (Apple Silicon + Intel universal binary)
- ✅ **Renderer**: Sprite sheet only (no Live2D)
- ✅ **AI tool integration**: Claude Code only (via hooks)
- ✅ **Characters shipped**: **1** (default "Linh" — sprite)
- ✅ **Character install methods**: drag-drop `.shikigami` + CLI `shikigami install <path>`
- ✅ **Transport**: Local HTTP POST with auth token (ADR-001)
- ✅ **Signal source**: Event-driven primary + text-parse fallback (ADR-002)
- ✅ **Speech bubble**: context-aware info display (FR-055)
- ✅ **Settings**: size, opacity, position, click-through, auto-start
- ✅ **System tray**: basic controls
- ✅ **Documentation**: contributor guide + template repo + packaging CLI

### Out of Scope v0.1 → Deferred Timeline

| Feature | Deferred to | Rationale |
|---------|-------------|-----------|
| Windows support | v0.2 (4 weeks after v0.1) | Test cross-platform transparent overlay separately |
| Linux support | v0.3 (6 weeks after v0.1) | WebKitGTK quirks need time |
| Live2D renderer (optional add-on) | Phase 5 | Licensing separation (ADR-000) |
| Cursor integration | v0.4+ | Prove Claude Code retention first |
| Windsurf integration | v0.4+ | Same as Cursor |
| ChatGPT browser adapter | v0.5+ | Adapter pattern shakedown first |
| Third-party renderer plugins | ❌ Removed indefinitely | Security + YAGNI |
| URL-based character install | v0.3+ | Signing & trust story needed first |
| Public character registry | v1.0 | Needs moderation + hosting story |
| VMC protocol export | v0.6+ | Power-user bridge, not core |
| TTS / voice | Future Scope (FR-F01) | Voice asset licensing, premium path |
| Cloud sync | Future Scope (FR-F03) | Premium tier material |

### Revised Success Metrics

| Metric | v0.1 Target (3 months) | v1.0 Target (9 months) |
|--------|-----------------------|------------------------|
| GitHub stars | 100–200 | 500–1,000 |
| Retention (weekly active) | 15–25 users | 100+ users |
| Community character packs | 2–3 | 10+ |
| State reaction correctness | ≥90% on test corpus | ≥95% |
| RAM footprint (idle) | <80MB | <80MB |
| Install-to-first-reaction | <3 min | <2 min |

**Previous "1000 stars in 6 months" target is retired as unrealistic.**

## Consequences

### Positive

- **Shippable in 4 weeks**. Every feature left in scope is justified by retention/delight/installability.
- **Quality over surface area**. One great character + one great integration > five mediocre ones.
- **Clear story for early adopters**: "Mac + Claude Code + sprite — that's the v0.1 promise."
- **Validated retention** before investing in Windows/Linux/Cursor adapters.
- Reviewers' "first user complaint" prediction becomes testable immediately.

### Negative

- Alienates Windows/Linux users for ~1–2 months post-v0.1.
- VTuber-model users (who want Live2D day one) must wait for the add-on.
- Niche narrows before broadening — counterintuitive for growth but correct for quality.

### Neutral

- Architecture (package format, event transport, renderer interface) is designed to accept the deferred features without rewrite.

## Alternatives Considered

1. **Keep PRD v0.1 scope as-is**
   - ❌ Rejected: both reviewers converged on "impossible in 4–6 weeks."

2. **Scope cut but keep Live2D in v0.1**
   - ❌ Rejected: licensing blocker (ADR-000).

3. **Even more radical — text-only status bar app, no character**
   - ❌ Rejected: removes the entire product thesis.

## References

- Codex review — "Constructive reset" section
- Gemini review — all sections (scope + licensing + realism)
- `docs/reviews/PRD-v0.1-adversarial-review.md`
