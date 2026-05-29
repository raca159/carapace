# Content Revamp: Asset Design & Lore Alignment Plan

**Issue:** CAR-271
**Author:** CTO / Game Designer
**Date:** 2026-05-26
**Status:** Planning — awaiting execution

---

## 1. Executive Summary

We have three concurrent opportunities converging:

1. **Lore Foundation:** The biological grimdark sci-fi world is fully defined in `docs/initial-concept/` — a 4-tier ecosystem (Great Carapace → Sanguine Elite → Familiars → Free Humanity/Remnant) with telomerase-based biology, no fantasy elements.

2. **Sprite Pipeline:** A Bevy GPU-accelerated render pipeline (`pipeline-bevy` feature) already exists with a sprite atlas system (`crates/render/src/spritesheet.rs`) that builds texture atlases from bitmap glyph data. The current `assets/sprites/atlas.toml` maps tile/entity types to glyph+color combos — suitable for terminal rendering but not for actual pixel art.

3. **New Hires:** A StoryTeller and an Asset Designer are now available to produce lore-aligned visual content.

**This plan coordinates all three** — ensuring the sprites/tiles we create are grounded in the established lore, the pipeline can render them efficiently, and the team has clear deliverables.

---

## 2. Current State Assessment

### What exists (good)

| Asset | Status |
|-------|--------|
| `spritesheet.rs` (atlas builder) | ✅ Built — generates texture atlases from glyph bitmaps, supports `tile_map`/`entity_map` lookup by string key |
| `bevy_pipeline.rs` (renderer) | ✅ Built — 3 render systems (tiles, dungeon tiles, entities) using `SpriteAtlas` |
| `atlas.toml` (config) | ✅ Built — TOML-driven sprite definitions, extensible without Rust changes |
| Entity templates (48 types) | ✅ Built — fully aligned with Carapace lore |
| Spawn rules (30+ entries) | ✅ Built — Carapace ecosystem, no fantasy remnants |
| Biome rules (18 biomes) | ✅ Built — tile glyph/color per biome |
| Lore documents | ✅ Built — initial concept, world build, elevator pitch |

### What needs improvement

| Area | Gap |
|------|-----|
| Sprite art | Glyph-based only (colored ASCII quads). No pixel art textures exist. |
| Atlas pipeline | `build_sprite_atlas()` generates bitmaps from hardcoded 5x7 pixel glyphs. Needs real sprite support. |
| Tile variation | Single glyph per biome = no visual variety. Tiles need 2-4 variants per biome. |
| Entity sprites | One glyph per entity type. Most entities share same `atlas.toml` entries (player/creature/item/npc). No unique sprites. |
| UI assets | Panels use colored rectangles only. No decorative elements. |
| Lore-to-asset bridge | No "art bible" translating lore descriptions into visual specifications for the Asset Designer. |

### Lore alignment status

The entity templates and spawn rules are already mostly aligned with Carapace lore (per CAR-164). The remaining fantasy-tag cleanup (removing `MYSTICAL`, `MAGICAL`, `ALCHEMICAL`, etc.) is tracked in the CAR-164 plan but hasn't been executed. This is a dependency — visual assets should be designed for the correct lore, not the incorrect one.

---

## 3. Design Approach: "Pixel-Atlas" Architecture

### 3.1 Sprite Pipeline Evolution

The current `SpriteAtlas` resource provides an abstraction layer with `tile_sprite()`, `entity_sprite()`, and `glyph_sprite()` lookups. We can swap the rendering backend from procedurally-generated glyph bitmaps to loaded sprite textures without changing the interface.

**Target architecture:**

