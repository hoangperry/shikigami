# Shikigami — Character Production Playbook

> **Status**: v0.1 Draft · **Last Updated**: 2026-04-22 · **Owner**: @hoangperry
> **Scope**: Full playbook for producing the default **Linh** character and onboarding community character contributors.
> **Reads with**: `docs/PRD.md`, `docs/TDD.md` §14, `docs/adr/003-character-package-format.md`.
> **Synthesized from**: Gemini CLI research (§1, §2, §3, §5, §6, §9) + in-house technical spec (§4, §7, §8).

---

## 1. Concept Direction — Linh (Default Character)

**Identity**: 25-year-old secretary in chibi format, professional but approachable. SFW. Warm, not corporate-cold.

**Voice (design intent)**: "She is competent enough to catch your `rm -rf` before you press enter, patient enough to sit through your five-hour debugging session, and kind enough to look relieved when your test suite finally passes."

### 1.1 Per-State Concept Briefs

Direct briefs for the animator (or AI-hybrid workflow). Each state targets the frame counts and FPS specified in TDD §14.3.

| State | Frames × FPS | Pose & Motion | Anime Reference |
|-------|-------------:|---------------|-----------------|
| **idle** | 12 × 12 loop | Neutral polite posture, hands clasped at waist. Subtle shoulder rise/fall (breathing). Double-blink at frames 4 and 10. | "Standard Wait" pose from *Working!! (Wagnaria)* |
| **happy** | 15 × 15 non-loop | Warm "smile bloom" — eyes curve to crescents, small blush, hands move toward chest as if pleasantly surprised. Returns to neutral on last frame. | Kobashigawa expressions from *New Game!* |
| **happy_relieved** | 20 × 12 non-loop | Eyes close, head tilts slightly back, shoulders drop visibly in a long exhale. One hand adjusts glasses as the breath finishes. | "Post-meeting exhale" common in *Servant × Service* |
| **focused** | 12 × 12 loop | Expression sharpens; holds translucent holographic clipboard (or folder). Eyes scan left-to-right reading. "Glasses glint" effect every ~3 seconds. | Akane Tsunemori's analytical look (*Psycho-Pass*) chibi-ified |
| **warning** | 18 × 18 loop | One hand up in "stop/wait" palm-out gesture, concerned "sweat-drop" icon pulses near temple. Slight forward lean to grab attention. | Mia Fey's "Wait a minute!" pose (*Ace Attorney*) |
| **sleepy** | 10 × 10 loop | Head slowly nods forward then jerks back up; eyes in "half-moon" drowsy state. Optional "z" bubble or snot bubble trope. | Any *Lucky Star* dozing sequence |

### 1.2 Style Direction & Palette

- **Proportion**: **1:3 Standard Chibi (SD)** ratio — enough vertical space to read blouse/pencil-skirt details without losing identity. Avoid 1:2 "bean" chibi (reads as mascot, not adult).
- **Line weight**: thick clean **vector-style outlines, 2–3px**, tapered ends for a hand-drawn feel.
- **Shading**: **soft 2-tone cel shading** — optimal for frame-to-frame consistency and future AI-assisted cleanup.

**Core Palette (5-color):**

| Role | Hex | Notes |
|------|-----|-------|
| Blouse | `#FFFFFF` | Crisp white |
| Skirt / vest | `#2C3E50` | Midnight navy |
| Hair | `#4A3728` | Warm espresso |
| Skin | `#FDF5E6` | Old lace / soft cream |
| Accent (tie / glasses rim) | `#E74C3C` | Ruby red pop |

**Avoid:**

- Generic VRoid exports (stiff 3D)
- Overly "moe" oversized eyes that mask the adult age
- Cold corporate flat-vector icon look
- Sexualized tropes — the persona is warm and competent, not fan-service

### 1.3 Reference Shopping List (Mood Board Inputs)

