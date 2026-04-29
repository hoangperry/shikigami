# Linh — Character Commission Brief (VGen)

> **Purpose**: send-ready brief for VGen / Skeb / direct-DM artist
> commission. Copy-paste sections into the platform's commission form;
> swap budget + timeline placeholders before hitting submit.
> **Reference docs**: [`linh-moodboard.md`](./linh-moodboard.md),
> `docs/CHARACTER-PLAYBOOK.md` §1.

---

## TL;DR for the artist

I'm commissioning a chibi anime character ("Linh") for an open-source
desktop overlay app. She reacts in real time to AI-coding events
(builds passing, errors, etc.). I need **6 illustrated states + 1
preview**, delivered as PNG sequences with transparent backgrounds.
SFW, professional warm tone — no fan-service. Style is 1:3 chibi with
soft 2-tone cel shading.

---

## 1. Project context

**Shikigami** is an MIT-licensed desktop app written in Rust + React.
Linh sits on the user's screen as a small transparent window and reacts
visually to their AI agent's actions. Think "VS Code status bar with a
character" rather than "VTuber rig". The character will be redistributed
publicly with the open-source project, so licensing has to allow
commercial-grade redistribution (CC-BY-4.0 acceptable; CC-BY-NC is not).

GitHub: https://github.com/hoangperry/shikigami

---

## 2. Character identity

> Quote pinned at the top of the mood board:
> *"She is competent enough to catch your `rm -rf` before you press
> enter, patient enough to sit through your five-hour debugging
> session, and kind enough to look relieved when your test suite
> finally passes."*

- **Age read**: 25, but expressed in chibi form
- **Vibe**: warm professional, not corporate-cold
- **Reference characters in tone**: Akane Tsunemori (analytical),
  Kobashigawa from *New Game!* (warm), Mia Fey from *Ace Attorney*
  (firm but kind)

## 3. Style direction

| Axis | Spec |
|---|---|
| Proportion | **1:3 chibi (SD)** — head ≈ 1/3 of total height. Not 1:2 "bean" |
| Line weight | Vector-style **2–3 px** tapered outlines, hand-drawn feel |
| Shading | Soft **2-tone cel** — no airbrush, no realism |
| Eyes | Adult-readable, **not** oversized moe |
| Hair | Stylised mass (one volume), not strand-by-strand |
| Background | Transparent, no environment |

Banned: VRoid 3D exports, photo-real, sexualised tropes, dark / horror
register, micro-skirt fan-service.

## 4. Palette (locked)

| Role | Hex |
|---|---|
| Blouse | `#FFFFFF` (crisp white) |
| Skirt / vest | `#2C3E50` (midnight navy) |
| Hair | `#4A3728` (warm espresso) |
| Skin | `#FDF5E6` (old lace / soft cream) |
| Accent | `#E74C3C` (ruby red — tie / glasses rim) |

Glasses are part of the character — thin frame, not cosplay/fashion.

## 5. Deliverables — 6 states

Each state is a short looping or one-shot animation. Frame counts and
target FPS below match the engine's expectations (`docs/TDD.md` §14.3,
`docs/CHARACTER-PLAYBOOK.md` §1.1).

| State | Frames × FPS | Loop? | Description |
|---|---|---|---|
| `idle` | 12 × 12 | yes | Neutral polite posture, hands clasped at waist. Subtle shoulder rise/fall (breathing). Double-blink at frames 4 + 10 |
| `happy` | 15 × 15 | no, → idle | Smile bloom — eyes curve to crescents, small blush, hands move toward chest. Returns to neutral on last frame |
| `happy_relieved` | 20 × 12 | no, → idle | Eyes close, head tilts back, shoulders drop in long exhale. One hand adjusts glasses. Dramatic state — fired after a long task finally succeeds |
| `focused` | 12 × 12 | yes | Expression sharpens. Holds a translucent holographic clipboard (or folder). Eyes scan left-to-right. "Glasses glint" effect every ~3s |
| `warning` | 18 × 18 | yes | One hand up palm-out ("stop / wait"), concerned sweat-drop pulses near temple. Slight forward lean |
| `sleepy` | 10 × 10 | yes | Head slowly nods forward then jerks back. Half-moon "drowsy" eyes. Optional small "z" bubble |

**Bonus** (not required, nice-to-have):
- `preview.png` — 512×512 thumbnail (used in the character library
  picker). Pose like `idle` mid-frame, full body visible.

Total: **87 frames** + 1 thumbnail.

## 6. File format & naming

- Format: **PNG with alpha channel**, 512×512 per frame
- Naming: `frame_00.png` … `frame_NN.png` per state, zero-padded
- Folder structure (artist hands back as a zip):
  ```
  linh/
  ├── idle/frame_00.png … frame_11.png
  ├── happy/frame_00.png … frame_14.png
  ├── happy_relieved/frame_00.png … frame_19.png
  ├── focused/frame_00.png … frame_11.png
  ├── warning/frame_00.png … frame_17.png
  ├── sleepy/frame_00.png … frame_09.png
  └── preview.png
  ```
- Backgrounds **fully transparent**. No matte fringe (use Krita's
  "Color → Adjust → Threshold alpha" if needed before export).

## 7. License + redistribution

I need rights to **publish + redistribute** the artwork as part of the
open-source repo. Two acceptable licenses:

- **MIT** — full freedom (rare in art commissions, fine if offered)
- **CC-BY-4.0** — credit + redistribution allowed; my preferred path

**Not acceptable**: CC-BY-NC (no commercial), all-rights-reserved,
"personal use only". The repo is MIT-licensed; the character ships
with it on every clone.

Artist gets:
- Credit in `characters/linh/LICENSE` + `manifest.json::author` (with
  optional URL to portfolio)
- Credit in the project README's "Characters" section
- Rights to use the artwork in their portfolio without restriction

## 8. Budget + timeline

> Replace `<...>` placeholders with your real numbers before sending.

- **Budget**: <e.g. $400–$600 USD for the full 6-state set>
- **Timeline**: <e.g. 3–4 weeks from approved keyframes>
- **Milestones**:
  1. Rough sketches for all 6 states (charge 25%)
  2. Approved line art for all 6 states (charge 25%)
  3. Final coloured + cel-shaded keyframes (charge 25%)
  4. All in-between frames + `preview.png` (charge final 25%)
- **Revisions**: 2 rounds included per milestone. Out-of-scope changes
  (new states, palette swaps) billed hourly.

## 9. Communication

- Primary channel: <Discord / Twitter DM / email>
- Updates expected: weekly check-in with WIP screenshots
- Approval: I'll respond within 48h to any milestone delivery

## 10. Mood board

Attached: `linh-moodboard.pur` (PureRef file) + `linh-moodboard.preview.png`.

If you don't have PureRef, the moodboard reference list is in
[`linh-moodboard.md`](./linh-moodboard.md) — the search queries section
is the fastest way to assemble your own working board.

---

## Send-off

If anything in this brief contradicts the mood board, the **mood board
wins** for visual direction; the brief wins for technical / licensing
constraints. Clarify before starting if unsure.

Thanks for considering — looking forward to seeing Linh in your style.
