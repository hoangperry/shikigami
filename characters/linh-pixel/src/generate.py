#!/usr/bin/env python3
"""
Pixel Linh sprite generator — dev fixture character for Shikigami.

Generates an 8-bit pixel-art placeholder character while the real Linh
assets are in production. All frames are rendered procedurally from
ASCII-art grids; no external image assets required.

Source resolution: 24x32 per frame.
Output resolution: 512x512 per frame, scaled 12x with nearest-neighbor
(preserves crisp 8-bit aesthetic) and centered on transparent canvas.

Usage:
    python3 generate.py
Produces PNG frames under ../raw/<state>/frame_NN.png
"""
from __future__ import annotations

import pathlib
from PIL import Image

# ──────────────────────────────────────────────────────────────────────
# Palette (RGBA tuples) — matches Linh's canonical palette where sensible
# ──────────────────────────────────────────────────────────────────────
TRANSPARENT = (0, 0, 0, 0)
SKIN        = (253, 245, 230, 255)  # #FDF5E6
SKIN_DARK   = (220, 200, 180, 255)  # shade
HAIR        = (74, 55, 40, 255)     # #4A3728
HAIR_LIGHT  = (120, 90, 65, 255)    # hair highlight
BLOUSE      = (255, 255, 255, 255)  # #FFFFFF
BLOUSE_SHADE = (220, 220, 225, 255) # blouse fold
NAVY        = (44, 62, 80, 255)     # #2C3E50
NAVY_LIGHT  = (70, 90, 115, 255)    # skirt highlight
RED         = (231, 76, 60, 255)    # #E74C3C tie
BLACK       = (30, 25, 20, 255)     # outline / glasses frame / shoes
PINK        = (255, 190, 190, 255)  # blush
EYE         = (40, 30, 20, 255)     # eye pupil

# Palette chars → color
CH = {
    '.': TRANSPARENT,
    'S': SKIN,
    's': SKIN_DARK,
    'H': HAIR,
    'h': HAIR_LIGHT,
    'W': BLOUSE,
    'w': BLOUSE_SHADE,
    'N': NAVY,
    'n': NAVY_LIGHT,
    'R': RED,
    'K': BLACK,
    'P': PINK,
    'E': EYE,
}

# ──────────────────────────────────────────────────────────────────────
# Base sprite template (24 wide × 32 tall)
#   K = outline/frame      H = hair       h = hair highlight
#   S = skin               s = skin shade P = blush
#   E = eye pupil          W = blouse    w = blouse shade
#   N = skirt (navy)       n = skirt hi  R = red tie
#   . = transparent
# ──────────────────────────────────────────────────────────────────────

def _body_template() -> list[str]:
    """Shared body rendering from neck down. Returns list of 18 rows (rows 14-31)."""
    return [
        ".........SSSS...........",  # 14 neck
        "........SWRRRWW.........",  # 15 tie knot + collar
        "........WWRRRWW.........",  # 16 tie mid-upper + collar
        "........WWWRRWW.........",  # 17 tie narrowing
        "........WwwRRwW.........",  # 18 tie + blouse shading
        "........WwwRRwW.........",  # 19
        "........WwwRRwW.........",  # 20
        "........WwwwwwW.........",  # 21 blouse under tie
        "........WwwwwwW.........",  # 22
        "........NnnNnnN.........",  # 23 skirt top (waistband)
        "........NnnnnnN.........",  # 24
        ".......NNnnnnnNN........",  # 25 skirt flare
        ".......NnnnnnnnN........",  # 26
        "........NnnnnnN.........",  # 27 skirt hem
        ".........SSSS...........",  # 28 upper legs
        ".........SSSS...........",  # 29 lower legs
        "........KKSSSKK.........",  # 30 ankles → shoe top
        ".......KKKK.KKKK........",  # 31 shoes
    ]

