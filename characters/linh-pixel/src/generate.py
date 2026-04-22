#!/usr/bin/env python3
"""
Pixel Linh sprite generator v2 — dev fixture character for Shikigami.

Improvements over v1:
  - Cleaner head/face geometry (glasses don't fuse with eyes anymore)
  - Separate chars for outline vs eye vs glasses frame (distinct shades)
  - Visible arms in every pose
  - Less awkward hair silhouette (more natural volume)
  - Slightly taller canvas (28x44) gives room for proper chibi 1:2 ratio
  - Warning state: arm clearly up in "stop" gesture
  - Sleepy state: head visibly nods
  - Happy_relieved: hand actually reaches glasses

Source resolution: 28x44 per frame.
Output resolution: 512x512 per frame, 10x nearest-neighbor scale
(280x440), centered on transparent canvas.

Usage:
    python3 generate.py
"""
from __future__ import annotations

import pathlib
from PIL import Image

# ──────────────────────────────────────────────────────────────────────
# Palette
# ──────────────────────────────────────────────────────────────────────
TRANSPARENT  = (0, 0, 0, 0)
SKIN         = (253, 245, 230, 255)   # #FDF5E6
SKIN_SHADE   = (228, 210, 190, 255)
HAIR         = (74, 55, 40, 255)      # #4A3728
HAIR_HI      = (120, 90, 65, 255)
BLOUSE       = (255, 255, 255, 255)   # #FFFFFF
BLOUSE_SHADE = (220, 220, 228, 255)
NAVY         = (44, 62, 80, 255)      # #2C3E50
NAVY_HI      = (70, 92, 115, 255)
RED          = (231, 76, 60, 255)     # #E74C3C
OUTLINE      = (30, 25, 20, 255)      # dark line work
GLASS_FRAME  = (40, 35, 30, 255)      # glasses frame
EYE          = (55, 40, 30, 255)      # eye pupil (warm dark)
MOUTH        = (180, 100, 90, 255)    # mouth color
BLUSH        = (255, 180, 180, 255)
SHOE         = (30, 25, 25, 255)

CH = {
    '.': TRANSPARENT,
    'S': SKIN,         's': SKIN_SHADE,
    'H': HAIR,         'h': HAIR_HI,
    'W': BLOUSE,       'w': BLOUSE_SHADE,
    'N': NAVY,         'n': NAVY_HI,
    'R': RED,
    'O': OUTLINE,      # body/hair outline
    'G': GLASS_FRAME,  # glasses frame (distinct from outline)
    'E': EYE,          # eye pupil
    'M': MOUTH,        # mouth
    'P': BLUSH,        # cheek blush
    'K': SHOE,         # shoes
}

# ──────────────────────────────────────────────────────────────────────
# Rendering helper
# ──────────────────────────────────────────────────────────────────────
CANVAS = (512, 512)
SCALE  = 10   # 28*10 = 280, 44*10 = 440 — fits with margins

def render(rows: list[str], out_path: pathlib.Path) -> None:
    src_h = len(rows)
    src_w = len(rows[0])
    for i, r in enumerate(rows):
        assert len(r) == src_w, f"{out_path}: row {i} width {len(r)} != {src_w}"

    sprite = Image.new("RGBA", (src_w, src_h), TRANSPARENT)
    px = sprite.load()
    for y, row in enumerate(rows):
        for x, c in enumerate(row):
            px[x, y] = CH[c]

    scaled = sprite.resize((src_w * SCALE, src_h * SCALE), Image.NEAREST)
    canvas = Image.new("RGBA", CANVAS, TRANSPARENT)
    off_x = (CANVAS[0] - scaled.width) // 2
    off_y = (CANVAS[1] - scaled.height) // 2
    canvas.paste(scaled, (off_x, off_y), scaled)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out_path, "PNG")


