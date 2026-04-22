# Codex UI Production Prompts — Linh Character Animation

> **Status**: v0.2 — Production-grade prompt kit · **Last Updated**: 2026-04-22
> **Goal**: Produce frame-consistent sprite animation for the Linh character using GPT-4o / Codex UI image generation ONLY. No ComfyUI, no Rive, no commission — just tight prompt engineering.
> **Core principle**: Generate multi-frame storyboards in a single image (identity + style stay locked within one generation), chain multiple storyboards per state, cherry-pick frames.

---

## 🧭 Core Philosophy

### Why Storyboards, Not Individual Frames

Each image-gen request is a fresh roll of the dice. Identity drifts between requests.

**Inside a single generation, identity holds**. So we force the model to produce **4–6 frames in one grid image** per request. Drift happens between storyboards, not within them.

**Math**: to fill a 12-frame idle loop with 4-frame storyboards, we need 3 generations (4×3=12). Each generation produces 4 coherent frames. Total drift boundaries = 3 (between storyboards), not 12 (between individual frames). 4× better coherence for free.

### Why Every Prompt Has the Identity Lock Block

GPT-4o is lazy. Without an aggressive identity lock, it starts inventing details (different hair length, different glasses, different skirt cut). The identity lock block makes the model re-render from reference on every generation.

### Why We Over-Specify Poses

Vague pose instructions → model picks "average pose" → all frames look identical → no animation. Over-specifying (with degrees, hand positions, finger positions) forces the model to actually vary the frames.

---

## 🔒 UNIVERSAL IDENTITY LOCK BLOCK

**Paste this verbatim into EVERY prompt after the first character sheet generation.** Keep the reference image attached to the message.

```
IDENTITY LOCK — THE CHARACTER IN EVERY PANEL MUST BE IDENTICAL:

Character name: Linh
Age appearance: 25 years old (adult, NOT child, NOT teen)
Face: soft rounded cheeks, warm amber almond-shaped eyes (medium size, NOT oversized moe), small natural mouth, light blush on cheeks
Hair: shoulder-length warm espresso brown (#4A3728), straight with slight inward curl at tips, side-parted
Glasses: thin-frame black rectangular glasses sitting mid-nose, minimal rim, NO thick frames
Outfit (EXACT):
  - Crisp white (#FFFFFF) long-sleeve blouse, small pointed collar, buttons down center
  - Midnight navy (#2C3E50) pencil skirt ending just above knee
  - Ruby red (#E74C3C) small necktie or scarf knotted at collar
  - Simple black low-heel closed-toe pumps
Skin tone: soft cream (#FDF5E6)
Proportion: anime moe / 1:5 head-to-body (NOT extreme chibi, NOT full realistic)
Body: slim, feminine, modest (NOT sexualized, NOT exaggerated)

ART STYLE LOCK:
  - Clean anime vector line work, 2–3 pixel lines, tapered ends
  - Soft 2-tone cel shading (NOT flat, NOT gradient rendering)
  - Single light source from upper-left at 45°
  - Consistent line weight across all panels
  - Palette EXACTLY: #FFFFFF blouse, #2C3E50 skirt, #4A3728 hair, #FDF5E6 skin, #E74C3C accent

NEGATIVE INSTRUCTIONS — DO NOT INCLUDE:
  - NO background, NO scene, NO floor, NO shadows on ground (pure alpha transparency only)
  - NO text, NO labels, NO panel numbers, NO watermark, NO signature
  - NO frame borders or grid lines between panels (panels flow on transparent bg)
  - NO additional characters or objects beyond what the pose requires
  - NO outfit variation across panels (same blouse, same skirt, same tie, every panel)
  - NO hair length change across panels
  - NO different glasses styles across panels
  - NO style drift (one panel realistic + another chibi is FAIL)
```

---

## 🪪 STEP 0 — Master Character Sheet (if not already done)

If you already have the Codex-generated character sheet from previous session, **skip this step**. Use that existing image as reference.

Otherwise, paste:

```
Create a professional 2D anime character reference sheet for an original character named "Linh". Output: transparent PNG, 1024×1024, three poses side-by-side on a horizontal canvas.

IDENTITY:
[paste the IDENTITY LOCK BLOCK here]

LAYOUT:
- Three poses side-by-side on a single horizontal 1536×1024 canvas (three 512×1024 panels)
- Panel 1 (left): full front view, hands clasped softly at waist, neutral friendly closed-mouth smile
- Panel 2 (center): 3/4 view turned slightly to her right, hands relaxed at sides, same neutral expression
- Panel 3 (right): side profile facing right, standing at rest, hands at sides
- Character fills ~80% of vertical space in each panel, centered horizontally in panel
- ~10% breathing room top and bottom of each panel
- Panels separated by empty transparent space (no borders)
- All three poses show EXACTLY the same outfit, proportion, hair, glasses

OUTPUT: 1536×1024 transparent PNG. Crisp clean lines. Color-accurate to hex codes. No text, no labels, no background.
```