```
assets/sprites/
├── atlas.toml               # Master config (already exists, extend)
├── tiles/
│   ├── grass_0.png
│   ├── grass_1.png
│   ├── dirt_0.png
│   ├── stone_0.png
│   ├── water_0.png
│   └── ... (per biome)
├── entities/
│   ├── player.png
│   ├── trench_lobster.png
│   ├── vampire_noble.png
│   ├── familiar_zealot.png
│   ├── remnant_hunter.png
│   └── ... (per entity type)
├── items/
│   ├── iron_sword.png
│   ├── healing_potion.png
│   └── ... (per item category)
├── ui/
│   ├── panel_bg.png
│   ├── button.png
│   └── ... (UI elements)
└── effects/
    ├── fire.png
    └── ... (status effects)
```

### 3.2 Rendering Strategy

**Phase A — Sprite loading (replaces glyph bitmaps):**
- Extend `build_sprite_atlas()` to load PNG images from disk via Bevy's `AssetServer`
- Fall back to glyph-based rendering for any sprite without a PNG
- Support nearest-neighbor filtering (crisp pixel art, no bilinear blur)

**Phase B — Tile variation:**
- Each biome tile type maps to 2-4 variant sprites
- `atlas.toml`: `grass = ["grass_0", "grass_1", "grass_2"]`
- Engine picks randomly at tile generation time (seeded by position for determinism)

**Phase C — Entity animation:**
- Multi-frame sprite sheets for animated entities (walk cycles, idle flicker)
- Bevy's `SpriteAnimation` or custom frame timer
- Priority: Carapace creatures (claw animations), Sanguine (cape flow/hover)

### 3.3 Coordinate System & Tile Size

Current setup:
- `TILE_SIZE = 16.0` (world units)
- `tile_size = 32` (atlas config, pixels per glyph sprite)

**Recommendation:** 16×16 pixel base tile size for pixel art sprites. This matches:
- The current `TILE_SIZE = 16.0` rendering units
- The retro pixel art aesthetic appropriate for the game's terminal heritage
- 2× scale for retina/HD displays via zoom

Entity sprites can be up to 32×32 to allow for larger creatures (Abyssal Siege Crab) — the renderer already handles this via z-ordering.

### 3.4 Color Palette Guidelines

The Carapace world has a distinct visual identity:

| Faction | Palette Keywords | Emotional Tone |
|---------|-----------------|----------------|
| The Great Carapace | Deep purples, bioluminescent cyan/green, chitin browns, blood reds | Alien, threatening, organic horror |
| The Sanguine Elite | Crimson reds, pale flesh, dark leather, gold trim | Gothic, aristocratic, predatory |
| The Familiars | Muted purples, sickly green, grey/brown rags | Desperate, degraded, cultish |
| Free Humanity | Warm browns, weathered metals, muted greens, sky blues | Survivalist, improvised, hopeful |
| Constructs | Cold steel greys, electric blue, warning orange | Mechanical, precise, aged |
| Biomes | Per existing biome_rules.toml color scheme | Environmental |

---

## 4. Work Breakdown

### 4.1 StoryTeller — Asset Design Bible (CAR-271-ST-1)

**Output:** `docs/superpowers/art-bible.md` — a visual specification for every tile, entity, item, and UI element in the game.

**Deliverables:**
1. For each of 18 biomes: 2-3 sentence visual description + key visual elements
2. For each of 48 entity templates: physical description, size relative to 16×16 tile, distinctive features, color scheme reference
3. For each faction: visual motifs, architectural style, color palette
4. For items: icon design briefs for each item category (weapons, armor, potions, artifacts, currency)
5. For UI: design brief for panel backgrounds, HUD elements, cursor style
6. Lore-consistency review of existing tileset configs (are WFC tilesets visually aligned?)

**Acceptance criteria:**
- All 48 entity types have visual descriptions
- All 18 biomes have visual theme docs
- Faction visual motifs are documented
- Art bible is committed to `docs/superpowers/art-bible.md`
- No fantasy elements (elves, dwarves, magic) appear in the art bible

### 4.2 Asset Designer — Sprite Creation (CAR-271-AD-1)

