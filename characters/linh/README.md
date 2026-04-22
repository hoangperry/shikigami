# Linh — Production Character Source

This directory is the source workspace for the real Linh character pack used by Shikigami. It mirrors the structure of `characters/linh-pixel`, but targets the production anime/chibi art direction defined in `docs/TDD.md` section 14 and `docs/CHARACTER-PLAYBOOK.md`.

Current workflow:

- `reference/` holds concept and anchor stills used to keep identity stable
- `raw/<state>/frame_NN.png` stores source sprite frames before packing
- `manifest.json` describes the final `.shikigami` package layout

The intended runtime camera is a single 3/4 full-body sprite centered in a square frame with alpha transparency. Source frames should be prepared at 1024x1024 and later exported/downscaled into the packaged 512x512 assets.