---

## 🎬 STATE 1 — IDLE (12 frames total, breathing loop)

### Motion Breakdown

Idle is a gentle breathing cycle with two blinks. 12 frames at 12fps = 1 second loop.

| Frame | Chest position | Shoulder position | Eyes | Head |
|------:|----------------|-------------------|------|------|
| 0 | neutral | neutral | open | straight |
| 1 | rising 20% | up 5% | open | straight |
| 2 | rising 50% | up 10% | open | straight |
| 3 | rising 60% | up 12% | **CLOSED (blink)** | straight |
| 4 | rising 80% | up 15% | opening | straight |
| 5 | peak | peak | open | straight |
| 6 | falling from peak | starting down | open | straight |
| 7 | falling 70% | down 10% | open | straight |
| 8 | falling 40% | down 5% | open | straight |
| 9 | falling 20% | down 2% | **CLOSED (blink)** | straight |
| 10 | returning neutral | returning | opening | straight |
| 11 | neutral | neutral | open | straight |

### Storyboard 1A — Frames 0, 1, 2, 3 (inhale start + first blink)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels arranged left-to-right, no borders between them, no numbers, no labels.

POSE INSTRUCTIONS (each panel shows Linh full body standing frontal, hands clasped softly at waist):

Panel 1 (frame 0): NEUTRAL START. Chest at rest position. Shoulders in relaxed neutral. Eyes fully open, amber, looking straight at viewer. Small closed-mouth neutral expression. Head straight, no tilt.

Panel 2 (frame 1): EARLY INHALE. Chest lifted 20% from frame 0, visible but subtle. Shoulders slightly raised. Eyes open same as panel 1. Same neutral expression.

Panel 3 (frame 2): MID INHALE. Chest clearly lifted, about 50% of maximum breath. Shoulders clearly raised. Eyes open, same expression.

Panel 4 (frame 3): BLINK AT PEAK OF MID-INHALE. Chest position same as panel 3. EYES ARE CLOSED (eyelids down, visible curved closed-eye lines). Same expression otherwise. This is a natural blink, NOT a happy squint.

The four panels form a sequential motion study. Character MUST be exactly identical across all panels except for the stated chest/shoulder/eye differences. Same outfit, same hair, same glasses, same everything else in every panel.

OUTPUT: 2048×512 transparent PNG, four panels clearly separable for individual frame extraction.
```

### Storyboard 1B — Frames 4, 5, 6, 7 (peak + exhale start)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders, no numbers.

POSE INSTRUCTIONS (each panel shows Linh full body standing frontal, hands clasped at waist):

Panel 1 (frame 4): POST-BLINK OPENING. Chest lifted 80% toward peak. Shoulders lifted. Eyes opening (eyelids half-up, eyes visible but partially shown). Neutral expression.

Panel 2 (frame 5): PEAK INHALE. Chest at maximum breath height (noticeably raised from neutral). Shoulders at peak. Eyes fully open wide and alert. Same neutral expression. This is the top of the breath cycle.

Panel 3 (frame 6): EXHALE STARTING. Chest beginning to fall from peak (very subtle, just a hint of descent). Shoulders starting to relax. Eyes open.

Panel 4 (frame 7): MID EXHALE. Chest 70% of the way down from peak. Shoulders descending visibly. Eyes open. Body posture relaxing.

Four panels, sequential breath exhale progression. Character must be IDENTICAL across panels except for the chest/shoulder/eye variations stated.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 1C — Frames 8, 9, 10, 11 (exhale completion + second blink)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

POSE INSTRUCTIONS (Linh full body standing frontal, hands clasped at waist, SAME as previous storyboards):

Panel 1 (frame 8): LATE EXHALE. Chest 40% of the way down from peak (mostly descended). Shoulders nearly at neutral. Eyes open alert.

Panel 2 (frame 9): BLINK AT LATE EXHALE. Chest at 20% above neutral (almost done exhaling). EYES CLOSED in natural blink (eyelids down, closed-eye curve lines visible). Same neutral expression.

Panel 3 (frame 10): BLINK RECOVERY. Chest at 10% above neutral. Eyes OPENING (eyelids half up, eyes partially visible).

Panel 4 (frame 11): RETURN TO NEUTRAL. Chest at rest position (same as frame 0). Shoulders neutral. Eyes fully open. Same neutral expression as frame 0. This frame is identical to frame 0 and loops back.

Character identical across all panels. Same outfit, same hair, same glasses, same everything.

OUTPUT: 2048×512 transparent PNG.
```

