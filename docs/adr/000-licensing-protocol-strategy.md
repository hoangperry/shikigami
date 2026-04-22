# ADR-000: Licensing & Open-Source Protocol Strategy

**Status**: ✅ Accepted
**Date**: 2026-04-22
**Deciders**: @hoangperry
**Informed by**: Gemini adversarial review (Live2D licensing red flag)

## Context

Shikigami aims to be a truly open-source project eligible for inclusion in FOSS distributions (Homebrew, Debian, Fedora repos) and trustable by the community. The PRD v0.1 proposed a **dual-renderer architecture** with Live2D Cubism Web SDK as a first-class citizen alongside sprite sheets.

The adversarial review surfaced a blocker:

- **Live2D Cubism SDK is proprietary**. The Core runtime is a closed binary blob.
- Redistribution requires agreeing to Live2D's commercial terms; the "Small-scale" free license has revenue caps and strict conditions.
- Including the SDK in the project tree prevents packaging in Debian/Fedora/Homebrew-core and contaminates the "Open Source" claim.

Secondary concern: the VMC/OSC protocol (OSC-over-UDP used by VTube Studio, Warudo, VSeeFace) is an open protocol. Emitting VMC data would let users render via their existing pro VTuber tooling — but adds an external dependency and undermines the plug-and-play v1 experience.

## Decision

1. **v1 ships sprite-only.** Core repository is 100% MIT-licensed and contains no Live2D SDK code, binaries, or models.
2. **Live2D support is deferred** to an **optional add-on package** (`shikigami-live2d`) shipped as a separate install. Users opt in explicitly and accept Live2D's terms themselves. The add-on is maintained in its own repo so core stays FOSS-clean.
3. **VMC protocol export is a future feature (FR-F10)** — not a pivot. Shikigami's default experience remains self-contained; VMC is a power-user bridge for those who already own VTuber models.
4. **Asset licensing**: code is MIT; default/sample characters ship with **CC-BY-SA 4.0**. Third-party character packs declare their own license in the manifest.
5. **No third-party renderers in v1** (see ADR-004). Removes plugin-loading attack surface and keeps licensing story clean.

## Consequences

### Positive

- Shikigami core can be packaged in FOSS distros and included in Homebrew-core.
- Reduces legal review burden for contributors and downstream packagers.
- Forces focus on sprite renderer quality rather than feature-parity across engines.
- Clean story for open-source users: "MIT all the way down."

### Negative

- Live2D-quality VTuber feel deferred to Phase 5+; early adopters get sprite-only experience.
- Flagship character ("Linh") must be designed for sprite animation, not Live2D rigging.
- Splitting core vs. Live2D add-on adds release coordination overhead when Live2D add-on ships later.

### Neutral

- Power users wanting Live2D have alternatives: use the future add-on, or use VMC export (v3+) into their existing VTube Studio.

## Alternatives Considered

1. **Ship Live2D bundled in core (v0.1 plan)**
   - ❌ Rejected: blocks truly-OSS status, creates licensing time bomb.

2. **Pivot entirely to VMC bridge, no built-in renderer** (Gemini's suggestion)
   - ❌ Rejected: destroys out-of-the-box experience, forces users to install+configure VTube Studio separately. Shikigami becomes a thin bridge with no standalone value.

3. **Dual-license model (MIT core, proprietary Live2D module)**
   - ❌ Rejected: confuses community, complicates contribution flow.

## References

- [Live2D Cubism SDK Licensing](https://www.live2d.com/en/sdk/license/)
- [VMC Protocol Specification](https://protocol.vmc.info/)
- `docs/reviews/PRD-v0.1-adversarial-review.md` — Gemini Section: Live2D OSS Licensing Reality
