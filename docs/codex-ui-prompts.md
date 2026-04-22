# Codex UI Prompts — Linh Character Generation

> **Purpose**: Copy-paste prompts for OpenAI Codex UI / ChatGPT-with-image-gen to generate the **Linh** default character for Shikigami.
> **Strategy**: Generate a master reference sheet first → lock identity via re-reference → produce keyframes per state. NOT frame-by-frame animation (image models cannot maintain tight frame coherence); use these outputs as **keyframes for a human animator or AnimateDiff** to interpolate between.
> **Source of truth**: visual direction, palette, and states are from `docs/CHARACTER-PLAYBOOK.md` §1 and `docs/TDD.md` §14.

---

## ⚙️ Strategy Overview

```
STEP 1: Generate master character sheet    ← lock identity + palette
            │
            ▼ use as reference image in every subsequent prompt
STEP 2: Generate 1 keyframe per state       ← 6 keyframes total (1 pose per state)
            │
            ▼ optional
STEP 3: Generate 2-3 keyframes for animated states (animator tweens in between)
            │
            ▼
Artist (or AnimateDiff) fills in-between frames using keyframes as anchors
```

---

## 🪪 STEP 1 — Master Character Sheet

Paste this **first**. Save the output image and upload it as a reference for every subsequent prompt.

### Prompt 1A: Full Turnaround Reference Sheet

```
Create a professional 2D anime character reference sheet for an original character named "Linh".

**Character identity**: A 25-year-old female secretary in chibi (Super-Deformed) proportion — body is 3 heads tall. She is warm, competent, and quietly observant — NOT corporate-cold, NOT childish, NOT sexualized. SFW. Professional yet approachable.

**Outfit**:
- Crisp white long-sleeve blouse with small collar
- Midnight navy (#2C3E50) pencil skirt, ending just above the knee
- Subtle ruby red (#E74C3C) necktie or scarf as accent
- Thin-frame black glasses, minimal rim, sitting mid-nose
- Simple black low-heel pumps

**Physical features**:
- Shoulder-length warm espresso (#4A3728) hair, straight with a slight inward curl at the tips
- Soft cream (#FDF5E6) skin tone
- Friendly almond-shaped eyes, medium size (NOT oversized moe eyes) — warm amber color
- Small natural smile, relaxed posture

**Art style**:
- Clean anime vector line work, 2–3 pixel lines, tapered ends (hand-drawn feel)
- Soft 2-tone cel shading, NOT flat
- Chibi SD proportion: 1:3 head-to-body ratio
- Reference mood: characters from "New Game!" and "Servant × Service" — SFW office anime

**Layout of the reference sheet**:
- Transparent background (no scene, no shadow ground — just the character on pure alpha transparency)
- Three poses side-by-side, evenly spaced:
  1. Front view, hands clasped at waist, neutral friendly smile
  2. 3/4 view (slightly turned right), hands relaxed at sides
  3. Side profile (facing right), standing at rest
- All three poses show the same outfit, same proportions, same hair style, same glasses
- Character height fills ~80% of the vertical space; ~10% breathing room top and bottom
- NO text labels, NO color swatches, NO grid lines, NO watermark

**Output requirements**:
- Square aspect ratio, 1024×1024 minimum
- Transparent PNG alpha channel
- High detail, crisp lines
- Color-accurate to the hex codes specified
```

### Prompt 1B: Palette Reference Card (optional, helps identity lock)

```
Generate a character color palette chart on a pure white background showing five labeled swatches for an anime character:

- Blouse: #FFFFFF (Crisp White)
- Skirt: #2C3E50 (Midnight Navy)
- Hair: #4A3728 (Warm Espresso)
- Skin: #FDF5E6 (Old Lace Cream)
- Accent: #E74C3C (Ruby Red)

Each swatch is a 200×200 rounded square with the hex code printed below in clean monospace font. Arrange all five horizontally with equal spacing. Simple, clinical, reference-card style.
```

---

## 🎬 STEP 2 — Per-State Keyframe Prompts

**Before running any of these**: upload the master sheet (Prompt 1A output) as the **reference image** in the Codex UI message, then paste the prompt below. Every prompt assumes the reference is attached.

### 2.1 IDLE — Neutral Breathing Keyframe

