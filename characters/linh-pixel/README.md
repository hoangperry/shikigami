# Linh (Pixel) — Dev Fixture Character

> A procedurally-generated 8-bit pixel-art placeholder for the **Linh** default character. This package exists to unblock Shikigami engineering while the production assets are in commission. It is **intentionally** simple and not meant for user-facing release.

## Why this exists

The real Linh character (anime vector-style) is being produced in parallel via Codex UI storyboards + potential artist commission. That pipeline takes weeks. This pixel version takes seconds.

This character lets us:
- Develop and test the sprite renderer with real frames
- Validate the `.shikigami` package format end-to-end
- Test the event → state machine → animation pipeline
- Ship a dev-preview build before production assets land

**Do not distribute this as the flagship character.** When real Linh ships, this package gets archived under `dev-fixtures/`.

## What's inside

- `src/generate.py` — Python script that draws every frame procedurally from ASCII-art grids
- `raw/<state>/frame_NN.png` — 512×512 PNG frames with alpha transparency (12 frames total across 6 states)
- `preview.png` — idle frame 0 used as the character thumbnail
- `manifest.json` — character manifest v1.0
- `LICENSE` — MIT

## States shipped

| State | Frames | FPS | Notes |
|-------|:------:|:---:|-------|
| `idle` | 2 | 4 | Breathing loop (heavy lids alternation) |
| `happy` | 2 | 6 | Crescent eyes + small smile |
| `happy_relieved` | 2 | 4 | Soft closed eyes + hand approaching glasses |
| `focused` | 2 | 4 | Knit brows + eye scan |
| `warning` | 2 | 6 | Wide eyes + raised arm (stop gesture) + sweat drop |
| `sleepy` | 2 | 3 | Half-closed drowsy eyes + nod |

Textures (`happy_relieved`, `cute`, etc. as defined in the core canonical vocab) are **not** supported by this fixture — all texture modifiers fall back to the base dominant state gracefully, as the Shikigami runtime is spec'd to do.

## Regenerating frames

```bash
cd characters/linh-pixel
python3 src/generate.py
```

Requires `Pillow` (`pip install pillow`).

Frames are drawn from ASCII grids defined in `src/generate.py`. Each grid is 24×32 at source resolution, scaled 12× nearest-neighbor to 288×384, centered on a 512×512 transparent canvas.

## Editing the sprite

Edit the head-row functions (`head_idle_a`, `head_happy_a`, etc.) in `src/generate.py`. The color palette is defined at the top of the file:

| Char | Color | Hex |
|:----:|-------|-----|
| `.` | transparent | — |
| `S` | skin | #FDF5E6 |
| `s` | skin shade | — |
| `H` | hair | #4A3728 |
| `h` | hair highlight | — |
| `W` | blouse | #FFFFFF |
| `w` | blouse shade | — |
| `N` | skirt (navy) | #2C3E50 |
| `n` | skirt highlight | — |
| `R` | red tie | #E74C3C |
| `K` | outline/frame/shoes | — |
| `P` | blush | — |
| `E` | eye pupil | — |

## License

MIT. Use freely in any context. When the production Linh ships, she will carry her own (probably CC-BY-SA-4.0) license — this pixel fixture remains MIT to keep it friction-free as test data.