**Output:** PNG sprite files in `assets/sprites/` following the art bible.

**Deliverables (first pass — 64 sprites target):**

**Tier 1 — Core tiles (18 sprites):**
- 2 variants each for 9 base biomes: grass, dirt, stone, sand, water, ice/snow, lava/magma, swamp, forest floor

**Tier 2 — Core entities (20 sprites):**
- Player (facing front)
- Carapace: Trench Lobster, Abyssal Dreadclaw, Spitter Crab, Molting Broodmother
- Sanguine: Vampire Noble, Vampire Enforcer, Vampire Courtesan
- Familiars: Familiar Zealot, Familiar Acolyte, Telomerase Ghoul
- Humans: Remnant Hunter, Settlement Guard, Artifact Scavenger
- Constructs: Security Drone, Cryo-Security Sentinel, Plasma Guardian
- Wildlife: Chimeric Brute, Mantis Slicer, Carrion Flapper

**Tier 3 — Core items (12 sprites):**
- Weapons: sword, bow, maul, dagger, rifle
- Armor: light armor, heavy armor, shield
- Items: healing potion, T-Fluid vial, Sanguis canister, cell battery

**Tier 4 — UI elements (14 sprites):**
- Panel backgrounds (sidebar, overlay)
- HUD elements (health bar, energy bar)
- Cursor, highlight, selection indicators
- Quest markers, faction icons
- Background/decorative elements

**Acceptance criteria:**
- All sprites are 16×16 pixels (entities may be 32×32)
- PNG format with transparency
- Nearest-neighbor friendly (no anti-aliasing at edges)
- Named following convention: `{type}_{variant}.png`
- Committed to `assets/sprites/{tiles|entities|items|ui}/`
- Atlas config updated in `assets/sprites/atlas.toml`

### 4.3 Founding Engineer — Pipeline Integration (CAR-271-FE-1)

**Output:** Updated Bevy pipeline that loads and renders the new sprites.

**Deliverables:**
1. Extend `spritesheet.rs` to load PNG textures from disk (falling back to glyph bitmaps)
2. Add tile variation support (multiple sprites per tile type, position-seeded selection)
3. Add nearest-neighbor filtering configuration
4. Support sprite sheet animations for entity sprites
5. Wire UI sprites into panel rendering
6. Test: verify all sprites render at correct position, size, and z-order
7. Test: verify fallback to glyphs works when PNG is missing
8. Update atlas config to reference new sprite files

**Acceptance criteria:**
- `cargo test --features pipeline-bevy` passes
- All sprites render at correct positions
- Glyph fallback works for missing sprites
- Tile variation produces visually distinct tiles
- Performance: 60 FPS at 200×200 map viewport
- No regression in terminal renderer

---

## 5. Implementation Order

```
Week 1: Phase 0 — Foundation
├── StoryTeller: Write Asset Design Bible (CAR-271-ST-1)
├── Founding Engineer: Extend pipeline for PNG loading (CAR-271-FE-1)
│   └── Depends on: CAR-164 lore cleanup (separate issue)
└── Asset Designer: Begin Tier 1 tiles

Week 2: Phase 1 — Core Assets
├── StoryTeller: Complete art bible, review Tier 1 sprites
├── Asset Designer: Complete Tier 1 tiles + begin Tier 2 entities
└── Founding Engineer: Complete tile variation + nearest-neighbor filtering

Week 3: Phase 2 — Full Sprite Pass
├── Asset Designer: Complete Tier 2 entities + begin Tier 3 items
├── Founding Engineer: Wire entity sprites + animations
└── CTO: Review, iterate, make pass/fail decisions

Week 4: Phase 3 — Polish
├── Asset Designer: Complete Tier 3 items + Tier 4 UI
├── Founding Engineer: Wire UI sprites, final integration
├── CTO: Full visual audit
└── All: Bug fixes, edge cases, save/load compatibility
```

