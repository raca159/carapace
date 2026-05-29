# Voidshell — Visual Design System

## Palette

All colors derived from existing TOML configs in `assets/config/wfc_tilesets/` and `assets/config/entity_templates.toml`. Terminal-first constraint: every sprite color maps to a terminal glyph color.

### Background / Void Tones
| Token | RGB | Use |
|-------|-----|-----|
| `bg_void` | (8,8,14) | Out-of-bounds, void |
| `bg_dark` | (16,16,26) | Interior shadows, deep water |
| `bg_mid` | (30,30,45) | General dark background |

### Material Ramps (3-shade)
| Material | Dark | Mid | Light |
|----------|------|-----|-------|
| Rock | (50,52,62) | (70,73,85) | (95,98,110) |
| Dirt | (55,40,25) | (75,55,35) | (100,75,50) |
| Grass | (20,50,18) | (35,75,30) | (55,100,45) |
| Sand | (90,80,55) | (130,115,80) | (160,140,100) |
| Metal | (55,58,65) | (80,84,92) | (110,114,120) |
| Water | (10,25,65) | (15,45,100) | (25,70,135) |

### Bioluminescent Accents
| Token | RGB | Source |
|-------|-----|--------|
| `cyan` | (0,200,220) | Entity `Lurejaw` lure, player glow |
| `cyan_dim` | (0,120,150) | Dim version |
| `magenta` | (200,40,100) | `Molting Broodmother` accent |
| `magenta_dim` | (120,20,70) | Dim version |
| `green_glow` | (80,200,60) | Console screens, toxin |
| `yellow_glow` | (220,200,40) | Generators, compound eyes |
| `red_glow` | (220,40,40) | Sanguine eyes, damage |

### Organic / Chitin
| Token | RGB | Source |
|-------|-----|--------|
| `chitin_dark` | (45,15,30) | `Trench Lobster` carapace |
| `chitin_mid` | (75,30,50) | Carapace body |
| `chitin_lit` | (100,50,70) | Highlight |
| `flesh_dark` | (60,20,35) | Ghoul exposed tissue |
| `flesh_mid` | (90,35,55) | Vampire skin |

### Human
| Token | RGB | Source |
|-------|-----|--------|
| `skin_dark` | (70,55,45) | Remnant skin shadow |
| `skin_mid` | (110,85,65) | Remnant skin base |
| `skin_lit` | (150,120,90) | Highlight |

---

## Sprite Layout Conventions

### 16x16 Grid
Each sprite occupies 16x16 pixels. Spritesheets use a regular grid with 2px padding between sprites.

### Padding Formula
```
cell_width  = sprite_size + padding = 18px
cell_height = sprite_size + padding = 18px
sprite_px(col, row) = (col * 18, row * 18)
```

### Spritesheet Index
- `tiles_terrain.png` — 4 cols x 3 rows (see `tiles_terrain.toml`)
- `creatures_carapace.png` — 4 cols x 4 rows (see `creatures_carapace.toml`)
- `ui_icons.png` — 4 cols x 3 rows (see `ui_icons.toml`)

---

## Design Rules

### Pixel Art Craft
- Use 3-shade ramps per material (dark / mid / light)
- Avoid dithering — solid-color clusters at 16x16
- Maintain cluster integrity: no isolated single-pixel noise
- Anti-aliasing only at diagonal edges

### Readability at Scale
- Test at 1x (16x16) and 2x (32x32 with nearest-neighbor)
- No two entity types share an ambiguous silhouette
- Player is cyan humanoid — the only cyan entity

### Color Contrast
- Background tiles: muted, desaturated (recede)
- Interactive entities: high contrast, warm-cool separation
- Squint test: standing player must still be findable

### Mood Consistency
- Grimdark biological aesthetic
- Dark desaturated backgrounds
- Bioluminescent accents only for significant elements
- No candy colors or high-chroma primaries

---

## HUD Information Hierarchy

```
[HUD top bar layout]
┌──────────────────────────────────────────────────┐
│  HP: 45/100 (45%)  Biome: Trench  Pos: (12,8)   │
│  Weapon: Claw  Armor: Plate                      │
└──────────────────────────────────────────────────┘

[Viewport]
│  (game world tiles + entities rendered here)     │

[Message bar]
└> You see a Trench Lobster lurking in the dark.
```

### Priority (most to least prominent)
1. HP value + bar — player's survival state
2. Position — situational awareness
3. Biome name — context cue
4. Equipment state — combat readiness
5. Message log — narrative / feedback

### UI Icon Set
| Icon | Purpose | Visual |
|------|---------|--------|
| HP | Health indicator | Red heart |
| Position | Spatial awareness | Cyan crosshair |
| Weapon | Equipment state | Crossed blades |
| Armor | Equipment state | Shield emblem |
| Backpack | Inventory state | Bag with buckle |
| Quest | Active quests | Scroll |
| Craft | Recipe list | Anvil + hammer |
| Examine | Inspection mode | Eye / magnifier |
| Message | Log indicator | Chat bubble |

---

## File Manifest

```
assets/sprites/
├── tiles_terrain.png       # 16x16 tile sprites (4x3 grid)
├── tiles_terrain.toml      # Tile sprite → grid position mapping
├── creatures_carapace.png  # 16x16 creature sprites (4x4 grid)
├── creatures_carapace.toml # Creature sprite → grid position mapping
├── ui_icons.png            # 16x16 UI icons (4x3 grid)
├── ui_icons.toml           # UI icon → grid position mapping
├── atlas.toml              # Glyph-based atlas config (existing)
└── design-system.md        # This document
```

---

## Handoff Notes for Coder

To integrate sprites:
1. Load the appropriate PNG spritesheet as a texture
2. Parse the companion TOML for sprite positions
3. Build a `TextureAtlasLayout` from the grid
4. Replace `glyph_index` lookups in `bevy_pipeline.rs` with sprite-index lookups
5. Tile types and entity types from `atlas.toml` map directly to the [sprites] keys in the companion TOMLs
