# ADR-003: Character Package Format v1.0

**Status**: ✅ Accepted
**Date**: 2026-04-22
**Deciders**: @hoangperry

## Context

Characters must be distributable, installable, and portable across OSes. PRD v0.1 proposed a `.shikigami` zip bundle format inspired by `.vsix` (VSCode), `.jar` (Java), and `.docx` (OOXML). The adversarial reviews confirmed this direction but surfaced concerns:

- Package corruption handling (Codex)
- Manifest schema versioning (Codex)
- Contributor pain when manifest requirements are heavy (both)

## Decision

**Format**: standard ZIP archive with `.shikigami` extension.

**Contents (v1.0 schema — sprite-only)**:

```
<character-id>.shikigami (zip)
├── manifest.json            ← required, schema-validated on install
├── preview.webp             ← required, 512x512 thumbnail
├── LICENSE                  ← required, SPDX-valid identifier in manifest
├── README.md                ← optional, character lore
└── assets/
    └── states/
        ├── idle/
        │   ├── frame_00.webp ... frame_11.webp
        │   └── animation.json   ← optional (auto-derived if omitted)
        ├── happy/
        ├── focused/
        ├── warning/
        └── <other states>/
```

**Manifest v1.0 (reduced from v0.1 proposal — less contributor burden)**:

```json
{
  "schemaVersion": "1.0",
  "id": "linh-secretary",
  "name": "Linh (Secretary)",
  "description": "Your charming executive assistant.",
  "author": "hoangperry",
  "version": "1.0.0",
  "license": "CC-BY-SA-4.0",
  "tags": ["anime", "chibi", "secretary"],
  "renderer": "sprite",
  "defaultState": "idle",
  "states": {
    "idle":     { "path": "assets/states/idle",     "fps": 12, "loop": true },
    "happy":    { "path": "assets/states/happy",    "fps": 15, "loop": false, "then": "idle" },
    "focused":  { "path": "assets/states/focused",  "fps": 12, "loop": true },
    "warning":  { "path": "assets/states/warning",  "fps": 18, "loop": true }
  }
}
```

### Minimum Viable Character

A contributor can ship a character with **only `idle` and `happy` states**. Missing states fall back to `idle` with a log warning. This addresses the "<30 min for an artist" claim with an honest minimum bar.

### Schema Validation

- JSON Schema v2020-12 for manifest validation
- CLI `shikigami validate <package>` runs full validation including:
  - SPDX license identifier
  - PNG/WebP frame dimensions consistent
  - Required states present
  - Performance budget (package <30MB compressed)

### Package Corruption Handling

On install failure or load-time corruption:
1. Character is marked `broken` in library, not silently dropped
2. User sees character card with `⚠ Broken package — click for details`
3. Details modal shows specific validation errors
4. Broken characters do not load into the renderer but are listed for manual cleanup/reinstall

### Schema Versioning

- `schemaVersion` field is required. v1.0 is the only supported version in v0.1 of the app.
- When v2.0 schema lands, app **reads v1.0 packages with an auto-migration shim** for one major version cycle. After that, legacy packages show `⚠ Upgrade required`.

## Consequences

### Positive

- Minimal manifest = low barrier for artists who know PNG export.
- Standard ZIP means existing tools (File Explorer, Finder, Keka) can inspect contents for debugging.
- Schema versioning future-proofs against format evolution.
- Broken-package visibility beats silent drops (better for contributor debugging).

### Negative

- Per-frame WebP files in a zip are slightly less efficient than atlas textures. We can auto-generate atlases at install time (inside the app) without burdening contributors.
- JSON Schema validation adds a dependency but is well-worth the install-time guarantees.

### Neutral

- Same format will support Live2D in the future via `renderer: "live2d"` — but Live2D contents remain in the optional add-on (ADR-000).

## Alternatives Considered

1. **Custom binary format**
   - ❌ Rejected: reinvents the wheel, harder for contributors to inspect.

2. **Per-frame JSON metadata required**
   - ❌ Rejected: too heavy. Auto-derive frame order from filename sort.

3. **Embed audio in v1**
   - ❌ Rejected: defer to when TTS lands (FR-F01).

## References

- VSCode Extension format: https://code.visualstudio.com/api/working-with-extensions/publishing-extension
- SPDX License List: https://spdx.org/licenses/
- JSON Schema: https://json-schema.org/