---

## 6. Technical Details — Bevy Sprite Pipeline

### 6.1 Sprite Loading Extension

Current flow:
```
atlas.toml → load_atlas_config() → build_sprite_atlas()
  → resolve_bitmap(glyph) → render 5x7 pixels → Image → TextureAtlas
```

Target flow:
```
atlas.toml + PNG files → load_atlas_config() → build_sprite_atlas()
  ├── If PNG exists: load via AssetServer → Image → TextureAtlas
  └── If PNG missing: resolve_bitmap(glyph) → render pixels → fallback
```

The `SpriteLookup` struct already supports `atlas_index` + `color` + `glyph`. Add:
```rust
pub struct SpriteLookup {
    pub atlas_index: usize,
    pub color: (f32, f32, f32),
    pub glyph: char,
    pub has_texture: bool,  // true if PNG-backed, false if glyph fallback
}
```

### 6.2 Texture Atlas Layout

Current atlas: 1 row, N columns (one per glyph).  
Target: Multi-row atlas with sprite sheets.

```toml
# Extended atlas.toml format
[sprites.grass_0]
file = "tiles/grass_0.png"     # Load from PNG
# OR fallback to glyph:
[sprites.grass_0_fallback]
glyph = '"'
color = { r = 0.2, g = 0.6, b = 0.1 }

[tiles.grass]
variants = ["grass_0", "grass_1", "grass_2"]  # Multiple variants
```

### 6.3 Performance Considerations

- 200×200 map = 40,000 tile sprites. Each is a Bevy `Entity` with `Sprite` + `Transform`.
- Current system spawns/despawns entities per frame (diff sync). This is fine for terminal rendering but causes frame drops at 40k entities.
- **Optimization:** Batch rendering via `RenderLayers` or custom shader. Alternatively, keep entity pool and only update transforms.
- **Target:** 30 FPS minimum with 40k sprites + entities + UI.

---

## 7. Dependencies & Risks

| Dependency | Risk | Mitigation |
|------------|------|------------|
| CAR-164 lore cleanup | Asset Designer creates art for wrong content | Must execute CAR-164 before Phase 1 starts |
| Font licensing | Terminal font used for glyphs may not be redistributable | Use open-source fonts (DejaVu, Fira Code, Iosevka) |
| Pipeline stability | Existing Bevy pipeline has no tests | FE must add pipeline tests before refactoring |
| StoryTeller output quality | Art bible may miss details, causing rework | Review cycle: CTO reviews art bible before Asset Designer starts |
| Asset Designer throughput | 64 sprites in 3 weeks is tight | Prioritize Tier 1+2, defer Tier 3+4 if needed |

---

## 8. Success Criteria

1. Game renders with actual pixel art sprites instead of colored glyphs
2. Every entity type has a unique, lore-aligned sprite
3. Biomes have visually distinct tile sets with 2-4 variants each
4. UI panels have styled backgrounds matching the game's aesthetic
5. Terminal renderer still works (no regression)
6. `cargo test` passes
7. Player can immediately see the difference — the game looks like a game, not a terminal

---

## 9. Open Questions for CEO

1. **Sprite resolution:** 16×16 matches terminal grid. Should we go larger (24×24, 32×32) for more detail, sacrificing the retro aesthetic?
   *Recommendation:* 16×16 base, with 32×32 for large entities. Preserves retro feel, gives space for detail on bosses.

2. **Animation scope:** Walk cycles for the player? Idle animations for entities? Animated tiles (water, fire)?
   *Recommendation:* Start static. Add player movement animation in Phase 2, environmental animations in Phase 3.

3. **Lore cleanup urgency:** Should the Asset Designer start immediately or wait for CAR-164 to land?
   *Recommendation:* Asset Designer starts on biomes/tiles (no lore dependency). StoryTeller does art bible + lore review simultaneously. CAR-164 must land before entity sprites are designed.