# ──────────────────────────────────────────────────────────────────────
# Base body (rows 16-43 of 44). Appears in every state unless overridden.
# 28 wide. Arms tucked, hands near waist.
# ──────────────────────────────────────────────────────────────────────
def body_base() -> list[str]:
    return [
        ".........OSSSSSSSO..........",  # 16 neck base
        "........OSSSSSSSSSO.........",  # 17 collarbone
        ".......OWRRRRRRRWWO.........",  # 18 tie top + collar
        "......OWWWRRRRRWWWWO........",  # 19 tie narrowing
        ".....OSWWwwRRRwwWWWSO.......",  # 20 arms visible at sides + tie
        ".....OSWWwwRRRwwWWWSO.......",  # 21
        ".....OSWWwwRRRwwWWWSO.......",  # 22
        ".....OSWWwwwwwwwwWWWSO......",  # 23 blouse below tie
        ".....OSWWwwwwwwwwWWWSO......",  # 24
        "......OSWwwwwwwwwWWSO.......",  # 25 waist taper
        "......ONNNNNnnNNNNNO........",  # 26 waistband
        "......ONNnnnnnnnnNNO........",  # 27 skirt
        ".....ONNnnnnnnnnnnNNO.......",  # 28 skirt flare
        ".....ONNnnnnnnnnnnNNO.......",  # 29
        "......ONnnnnnnnnnnNO........",  # 30 skirt hem
        ".......ONnnnnnnnnNO.........",  # 31
        "........OSSSSSSSSSO.........",  # 32 upper legs
        "........OSSSSSSSSSO.........",  # 33
        ".........OSSSSSSSO..........",  # 34 mid legs
        ".........OSSSSSSSO..........",  # 35
        "........OSSSS.SSSSO.........",  # 36 lower legs (gap)
        "........OSSSS.SSSSO.........",  # 37
        ".......OKKKKK.KKKKKO........",  # 38 shoe tops
        ".......KKKKKK.KKKKKK........",  # 39 shoes
        "......KKKKKKK.KKKKKKK.......",  # 40 shoe widest
        ".......OOOOO...OOOOO........",  # 41 shoe outline
        "............................",  # 42
        "............................",  # 43
    ]

def body_warning() -> list[str]:
    """Right arm raised in 'stop' gesture, palm facing viewer."""
    return [
        ".........OSSSSSSSO.......OSO",  # 16 neck + arm raising to upper right
        "........OSSSSSSSSSO.....OSSO",  # 17 + arm rising
        ".......OWRRRRRRRWWO...OSSSO.",  # 18 + hand approaching
        "......OWWWRRRRRWWWWO.OSSSO..",  # 19 + hand
        ".....OSWWwwRRRwwWWWWO.OSO...",  # 20 + palm
        ".....OSWWwwRRRwwWWWSO.......",  # 21
        ".....OSWWwwRRRwwWWWSO.......",  # 22
        ".....OSWWwwwwwwwwWWWSO......",  # 23
        ".....OSWWwwwwwwwwWWWSO......",  # 24
        "......OSWwwwwwwwwWWSO.......",  # 25
        "......ONNNNNnnNNNNNO........",  # 26
        "......ONNnnnnnnnnNNO........",  # 27
        ".....ONNnnnnnnnnnnNNO.......",  # 28
        ".....ONNnnnnnnnnnnNNO.......",  # 29
        "......ONnnnnnnnnnnNO........",  # 30
        ".......ONnnnnnnnnNO.........",  # 31
        "........OSSSSSSSSSO.........",  # 32
        "........OSSSSSSSSSO.........",  # 33
        ".........OSSSSSSSO..........",  # 34
        ".........OSSSSSSSO..........",  # 35
        "........OSSSS.SSSSO.........",  # 36
        "........OSSSS.SSSSO.........",  # 37
        ".......OKKKKK.KKKKKO........",  # 38
        ".......KKKKKK.KKKKKK........",  # 39
        "......KKKKKKK.KKKKKKK.......",  # 40
        ".......OOOOO...OOOOO........",  # 41
        "............................",  # 42
        "............................",  # 43
    ]

def body_focused() -> list[str]:
    """Holding folder + pen at chest."""
    return [
        ".........OSSSSSSSO..........",  # 16
        "........OSSSSSSSSSO.........",  # 17
        ".......OWRRRRRRRWWO.........",  # 18
        "......OWWWRRRRRWWWWO........",  # 19
        ".....OSNNNNNNNNNNNNSO.......",  # 20 folder (navy) held at chest
        ".....OSNNNNNNNNNNNNSO.......",  # 21
        ".....OSNNNNNNNNNNNNSO.......",  # 22 pen (small S dot top-right)
        ".....OSWNNNNNNNNNNWSO.......",  # 23
        ".....OSWWwwwwwwwwWWWSO......",  # 24
        "......OSWwwwwwwwwWWSO.......",  # 25
        "......ONNNNNnnNNNNNO........",  # 26
        "......ONNnnnnnnnnNNO........",  # 27
        ".....ONNnnnnnnnnnnNNO.......",  # 28
        ".....ONNnnnnnnnnnnNNO.......",  # 29
        "......ONnnnnnnnnnnNO........",  # 30
        ".......ONnnnnnnnnNO.........",  # 31
        "........OSSSSSSSSSO.........",  # 32
        "........OSSSSSSSSSO.........",  # 33
        ".........OSSSSSSSO..........",  # 34
        ".........OSSSSSSSO..........",  # 35
        "........OSSSS.SSSSO.........",  # 36
        "........OSSSS.SSSSO.........",  # 37
        ".......OKKKKK.KKKKKO........",  # 38
        ".......KKKKKK.KKKKKK........",  # 39
        "......KKKKKKK.KKKKKKK.......",  # 40
        ".......OOOOO...OOOOO........",  # 41
        "............................",  # 42
        "............................",  # 43
    ]