---

## 🎬 STATE 2 — HAPPY (15 frames total, smile bloom non-loop)

### Motion Breakdown

Happy is a smile bloom from neutral → peak → hold → settle. 15 frames at 15fps = 1 second.

| Frame | Mouth | Eyes | Cheeks | Hand position | Head tilt |
|------:|-------|------|--------|---------------|-----------|
| 0 | neutral closed | open alert | normal | clasped waist | 0° |
| 1 | corners lifting 10% | open | very slight warmth | clasped waist | 0° |
| 2 | corners up 30% | open, narrowing | light blush beginning | clasped waist | 2° left |
| 3 | small smile forming | narrowing to crescent | blush light | hands beginning to lift | 3° left |
| 4 | bigger smile | crescent-shaping | blush more visible | hands mid-lift | 4° left |
| 5 | near-peak smile | closing to crescent | blush clear | hands near chest | 5° left |
| 6 | full closed-eye smile | fully crescent | peak blush | hands at chest sternum | 5° left |
| 7 | **peak hold** | crescent hold | peak blush hold | hands at chest | 5° left hold |
| 8 | peak hold | crescent hold | peak blush | hands at chest | 5° left hold |
| 9 | smile softening | crescent opening slightly | blush softening | hands beginning to lower | 4° left |
| 10 | smaller smile | eyes reopening | blush fading | hands mid-descent | 3° left |
| 11 | small smile | eyes mostly open | blush very light | hands descending | 2° left |
| 12 | corners lowering | open | blush trace | hands near waist | 1° left |
| 13 | near neutral | open | normal | hands at waist | 0° |
| 14 | neutral (back to idle-compatible) | open | normal | clasped at waist | 0° |

### Storyboard 2A — Frames 0, 3, 6, 7 (4 keyframes covering onset + peak)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders, no numbers.

POSE INSTRUCTIONS (Linh full body, frontal):

Panel 1 (frame 0 — START NEUTRAL): Hands clasped softly at waist. Mouth closed in small neutral line (NOT smiling yet). Eyes fully open, alert, straight. Cheeks normal skin tone. Head straight, no tilt. This is her baseline resting pose before the smile begins.

Panel 2 (frame 3 — SMILE FORMING): Hands BEGINNING TO LIFT from waist, fingertips at lower ribcage level. Mouth in small definite smile (corners clearly up, lips still closed). Eyes narrowing slightly (starting to curve toward crescent but still mostly open). Light pink blush appearing on both cheeks. Head tilted 3 degrees to her left.

Panel 3 (frame 6 — FULL SMILE NEAR PEAK): Hands at chest sternum level, fingertips curled slightly inward near heart. Mouth in full warm closed-mouth smile, corners noticeably raised. Eyes closing into upward-curving crescents (happy closed-eye shape, visible eyelash curves). Clear pink blush on both cheeks. Head tilted 5 degrees to her left.

Panel 4 (frame 7 — PEAK HOLD): IDENTICAL to panel 3 (this is the held peak frame). Same hand position, same eye crescent, same blush, same head tilt, same smile. This frame captures the held expression.

Character must be IDENTICAL in outfit, hair, glasses, proportion, and style across all four panels. Only the mouth, eyes, cheeks, hands, and head-tilt vary as specified above.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 2B — Frames 1, 2, 4, 5 (4 transition frames between 0 and 7)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: These are in-between frames for a smile-bloom animation. Panel 1 corresponds to ~10% of the way from neutral to peak; Panel 4 corresponds to ~80% of the way. Poses must form a smooth in-between progression.

Panel 1 (frame 1 — 10% smile): Hands clasped at waist (unchanged from neutral). Mouth corners barely lifting, a hint of smile but still very close to neutral. Eyes fully open. Very faint warmth in cheeks (almost invisible). Head straight.

Panel 2 (frame 2 — 30% smile): Hands still at waist, fingers beginning to relax their clasp. Mouth in small visible smile (corners clearly up). Eyes slightly narrowing. Light blush visible on cheeks. Head tilted 2 degrees to her left.

Panel 3 (frame 4 — 60% smile): Hands halfway up from waist to chest, around mid-torso. Mouth in medium-sized warm smile. Eyes narrowing into early crescent shape. Blush more visible. Head tilted 4 degrees to her left.

Panel 4 (frame 5 — 80% smile): Hands near chest but not yet at sternum, fingertips at ribs. Mouth in large smile, almost full crescent. Eyes closing to near-crescent (but not fully closed yet — eyes still slightly visible). Clear blush. Head tilted 5 degrees to her left.