| What the reference captures | Where to search | What to extract |
|-----------------------------|-----------------|-----------------|
| Professional chibi proportion | Pinterest: *"Nendoroid office lady"* | 1:3 head-body scale, joint placement |
| Secretary attire | ArtStation: *"chibi secretary design"* | Simplified blouse collar, skirt silhouette |
| Pencil skirt movement | Booru: `pencil_skirt + animated` | Fabric stretch during pose changes |
| Warm secretary expression | Pinterest: *"anime secretary warm expression"* | Eye shape, soft coloring style |
| Office palette | Design Seeds: *"corporate professional soft"* | Alt palette ideas |
| Thin-frame glasses at small scale | ArtStation: *"anime glasses shader"* | Transparency + rim thickness at 512px |
| Chibi hair physics | Twitter/X: `#chibiart #animation` | Hair-as-solid-mass bounce |
| SFW office poses | Pinterest: *"office lady pose reference"* | Classy standing / sitting |

Assemble 8–12 images into a PureRef / Milanote board before commissioning or starting production.

---

## 2. Production Toolchain

### 2.1 Tool Comparison Matrix

| Tool | Price (2026) | OS | Frame-sheet support | Learning curve | Output fit for Shikigami |
|------|-------------:|----|---------------------|----------------|-------------------------|
| **Aseprite** | \$20 one-time | Mac / Win / Linux | ✅ Native sprite sheet export | Low–medium | 🟢 Best for pure pixel sprite workflow |
| **Clip Studio Paint EX** | \$230 one-time / \$5 mo | Mac / Win / iPad | ✅ Animation timeline, export as image sequence | Medium | 🟢 **Best for anime/chibi vector-style character** — industry standard |
| **Procreate + Procreate Dreams** | \$13 + \$20 one-time | iPad only | ✅ Animation Assist → export frames | Low | 🟡 Great quality, iPad-locked |
| **Krita** | Free | Mac / Win / Linux | ✅ Animation workspace, export PNG sequence | Medium–high | 🟢 Best free option |
| **Toon Boom Harmony** | $\$$$ subscription | Mac / Win | ✅ Professional rigging + frame | Very high | 🔴 Overkill |
| **AI-assisted (SDXL + AnimateDiff + ControlNet)** | \$0–\$50 (GPU) | Any | ⚠️ Output needs cleanup | High (workflow) | 🟡 Viable for base frames + human polish |

### 2.2 Recommended Combinations

**For the Linh default character (solo hobbyist, ships in 3–4 weeks):**

> **Clip Studio Paint EX** — character illustration + animation timeline
> **Krita** — frame cleanup and transparency polish
> **ImageMagick + cwebp** — batch export pipeline
> **AI-assisted** for base pose exploration only (*not* production frames)

Rationale: Clip Studio handles anime vector line work better than any other tool; its Animation Timeline is simpler than Krita for 10–20 frame sequences; pairs smoothly with a CLI pipeline for format conversion.

**For community contributors (minimum barrier):**

> **Krita** (free) + batch-conversion pipeline provided by Shikigami CLI

Shipping a `shikigami pack` command means contributors can use *any* tool that exports PNG frames.

### 2.3 AI-Assisted Reality Check (2026)

**What works:**

- Flux.1 / SDXL → high-identity **character reference sheets** (front / side / 3/4 view)
- AnimateDiff + ControlNet OpenPose → raw **idle-loop** and **happy-bloom** sequences if prompt is tight
- LoRA training on 20–30 curated images → identity stability across >80 frames
- Auto-masking (SAM 2) → clean alpha-channel extraction

**What fails:**

- High-velocity motion (warning gesture, surprise) → ghosting and identity drift
- Consistent clothing detail at small frame counts (buttons, tie knot) → drifts
- Precise fps-locked timing → AnimateDiff's 8–16 fps native output needs retiming
- Licensing purity for CC-BY-SA — depends entirely on which model was used

**Licensing-clean AI workflow** (if you go this route):

- Use **Mitsua Diffusion** (trained on public-domain + opt-in data) or **Flux Schnell Apache 2.0** base
- Avoid NovelAI / CivitAI models trained on scraped art — CC-BY-SA downstream becomes legally murky

**Realistic hybrid workflow:**

1. AI: generate 1:3 character reference sheet (Flux.1)
2. AI: AnimateDiff + ControlNet OpenPose → raw frame sequences for **idle** and **sleepy** (slow states)
3. Human: line-over + transparency cleanup in Krita
4. Human: hand-animate **warning**, **happy**, **happy_relieved** (high-motion states fail AI)