# ──────────────────────────────────────────────────────────────────────
# Head variants (rows 0-15 of 44). 28 wide.
# Hair silhouette is consistent across all heads for identity.
# ──────────────────────────────────────────────────────────────────────
def _hair_top() -> list[str]:
    """Rows 0-4: upper hair crown. Same in every head."""
    return [
        "............................",  # 0
        "............OHHHHHHO........",  # 1 crown peak
        "..........OHHhhhhhhHHO......",  # 2
        ".........OHHhhhhhhhhHHO.....",  # 3
        "........OHhhSSSSSSSShhHO....",  # 4 hairline + forehead
    ]

def head_idle_a() -> list[str]:
    """Idle frame A — eyes open, chest low."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",  # 5 upper face
        "........OhSSSSSSSSSSShhHO...",  # 6
        "........OhSSGGGSSSGGGShHO...",  # 7 glasses top
        "........OhSGEEEGSGEEEGSHO...",  # 8 eyes
        "........OhSGGGGGGGGGGGSHO...",  # 9 glasses bridge (one line)
        "........OhSSSSSSSSSSSShHO...",  # 10
        ".........OSSPPSSSSPPSSSO....",  # 11 blush
        "..........OSSSSMMSSSSSO.....",  # 12 mouth (small neutral)
        "...........OSSSSSSSSSO......",  # 13
        "............OSSSSSSSO.......",  # 14 chin
        ".............OSSSSSO........",  # 15 neck
    ]

def head_idle_b() -> list[str]:
    """Idle frame B — eyes open, chest high (breath peak). Blink at this frame."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSSSSSSSSSShhHO...",
        "........OhSSGGGSSSGGGShHO...",
        "........OhSGGGGGGGGGGGGHO...",  # blink — eyes become thin line
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSSSSSSSSSSShHO...",
        ".........OSSPPSSSSPPSSSO....",
        "..........OSSSSMMSSSSSO.....",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_happy_a() -> list[str]:
    """Peak smile — crescent eyes + smile mouth."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSSSSSSSSSShhHO...",
        "........OhSSSSSSSSSSSShHO...",
        "........OhSHhhhSSShhhHShHO..",  # crescent closed eyes (happy)
        "........OhSGGGGGGGGGGGSHO...",  # glasses
        "........OhSSPPSSSSPPSSShHO..",
        ".........OSSMMMMMMMMMSSO....",  # smile mouth (wide)
        "..........OSSSSSSSSSSSO.....",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_happy_b() -> list[str]:
    """Mid-smile transition."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSSSSSSSSSShhHO...",
        "........OhSSGGGSSSGGGShHO...",
        "........OhSGEEhGSGhEEGSHO...",  # eyes half-closing
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSPPSSSSPPSSShHO..",
        ".........OSSSMMMMMMMSSSO....",
        "..........OSSSSSSSSSSSO.....",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_focused_a() -> list[str]:
    """Concentrated — brows drawn, eyes looking down-left."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSOOSSSSSOOSShHO..",  # knit brows
        "........OhSSGGGSSSGGGShHO...",
        "........OhSGEEEGSGSSSSGHO...",  # eyes look left-down
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSSSSSSSSSSShHO...",
        ".........OSSSSMMSSSSSSO.....",  # thoughtful mouth
        "..........OSSSSSSSSSSO......",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_focused_b() -> list[str]:
    """Same concentration — eyes scan right."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSOOSSSSSOOSShHO..",
        "........OhSSGGGSSSGGGShHO...",
        "........OhSGSSSSGSGEEEGSHO..",  # eyes look right-down
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSSSSSSSSSSShHO...",
        ".........OSSSSMMSSSSSSO.....",
        "..........OSSSSSSSSSSO......",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_warning_a() -> list[str]:
    """Concerned — wide eyes + sweat drop."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSOOOSSSSSOOOhHO...",  # concerned brows
        "........OhSGGGSSSGGGShHO....",
        "........OhSGEEEGSGEEEGSHO...",  # eyes wide
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSSSSSSSSSSShHO...",  # no blush in alarm
        ".........OSSSMMMMMSSSSO....P",  # open worried mouth + sweat drop P
        "..........OSSSSSSSSSSO.....P",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_warning_b() -> list[str]:
    """Alarm — mouth slightly different, sweat drop smaller."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSOOOSSSSSOOOhHO...",
        "........OhSGGGSSSGGGShHO....",
        "........OhSGEEEGSGEEEGSHO...",
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSSSSSSSSSSShHO...",
        ".........OSSSSMMMSSSSSSO....",
        "..........OSSSSSSSSSSO.....P",  # sweat drop pulsing
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_sleepy_a() -> list[str]:
    """Drowsy — half-closed eyes."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSSSSSSSSSShhHO...",
        "........OhSSGGGSSSGGGShHO...",
        "........OhSGhhhGSGhhhGSHO...",  # heavy half-closed eyes
        "........OhSGGGGGGGGGGGSHO...",
        "........OhSSPPSSSSPPSSShHO..",
        ".........OSSSSSSSSSSSSSO....",  # relaxed mouth
        "..........OSSSSSSSSSSSO.....",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_sleepy_b() -> list[str]:
    """Nodding forward — head tilts down visibly."""
    return [
        "............................",  # 0
        "............................",  # 1 head nodded 1 row down
        "............OHHHHHHO........",  # 2
        "..........OHHhhhhhhHHO......",  # 3
        ".........OHHhhhhhhhhHHO.....",  # 4
        "........OHhhSSSSSSSShhHO....",  # 5 forehead visible
        "........OHhSSSSSSSSShhHHO...",  # 6
        "........OhSSSSSSSSSSShhHO...",  # 7
        "........OhSGGGGGGGGGGGSHO...",  # 8 eyes fully closed (just frame)
        "........OhSSSSSSSSSSSShHO...",  # 9
        "........OhSSSSSSSSSSSShHO...",  # 10
        ".........OSSSSSSSSSSSSO.....",  # 11
        "..........OSSSSSSSSSSO......",  # 12 mouth still relaxed
        "...........OSSSSSSSSSO......",  # 13
        "............OSSSSSSSO.......",  # 14
        ".............OSSSSSO........",  # 15
    ]

def head_happy_relieved_a() -> list[str]:
    """Eyes closed softly, small contented smile, head tilted slightly back."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO...",
        "........OhSSSSSSSSSSShhHO...",
        "........OhSShhhSSShhhShHO...",  # soft closed eye curves
        "........OhSGGGGGGGGGGGSHO...",  # glasses
        "........OhSSSSSSSSSSSShHO...",
        "........OhSSPPSSSSPPSSShHO..",
        ".........OSSSSMMMSSSSSSO....",  # soft smile
        "..........OSSSSSSSSSSO......",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]