Character identical in outfit, hair, glasses, proportion across all four panels.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 2C — Frames 9, 11, 13, 14 (smile recovery to neutral)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: After the smile peak (frame 7), Linh's expression recovers back to neutral over frames 8-14. These four panels capture the recovery arc.

Panel 1 (frame 9 — 80% smile softening): Hands BEGINNING TO LOWER from chest (fingertips at upper ribs, moving downward). Mouth in full smile but corners beginning to relax. Eyes crescent-shape opening slightly (eyes becoming partially visible). Blush still clear but softening. Head tilted 4 degrees to her left.

Panel 2 (frame 11 — 50% smile): Hands descending, now at mid-torso/ribcage. Mouth in medium smile. Eyes open mostly, slight narrowing. Light blush. Head tilted 2 degrees to her left.

Panel 3 (frame 13 — 15% smile): Hands almost back at waist, near final position. Mouth in tiny remaining smile, corners barely up. Eyes fully open alert. Cheeks nearly normal color. Head nearly straight (1 degree tilt).

Panel 4 (frame 14 — BACK TO NEUTRAL): Hands clasped at waist (identical to frame 0 / idle neutral). Mouth in neutral closed line, no smile. Eyes fully open alert. Cheeks normal. Head straight. This frame is identical to the idle neutral pose and allows seamless transition back to idle.

Character identical in outfit, hair, glasses, proportion across all four panels.

OUTPUT: 2048×512 transparent PNG.
```

---

## 🎬 STATE 3 — HAPPY_RELIEVED (20 frames, slow exhale)

### Motion Breakdown

This is a SLOW relief breath. 20 frames at 12fps = ~1.67 seconds. Key moments:

| Phase | Frames | Action |
|-------|-------:|--------|
| Onset | 0–3 | eyes still open, tension visible |
| Eyes close | 4–7 | gradual eye close, head starts tilting back |
| Exhale peak | 8–12 | head tilted back, eyes closed, shoulders drop, hand moves to glasses |
| Hold | 13–16 | hold the relieved position |
| Return | 17–19 | settle back to near-neutral |

### Storyboard 3A — Frames 0, 3, 5, 7 (onset + eyes closing)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: Linh is about to exhale in relief after tension. The animation is slow and weighted — each panel is visibly different.

Panel 1 (frame 0 — TENSION START): Hands at waist, shoulders slightly high from held tension. Mouth in small line. Eyes OPEN, looking slightly downward (NOT at viewer, looking at something below). Subtle tight expression around eyes. Head straight.

Panel 2 (frame 3 — BREATH STARTING): Shoulders beginning to drop from tension. Eyes starting to close (upper eyelids descending, eyes half-lidded). Mouth slightly parted. A hint of relief entering the expression. Head beginning to tilt back 2 degrees.

Panel 3 (frame 5 — EYES NEAR CLOSED): Shoulders dropping visibly. Eyes nearly closed (just a slit visible). Mouth in soft relaxed line. Head tilted back 4 degrees. One hand (her right) beginning to lift from the waist toward her face.

Panel 4 (frame 7 — EYES CLOSED): Eyes fully closed (eyelid curve lines visible). Shoulders noticeably lower. Mouth in gentle soft line, corner slightly lifted (half-smile starting). Head tilted back 6 degrees. Right hand reaching mid-way to glasses level.

Character identical in outfit, hair, glasses, proportion, style across all panels.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 3B — Frames 9, 11, 13, 15 (peak exhale + hold)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

Panel 1 (frame 9 — RIGHT HAND AT GLASSES): Eyes closed softly. Shoulders clearly dropped low (relief posture). Mouth in half-smile, corner up. Head tilted back 8 degrees. Right hand now TOUCHING the bridge of her glasses, fingertips resting on the rim. Left hand at her side or loosely at waist.

Panel 2 (frame 11 — PEAK EXHALE): Same posture as panel 1 but deepened — shoulders at lowest point, head tilted back maximum 10 degrees, right hand adjusting glasses (fingers gently pushing them up slightly). Mouth in soft half-smile, peaceful. Eyes closed.

Panel 3 (frame 13 — HOLD 1): Hold the peak relief position. Same as panel 2 — eyes closed, head tilted back, right hand on glasses, shoulders low. Very subtle breath settling.

Panel 4 (frame 15 — HOLD 2): Same held position, beginning the transition out. Right hand starting to lower slightly from glasses (fingertips still near rim). Shoulders starting to rise ever so slightly.

Character identical in all non-specified details across panels.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 3C — Frames 17, 18, 19 + return to neutral

```
[paste IDENTITY LOCK BLOCK here]