Cuts total animator time roughly 80h → 25h for a 5-state character.

---

## 3. Sourcing Strategy

### 3.1 Commission Marketplace Landscape (2026)

| Platform | Typical Price (USD) | Turnaround | CC-BY-SA Friendliness |
|----------|--------------------:|-----------:|----------------------|
| **VGen** (https://vgen.co) | \$250–600 | 2–4 weeks | 🟢 **Best.** Clear commercial-use addons, easy to negotiate share-alike terms |
| **Skeb** (Japan) | \$80–200 | 1–2 weeks | 🔴 Personal use only by default; no back-and-forth means SA cannot be negotiated |
| **Fiverr** | \$100–300 | ~1 week | 🟡 Hit-or-miss; high risk of AI re-rolls or stolen work |
| **ArtStation commissions** | \$500–1,200 | 4+ weeks | 🟢 Premium quality, professional contracts, expensive |
| **Twitter/X DM** (via Artistree escrow) | \$150–400 | Variable | 🟡 Best for niche anime aesthetic, use Artistree for contract safety |
| **Ko-fi Commissions** | \$100–350 | ~2 weeks | 🟢 Flexible; good for long-term indie relationships |
| **DeviantArt commissions** | \$80–250 | Variable | 🟡 Legacy community; quality varies widely |

**Top picks for Shikigami**:

1. 🥇 **VGen** — the platform already serves the PNGtuber and sprite-artist niche; most artists there understand sprite-sheet deliverables and offer clear commercial / share-alike add-ons. This is the path of least legal friction for a flagship OSS character.
2. 🥈 **Twitter/X + Artistree** — for locking in the exact "anime secretary" aesthetic from a specific artist. Artistree provides the escrow + license contract layer that makes CC-BY-SA enforceable.

### 3.2 Commission Brief Template

When reaching out to a VGen artist, send:

- Link to `docs/TDD.md §14` (public once repo goes public) as authoritative spec
- PureRef board assembled from §1.3 above
- Explicit license requirement: **CC-BY-SA 4.0** with credit string: *"Linh character by <artist>, licensed CC-BY-SA 4.0, commissioned for the Shikigami open-source project"*
- Deliverable expectation: **PNG sequence per state** (we convert to WebP + pack), transparent alpha
- Frame specs copied from TDD §14.3
- Milestone structure: (a) character sheet approved → 40% payment; (b) idle + happy delivered → 30%; (c) remaining 3 states → 30%

### 3.3 Pricing Reality Check

For Linh's 5 states (85 total frames, 1:3 chibi SD, anime cel-shade):

- **Low end VGen**: ~\$350–450
- **Mid VGen / Twitter commission**: ~\$600–800
- **ArtStation pro**: ~\$1,200–1,800

Budget **\$600–800** for a production-quality flagship.

---

## 4. Processing Pipeline (Shell + Python)

> These scripts live in `cli/scripts/` once implemented. They wrap the `shikigami pack` command documented in TDD §2 but are also usable standalone by contributors.

### 4.1 Directory Contract (Raw Input)

Contributor provides:

```
my-character/
├── manifest.json              ← contributor-authored, validated
├── preview.webp               ← 512×512
├── LICENSE                    ← SPDX-matching plaintext
├── README.md                  (optional)
└── raw/
    ├── idle/
    │   ├── frame_00.png
    │   ├── frame_01.png
    │   └── ...
    ├── happy/
    └── ...
```

Pipeline output:

```
my-character.shikigami (zip)
├── manifest.json
├── preview.webp
├── LICENSE
├── README.md
└── assets/states/
    ├── idle/
    │   ├── frame_00.webp
    │   ├── ...
    └── ...
```

### 4.2 `pack-character.sh` (Orchestrator)

```bash
#!/usr/bin/env bash
# cli/scripts/pack-character.sh — build .shikigami from raw/ directory
set -euo pipefail

SRC="${1:?usage: pack-character.sh <character-dir>}"
OUT_DIR="${2:-./dist}"

# Dependencies
for cmd in cwebp jq python3 zip; do
  command -v "$cmd" >/dev/null 2>&1 || { echo "✗ missing: $cmd"; exit 2; }
done

ID="$(jq -r '.id' "$SRC/manifest.json")"
STAGE="$(mktemp -d -t shikigami-pack-XXXX)"
trap 'rm -rf "$STAGE"' EXIT

# 1. Validate manifest + license + previews (Python validator)
python3 "$(dirname "$0")/validate_manifest.py" "$SRC"

# 2. Mirror top-level files
cp "$SRC/manifest.json" "$SRC/preview.webp" "$SRC/LICENSE" "$STAGE/"
[ -f "$SRC/README.md" ] && cp "$SRC/README.md" "$STAGE/"

# 3. Convert every raw/<state>/frame_NN.png → assets/states/<state>/frame_NN.webp
for state_dir in "$SRC/raw"/*/; do
  state="$(basename "$state_dir")"
  out="$STAGE/assets/states/$state"
  mkdir -p "$out"
  for frame in "$state_dir"frame_*.png; do
    [ -f "$frame" ] || continue
    base="$(basename "${frame%.png}")"
    cwebp -quiet -lossless -exact -alpha_q 100 "$frame" -o "$out/$base.webp"
  done
  echo "✓ $state: $(ls "$out" | wc -l | tr -d ' ') frames"
done

# 4. Zip
OUT_FILE="$OUT_DIR/${ID}.shikigami"
mkdir -p "$OUT_DIR"
(cd "$STAGE" && zip -qr9 "$OUT_FILE" .)

# 5. Report
SIZE_BYTES=$(stat -f%z "$OUT_FILE" 2>/dev/null || stat -c%s "$OUT_FILE")
SIZE_MB=$((SIZE_BYTES / 1024 / 1024))
echo "✓ packed: $OUT_FILE (${SIZE_MB}MB)"
[ "$SIZE_MB" -gt 30 ] && { echo "✗ exceeds 30MB cap"; exit 3; }
```

### 4.3 `validate_manifest.py` (Deep Validator)

```python
#!/usr/bin/env python3
"""validate_manifest.py — deep-validate a character source directory.

Checks:
  - manifest.json parses and matches schema v1.0
  - preview.webp exists and is 512×512
  - LICENSE exists and file content matches a known SPDX identifier
  - Each state path in manifest.states exists in raw/
  - Every frame in a state has identical dimensions
  - Frame filenames are zero-padded and contiguous
  - License SPDX-matches the manifest.license field
"""
from __future__ import annotations
import json
import re
import sys
import pathlib
from typing import Iterator

try:
    from PIL import Image
except ImportError:
    sys.stderr.write("Install Pillow: pip install pillow\n")
    sys.exit(2)

FRAME_RX = re.compile(r"^frame_(\d{2,4})\.(?:png|webp)$")
VALID_SPDX = {"MIT", "CC-BY-4.0", "CC-BY-SA-4.0", "CC0-1.0", "Apache-2.0"}

class ValidationError(Exception):
    pass

def fail(msg: str) -> None:
    raise ValidationError(msg)

def iter_frames(state_dir: pathlib.Path) -> Iterator[tuple[int, pathlib.Path]]:
    frames = []
    for p in sorted(state_dir.iterdir()):
        m = FRAME_RX.match(p.name)
        if m:
            frames.append((int(m.group(1)), p))
    for expected, (got_idx, path) in enumerate(frames):
        if expected != got_idx:
            fail(f"{state_dir}: frame index gap at position {expected} (found {got_idx})")
    yield from frames

def validate(src: pathlib.Path) -> None:
    manifest_path = src / "manifest.json"
    if not manifest_path.is_file():
        fail("manifest.json missing")

    try:
        manifest = json.loads(manifest_path.read_text("utf-8"))
    except json.JSONDecodeError as e:
        fail(f"manifest.json invalid JSON: {e}")

    # Top-level fields
    for key in ("schemaVersion", "id", "name", "author", "version", "license", "renderer", "states"):
        if key not in manifest:
            fail(f"manifest.json missing field: {key}")
    if manifest["schemaVersion"] != "1.0":
        fail(f"unsupported schemaVersion: {manifest['schemaVersion']}")
    if not re.fullmatch(r"[a-z0-9][a-z0-9-]{2,63}", manifest["id"]):
        fail(f"invalid id: {manifest['id']}")
    if manifest["license"] not in VALID_SPDX:
        fail(f"license {manifest['license']!r} not in {VALID_SPDX}")
    if manifest["renderer"] != "sprite":
        fail(f"renderer must be 'sprite' in v1.0, got {manifest['renderer']!r}")

    # Preview + LICENSE + README
    preview = src / "preview.webp"
    if not preview.is_file():
        fail("preview.webp missing")
    with Image.open(preview) as im:
        if im.size != (512, 512):
            fail(f"preview.webp must be 512x512 (got {im.size})")
    license_path = src / "LICENSE"
    if not license_path.is_file():
        fail("LICENSE missing")

    # States: each state path under raw/ must exist and have consistent frames
    raw = src / "raw"
    if not raw.is_dir():
        fail("raw/ directory missing — provide raw PNG frames per state")

    for state_name, state_cfg in manifest["states"].items():
        state_dir = raw / state_name
        if not state_dir.is_dir():
            fail(f"state {state_name!r}: raw/{state_name}/ missing")
        frames = list(iter_frames(state_dir))
        if not frames:
            fail(f"state {state_name!r}: no frames found")
        # Dimension consistency
        with Image.open(frames[0][1]) as first:
            base_size = first.size
            base_mode = first.mode
        for idx, path in frames[1:]:
            with Image.open(path) as im:
                if im.size != base_size:
                    fail(f"state {state_name!r}: frame {idx} size {im.size} != {base_size}")
                if im.mode != base_mode:
                    fail(f"state {state_name!r}: frame {idx} mode {im.mode} != {base_mode}")
        print(f"✓ {state_name}: {len(frames)} frames @ {base_size[0]}×{base_size[1]}")

    # Minimum viable character
    if "idle" not in manifest["states"] or "happy" not in manifest["states"]:
        fail("minimum viable character requires 'idle' and 'happy' states")

    print(f"✓ {manifest['id']} ready to pack")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.stderr.write("usage: validate_manifest.py <character-dir>\n")
        sys.exit(2)
    try:
        validate(pathlib.Path(sys.argv[1]))
    except ValidationError as e:
        sys.stderr.write(f"✗ {e}\n")
        sys.exit(1)
```

### 4.4 One-Shot Usage

```bash
# For contributors — three commands to ship
git clone https://github.com/shikigami-project/character-template my-oc
# ... draw frames into my-oc/raw/<state>/*.png ...
./cli/scripts/pack-character.sh my-oc ./out
# → ./out/my-oc.shikigami ready to distribute
```

---

## 5. CI Hooks & Community Tooling

### 5.1 Pre-Commit Hook (drops into contributor repo)

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit — validate manifest before allowing commit
set -e
if [ -f manifest.json ]; then
  python3 cli/scripts/validate_manifest.py .
fi
```

### 5.2 GitHub Action for Character Repos

File: `.github/workflows/validate-character.yml` (ships in template repo)

```yaml
name: validate-character
on: [pull_request, push]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with: { python-version: "3.12" }
      - run: pip install pillow
      - name: Install shikigami CLI
        run: |
          curl -sL https://github.com/hoangperry/shikigami/releases/latest/download/shikigami-linux-x64.tar.gz | tar xz
          sudo mv shikigami /usr/local/bin/
      - name: Validate manifest
        run: shikigami validate .
      - name: Attempt pack
        run: shikigami pack . --out dist/
      - name: Upload .shikigami artifact
        uses: actions/upload-artifact@v4
        with:
          name: character-bundle
          path: dist/*.shikigami
```

### 5.3 Template Repo

`hoangperry/shikigami-character-template` — scaffold with:

- Empty `raw/idle/`, `raw/happy/` folders with `.gitkeep`
- `manifest.json` stub with TODO comments
- `LICENSE` template (CC-BY-SA-4.0 preselected)
- Pre-commit hook installed
- GitHub Action from §5.2
- `README.md` with a 10-minute tutorial

Link from main Shikigami README: *"Want to make your own character? Use the template →"*

---

## 6. Timeline + Cost + Recommendation

### 6.1 Scenarios Side-by-Side

| Scenario | Time | Cost (USD) | Quality Risk | License Risk |
|----------|-----:|-----------:|-------------|-------------|
| **A — Solo Hobbyist** (Krita / Clip Studio, 10h/week from scratch) | 8–12 weeks | \$230 tool cost | 🔴 High — burnout + inconsistency across 85 frames | 🟢 Clean (you own it) |
| **B — Commissioned Pro** (VGen, top pick) | 3–5 weeks | \$600–800 incl. CC-BY-SA buyout | 🟢 Low — contracted quality | 🟢 Clean (contract specifies CC-BY-SA) |
| **C — AI-Hybrid** (Flux/AnimateDiff + human cleanup) | 2–3 weeks | \$50–100 (GPU rental) + ~25h human | 🟡 Medium — identity drift requires skilled cleanup | 🟡 Depends on base model (use Mitsua/Flux Schnell) |

### 6.2 Recommendation for the Flagship Linh

🏆 **Scenario B — VGen Commission** for v0.1.

Reasoning:
- The flagship character is the *first thing* every user sees. Quality is disproportionate leverage for open-source reception.
- License cleanliness matters for public/pinned demo usage and for inspiring community contributions.
- \$600–800 is a rounding error compared to the engineering time being invested in the rest of the project.
- Commissioned artist + CC-BY-SA attribution becomes marketing: the artist gets exposure, the project gets quality, contributors see a professional bar to aspire to.

**Scenario C** remains the recommended path for **community-contributed expansion packs** in v0.2+ — lower stakes per character, lets 10+ characters ship when 1 would otherwise.

**Scenario A** is only advisable if the project lead is themselves a trained anime-style animator willing to commit the time budget.

### 6.3 Execution Plan (Pre-Phase-2)

Because character asset production runs in parallel with Phase 0 + Phase 1 engineering:

1. **Week 1 (Phase 0 engineering in parallel)**: Assemble PureRef mood board (§1.3). Write VGen commission brief. Post listings on VGen + 1 Twitter DM outreach.
2. **Week 2**: Select artist, pay milestone 1, provide PRD + TDD §14 links. Engineer continues Phase 0.
3. **Week 3–4**: Artist delivers character sheet + idle + happy. Engineer completes Phase 1 (state machine). Validate frames via `validate_manifest.py`.
4. **Week 5**: Final 3 states delivered. Pack `.shikigami`. Test in dev build. Publish as first community-visible artifact.

Total: **4–5 weeks character budget** aligned with **4-week Phase 0–2 engineering budget**. Character arrives just as Phase 2 renderer is ready to load it.

---

## 7. Post-Linh: Community Character Roadmap

Once Linh ships and `shikigami pack` CLI is stable:

- **Launch a "Character Jam"** — 2-week call for community submissions with a defined theme (e.g., "Office Oni", "Seasonal Sprites")
- Curate 5 quality submissions into a **starter pack** bundled with the app
- Dedicated GitHub org `shikigami-characters/` hosts individual character repos
- README badge: *"Shipped with Shikigami: <character-count> community characters"*

Goal metric: **10+ community-shipped characters by 3 months post-v0.1 launch** (matches PRD G9).

---

## 8. Appendix — Provenance

- Sections 1, 3, 6 (concept + commission marketplace + timeline table): synthesized from Gemini CLI research dispatch on 2026-04-22.
- Sections 2, 4, 5: in-house technical spec drawing on the Shikigami TDD §14 and general 2026 tooling knowledge.
- Codex CLI dispatch for pipeline scripts was attempted but timed out; pipeline content was hand-authored as proxy.
- Full Gemini raw output archived at `docs/debates/2026-04-22-signal-source/` was separate; this playbook's Gemini content from a follow-up dispatch (not archived as a standalone artifact but incorporated here).

---

*"Every character is a small act of trust. Build the pipeline once, let every artist after you have an easier day."*