```
Using the attached reference sheet as the exact character identity (same outfit, hair, glasses, proportions, palette), create a single keyframe for the "idle" animation state.

**Pose**: Linh stands facing the viewer, polite neutral posture. Hands clasped softly at her waist in front of her body. Shoulders relaxed. Small pleasant neutral expression — NOT smiling, NOT frowning. Eyes open and alert but calm.

**Framing**:
- Full body visible, 1:3 chibi SD proportion
- Character centered, 512×512 canvas, ~10% breathing room top/bottom
- Transparent background, NO shadow ground, NO scene

**Style consistency**:
- Match reference sheet EXACTLY: same hair curl, same glasses rim, same blouse collar shape, same skirt length
- 2-tone soft cel shading
- Clean vector lines 2–3px, tapered
- Pure alpha transparency, NO watermark, NO text
```

### 2.2 HAPPY — Smile Bloom Keyframe

```
Using the attached reference sheet as the exact character identity, create a single keyframe for the "happy" animation state (moment of peak expression).

**Pose**: Linh's eyes curve into warm crescents (closed-eye smile). Small natural blush on cheeks (subtle, not heavy). Her hands have moved up slightly toward her chest, fingertips near sternum, as if she's pleasantly surprised. Head tilted ~5° to one side. Shoulders relaxed and slightly raised from excitement.

**Framing**:
- Full body, 1:3 chibi SD proportion
- 512×512 canvas, transparent background
- Character centered

**Style consistency**:
- Match reference sheet exactly (outfit, hair, glasses, palette)
- 2-tone cel shading
- Clean vector lines 2–3px tapered
- Pure alpha transparency, NO scene, NO watermark, NO text
```

### 2.3 HAPPY_RELIEVED — Slow Exhale Keyframe

```
Using the attached reference sheet as the exact character identity, create a single keyframe for the "happy_relieved" state — the moment of exhale after tension releases.

**Pose**: Linh's eyes are softly closed, head tilted slightly back, face relaxed with the faintest smile at the corner of her mouth. Her shoulders have dropped low (visible relaxation of tension). One hand has moved up to her glasses, fingertips touching the rim as if adjusting them. Body weight shifted slightly to one foot. A gentle exhale sense — NOT sleep, NOT sadness, just quiet relief.

**Framing**:
- Full body, 1:3 chibi SD proportion
- 512×512 canvas, transparent background
- Character centered

**Style consistency**:
- Match reference sheet exactly
- 2-tone cel shading
- Clean vector lines 2–3px tapered
- Pure alpha transparency, NO scene, NO watermark, NO text
- The mood should read as "phew, that's done" — warm, not exhausted
```

### 2.4 FOCUSED — Working Concentration Keyframe

```
Using the attached reference sheet as the exact character identity, create a single keyframe for the "focused" animation state.

**Pose**: Linh holds a slim dark folder or clipboard in her left hand at chest height, reading it with concentrated attention. Her right hand is raised holding a silver pen, poised to write or tap. Eyebrows slightly drawn in concentration (NOT angry). Glasses catch a subtle light reflection (small white highlight on the left lens). Body leans forward very slightly. Mouth closed in a thoughtful line.

**Framing**:
- Full body, 1:3 chibi SD proportion
- 512×512 canvas, transparent background
- Character centered

**Style consistency**:
- Match reference sheet exactly
- 2-tone cel shading
- Clean vector lines 2–3px tapered
- Pure alpha transparency, NO scene, NO watermark, NO text
- Mood: sharp and engaged, NOT frustrated
```

### 2.5 WARNING — "Wait" Gesture Keyframe

```
Using the attached reference sheet as the exact character identity, create a single keyframe for the "warning" animation state — caution, attention required.

**Pose**: Linh holds her right hand up in a clear "stop/wait" gesture, palm facing the viewer, fingers together, arm bent at the elbow. Her left hand is at her side or slightly back. Expression: concerned but composed — brows slightly knit, mouth in a firm small "o" or slight frown. A single classic anime "sweat drop" icon is rendered next to her temple (small, tasteful). Body leans forward slightly, weight on front foot, conveying urgency.

**Framing**:
- Full body, 1:3 chibi SD proportion
- 512×512 canvas, transparent background
- Character centered
- The raised hand must be fully in frame

**Style consistency**:
- Match reference sheet exactly
- 2-tone cel shading
- Clean vector lines 2–3px tapered
- Pure alpha transparency, NO scene, NO watermark, NO text
- Mood: serious alert, NOT angry, NOT panicked
```

### 2.6 SLEEPY — Dozing Keyframe