Create a 3-panel animation storyboard on a single 1536×512 horizontal canvas with pure transparent alpha background. Three panels left-to-right, no borders.

Panel 1 (frame 17 — RECOVERING): Head tilting back toward neutral (tilted 4 degrees). Eyes still closed but upper lids beginning to lift. Right hand descending from glasses, back at chest level. Shoulders rising back toward normal. Small contented smile.

Panel 2 (frame 18 — EYES OPENING): Head at 2 degrees tilt. Eyes opening (eyelids lifting, pupils becoming visible). Right hand continuing to descend, now at waist level. Small gentle smile.

Panel 3 (frame 19 — BACK TO READY): Head straight. Eyes fully open, soft alert gaze. Right hand returned to clasped position at waist with left hand. Small gentle smile (slightly warmer than pure neutral — the relief has left a small afterglow smile). Shoulders at normal position.

Character identical in outfit/hair/glasses/style across all three panels.

OUTPUT: 1536×512 transparent PNG.
```

---

## 🎬 STATE 4 — FOCUSED (12 frames loop, reading/working)

### Motion Breakdown

Linh reads a folder/clipboard with concentrated attention. Subtle motion: eyes scan left-right, occasional glasses glint, slight head tilt as she processes.

| Phase | Frames |
|-------|--------|
| Scanning left | 0–3 |
| Scanning right | 4–7 |
| Glint + think | 8–11 |

### Storyboard 4A — Frames 0, 1, 2, 3 (eyes scanning left)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: Linh is in focused reading mode. In ALL panels she is holding a slim dark navy folder/clipboard in her left hand at chest height, with her right hand holding a silver pen near the folder. The folder is held like one would hold a menu — angled slightly toward her so she can read. Her body is facing viewer but her gaze is down-and-to-her-left on the folder.

Panel 1 (frame 0): Eyes looking down-LEFT at the folder (gaze direction clearly left). Eyebrows slightly drawn in concentration. Mouth in small thoughtful closed line. Head tilted 2 degrees forward and 3 degrees to her right. Pen held loosely, tip near folder but not touching.

Panel 2 (frame 1): Eyes still scanning left, now looking farther left-down. Same concentration. Same hand positions.

Panel 3 (frame 2): Eyes at far-left position of the scan. Head tilted 3 degrees forward, 4 degrees right (slightly more tilted than panel 1). Pen still poised.

Panel 4 (frame 3): Eyes beginning the return to center. Head tilt returning toward panel 1 position. Same concentration in expression.

Character identical in outfit/hair/glasses/proportion across all panels. The folder and pen must be IDENTICAL in all four panels (same color, same shape, same size).

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 4B — Frames 4, 5, 6, 7 (eyes scanning right)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

Panel 1 (frame 4): Eyes looking down at center of folder. Head slightly less tilted than previous, 2 degrees forward, 2 degrees right. Concentration.

Panel 2 (frame 5): Eyes beginning to scan right (gaze direction shifting right-down). Head tilt shifting slightly — 2 degrees forward, 1 degree right.

Panel 3 (frame 6): Eyes scanning right, clearly looking right-down on the folder. Head tilt — 2 degrees forward, 0 degrees side (straight).

Panel 4 (frame 7): Eyes at far-right position. Head tilted 2 degrees forward, 2 degrees LEFT (opposite of scan-left storyboard). Same concentration. Pen still poised.

Character identical in outfit, hair, glasses, folder, pen across all panels.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 4C — Frames 8, 9, 10, 11 (glint + think + loop return)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

Panel 1 (frame 8): Eyes returning toward center of folder. Head tilt returning toward neutral forward lean.

Panel 2 (frame 9): GLASSES GLINT. A small bright white reflective highlight visible on the LEFT lens of her glasses (like a diagonal shine streak). This is a brief moment of reflection catching the light. All other elements same as panel 1.

Panel 3 (frame 10): Glint fading (small remnant of reflection). Eyes at center-down on folder. Concentration unchanged.

Panel 4 (frame 11): RETURN TO FRAME 0 POSITION. No glint. Eyes at starting scan-left position. Head tilt at 2 forward, 3 right (matches frame 0). This frame loops back to frame 0 seamlessly.

Character identical. Folder and pen identical. Only the glint and eye direction vary as specified.

OUTPUT: 2048×512 transparent PNG.
```

---

## 🎬 STATE 5 — WARNING (18 frames loop, "wait" gesture)

### Motion Breakdown

Linh holds a "stop/wait" hand gesture with concerned expression. Subtle rocking motion gives it urgency without chaos.

