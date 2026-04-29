# Linh — AI Keyframe Generation Prompts

> **Purpose**: per-state prompt kit for AI image generation (SDXL +
> ControlNet, DALL-E 3, Midjourney v6, Imagen 3, OpenAI Images via
> Codex / responses API, etc.). Outputs are *keyframes* — single-image
> reference seeds the human artist (or `generate.py`-style frame
> tweener) uses as the basis for a 10–20 frame sequence.
> **Pre-reads**: `linh-commission-brief.md`, `linh-moodboard.md`,
> `docs/CHARACTER-PLAYBOOK.md` §1.

---

## 1. Why keyframes, not full animations

AI image gen is great at single coherent frames, terrible at
maintaining identity across a 10-frame loop. The realistic workflow:

```
AI generates 1 reference keyframe per state
   ↓
Artist (or img2img with low denoise + ControlNet pose) tweens between
   ↓
Cleanup pass in Krita (transparency, line consistency)
   ↓
Export as frame_NN.png sequence
```

This doc covers step 1. Step 2–4 are out of scope; tracked in
[#28](https://github.com/hoangperry/shikigami/issues/28).

## 2. Universal prefix

Prepend to every state prompt below. Keeps identity consistent across
the 6+ generations.

```
chibi character design, single character, 1:3 head-to-body proportion,
adult-readable face (NOT oversized moe eyes), soft 2-tone cel shading,
2-3px tapered vector outlines, transparent background, full body visible,
warm professional secretary persona, glasses (thin red frame),
white blouse, navy pencil skirt, espresso brown shoulder-length hair,
warm cream skin tone, ruby red tie/accent,
clean Clip Studio Paint or Krita illustration style
```

Negative prompt (SDXL / Midjourney `--no` flag):

```
3D render, vroid, photo-real, semi-realism, dark / horror,
sexualised, micro-skirt, fan-service, oversized moe eyes,
cluttered background, environment, multiple characters, watermark,
low quality, jpeg artifacts, signature
```

## 3. Per-state keyframe prompts

For each state, the resulting image is the **midpoint frame** of the
target animation — the visual peak the tweener walks toward and away
from.

### `idle` — frame 06 of 12 (looping)

```
Linh standing relaxed-polite, hands clasped at waist over the navy
skirt, slight smile, eyes open and looking forward, neutral head tilt,
shoulders mid-breath (slightly raised), gentle ambient feel
```

Weight knobs: shoulders breathing > hand position > expression.

### `happy` — frame 08 of 15

```
Linh smiling warmly, eyes curved into crescent shape ( ^_^ ), small
pink blush on cheeks, both hands raised toward chest in a "pleasantly
surprised" pose, soft warm glow on the face, head tilted slightly to
one side
```

### `happy_relieved` — frame 12 of 20

```
Linh with eyes closed in a long exhale, head tilted slightly back,
shoulders visibly dropped, one hand mid-air adjusting the glasses
(thumb on temple piece), faint smile of relief, very subtle blush.
Animation context: this is the "post-meeting exhale" / "tests finally
passed" moment — convey relief, not joy.
```

### `focused` — frame 06 of 12 (looping)

```
Linh in concentration: eyes sharp and slightly narrowed, scanning
left-to-right, glasses catching a subtle glint highlight on the right
lens, holding a translucent holographic clipboard or folder in front
of her, mouth in a thin focused line, body angled slightly toward the
clipboard
```

Weight knobs: glasses glint > eye sharpness > clipboard pose.

### `warning` — frame 09 of 18 (looping)

```
Linh leaning slightly forward with one palm raised in a "wait/stop"
gesture (palm facing the viewer at chest height), concerned expression
with knit eyebrows, single small sweat-drop floating near her right
temple, mouth slightly open as if about to speak. Composition reads as
"hold on" not "panic"
```

### `sleepy` — frame 05 of 10 (looping)

```
Linh with half-closed drowsy eyes ( -_- ), head nodded forward
slightly, body posture relaxed-loose, optional small "z" or "zzz"
bubble floating near her ear, glasses sliding down the nose by a few
millimetres. Ambient slow vibe.
```

### `preview.png` thumbnail (512×512)

```
Linh in idle pose mid-frame (matches frame 06 of idle state),
centred on a fully transparent canvas, full body visible from feet
to head with ~10% padding on all sides, faint contact shadow on the
ground, no environment elements
```

## 4. Generation tips per provider

### SDXL + ControlNet (recommended — most controllable)

- Base model: any anime-tuned SDXL checkpoint (e.g. AnythingXL, Animagine)
- ControlNet: **OpenPose** for pose lock + **Lineart** at low strength for outline cleanliness
- CFG: 6–8, steps: 28–35
- Sampler: DPM++ 2M Karras
- Seed: pin a single seed across all 6 states for identity stability,
  vary only the prompt + ControlNet pose

### DALL-E 3 (fastest, hardest to lock identity)

- Run idle first; copy the description verbatim into subsequent prompts
  prefixed with "Same character as before, now …"
- Use the OpenAI Images API with `style: "vivid"`, `quality: "hd"`
- Expect 30–40% reroll rate before identity is consistent

### Midjourney v6

- Use `--cref <idle-image-url> --cw 80` for character reference
  consistency on subsequent states
- `--ar 1:1`, `--style raw` for cleaner outlines
- `--no` flag with the negative-prompt list from §2

### Imagen 3 (via Vertex AI / Gemini API)

- Lower stylistic flexibility than SDXL but handles transparent
  backgrounds well
- Run with `safety_filter_level: "block_only_high"` to avoid false
  positives on the warning-state sweat-drop

## 5. Acceptance criteria per generation

Before promoting a generation to "keyframe", verify:

- [ ] Proportion is 1:3 (measure: head height × 3 ≈ total height)
- [ ] Eyes are adult-readable (test: cover the body and ask "could this
      be a 25-year-old?")
- [ ] Palette matches §4 of the commission brief within ΔE 10
- [ ] Outline weight is 2–3 px equivalent at the rendered resolution
- [ ] Transparent background (no matte / fringe pixels)
- [ ] Pose matches the state's animation context — re-read the
      `CHARACTER-PLAYBOOK.md` §1.1 row before approving

## 6. Export from chosen tool

Save each approved keyframe as:

```
docs/character/keyframes/<state>.png       # 512×512, transparent
docs/character/keyframes/<state>.prompt    # exact prompt used (for reproducibility)
docs/character/keyframes/<state>.seed      # SDXL seed if applicable
```

Hand the keyframes folder + prompts to the tweening / cleanup step
(#28). Without the recorded prompt + seed it's impossible to regenerate
a consistent variant later.

## 7. Output license note

If you used a commercial-API provider (DALL-E 3, Imagen 3 paid),
double-check the provider's terms of service — some grant the user
full commercial rights, others restrict redistribution. The Shikigami
repo ships under MIT, so the keyframes need to clear the redistribution
bar set in [`linh-commission-brief.md`](./linh-commission-brief.md) §7.

For SDXL local generation: outputs from open checkpoints (CC0, MIT,
OpenRAIL with attribution) are typically safe to ship. For SDXL fine-
tunes published on Civitai, **read the model card** — many are
non-commercial-only.