def render_pixels(rows: list[str], out_path: pathlib.Path, scale: int = 12,
                  canvas: tuple[int, int] = (512, 512)) -> None:
    """Render an ASCII-art grid to a PNG centered on a transparent canvas."""
    src_h = len(rows)
    src_w = len(rows[0])
    for r in rows:
        assert len(r) == src_w, f"row width mismatch in {out_path}"

    sprite = Image.new("RGBA", (src_w, src_h), TRANSPARENT)
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            sprite.putpixel((x, y), CH[ch])

    scaled = sprite.resize((src_w * scale, src_h * scale), Image.NEAREST)
    canvas_img = Image.new("RGBA", canvas, TRANSPARENT)
    off_x = (canvas[0] - scaled.width) // 2
    off_y = (canvas[1] - scaled.height) // 2
    canvas_img.paste(scaled, (off_x, off_y), scaled)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas_img.save(out_path, "PNG")


# ──────────────────────────────────────────────────────────────────────
# Head variants — head occupies rows 0-13 (14 rows, 24 wide)
# ──────────────────────────────────────────────────────────────────────

def head_idle_a() -> list[str]:
    """Eyes open, neutral. Frame A of idle breathing cycle (chest at rest)."""
    return [
        "........................",  # 0
        "........HHHHHHHH........",  # 1
        ".......HHhhhhhHHH.......",  # 2
        "......HHhSSSSShhH.......",  # 3
        "......HhSSSSSSShH.......",  # 4
        "......HSSSSSSSSSH.......",  # 5
        "......HSKKSESKKSH.......",  # 6 glasses outline + eyes
        "......HSKESKSKESH.......",  # 7 eye pupils visible
        "......HSKKSSSKKSH.......",  # 8 glasses bottom
        "......HSSSSSSSSSH.......",  # 9
        "......HSSSPPPSSsH.......",  # 10 blush cheeks
        ".......HSSSSSSSsH.......",  # 11 cheeks
        "........HSSSSSsH........",  # 12 chin
        ".........SSSSSS.........",  # 13 jaw/neck
    ]