| Phase | Frames |
|-------|--------|
| Initial raise | 0–5 |
| Holding + rocking | 6–11 |
| Sweat drop pulse | 12–17 |

### Storyboard 5A — Frames 0, 2, 4, 5 (hand raise)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: Linh is raising her right hand in a clear "stop/wait" gesture. Concerned but composed expression. Body leaning slightly forward with urgency.

Panel 1 (frame 0): Right hand beginning to raise from waist, now at chest level, palm starting to face viewer. Left hand at side. Mouth in small firm line, brows slightly knit with concern (NOT angry, just attentive warning). Eyes alert wide open. Body leaning forward 3 degrees from upright.

Panel 2 (frame 2): Right hand further up, now at shoulder height, palm clearly facing viewer (showing palm). Fingers together pointing upward. Same concerned expression. Body leaning forward 4 degrees.

Panel 3 (frame 4): Right hand at final "wait" position — elbow bent, palm fully facing viewer, fingers together pointing up, hand in front of her face level but not blocking face. Left hand still at side. Body leaning forward 5 degrees. Concerned expression peak — brows clearly knit.

Panel 4 (frame 5): Same position as panel 3 but a small classic anime "sweat drop" icon appearing near her right temple (small translucent teardrop shape floating just off her head). This is the tasteful classic anime trope, not exaggerated.

Character identical in outfit/hair/glasses across all panels. The raised hand must be fully in frame and clearly visible.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 5B — Frames 7, 9, 11, 12 (hold + rocking)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: Linh holds the "wait" position with subtle body rocking. Sweat drop pulses.

Panel 1 (frame 7): Hold position — right hand raised palm-out at face level, left hand at side, body leaning forward 5 degrees. Sweat drop at full visible size near right temple. Concerned brows, mouth small firm line.

Panel 2 (frame 9): Slight rocking — body leaning forward 6 degrees (1 degree more than panel 1). Hand position same. Sweat drop slightly smaller (pulsing — this is mid-shrink). Same expression.

Panel 3 (frame 11): Rocking back — body leaning forward 4 degrees (1 less than panel 1). Sweat drop at minimum visible size. Hand position same. Same expression.

Panel 4 (frame 12): Return to lean 5 degrees (same as panel 1). Sweat drop growing back to normal size. Concerned expression hold.

Character identical except for the subtle body lean rocking and sweat drop size pulse.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 5C — Frames 14, 15, 16, 17 (final rock + loop back)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

Panel 1 (frame 14): Body lean 6 degrees (forward peak of second rock cycle). Hand still raised palm-out. Sweat drop at full size. Concerned expression.

Panel 2 (frame 15): Body lean 4 degrees. Sweat drop shrinking. Same hand position. Same expression.

Panel 3 (frame 16): Body lean 5 degrees (returning to neutral hold). Sweat drop normal size. Same pose.

Panel 4 (frame 17): LOOP RETURN. Identical to frame 0 position — right hand beginning to raise (lower position, near chest), left hand at side. Body lean 3 degrees. Sweat drop still present (full size). This frame allows the loop to restart seamlessly with the hand re-raising.

Character identical across all panels.

OUTPUT: 2048×512 transparent PNG.
```

---

## 🎬 STATE 6 — SLEEPY (10 frames loop, dozing)

### Motion Breakdown

Slow nod-and-catch cycle with half-moon eyes. 10 frames at 10fps = 1 second.

### Storyboard 6A — Frames 0, 2, 4, 5 (slow nod down)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

CONTEXT: Linh is dozing off slowly. Her head slowly nods forward. Eyes in drowsy half-moon state. Tasteful sleepy trope.

Panel 1 (frame 0): Head straight upright, neutral position. Eyes in HALF-MOON DROWSY shape (upper eyelids heavy, covering upper half of eyes, lower portion visible). Mouth slightly parted in relaxed soft line. Shoulders slightly slumped. Hands loosely folded in front or at sides. A small classic anime "z" thought-bubble floats near upper-right of head (tasteful, small).

Panel 2 (frame 2): Head nodding forward 10 degrees. Eyes half-moon drowsy (same). Mouth same. Shoulders slightly more slumped. "z" bubble still visible.

Panel 3 (frame 4): Head nodding forward 20 degrees (clearly nodding). Eyes half-moon. Mouth same. "z" bubble still present.

Panel 4 (frame 5): Head at maximum nod forward, 25 degrees. Chin near collarbone. Eyes half-moon still. Mouth soft relaxed line. "z" bubble visible (maybe slightly larger to emphasize sleepy moment).

Character identical in outfit/hair/glasses across panels. Only the head angle and subtle shoulder slump vary.

OUTPUT: 2048×512 transparent PNG.
```