def head_happy_relieved_b() -> list[str]:
    """Hand approaching glasses — visible hand shape to right of head."""
    return _hair_top() + [
        "........OHhSSSSSSSSShhHHO.SS",  # hand approaching top-right
        "........OhSSSSSSSSSSShhHOSSS",
        "........OhSShhhSSShhhShHOSSS",
        "........OhSGGGGGGGGGGGSHOSSS",  # hand at glasses level
        "........OhSSSSSSSSSSSShHOSS.",
        "........OhSSPPSSSSPPSSShHO..",
        ".........OSSSSMMMSSSSSSO....",
        "..........OSSSSSSSSSSO......",
        "...........OSSSSSSSSSO......",
        "............OSSSSSSSO.......",
        ".............OSSSSSO........",
    ]


# ──────────────────────────────────────────────────────────────────────
# Composition
# ──────────────────────────────────────────────────────────────────────
def compose(head_rows: list[str], body_rows: list[str] | None = None) -> list[str]:
    body = body_rows if body_rows is not None else body_base()
    return head_rows + body


# ──────────────────────────────────────────────────────────────────────
# Main
# ──────────────────────────────────────────────────────────────────────
def main() -> None:
    here = pathlib.Path(__file__).parent.resolve()
    root = here.parent
    raw  = root / "raw"

    catalog: dict[str, list[list[str]]] = {
        "idle": [
            compose(head_idle_a()),
            compose(head_idle_b()),
        ],
        "happy": [
            compose(head_happy_a()),
            compose(head_happy_b()),
        ],
        "happy_relieved": [
            compose(head_happy_relieved_a()),
            compose(head_happy_relieved_b()),
        ],
        "focused": [
            compose(head_focused_a(), body_focused()),
            compose(head_focused_b(), body_focused()),
        ],
        "warning": [
            compose(head_warning_a(), body_warning()),
            compose(head_warning_b(), body_warning()),
        ],
        "sleepy": [
            compose(head_sleepy_a()),
            compose(head_sleepy_b()),
        ],
    }

    # Verify row widths + counts
    for name, frames in catalog.items():
        for i, rows in enumerate(frames):
            assert len(rows) == 44, f"{name}/{i}: expected 44 rows, got {len(rows)}"
            for y, r in enumerate(rows):
                assert len(r) == 28, f"{name}/{i}/row{y}: width {len(r)} != 28"

    for state, frames in catalog.items():
        for idx, rows in enumerate(frames):
            out = raw / state / f"frame_{idx:02d}.png"
            render(rows, out)
            print(f"✓ {state}/frame_{idx:02d}.png")

    preview = root / "preview.png"
    render(catalog["idle"][0], preview)
    print(f"✓ preview.png")


if __name__ == "__main__":
    main()