```
Using the attached reference sheet as the exact character identity, create a single keyframe for the "sleepy" animation state.

**Pose**: Linh's head has nodded forward slightly, chin near her collarbone. Eyes are in a drowsy "half-moon" shape — not fully closed, just heavy-lidded. Mouth slightly open in a soft relaxed line. Shoulders slumped slightly. Hands are loosely folded in front of her or resting at her sides. A classic small anime "z" bubble (thought-bubble style, tasteful) floats near her head.

**Framing**:
- Full body, 1:3 chibi SD proportion
- 512×512 canvas, transparent background
- Character centered

**Style consistency**:
- Match reference sheet exactly
- 2-tone cel shading
- Clean vector lines 2–3px tapered
- Pure alpha transparency, NO scene, NO watermark, NO text
- Mood: gentle, quiet — she is tired but at peace, NOT passed out
```

---

## 🔄 STEP 3 — Animation In-Betweens (optional)

If you want multi-frame outputs from GPT directly (lower quality than human animation but viable for v0.1 MVP), use one of these approaches:

### 3.1 Two-Frame Arc (start + peak + end for non-looping states)

For `happy` and `happy_relieved`, ask for **3 frames** in one image: neutral → mid-expression → peak. Example:

```
Using the attached reference sheet, create a 3-frame animation storyboard for the "happy" state, arranged horizontally:

Frame 1 (left): neutral posture, hands at waist, no smile yet (this matches idle)
Frame 2 (center): mid-expression, a small smile forming, eyes beginning to curve
Frame 3 (right): peak expression, full closed-eye smile, hands raised to chest

- All three on transparent background
- Character identical across frames (same outfit, hair, glasses, proportions)
- 1024 wide × 512 tall canvas (three 512×512 frames side by side)
- 2-tone cel shading, clean lines, NO text labels, NO watermark
```

Then a human (or ffmpeg + frame interpolation like RIFE / DAIN) interpolates the missing frames.

### 3.2 Loop Keyframe Pair (for idle, focused, warning, sleepy)

For looping states, ask for **2 frames**: the "A" position and the "B" position. Interpolate between them.

```
Using the attached reference sheet, create a 2-frame animation storyboard for the "idle" state:

Frame 1 (left): breathing-in pose, chest slightly raised, shoulders up
Frame 2 (right): breathing-out pose, chest relaxed, shoulders down

Everything else identical. 1024 wide × 512 tall canvas, transparent background, 2-tone cel shading, NO text, NO watermark.
```

---

## 🎯 Iteration Tips in Codex UI

1. **Always attach the master reference sheet** to every prompt after Step 1. Without it, identity drifts.
2. **Re-generate, don't edit**: if a frame is off, paste the prompt again rather than asking to edit. Edits tend to drift.
3. **Lock the palette verbally**: repeat `"#FFFFFF blouse, #2C3E50 skirt, #4A3728 hair, #FDF5E6 skin, #E74C3C accent"` in follow-up messages if colors start wandering.
4. **Reject drifted outputs aggressively** — if the chibi ratio becomes 1:2 or the glasses disappear, do NOT use the frame. Regenerate.
5. **Export with transparent background**: if the model delivers a frame with a background, explicitly say in the next message: *"The previous image had a background — regenerate with pure alpha transparency, no scene."*
6. **Use Codex UI's "consistent character" feature** if available — it helps maintain identity across multiple generations.

---

## 📤 Post-Generation Checklist

Before feeding outputs into the Shikigami pipeline:

- [ ] All 6 keyframes generated at 512×512 minimum
- [ ] Transparent alpha channels (no white/colored background)
- [ ] Character identity consistent (visual diff the reference sheet vs each keyframe side-by-side)
- [ ] Palette consistent (use a color-picker on key regions to verify hex codes)
- [ ] No text, watermarks, signatures, or UI artifacts
- [ ] SFW across all frames

**Next step**: hand keyframes to animator (human or AnimateDiff) to tween in-between frames per TDD §14.3 frame counts. Or: accept keyframes-only as v0.1 minimum and schedule full animation for v0.2.

---

## 🛑 Known Limitations of Image Generation for Sprite Animation

Be honest with yourself going in:

- **Identity drift** is near-guaranteed across 6+ prompts even with reference. Expect to regenerate 30–50% of outputs.
- **Transparent backgrounds** are inconsistent — some outputs arrive with faint halos or off-white fills; clean in Krita/Photoshop before packing.
- **Line weight and shading style** may drift between outputs. Pass final frames through a consistency filter (e.g., run all through a matching Krita filter pass).
- **Frame-count animations** (e.g., 18 frames at consistent quality) are **not** achievable from this prompt alone — you get keyframes. Everything in between needs a human animator or an AI interpolator (RIFE, DAIN, AnimateDiff).

For a true production-grade Linh, the Codex UI output is **Step 1 of 3**. Treat it as concept art / keyframe anchors, not deliverable assets.

---

*"Prompt the skeleton. Animate the soul."*
