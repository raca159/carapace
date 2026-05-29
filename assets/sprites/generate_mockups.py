#!/usr/bin/env python3
"""Generate 16x16 pixel art mockup spritesheets for Carapace (Voidshell).

Produces three PNG spritesheets:
  - tiles_terrain.png   — core terrain tiles (grass, dirt, stone, sand, water, wall, floor, etc.)
  - creatures_carapace.png — player, carapace enemies, vampires, humans, constructs
  - ui_icons.png        — HUD/UI icon set (HP, position, equipment, etc.)

Each sheet has a companion TOML mapping file.

Design constraints:
  - Grimdark biological aesthetic: dark desaturated backgrounds, bioluminescent accents
  - Terminal-first palette: colors match existing TOML configs
  - 16x16 per sprite, 2px padding between sprites
  - Silhouette readability at 1x and 2x
"""

from PIL import Image
import os

OUT = os.path.dirname(os.path.abspath(__file__))

SP = 16    # sprite pixel size
PAD = 2    # padding between sprites
CELL = SP + PAD

# ─── Color Palette (derived from existing TOML configs) ──────────────────────

# Dark backgrounds / base tones
BG_VOID    = (8, 8, 14)
BG_DARK    = (16, 16, 26)
BG_MID     = (30, 30, 45)

# Material ramps (3-shade)
ROCK_DARK  = (50, 52, 62)
ROCK_MID   = (70, 73, 85)
ROCK_LIT   = (95, 98, 110)

DIRT_DARK  = (55, 40, 25)
DIRT_MID   = (75, 55, 35)
DIRT_LIT   = (100, 75, 50)

GRASS_DARK = (20, 50, 18)
GRASS_MID  = (35, 75, 30)
GRASS_LIT  = (55, 100, 45)

SAND_DARK  = (90, 80, 55)
SAND_MID   = (130, 115, 80)
SAND_LIT   = (160, 140, 100)

METAL_DARK = (55, 58, 65)
METAL_MID  = (80, 84, 92)
METAL_LIT  = (110, 114, 120)

WATER_DARK = (10, 25, 65)
WATER_MID  = (15, 45, 100)
WATER_LIT  = (25, 70, 135)

# Bioluminescent accents
CYAN       = (0, 200, 220)
CYAN_DIM   = (0, 120, 150)
MAGENTA    = (200, 40, 100)
MAGENTA_DIM= (120, 20, 70)
GREEN_GLOW = (80, 200, 60)
GREEN_DIM  = (50, 130, 40)
YELLOW_GLOW= (220, 200, 40)
RED_GLOW   = (220, 40, 40)

# Organic / flesh
FLESH_DARK = (60, 20, 35)
FLESH_MID  = (90, 35, 55)
CHITIN_DARK= (45, 15, 30)
CHITIN_MID = (75, 30, 50)
CHITIN_LIT = (100, 50, 70)

# Human tones
SKIN_DARK  = (70, 55, 45)
SKIN_MID   = (110, 85, 65)
SKIN_LIT   = (150, 120, 90)

# UI colors
UI_BG      = (12, 12, 20)
UI_BORDER  = (60, 60, 80)
UI_TEXT    = (180, 180, 190)
UI_HP      = (200, 30, 30)
UI_HP_BG   = (50, 10, 10)

WHITE = (255, 255, 255)


def new_sheet(cols, rows):
    """Create a new transparent spritesheet canvas."""
    w = cols * CELL - PAD
    h = rows * CELL - PAD
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def pos(col, row):
    """Top-left pixel coordinate for sprite at (col, row)."""
    x = col * CELL
    y = row * CELL
    return x, y


def put(sheet, col, row, pixels, offset_x=0, offset_y=0):
    """Paint a 16x16 sprite onto the sheet at (col, row).

    `pixels` is a 16-element list of 16-element lists, each entry either
    None (transparent) or an (R,G,B) tuple.
    """
    ox, oy = pos(col, row)
    for py in range(16):
        for px in range(16):
            c = pixels[py][px]
            if c is not None:
                sheet.putpixel((ox + px + offset_x, oy + py + offset_y), c)


def hline(y, color):
    """Return a 16-pixel row with a horizontal line at the given y offset."""
    row = [None] * 16
    row[y] = color
    return row


# ═══════════════════════════════════════════════════════════════════════════════
# TILE SPRITES  (4 cols x 3 rows)
# ═══════════════════════════════════════════════════════════════════════════════

def sprite_grass():
    p = [[None]*16 for _ in range(16)]
    # Dark earthy ground with grass tufts
    for y in range(16):
        for x in range(16):
            if y >= 6:
                p[y][x] = DIRT_DARK if y < 10 else (DIRT_MID if ((x+y)%3==0) else DIRT_DARK)
    # Grass blades
    tufts = [(3,5),(7,4),(11,5),(5,3),(13,4)]
    for tx, ty in tufts:
        p[ty][tx] = GRASS_LIT
        if ty+1 < 16: p[ty+1][tx] = GRASS_MID
        if ty+1 < 16 and tx+1 < 16: p[ty+1][tx+1] = None
    # Sparse lighter dots
    for x in range(0,16,4):
        for y in range(10,15,2):
            if (x+y)%5==0:
                p[y][x] = GRASS_DARK
    return p