### Storyboard 6B — Frames 6, 7, 8, 9 (jerk back + settle + loop)

```
[paste IDENTITY LOCK BLOCK here]

Create a 4-panel animation storyboard on a single 2048×512 horizontal canvas with pure transparent alpha background. Four panels left-to-right, no borders.

Panel 1 (frame 6): HEAD JERKS UP slightly (startled awake subtly). Head at 15 degrees forward (from peak nod of 25). Eyes slightly wider open (still drowsy but briefly more awake — like catching herself dozing). Mouth slightly more open. "z" bubble visible.

Panel 2 (frame 7): Head continuing to rise, now at 10 degrees forward. Eyes returning to half-moon drowsy. Body settling. "z" bubble present.

Panel 3 (frame 8): Head at 5 degrees forward. Eyes half-moon. Relaxed settling into the sleep cycle. "z" bubble.

Panel 4 (frame 9): Head back to straight upright (same as frame 0). Eyes half-moon drowsy. Mouth soft relaxed. Shoulders slightly slumped. "z" bubble. This frame loops back to frame 0 seamlessly.

Character identical across panels.

OUTPUT: 2048×512 transparent PNG.
```

---

## ✂️ Frame Extraction Guide

Each storyboard is a horizontal composite. Extract individual frames using any image editor:

### Using ImageMagick (CLI)

```bash
# For a 4-panel 2048×512 storyboard — extract 4 frames of 512×512
convert storyboard-1A.png -crop 512x512 +repage frame_%02d.png
# Renames to frame_00.png, frame_01.png, frame_02.png, frame_03.png

# For 3-panel 1536×512 storyboard
convert storyboard-3C.png -crop 512x512 +repage frame_%02d.png
```

### Using Photoshop / Krita (GUI)

- Import storyboard
- Grid slice at 512px intervals
- Export each slice as `frame_NN.webp` with alpha channel preserved

### Frame Numbering Reminder

When saving extracted frames, keep the CANONICAL frame numbers (not the panel numbers within a storyboard):

- Storyboard 1A (idle) → `frame_00.webp`, `frame_01.webp`, `frame_02.webp`, `frame_03.webp`
- Storyboard 1B (idle) → `frame_04.webp`, `frame_05.webp`, `frame_06.webp`, `frame_07.webp`
- Storyboard 1C (idle) → `frame_08.webp`, `frame_09.webp`, `frame_10.webp`, `frame_11.webp`

---

## 🔄 Regeneration Protocols

### When to Regenerate

**ALWAYS regenerate** if any of:

- Character proportion shifts between panels within one storyboard
- Outfit detail changes (tie color, skirt length, collar shape)
- Hair length or style differs
- Glasses disappear in any panel
- Background bleeds in (gray halo, white fill)
- Line weight or shading style shifts noticeably between panels
- Model ignores the pose spec and generates "average character" across all panels

**SOMETIMES accept with cleanup** if:

- Minor color shift (fixable in Krita)
- Slight line weight inconsistency (one light pass in Krita fixes)
- Small alpha halo (mattting-out fixes)

**ALWAYS accept** if:

- All identity elements consistent
- Pose spec roughly followed (within 20% of described positions)
- Transparent background clean
- No drift between panels

### Regeneration Prompt Pattern

When regenerating, send this follow-up message:

```
The previous storyboard had [LIST SPECIFIC ISSUES: e.g., "a gray background bleed in panel 3", "glasses missing in panel 2", "tie changed from red to blue in panel 4"].

Regenerate the same 4-panel storyboard with the EXACT same pose specifications and identity. Do not deviate. Emphasis on:
- Pure alpha transparency, absolutely NO background fill, no gray, no white
- Glasses present and consistent in every panel
- Tie must be red #E74C3C in every panel
- [OTHER SPECIFIC CONCERNS]

[paste full original prompt again — do not shortcut]
```

### Cherry-Picking Strategy

When you have multiple generations with partial success:

1. Extract frames from every successful generation
2. Sort frames by canonical frame number (00, 01, 02, ...)
3. For each canonical frame, pick the version with **best identity match to the reference sheet**
4. Some frames may come from generation 1, others from generation 2 — this is fine as long as each frame matches identity
5. Run final frame sequence through a "consistency pass" in Krita (same filter on all frames) to normalize line weight and shading before packing

---

## 🎚️ Full Generation Sequence (Recommended Order)

Run in this exact order for best results:

### Day 1 (setup)

1. Open fresh Codex UI session
2. Run Step 0 master character sheet (or reuse existing sheet)
3. Save output locally; upload to Codex UI as sticky reference attachment for subsequent messages in the same session
4. Generate Storyboard 1A (idle frames 0-3) — verify quality
5. Regenerate if needed until satisfied
6. Generate Storyboard 1B, then 1C
7. Extract all 12 idle frames