def head_idle_b() -> list[str]:
    """Eyes open, slight blink hint. Frame B of idle (chest at peak breath)."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSKKSSSSKKSH......",  # glasses wider (perspective shift)
        "......HSKSSSSSKSH.......",  # eyes slightly open less (blink)
        "......HSKKSSSSKKSH......",
        "......HSSSSSSSSSH.......",
        "......HSSSPPPSSsH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_happy_a() -> list[str]:
    """Big smile, eyes crescent (closed happy). Peak smile frame."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSKKSSSKKSSH......",
        "......HSKHKSHKKSH.......",  # crescent closed-eye lines
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSPPEPPSSH.......",  # mouth small smile
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_happy_b() -> list[str]:
    """Mid-smile, eyes beginning to crescent."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSKKSESKKSH.......",
        "......HSKEHKHKESH.......",  # eyes mid-close
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSPPEPPSSH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_focused_a() -> list[str]:
    """Brows drawn, eyes looking down-left (at folder)."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSKKSKKSSSH......",  # knitted brows
        "......HSKKSESKKSH.......",
        "......HSKEKSSKSSH.......",  # eyes look left-down
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSESSSsH.......",  # thoughtful mouth
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_focused_b() -> list[str]:
    """Same concentration, eyes scanning right (alternate frame for loop)."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSKKSKKSSSH......",
        "......HSKKSESKKSH.......",
        "......HSSSKSSEKSH.......",  # eyes look right-down
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSESSSsH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_warning_a() -> list[str]:
    """Concerned brows, small open mouth, slight sweat drop."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH......P",  # sweat drop upper-right
        "......HhSSSSSSShH......P",
        "......HSKKSSSKKSSH......",  # brows up (alarm)
        "......HSKKSESKKSH.......",
        "......HSKESKSKESH.......",  # eyes wide open
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSEEESSSsH......",  # open worried mouth
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_warning_b() -> list[str]:
    """Same concern, mouth shape slightly varied."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH......P",  # sweat drop shrinking
        "......HSKKSSSKKSSH......",
        "......HSKKSESKKSH.......",
        "......HSKESKSKESH.......",
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSESSSsH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_sleepy_a() -> list[str]:
    """Drowsy half-moon eyes, slight head nod."""
    return [
        "........................",
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSHHSSSHHSH.......",  # eyes half-closed (heavy lids)
        "......HSKKHKHKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSPPSSSsH.......",
        "......HSSSSSSSSsH.......",  # relaxed mouth (neutral)
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_sleepy_b() -> list[str]:
    """Nodding forward — head lower, more slumped."""
    return [
        "........................",
        "........................",  # head nodded down 1 row
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSHHHHHHHHSH......",  # eyes more closed
        "......HSSSSSSSSSH.......",
        "......HSSSPPSSSsH.......",
        "......HSSSSSSSSsH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_happy_relieved_a() -> list[str]:
    """Eyes closed softly, small contented smile, head slightly tilted back."""
    return [
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSKKSSSKKSSH......",
        "......HSSHKHKHSSH.......",  # eyes closed soft curves
        "......HSKKSSSKKSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSESSSsH.......",  # soft mouth corner up
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]

def head_happy_relieved_b() -> list[str]:
    """Similar to A but hand up near glasses (indicated by extra pixels beside head)."""
    return [
        "........HHHHHHHH........",
        ".......HHhhhhhHHH.......",
        "......HHhSSSSShhH.......",
        "......HhSSSSSSShH.......",
        "......HSSSSSSSSSH.......",
        "......HSKKSSSKKSSH.SSSS.",  # glasses + hand approaching (right side skin)
        "......HSSHKHKHSSH..SSSS.",  # hand silhouette at head level
        "......HSKKSSSKKSH...SSS.",
        "......HSSSSSSSSSH.......",
        "......HSSSSSSSSSH.......",
        "......HSSSSESSSsH.......",
        ".......HSSSSSSSsH.......",
        "........HSSSSSsH........",
        ".........SSSSSS.........",
    ]


# ──────────────────────────────────────────────────────────────────────
# Compose head + body into full frame
# ──────────────────────────────────────────────────────────────────────

def compose(head_rows: list[str], body_rows: list[str] | None = None) -> list[str]:
    body = body_rows if body_rows is not None else _body_template()
    return head_rows + body


def warning_body_arm_raised() -> list[str]:
    """Body variant for warning state — right arm raised in 'stop' gesture."""
    return [
        ".........SSSS...........",  # 14 neck
        "........SWRRRWW......S..",  # 15 tie + hand appearing upper right
        "........WWRRRWW.....SSS.",  # 16 hand forming
        "........WWWRRWW....SSSS.",  # 17 hand rising
        "........WwwRRwW....SSSS.",  # 18
        "........WwwRRwW....SSSS.",  # 19
        "........WwwRRwW.........",  # 20
        "........WwwwwwW.........",  # 21
        "........WwwwwwW.........",  # 22
        "........NnnNnnN.........",  # 23 skirt
        "........NnnnnnN.........",  # 24
        ".......NNnnnnnNN........",  # 25
        ".......NnnnnnnnN........",  # 26
        "........NnnnnnN.........",  # 27
        ".........SSSS...........",  # 28 legs
        ".........SSSS...........",  # 29
        "........KKSSSKK.........",  # 30
        ".......KKKK.KKKK........",  # 31
    ]


# ──────────────────────────────────────────────────────────────────────
# Main — produce all state frames
# ──────────────────────────────────────────────────────────────────────

def main() -> None:
    here = pathlib.Path(__file__).parent.resolve()
    root = here.parent
    raw = root / "raw"

    # (state_name, list of frame-row-grids)
    catalog = {
        "idle":           [compose(head_idle_a()),
                           compose(head_idle_b())],
        "happy":          [compose(head_happy_a()),
                           compose(head_happy_b())],
        "happy_relieved": [compose(head_happy_relieved_a()),
                           compose(head_happy_relieved_b())],
        "focused":        [compose(head_focused_a()),
                           compose(head_focused_b())],
        "warning":        [compose(head_warning_a(), warning_body_arm_raised()),
                           compose(head_warning_b(), warning_body_arm_raised())],
        "sleepy":         [compose(head_sleepy_a()),
                           compose(head_sleepy_b())],
    }

    for state, frames in catalog.items():
        for idx, rows in enumerate(frames):
            out = raw / state / f"frame_{idx:02d}.png"
            render_pixels(rows, out)
            print(f"✓ {state}/frame_{idx:02d}.png")

    # preview = idle frame 0
    preview = root / "preview.png"
    render_pixels(catalog["idle"][0], preview)
    print(f"✓ preview.png")

    print("\nAll frames generated. Pack with shikigami pack when CLI lands.")


if __name__ == "__main__":
    main()