def sprite_dirt():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = DIRT_DARK
    # Speckle with lighter bits
    for _ in range(20):
        dx, dy = _, _ % 16
        p[(_*7)%16][(_*13)%16] = DIRT_MID
        p[(_*11)%16][(_*5)%16] = DIRT_LIT
    return p


def sprite_stone():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            base = ROCK_DARK
            # Crack lines
            if (x == 5 or x == 11) and y > 2 and y < 14:
                base = ROCK_MID
            if y == 8 and x > 6 and x < 13:
                base = ROCK_MID
            p[y][x] = base
    # Highlight edges
    for x in range(16):
        if x < 2 or x > 13: p[0][x] = ROCK_LIT
    for y in range(16):
        if y < 2 or y > 13: p[y][0] = ROCK_LIT
    return p


def sprite_sand():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = SAND_DARK
    # Dune ripples
    for y in range(3, 14, 4):
        for x in range(16):
            if (x + y//4) % 4 == 0:
                p[y][x] = SAND_MID
    # Speckles
    for _ in range(12):
        sx = (_ * 7 + 3) % 16
        sy = (_ * 11 + 5) % 16
        p[sy][sx] = SAND_LIT
    return p


def sprite_water():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = WATER_DARK
    # Wave bands
    for y in [2, 6, 10, 14]:
        for x in range(16):
            offset = y // 2
            if (x + offset) % 4 < 2:
                p[y][x] = WATER_MID
                if y > 0: p[y-1][x] = WATER_MID
    # Highlight crests
    for y in [3, 7, 11]:
        for x in range(16):
            offset = y // 2
            if (x + offset) % 4 == 0:
                p[y][x] = WATER_LIT
    return p


def sprite_wall():
    p = [[None]*16 for _ in range(16)]
    # Metal panel wall
    for y in range(16):
        for x in range(16):
            p[y][x] = METAL_DARK
    # Panel divisions
    for y in [4, 10]:
        for x in range(16):
            c = METAL_MID if x % 4 < 3 else METAL_LIT
            p[y][x] = c
    # Rivets
    rivets = [(3,2),(12,2),(3,8),(12,8),(3,14),(12,14)]
    for rx, ry in rivets:
        for dy in range(-1,2):
            for dx in range(-1,2):
                if 0 <= rx+dx < 16 and 0 <= ry+dy < 16:
                    p[ry+dy][rx+dx] = METAL_LIT
        p[ry][rx] = (160, 165, 175)
    return p


def sprite_floor():
    p = [[None]*16 for _ in range(16)]
    # Metal grating floor
    for y in range(16):
        for x in range(16):
            p[y][x] = METAL_DARK
    # Grating lines
    for y in range(0, 16, 4):
        for x in range(16):
            p[y][x] = METAL_MID
            if y+1 < 16: p[y+1][x] = METAL_MID
    for x in range(0, 16, 4):
        for y in range(16):
            p[y][x] = METAL_MID
            if x+1 < 16: p[y][x+1] = METAL_MID
    return p


def sprite_corridor():
    p = [[None]*16 for _ in range(16)]
    # Darker floor with directional striping
    for y in range(16):
        for x in range(16):
            p[y][x] = (40, 42, 55)
    # Run-length directional lines
    for y in range(0, 16, 3):
        for x in range(16):
            if x % 6 < 3:
                p[y][x] = METAL_DARK
    return p


def sprite_door():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            # Door frame
            if x < 1 or x > 14 or y < 1 or y > 14:
                p[y][x] = METAL_MID
            else:
                p[y][x] = (60, 80, 100)
    # Handle
    for dy in range(-1, 2):
        for dx in range(-1, 2):
            if 0 <= 10+dx < 16 and 0 <= 8+dy < 16:
                p[8+dy][10+dx] = METAL_LIT
    # Panel lines
    for y in [3, 7, 11]:
        for x in range(3, 13):
            p[y][x] = (50, 65, 85)
    return p


def sprite_cryo():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = METAL_DARK
    # Glass tube in center
    for x in range(5, 11):
        for y in range(2, 14):
            p[y][x] = (10, 60, 100)
    # Glow effect
    for x in range(6, 10):
        for y in range(3, 13):
            p[y][x] = (0, 80, 150)
    for x in range(7, 9):
        for y in range(4, 12):
            p[y][x] = CYAN_DIM
    p[7][7] = CYAN
    p[8][8] = CYAN
    # Frame
    for x in range(16):
        p[1][x] = METAL_MID
        p[14][x] = METAL_MID
    for y in range(16):
        p[y][4] = METAL_MID
        p[y][11] = METAL_MID
    return p


def sprite_console():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = METAL_DARK
    # Screen area
    for x in range(3, 13):
        for y in range(2, 8):
            p[y][x] = (10, 40, 20)
    # Screen glow
    for x in range(4, 12):
        for y in range(3, 7):
            p[y][x] = GREEN_DIM
    # Screen content
    for px in [5, 7, 9]:
        p[4][px] = GREEN_GLOW
    p[5][6] = GREEN_GLOW
    # Keyboard/base
    for y in range(10, 14):
        for x in range(4, 12):
            p[y][x] = METAL_MID
    # Keys
    for kx in range(5, 11, 2):
        for ky in [11]:
            p[ky][kx] = METAL_LIT
    return p


def sprite_pipe():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            p[y][x] = (20, 20, 30)
    # Vertical pipe
    for x in range(6, 10):
        for y in range(16):
            rust = (140 + (y % 3) * 10 - 10, 70 + (y % 2) * 10, 20 + (y % 4) * 5)
            p[y][x] = rust
    # Pipe highlight
    for x in range(7, 9):
        for y in range(16):
            r, g, b = p[y][x]
            p[y][x] = (min(r+30, 255), min(g+15, 255), min(b+5, 255))
    # Flanges
    for y in [0, 7, 15]:
        for x in range(5, 11):
            p[y][x] = (100, 55, 25)
    return p

TILES = [
    # Row 0: surface terrain
    ("grass", sprite_grass),
    ("dirt", sprite_dirt),
    ("stone", sprite_stone),
    ("sand", sprite_sand),
    # Row 1: dungeon tiles
    ("water", sprite_water),
    ("wall", sprite_wall),
    ("floor", sprite_floor),
    ("corridor", sprite_corridor),
    # Row 2: features
    ("door", sprite_door),
    ("cryo_chamber", sprite_cryo),
    ("console", sprite_console),
    ("pipe", sprite_pipe),
]


# ═══════════════════════════════════════════════════════════════════════════════
# CREATURE SPRITES  (4 cols x 4 rows)
# ═══════════════════════════════════════════════════════════════════════════════

def creature_player():
    p = [[None]*16 for _ in range(16)]
    # Bioluminescent cyan humanoid
    # Head
    for x in range(5, 11):
        for y in range(1, 5):
            p[y][x] = CYAN_DIM
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = CYAN
    # Eyes
    p[2][6] = WHITE
    p[2][9] = WHITE
    # Body
    for x in range(5, 11):
        for y in range(5, 10):
            p[y][x] = CYAN_DIM
    for x in range(6, 10):
        for y in range(5, 9):
            p[y][x] = CYAN
    # Arms
    for x in range(3, 5):
        for y in range(6, 9):
            p[y][x] = CYAN_DIM
    for x in range(11, 13):
        for y in range(6, 9):
            p[y][x] = CYAN_DIM
    # Legs
    for x in range(5, 8):
        for y in range(10, 14):
            p[y][x] = CYAN_DIM
    for x in range(9, 12):
        for y in range(10, 14):
            p[y][x] = CYAN_DIM
    # Feet
    for x in range(4, 8):
        p[14][x] = (0, 80, 100)
    for x in range(9, 13):
        p[14][x] = (0, 80, 100)
    return p


def creature_trench_lobster():
    p = [[None]*16 for _ in range(16)]
    # Red crustacean — carapace body
    for x in range(3, 13):
        for y in range(3, 11):
            p[y][x] = CHITIN_DARK
    for x in range(4, 12):
        for y in range(4, 10):
            p[y][x] = CHITIN_MID
    # Head
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = CHITIN_DARK
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = CHITIN_MID
    # Eyes (compound)
    p[2][6] = YELLOW_GLOW
    p[2][9] = YELLOW_GLOW
    # Claws
    for dx in range(3):
        p[5][0+dx] = CHITIN_LIT
        p[6][0+dx] = CHITIN_MID
        p[5][13+dx] = CHITIN_LIT
        p[6][13+dx] = CHITIN_MID
    # Legs (small)
    for lx in [4, 6, 8, 10]:
        for ly in [11, 12, 13]:
            if lx % 3 == 1:
                p[ly][lx] = CHITIN_DARK
    # Carapace ridge
    for x in range(6, 10):
        p[3][x] = CHITIN_LIT
    return p


def creature_dreadclaw():
    p = [[None]*16 for _ in range(16)]
    # Abyssal Dreadclaw — larger, darker, more menacing
    for x in range(2, 14):
        for y in range(2, 12):
            p[y][x] = (30, 10, 25)
    for x in range(3, 13):
        for y in range(3, 11):
            p[y][x] = (55, 20, 40)
    # Head with bioluminescent lure
    for x in range(4, 12):
        for y in range(0, 4):
            p[y][x] = (40, 15, 30)
    # Lure organ
    p[0][7] = CYAN
    p[1][7] = CYAN_DIM
    p[1][8] = CYAN_DIM
    # Massive claws
    for dx in range(4):
        p[5][0+dx] = (80, 30, 50)
        p[6][0+dx] = (65, 25, 40)
        p[5][12+dx] = (80, 30, 50)
        p[6][12+dx] = (65, 25, 40)
    # Eyes
    p[3][5] = RED_GLOW
    p[3][10] = RED_GLOW
    # Legs
    for lx in [3, 5, 8, 10]:
        for ly in [12, 13, 14]:
            p[ly][lx] = (35, 12, 28)
    return p


def creature_spitter():
    p = [[None]*16 for _ in range(16)]
    # Spitter Crab — orange/yellow chemical
    for x in range(4, 12):
        for y in range(3, 10):
            p[y][x] = (120, 60, 10)
    for x in range(5, 11):
        for y in range(4, 9):
            p[y][x] = (160, 90, 15)
    # Chemical gland (back)
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = (180, 140, 0)
    for x in range(6, 10):
        p[2][x] = (220, 180, 20)
    # Gland nozzle
    p[1][7] = (200, 120, 0)
    # Eyes
    p[4][5] = (100, 60, 5)
    p[4][10] = (100, 60, 5)
    # Legs
    for lx in [4, 7, 10]:
        for ly in [10, 11, 12]:
            p[ly][lx] = (100, 50, 8)
        p[13][lx] = (80, 40, 5)
    return p


def creature_broodmother():
    p = [[None]*16 for _ in range(16)]
    # Massive body
    for x in range(1, 15):
        for y in range(3, 13):
            p[y][x] = (50, 20, 40)
    for x in range(2, 14):
        for y in range(4, 12):
            p[y][x] = (70, 30, 55)
    # Head (small relative to body)
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = (55, 22, 42)
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = (75, 32, 58)
    # Many eyes
    for ex in [5, 7, 9]:
        p[2][ex] = RED_GLOW
    # Egg sac (underside)
    for x in range(4, 12):
        for y in range(12, 15):
            p[y][x] = (90, 50, 70)
    # Many legs
    for i in range(7):
        lx = 2 + i*2
        p[13][lx] = (40, 15, 30)
    return p


def creature_pressure_crawler():
    p = [[None]*16 for _ in range(16)]
    # Small dog-sized carapace creature
    for x in range(4, 12):
        for y in range(4, 10):
            p[y][x] = (55, 40, 70)
    for x in range(5, 11):
        for y in range(5, 9):
            p[y][x] = (75, 55, 90)
    # Head
    for x in range(6, 10):
        for y in range(2, 5):
            p[y][x] = (60, 45, 75)
    for x in range(7, 9):
        for y in range(3, 5):
            p[y][x] = (80, 60, 95)
    # Eyes
    p[3][6] = CYAN
    p[3][9] = CYAN
    # Tail
    for x in range(2, 5):
        p[5][x] = (50, 35, 65)
        p[6][x] = (45, 30, 60)
    # Legs
    for lx in [5, 7, 9]:
        for ly in [10, 11]:
            p[ly][lx] = (50, 35, 65)
    return p


def creature_siege_crab():
    p = [[None]*16 for _ in range(16)]
    # Huge fortress-like crab
    for x in range(0, 16):
        for y in range(3, 13):
            p[y][x] = (35, 12, 28)
    for x in range(1, 15):
        for y in range(4, 12):
            p[y][x] = (50, 18, 38)
    # Mineral-encrusted shell
    for y in range(3, 6):
        for x in range(2, 14):
            p[y][x] = (60, 25, 45)
    for x in range(3, 13):
        p[3][x] = (75, 35, 55)
    # Crystal growths
    for (cx, cy) in [(2,4), (5,2), (10,2), (13,5)]:
        p[cy][cx] = MAGENTA_DIM
        if cy+1 < 16: p[cy+1][cx] = MAGENTA
    # Small eyes
    p[5][6] = YELLOW_GLOW
    p[5][9] = YELLOW_GLOW
    # Many legs
    for i in range(6):
        lx = 2 + i*2
        p[12][lx] = (30, 10, 22)
        p[13][lx] = (25, 8, 18)
        if lx+1 < 16: p[12][lx+1] = (30, 10, 22)
        if lx+1 < 16: p[13][lx+1] = (25, 8, 18)
    return p


def creature_lurejaw():
    p = [[None]*16 for _ in range(16)]
    # Angler-style predator
    for x in range(4, 12):
        for y in range(3, 10):
            p[y][x] = (15, 70, 85)
    for x in range(5, 11):
        for y in range(4, 9):
            p[y][x] = (25, 100, 120)
    # Head / jaws
    for x in range(2, 14):
        for y in range(1, 4):
            p[y][x] = (20, 80, 95)
    # Lure
    p[0][6] = CYAN
    p[0][7] = WHITE
    p[1][7] = CYAN_DIM
    # Teeth
    for tx in [3, 5, 7, 9, 11]:
        p[3][tx] = WHITE
        p[4][tx] = WHITE
    # Eye
    p[2][5] = YELLOW_GLOW
    p[2][10] = YELLOW_GLOW
    # Fins
    for fy in range(5, 9):
        p[fy][1] = (10, 55, 70)
        p[fy][14] = (10, 55, 70)
    # Tail
    for tx in range(12, 15):
        p[8][tx] = (15, 60, 75)
        p[9][tx] = (12, 50, 65)
    return p


# creature_vampire_noble defined below


def creature_vampire_noble():
    p = [[None]*16 for _ in range(16)]
    for x in range(5, 11):
        for y in range(1, 5):
            p[y][x] = SKIN_DARK
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = SKIN_MID
    # Hair
    for x in range(4, 12):
        p[0][x] = (15, 5, 10)
        p[1][x] = (25, 8, 15)
    # Coat
    for x in range(4, 12):
        for y in range(5, 10):
            p[y][x] = (35, 8, 18)
    for x in range(5, 11):
        for y in range(5, 9):
            p[y][x] = (55, 12, 28)
    # Red sash
    for x in range(7, 9):
        p[6][x] = RED_GLOW
    # Eyes
    p[3][6] = RED_GLOW
    p[3][9] = RED_GLOW
    # Legs
    for x in range(6, 8):
        for y in range(10, 14):
            p[y][x] = (25, 6, 12)
    for x in range(9, 11):
        for y in range(10, 14):
            p[y][x] = (25, 6, 12)
    return p


def creature_vampire_enforcer():
    p = [[None]*16 for _ in range(16)]
    # Bruiser vampire
    for x in range(4, 12):
        for y in range(1, 4):
            p[y][x] = (60, 20, 30)
    for x in range(5, 11):
        for y in range(2, 4):
            p[y][x] = (80, 30, 40)
    # Armored body
    for x in range(3, 13):
        for y in range(4, 10):
            p[y][x] = (50, 15, 30)
    for x in range(4, 12):
        for y in range(4, 9):
            p[y][x] = (75, 25, 45)
    # Chest plate
    for x in range(6, 10):
        p[5][x] = METAL_MID
        p[6][x] = METAL_MID
    # Eyes (menacing)
    p[2][5] = RED_GLOW
    p[2][10] = RED_GLOW
    # Large fists
    for dx in range(3):
        p[7][1+dx] = (65, 20, 35)
        p[7][12+dx] = (65, 20, 35)
    # Legs
    for x in range(5, 8):
        for y in range(10, 13):
            p[y][x] = (40, 12, 25)
    for x in range(9, 12):
        for y in range(10, 13):
            p[y][x] = (40, 12, 25)
    return p


def creature_familiar():
    p = [[None]*16 for _ in range(16)]
    # Cultist — tattered robes
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = (55, 30, 50)
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = (75, 40, 65)
    # Hood
    for x in range(4, 12):
        p[0][x] = (40, 20, 35)
        p[1][x] = (50, 25, 45)
    # Eyes (glowing violet)
    p[2][6] = MAGENTA
    p[2][9] = MAGENTA
    # Robes
    for x in range(4, 12):
        for y in range(4, 11):
            p[y][x] = (40, 20, 35)
    for x in range(5, 11):
        for y in range(5, 10):
            p[y][x] = (60, 30, 50)
    # Ceremonial dagger
    p[7][1] = METAL_LIT
    p[8][2] = METAL_LIT
    # Legs
    for x in range(6, 8):
        for y in range(11, 13):
            p[y][x] = (35, 15, 30)
    for x in range(9, 11):
        for y in range(11, 13):
            p[y][x] = (35, 15, 30)
    return p


def creature_ghoul():
    p = [[None]*16 for _ in range(16)]
    # Collapsed, shambling form
    for x in range(4, 12):
        for y in range(2, 10):
            p[y][x] = (35, 10, 30)
    for x in range(5, 11):
        for y in range(3, 9):
            p[y][x] = (50, 18, 42)
    # Hollow eyes
    p[3][6] = (10, 2, 8)
    p[3][9] = (10, 2, 8)
    p[2][6] = YELLOW_GLOW
    p[2][9] = YELLOW_GLOW
    # Open jaw
    for x in range(6, 10):
        p[4][x] = (60, 25, 50)
    p[5][7] = RED_GLOW
    # Claw arms
    for dx in range(2):
        p[6][2+dx] = (40, 12, 35)
        p[7][2+dx] = (40, 12, 35)
        p[6][12+dx] = (40, 12, 35)
        p[7][12+dx] = (40, 12, 35)
    # Shambling legs
    for x in range(5, 8):
        for y in range(10, 13):
            p[y][x] = (30, 8, 25)
    for x in range(9, 12):
        for y in range(10, 13):
            p[y][x] = (30, 8, 25)
    return p


def creature_hunter():
    p = [[None]*16 for _ in range(16)]
    # Remnant hunter — practical gear
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = SKIN_DARK
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = SKIN_MID
    # Hat
    for x in range(4, 12):
        p[0][x] = (40, 35, 30)
        p[1][x] = (55, 45, 35)
    # Vest
    for x in range(5, 11):
        for y in range(4, 9):
            p[y][x] = (60, 50, 40)
    for x in range(6, 10):
        for y in range(4, 8):
            p[y][x] = (80, 65, 50)
    # Rifle
    for x in range(1, 6):
        p[7][x] = (100, 95, 85)
        p[8][x] = (70, 65, 55)
    # Eyes
    p[2][6] = WHITE
    p[2][9] = WHITE
    p[2][7] = (60, 55, 45)
    # Legs
    for x in range(6, 8):z
        for y in range(9, 13):
            p[y][x] = (45, 38, 30)
    for x in range(9, 11):
        for y in range(9, 13):
            p[y][x] = (45, 38, 30)
    # Boots
    for x in range(5, 8):
        p[13][x] = (35, 30, 25)
    for x in range(9, 12):
        p[13][x] = (35, 30, 25)
    return p


def creature_guard():
    p = [[None]*16 for _ in range(16)]
    # Settlement guard — uniformed
    for x in range(5, 11):
        for y in range(1, 4):
            p[y][x] = SKIN_DARK
    for x in range(6, 10):
        for y in range(2, 4):
            p[y][x] = SKIN_MID
    # Helmet
    for x in range(4, 12):
        p[0][x] = METAL_MID
        p[1][x] = METAL_LIT
    # Uniform
    for x in range(5, 11):
        for y in range(4, 9):
            p[y][x] = (40, 55, 75)
    for x in range(6, 10):
        for y in range(4, 8):
            p[y][x] = (55, 75, 100)
    # Baton
    for x in range(2, 5):
        p[7][x] = (80, 110, 140)
    # Eyes
    p[2][6] = WHITE
    p[2][9] = WHITE
    # Legs
    for x in range(6, 8):
        for y in range(9, 13):
            p[y][x] = (35, 45, 60)
    for x in range(9, 11):
        for y in range(9, 13):
            p[y][x] = (35, 45, 60)
    return p


def creature_security_drone():
    p = [[None]*16 for _ in range(16)]
    # Floating machine
    for x in range(3, 13):
        for y in range(4, 10):
            p[y][x] = (80, 80, 85)
    for x in range(4, 12):
        for y in range(5, 9):
            p[y][x] = (110, 110, 115)
    # Core glow
    for x in range(6, 10):
        for y in range(6, 8):
            p[y][x] = RED_GLOW
    p[7][7] = (255, 255, 200)
    # Sensor eye
    for x in range(7, 9):
        p[4][x] = (30, 30, 40)
    p[4][7] = RED_GLOW
    p[4][8] = RED_GLOW
    # Arms/weapons
    for dx in range(3):
        p[7][0+dx] = (90, 90, 95)
        p[7][13+dx] = (90, 90, 95)
    # Stabilizers
    for x in range(4, 6):
        p[2][x] = (60, 60, 65)
    for x in range(10, 12):
        p[2][x] = (60, 60, 65)
    return p


def creature_nanite_swarm():
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        for x in range(16):
            d = ((x-8)**2 + (y-8)**2)**0.5
            if d < 7:
                v = max(0, int(255 * (1 - d/7)))
                p[y][x] = (v//8, v//2 + 50, v)
    spots = [(7,6), (9,7), (6,9), (8,8), (10,6), (7,10)]
    for sx, sy in spots:
        if 0 <= sx < 16 and 0 <= sy < 16:
            p[sy][sx] = (200, 255, 255)
    p[8][8] = (255, 255, 255)
    return p


# Creature catalog (4 cols x 4 rows)
CREATURES = [
    ("player", creature_player),
    ("trench_lobster", creature_trench_lobster),
    ("dreadclaw", creature_dreadclaw),
    ("spitter_crab", creature_spitter),
    ("broodmother", creature_broodmother),
    ("pressure_crawler", creature_pressure_crawler),
    ("siege_crab", creature_siege_crab),
    ("lurejaw", creature_lurejaw),
    ("vampire_noble", creature_vampire_noble),
    ("vampire_enforcer", creature_vampire_enforcer),
    ("familiar_zealot", creature_familiar),
    ("ghoul", creature_ghoul),
    ("remnant_hunter", creature_hunter),
    ("settlement_guard", creature_guard),
    ("security_drone", creature_security_drone),
    ("nanite_swarm", creature_nanite_swarm),
]


# ═══════════════════════════════════════════════════════════════════════════════
# UI ICONS  (4 cols x 4 rows)
# ═══════════════════════════════════════════════════════════════════════════════

def icon_hp():
    p = [[None]*16 for _ in range(16)]
    # Heart shape
    heart = [
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, 1, 1, None, None, None, None, 1, 1, None, None, None, None,
        None, None, None, 1, 1, 1, 1, None, None, 1, 1, 1, 1, None, None, None,
        None, None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None, None,
        None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None,
        None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None,
        None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None,
        None, None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None, None,
        None, None, None, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, None, None, None,
        None, None, None, None, 1, 1, 1, 1, 1, 1, 1, 1, None, None, None, None,
        None, None, None, None, None, 1, 1, 1, 1, 1, 1, None, None, None, None, None,
        None, None, None, None, None, None, 1, 1, 1, 1, None, None, None, None, None, None,
        None, None, None, None, None, None, None, 1, 1, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ]
    for y in range(16):
        for x in range(16):
            if heart[y*16 + x] == 1:
                p[y][x] = UI_HP if x < 8 else (180, 20, 20)
    return p


def icon_position():
    p = [[None]*16 for _ in range(16)]
    # Crosshair / target
    p = [[None]*16 for _ in range(16)]
    for y in range(16):
        p[y][6] = CYAN_DIM
        p[y][9] = CYAN_DIM
    for x in range(16):
        p[6][x] = CYAN_DIM
        p[9][x] = CYAN_DIM
    p[6][6] = CYAN
    p[6][9] = CYAN
    p[9][6] = CYAN
    p[9][9] = CYAN
    p[7][7] = CYAN
    p[8][8] = CYAN
    return p


def icon_weapon():
    p = [[None]*16 for _ in range(16)]
    # Crossed swords / weapon
    for x in range(7, 9):
        for y in range(2, 12):
            p[y][x] = METAL_LIT
    p[1][8] = METAL_LIT
    p[2][7] = METAL_LIT
    # Blade edge
    for x in range(6, 10):
        p[2][x] = METAL_MID
    # Handle
    for y in range(12, 14):
        p[7][y] = (80, 50, 20)
        p[8][y] = (80, 50, 20)
    return p


def icon_armor():
    p = [[None]*16 for _ in range(16)]
    # Shield / armor plate
    for x in range(3, 13):
        for y in range(2, 14):
            p[y][x] = METAL_DARK
    for x in range(4, 12):
        for y in range(3, 13):
            p[y][x] = METAL_MID
    # Shield emblem
    for x in range(6, 10):
        for y in range(5, 11):
            p[y][x] = METAL_LIT
    # Cross
    for y in range(6, 10):
        p[y][7] = (40, 80, 120)
    for x in range(7, 9):
        p[7][x] = (40, 80, 120)
    return p


def icon_backpack():
    p = [[None]*16 for _ in range(16)]
    # Backpack shape
    for x in range(4, 12):
        for y in range(4, 12):
            p[y][x] = (70, 55, 35)
    for x in range(5, 11):
        for y in range(5, 11):
            p[y][x] = (90, 70, 45)
    # Flap
    for x in range(5, 11):
        p[4][x] = (110, 85, 55)
    # Buckle
    for x in range(7, 9):
        p[7][x] = METAL_MID
    # Straps
    for x in range(5, 7):
        for y in range(3, 5):
            p[y][x] = (55, 40, 25)
    for x in range(9, 11):
        for y in range(3, 5):
            p[y][x] = (55, 40, 25)
    return p


def icon_quest():
    p = [[None]*16 for _ in range(16)]
    # Scroll / quest marker
    for x in range(3, 13):
        for y in range(2, 14):
            p[y][x] = (100, 85, 55)
    for x in range(4, 12):
        for y in range(3, 13):
            p[y][x] = (130, 110, 70)
    # Rolled ends
    for y in range(2, 14):
        p[3][y] = (90, 75, 45)
        p[12][y] = (90, 75, 45)
    # Text lines
    for line_y in [5, 8, 11]:
        for lx in range(5, 11):
            p[line_y][lx] = (60, 50, 30)
    return p


def icon_craft():
    p = [[None]*16 for _ in range(16)]
    # Anvil / hammer
    for x in range(3, 13):
        for y in range(9, 13):
            p[y][x] = METAL_DARK
    for x in range(4, 12):
        for y in range(10, 12):
            p[y][x] = METAL_MID
    # Anvil top
    for x in range(5, 11):
        p[8][x] = METAL_MID
    for x in range(6, 10):
        p[7][x] = METAL_MID
    # Hammer
    for x in range(10, 13):
        for y in range(4, 7):
            p[y][x] = METAL_LIT
    for x in range(11, 13):
        for y in range(7, 9):
            p[y][x] = (80, 50, 20)
    return p


def icon_examine():
    p = [[None]*16 for _ in range(16)]
    # Eye / magnifying glass
    for x in range(4, 12):
        for y in range(3, 11):
            p[y][x] = (20, 20, 30)
    for x in range(5, 11):
        for y in range(4, 10):
            p[y][x] = (40, 40, 55)
    # Pupil
    for x in range(6, 10):
        for y in range(5, 9):
            p[y][x] = CYAN_DIM
    for x in range(7, 9):
        for y in range(6, 8):
            p[y][x] = CYAN
    # Handle
    for x in range(11, 14):
        for y in range(10, 13):
            p[y][x] = METAL_MID
    return p


def icon_message():
    p = [[None]*16 for _ in range(16)]
    # Chat bubble
    for x in range(2, 14):
        for y in range(2, 11):
            p[y][x] = (25, 30, 45)
    for x in range(3, 13):
        for y in range(3, 10):
            p[y][x] = (40, 50, 70)
    # Text cursor
    p[5][5] = UI_TEXT
    p[5][6] = UI_TEXT
    p[5][7] = UI_TEXT
    p[7][5] = UI_TEXT
    p[7][6] = UI_TEXT
    p[7][7] = UI_TEXT
    # Tail (triangle)
    p[11][5] = (25, 30, 45)
    p[11][6] = (25, 30, 45)
    p[10][6] = (25, 30, 45)
    return p


def icon_door_ui():
    p = [[None]*16 for _ in range(16)]
    # Door icon (simplified)
    for x in range(3, 13):
        for y in range(2, 14):
            p[y][x] = (50, 50, 60)
    for x in range(4, 12):
        for y in range(3, 13):
            p[y][x] = (70, 70, 85)
    # Door open
    for x in range(5, 8):
        for y in range(4, 12):
            p[y][x] = (90, 90, 105)
    # Arrow
    for x in range(5, 10):
        p[9][x] = (180, 180, 100)
    p[8][8] = (180, 180, 100)
    p[10][8] = (180, 180, 100)
    return p


def icon_stairs():
    p = [[None]*16 for _ in range(16)]
    # Stairs going down
    for step in range(6):
        sx = 2 + step * 2
        sy = 2 + step * 2
        for x in range(sx, 14):
            p[sy][x] = METAL_MID
            if sy+1 < 16: p[sy+1][x] = METAL_DARK
    # Arrow down
    p[12][7] = UI_TEXT
    p[12][8] = UI_TEXT
    p[11][7] = UI_TEXT
    p[13][7] = UI_TEXT
    p[12][9] = UI_TEXT
    return p


def icon_loot():
    p = [[None]*16 for _ in range(16)]
    # Treasure chest
    for x in range(3, 13):
        for y in range(5, 13):
            p[y][x] = (80, 55, 25)
    for x in range(4, 12):
        for y in range(6, 12):
            p[y][x] = (110, 75, 35)
    # Lid
    for x in range(2, 14):
        p[4][x] = (90, 60, 30)
        p[5][x] = (100, 70, 35)
    # Lock
    for x in range(7, 9):
        for y in range(7, 9):
            p[y][x] = METAL_LIT
    # Glow
    p[11][7] = YELLOW_GLOW
    p[11][8] = YELLOW_GLOW
    return p


UI_ICONS = [
    ("hp", icon_hp),
    ("position", icon_position),
    ("weapon", icon_weapon),
    ("armor", icon_armor),
    ("backpack", icon_backpack),
    ("quest", icon_quest),
    ("craft", icon_craft),
    ("examine", icon_examine),
    ("message", icon_message),
    ("door", icon_door_ui),
    ("stairs", icon_stairs),
    ("loot", icon_loot),
]


# ═══════════════════════════════════════════════════════════════════════════════
# GENERATE SPRITESHEETS
# ═══════════════════════════════════════════════════════════════════════════════

def make_sheet(catalog, cols):
    rows = (len(catalog) + cols - 1) // cols
    sheet = new_sheet(cols, rows)
    for i, (name, func) in enumerate(catalog):
        col = i % cols
        row = i // cols
        sprite = func()
        put(sheet, col, row, sprite)
    return sheet


def make_toml(catalog, cols, category):
    """Build a TOML mapping for the spritesheet."""
    lines = [
        f"# {category.title()} spritesheet mapping",
        f"# Companion to assets/sprites/{category}.png",
        f"# Grid: {cols} columns, 16x16 per sprite, 2px padding",
        f"",
        f"tile_size = 16",
        f"padding = 2",
        f"",
        f"[sprites]",
    ]
    for i, (name, _) in enumerate(catalog):
        col = i % cols
        row = i // cols
        x = col * (SP + PAD)
        y = row * (SP + PAD)
        lines.append(f'{name} = {{ col = {col}, row = {row}, x = {x}, y = {y} }}')
    lines.append("")
    return "\n".join(lines)


def main():
    # Generate tiles spritesheet
    tiles_sheet = make_sheet(TILES, 4)
    tiles_path = os.path.join(OUT, "tiles_terrain.png")
    tiles_sheet.save(tiles_path)
    print(f"Created: {tiles_path}")

    tiles_toml = make_toml(TILES, 4, "tiles")
    tiles_toml_path = os.path.join(OUT, "tiles_terrain.toml")
    with open(tiles_toml_path, "w") as f:
        f.write(tiles_toml)
    print(f"Created: {tiles_toml_path}")

    # Generate creatures spritesheet
    critters_sheet = make_sheet(CREATURES, 4)
    critters_path = os.path.join(OUT, "creatures_carapace.png")
    critters_sheet.save(critters_path)
    print(f"Created: {critters_path}")

    critters_toml = make_toml(CREATURES, 4, "creatures")
    critters_toml_path = os.path.join(OUT, "creatures_carapace.toml")
    with open(critters_toml_path, "w") as f:
        f.write(critters_toml)
    print(f"Created: {critters_toml_path}")

    # Generate UI icons spritesheet
    ui_sheet = make_sheet(UI_ICONS, 4)
    ui_path = os.path.join(OUT, "ui_icons.png")
    ui_sheet.save(ui_path)
    print(f"Created: {ui_path}")

    ui_toml = make_toml(UI_ICONS, 4, "ui")
    ui_toml_path = os.path.join(OUT, "ui_icons.toml")
    with open(ui_toml_path, "w") as f:
        f.write(ui_toml)
    print(f"Created: {ui_toml_path}")

    print("\nDone — 3 spritesheets + 3 TOML mappings generated.")


if __name__ == "__main__":
    main()