### Day 2

8. Storyboards 2A → 2B → 2C (happy, 15 frames)
9. Extract 15 happy frames
10. Storyboards 3A → 3B → 3C (happy_relieved, 20 frames)

### Day 3

11. Storyboards 4A → 4B → 4C (focused, 12 frames)
12. Storyboards 5A → 5B → 5C (warning, 18 frames — but 2 of those storyboards cover 16 frames and the rest loops; cherry-pick your 18)
13. Storyboards 6A → 6B (sleepy, 10 frames)

### Day 4 (cleanup)

14. Krita consistency pass on all frames (normalize line weight, alpha cleanup)
15. Run `validate_manifest.py` on character directory
16. Run `pack-character.sh` → `linh.shikigami`
17. Test load in Shikigami dev build

**Total generation budget**: ~20–25 storyboard prompts, each ~3 minutes = ~1-1.5 hour active time, spread over 3-4 days to avoid fatigue and allow re-roll cycles.

---

## 🎯 Quality Gate Checklist (per storyboard)

Before accepting a storyboard output, tick every box:

- [ ] All panels on fully transparent alpha (no background fill, no halos)
- [ ] Character identity matches master reference sheet in every panel (face, hair, glasses, outfit, palette)
- [ ] Pose varies across panels per the prompt spec (not all identical)
- [ ] No text, watermarks, or labels anywhere in the image
- [ ] No panel borders or grid lines
- [ ] Line weight consistent across panels (no panel suddenly styled differently)
- [ ] Palette consistent (hex codes still match in every panel)
- [ ] No extra characters or objects beyond what the pose requires
- [ ] Frame dimensions extractable cleanly at 512×512 per panel

If fewer than 8/9 boxes tick → regenerate.

---

## 🚨 Failure Modes & Fixes

### "The character looks different in panel 4 than panel 1"

- Cause: Model drift within a single generation. Rare but happens with complex prompts.
- Fix: Regenerate the same prompt. If persistent, simplify the pose description in panels that drift.

### "Background keeps coming through"

- Cause: Model defaults to "scene" when prompts are too descriptive.
- Fix: Start the prompt with `ABSOLUTELY TRANSPARENT ALPHA BACKGROUND — NO SCENE, NO FLOOR, NO SHADOWS` in ALL CAPS. Add it to the IDENTITY LOCK BLOCK for that session.

### "Two of the four panels have great identity but the other two drifted"

- Cause: Prompt has too many pose variations; model averaged.
- Fix: Split the storyboard into 2 storyboards of 2 panels each. Smaller storyboards = better coherence.

### "The model refuses with content policy warning"

- Cause: "Secretary" + "young woman" sometimes triggers false-positive SFW filter. "Office lady" or "professional" may word-pass.
- Fix: Rephrase: "25-year-old professional office worker, polite and SFW" instead of "secretary". Emphasize "professional attire" instead of "pencil skirt" if needed.

### "Glasses keep disappearing in one panel"

- Cause: Eye-heavy expressions (closed eyes, crescents) sometimes make the model drop glasses.
- Fix: Add explicit line: "Glasses must remain visible and in-place in every panel, even when eyes are closed."

### "Style shifts from anime vector to realistic"

- Cause: Prompts that describe pose in detail sometimes trigger realistic-mode.
- Fix: Repeat `ART STYLE LOCK` block twice in the prompt. Add "in the style of New Game! anime" as an explicit style anchor.

### "Output is blurry or low detail"

- Cause: Too many panels in one image at too-small individual resolution.
- Fix: Use the 2048×512 layout for 4 panels (512×512 each). Do NOT use 4096×512 layouts — resolution per panel becomes too small.

---

## 🧾 Appendix — Why This Works Better Than Single-Frame Prompts

**Traditional approach (single frame per prompt)**:
- 12 idle frames = 12 separate generations
- Identity drift between every frame
- Estimated usable frames after cherry-pick: 4-6 out of 12 (33-50%)
- Net yield per hour: ~5 usable frames

**Storyboard approach (this document)**:
- 12 idle frames = 3 storyboard generations (4 panels each)
- Identity drift only between storyboards (3 boundaries instead of 12)
- Estimated usable frames after cherry-pick: 9-11 out of 12 (75-92%)
- Net yield per hour: ~15-20 usable frames

**3–4× efficiency gain** from batching frames into coherent storyboard generations. This is why the document is long and detailed — the payoff is concentrated in the execution.

---

*"Prompts are contracts. Ambiguity is breach of contract. Specify the pose down to the degree of head tilt."*
