# Weather & Location Deep Integration Design

> Date: 2026-05-28
> Status: Approved
> Depends on: tag system (tags.toml, TagRegistry, Tags component), interaction system (interactions.toml, status.rs), cascade pipeline (economy, spawner)

---

## 1. Overview

Two systems — weather and location interiors — are refactored from hardcoded Rust logic to **config-driven tag templates**. New weather types and location types are added via TOML files with zero Rust changes. Tags compose through the existing interaction pipeline to produce emergent behavior.

**Core principle:** Tags are the interface between systems. No system needs to know about another system's implementation — they read and write tags, and the interaction rules in `interactions.toml` handle the rest.

---

## 2. Environmental System

### 2.1 Two-Tier Model

Environmental effects use a two-tier approach:

**Tier 1 — Core axes with score→threshold mapping.** A small set of continuous environmental axes (light, temperature, moisture) where granular control makes sense. Each axis is a 0-100 score that crosses thresholds to produce tags. This is the same pattern as personality scores → behavioral tags.

**Tier 2 — Arbitrary modifier tags.** Open-ended tags applied directly from weather/location TOMLs. No axis needed — these are flags or qualitative descriptors. TOXIC, TAINTED, BURNING, RADIOACTIVE, whatever is needed. Just register the tag in `tags.toml` and add interaction rules if desired.

**Why two tiers:** Not every environmental effect needs a continuous score. Toxic rain isn't "70% toxic" — it's either toxic or it isn't. But light level is genuinely continuous (0-100) and maps meaningfully to DARK/DIM/BRIGHT thresholds. The core axes handle what benefits from granular control. Modifier tags handle everything else as simple on/off flags that compose through the interaction system.

### 2.2 Core Axes

| Axis | Score Range | Threshold → Tag |
|------|------------|-----------------|
| Light | 0–100 | 0-20 → DARK, 20-40 → DIM, 40-60 → (none), 60-100 → BRIGHT |
| Temperature | 0–100 | 0-15 → FREEZING, 15-35 → COLD, 35-65 → NEUTRAL, 65-85 → WARM, 85-100 → HOT |
| Moisture | 0–100 | 0-20 → DRY, 20-40 → DAMP, 40-70 → WET, 70-100 → SOAKED |

These are the **only** axes. Adding a new axis is a design decision, not the default path. Most environmental effects should use modifier tags (Tier 2) instead.

**Threshold definitions** live in `tags.toml` as `range` fields on each tag:

```toml
[[archetype.tags]]
id = "DARK"
range = [0, 20]          # light score 0-20 → DARK tag applied

[[archetype.tags]]
id = "DIM"
range = [20, 40]          # light score 20-40 → DIM tag applied

[[archetype.tags]]
id = "FREEZING"
range = [0, 15]           # temperature score 0-15 → FREEZING tag applied

[[archetype.tags]]
id = "WET"
range = [40, 70]          # moisture score 40-70 → WET tag applied
```

### 2.3 Score Composition

Environmental scores are computed from three sources, stacked:

1. **Base score from tile** — biome tags define base environmental values. Desert tiles: `temperature: 80, moisture: 10, light: 70`. Swamp tiles: `temperature: 50, moisture: 80, light: 40`.
2. **Weather modifier** — active weather adds to base scores. Rain: `moisture: +50`. Snow: `temperature: -40, moisture: +30`.
3. **Location interior override** — interior tiles replace scores entirely. Dungeon: `light: 5, temperature: 30, moisture: 20`. These are fixed regardless of weather.

**Final score = base (from tile or interior) + weather modifier**, clamped to 0-100.

### 2.4 Modifier Tags

Weather and location TOMLs can include arbitrary modifier tags alongside axis modifiers. These are applied directly as `TagId` — no score, no threshold:

```toml
# weather_toxic_rain.toml
name = "ToxicRain"
modifiers = { moisture = 50 }     # axis modifier → WET from threshold
tags = ["RAINY", "TOXIC"]         # modifier tags applied directly
```

TOXIC is just a tag registered in `tags.toml`. It doesn't need a toxicity axis. Interaction rules handle the emergent behavior: `TOXIC + FLESH → POISONED`, `TOXIC + PLANT → NECROTIC_DAMAGE`, `TOXIC + CONSTRUCT → (no effect)`. Adding a new modifier tag = register in `tags.toml` + optionally add interaction rules. No axis, no score, no threshold.

### 2.5 Weather as TOML Templates

Weather types are defined as TOML files that produce **score modifiers** and **descriptive tags**:

```toml
# assets/config/weather/weather_rain.toml
name = "Rain"
weight = 10
duration = [5, 15]
visibility = 0.6
modifiers = { moisture = 50 }     # adds 50 to moisture score → pushes past WET threshold
tags = ["RAINY"]                   # descriptive tags applied directly
```

```toml
# assets/config/weather/weather_fire_storm.toml
name = "FireStorm"
weight = 3
duration = [3, 8]
visibility = 0.4
modifiers = { temperature = 50 }   # adds 50 to temperature score → pushes past HOT threshold
tags = []                           # HOT derived from score, not direct
```

```toml
# assets/config/weather/weather_clear.toml
name = "Clear"
weight = 30
duration = [8, 25]
visibility = 1.0
modifiers = {}
tags = []
```

**Fields:**
- `name` — unique identifier, replaces the Rust enum variant
- `weight` — selection probability weight during weather rolls
- `duration` — `[min, max]` turn duration before weather changes
- `visibility` — multiplier for NPC detection range (0.0–1.0)
- `modifiers` — score deltas applied to environmental axes (`light`, `temperature`, `moisture`)
- `tags` — descriptive tags applied directly (RAINY, STORMY, FOGGY — state-of-weather, not environmental conditions)

**Time-of-day** becomes a modifier to the light axis: Night adds `light: -80`, Dusk/Dawn adds `light: -40`. The light score then crosses the DARK/DIM thresholds naturally.

**Rust side:** `WeatherState` holds the active `WeatherDef` (loaded from TOML at startup) and a timer. `advance_time()` rolls a new weather by weight when the timer expires.

### 2.5 Weather TOML Inventory

Initial set (migrating existing hardcoded types):

| File | name | visibility | modifiers | tags |
|------|------|-----------|-----------|------|
| `weather_clear.toml` | Clear | 1.0 | `{}` | — |
| `weather_cloudy.toml` | Cloudy | 0.9 | `{ light: -10 }` | — |
| `weather_fog.toml` | Fog | 0.3 | `{ moisture: 20 }` | FOGGY, REDUCED_VISIBILITY |
| `weather_rain.toml` | Rain | 0.6 | `{ moisture: 50 }` | RAINY |
| `weather_storm.toml` | Storm | 0.3 | `{ moisture: 60 }` | STORMY, WINDY |
| `weather_snow.toml` | Snow | 0.5 | `{ temperature: -40, moisture: 30 }` | SNOWY |
| `weather_sandstorm.toml` | Sandstorm | 0.3 | `{ moisture: -40 }` | REDUCED_VISIBILITY |
| `weather_ashfall.toml` | AshFall | 0.4 | `{ light: -50 }` | REDUCED_VISIBILITY |

Note: WET, COLD, HOT, DARK, DIM, DRY are NOT in `tags` — they're derived from score thresholds.

Future additions (zero Rust changes):
- `weather_fire_storm.toml` — `{ temperature: 50 }` → HOT threshold crossed
- `weather_toxic_mist.toml` — new `toxicity` axis with TOXIC tag threshold
- `weather_blood_rain.toml` — `{ moisture: 50 }` + tag TAINTED

### 2.8 Extensibility

**Adding a modifier tag (default path — no axis):**
1. Register tag in `tags.toml` (e.g., TOXIC under `interaction` or new archetype)
2. Optionally add interaction rules to `interactions.toml` (e.g., `TOXIC + FLESH → POISONED`)
3. Use it in weather TOMLs (`tags = ["TOXIC"]`) or location TOMLs
4. No Rust changes

**Adding a core axis (rare — requires design decision):**
1. Add threshold tags to `tags.toml` with `range` fields
2. Add base scores to biome rules in `biome_rules.toml`
3. Weather TOMLs gain the new modifier key
4. Update score computation and tag resolution in Rust
5. Only do this when granular continuous control genuinely adds value over a simple tag

---

## 3. Weather Tag Application Pipeline

### 3.1 Overview

Each turn, environmental scores are recomputed from tile base + weather modifiers + time-of-day. Score thresholds produce tags. Tags are applied to entities and tiles as `TagId`. The existing interaction system in `status.rs` picks them up naturally.

### 3.2 Pipeline Steps

Executed in `finish_npc_turns()` after weather advancement:

**Step 1 — Compute environmental scores.** For each tile in the player's viewport:
- Read base scores from tile's biome tags (stored on the tile entity or looked up from biome config)
- Add weather modifiers from active `WeatherDef`
- Add time-of-day modifier to light axis (Night: `light -80`, Dusk/Dawn: `light -40`)
- If tile has interior overrides (from location `[interior].environment`), use those instead of base + weather
- Clamp all scores to 0-100

**Step 2 — Resolve scores to tags.** For each axis, check which threshold range the score falls into. E.g., light score = 15 → DARK tag. Temperature score = 75 → WARM tag. Moisture score = 55 → WET tag.

**Step 3 — Remove stale environmental tags.** `WeatherContext` holds `applied_tags: Vec<TagId>` tracking what was applied last turn. Remove these from tiles and entities. Tags with tick-based persistence (WET lingers 2-3 turns) use `TagValue::Ticks`.

**Step 4 — Apply tags to tiles.** Resolved environmental tags + weather descriptive tags applied to tile entities within viewport. Tiles with `BLOCKS_WEATHER` tag (interior tiles) skip weather modifiers — they use interior override scores only. Descriptive weather tags (RAINY, STORMY) are NOT applied to interior tiles.

**Step 5 — Apply tags to entities.** Environmental tags applied to all entities with `WeatherSensitive` component. Creatures get this by default during spawning. Player is `WeatherSensitive`. FIREPROOF on an entity → `conflicts: [BURNING]` → prevents burning even if temperature score is high.

**Step 6 — Interaction system fires.** After environmental tags are applied, `process_status_effects()` runs as normal. Self-interactions fire. Cross-interactions between entities and tiles fire. No special weather code in the interaction system.

### 3.3 Visibility Integration

`WeatherDef.visibility` multiplies NPC sense range. Computed as `visibility × light_score / 100` to factor in both weather and light level. DARKVISION creatures ignore the light component. THERMAL_SENSE creatures ignore visibility entirely.

### 3.4 Data Flow

```
WeatherDef TOML files
    → loaded at startup into WeatherState resource
    → finish_npc_turns advances weather timer
    → when timer expires: roll new WeatherDef by weight

    → compute_environmental_scores():
        for each viewport tile:
          base = biome tag scores (from biome_rules.toml)
          + weather modifiers (from active WeatherDef)
          + time-of-day modifier (light axis)
          override = interior scores if BLOCKS_WEATHER tile
          clamp to 0-100

    → resolve_scores_to_tags():
        light score → DARK/DIM/none/BRIGHT based on thresholds
        temperature score → FREEZING/COLD/NEUTRAL/WARM/HOT
        moisture score → DRY/DAMP/WET/SOAKED

    → apply_environmental_tags():
        remove stale tags from last turn
        apply resolved tags + descriptive weather tags to viewport tiles
        apply resolved tags to WeatherSensitive entities

    → process_status_effects():
        tick_status on all entities
        check_self_interactions → environmental + entity tags fire rules
        check_cross_interactions → entity + tile tag rules fire

    → get_sense_range() reads visibility × light_score
```

### 3.5 Tags to Register

**New `light` archetype** in `tags.toml` (exclusivity: mutual):

| Tag | Range |
|-----|-------|
| DARK | light 0-20 |
| DIM | light 20-40 |

**New `weather` archetype** in `tags.toml` (exclusivity: any) — descriptive weather state:

| Tag | Notes |
|-----|-------|
| RAINY | It's raining |
| STORMY | Storm active |
| SNOWY | Snowing |
| FOGGY | Fog present |
| WINDY | Strong winds |
| REDUCED_VISIBILITY | Visibility impaired |
| BRIGHT | High light level |

**Existing tags that are now score-derived** (stay in their archetypes, gain `range` field):
- `temperature` archetype: FREEZING, COLD, NEUTRAL, WARM, HOT — each gets a threshold range
- `moisture` archetype: DRY, DAMP, WET, SOAKED — each gets a threshold range
- BURNING stays in `status` archetype — can be produced by interaction rules when HOT + FLAMMABLE coexist

### 3.6 Files Changed

| File | Change |
|------|--------|
| `crates/core/src/weather.rs` | Refactor: `Weather` enum → generic `WeatherDef` loader from TOML, `modifiers` HashMap, score computation, tag resolution |
| `src/game/mod.rs` | Add `apply_environmental_tags()` call in `finish_npc_turns()`, add `WeatherSensitive` component to player |
| `crates/world/src/spawner.rs` | Add `WeatherSensitive` component to all spawned creatures |
| `crates/world/src/behavior.rs` | `get_sense_range()` reads visibility × light_score |
| `crates/core/src/encounters.rs` | Convert string checks to `TagId` checks |
| `crates/core/src/npc_action.rs` | Convert string checks to `TagId` checks |
| `src/interact/talk.rs` | Convert string checks to `TagId` checks |
| `assets/config/tags.toml` | Add `light` archetype (DARK, DIM, BRIGHT), `weather` archetype, `range` fields on temperature/moisture tags |
| `assets/config/weather/*.toml` | New weather template files |
| `assets/config/biome_rules.toml` | Add base environmental scores to each biome rule |

---

## 4. Location Templates

### 4.1 Current State

`location_types.toml` defines 7 location types as flat entries with tags. No interior definition. No connection to BSP/WFC generators. The `>` key prints a message but does nothing. `MapLayer` has `active_dungeon` field but it's never populated.

### 4.2 New Design

Each location type gains an optional `[interior]` section that groups tags + generator hint + tileset + spawn rules:

```toml
[[location_type]]
id = "dungeon"
pass = 1
weight = 20
min_distance_from_same = 30
zone_radius = 10
habitability_threshold = 0.0
tags = ["HOSTILE_ZONE", "HAS_INTERIOR", "UNDERGROUND"]
biome_affinity = ["BIOME_MOUNTAIN", "BIOME_SWAMP", "BIOME_DESERT", "BIOME_TUNDRA"]

[interior]
  generator = "bsp"
  tileset = "trench_nest"
  scale = [30, 50]
  tags = ["INDOORS", "BLOCKS_WEATHER"]
  environment = { light = 5, temperature = 30, moisture = 20 }  # overrides biome base + weather
  spawn_rules = ["hostile", "loot"]
  depth_range = [1, 5]

[[location_type]]
id = "city"
pass = 1
weight = 30
min_distance_from_same = 40
zone_radius = 20
habitability_threshold = 0.6
tags = ["SETTLEMENT", "HAS_ECONOMY", "HAS_TRADE", "HAS_INTERIOR"]
biome_affinity = ["BIOME_GRASSLAND", "BIOME_TEMPERATE_FOREST"]

[interior]
  generator = "wfc"
  tileset = "human_settlement"
  scale = [40, 60]
  tags = ["INDOORS", "SETTLEMENT"]
  environment = { light = 70, temperature = 50, moisture = 30 }  # well-lit, comfortable
  spawn_rules = ["friendly", "shops"]

[[location_type]]
id = "ruin"
pass = 3
weight = 25
min_distance_from_same = 10
zone_radius = 4
habitability_threshold = 0.0
tags = ["HAS_LOOT", "HAS_INTERIOR", "UNDERGROUND"]
biome_affinity = ["BIOME_TEMPERATE_FOREST", "BIOME_DESERT"]

[interior]
  scale = [15, 25]
  tags = ["INDOORS", "BLOCKS_WEATHER", "HOSTILE_ZONE"]
  environment = { light = 25, temperature = 40, moisture = 50 }  # dim, damp
  spawn_rules = ["hostile", "loot"]
```

**`[interior]` fields:**
- `generator` — optional hint: `"bsp"` or `"wfc"`. If absent, derived from tags: `UNDERGROUND` → BSP, `SETTLEMENT` + `SURFACE` → WFC, small scale → BSP.
- `tileset` — optional WFC tileset name for the generation
- `scale` — `[min, max]` for interior map width/height
- `tags` — applied to every interior tile entity (like biome tags on overworld tiles)
- `environment` — environmental score overrides for interior tiles. `{ light: 5, temperature: 30, moisture: 20 }` replaces biome base + weather entirely. Light score 5 → crosses DARK threshold. Adding a volcanic lair: `{ light: 40, temperature: 90, moisture: 5 }` → crosses HOT threshold, dim but not dark.
- `spawn_rules` — list of tag names that drive entity spawning inside: `"hostile"` → spawn creatures using cascade spawner, `"loot"` → spawn chests using `place_dungeon_chests`, `"friendly"` → spawn NPCs with CAN_TALK/CAN_BARTER, `"shops"` → spawn shopkeeper NPCs near SETTLEMENT-tagged tiles. These are tag-based hints, not hardcoded keywords — the spawn system checks for these tags in its spawn logic.
- `depth_range` — `[min, max]` dungeon depth levels (only for `UNDERGROUND` locations)

### 4.3 Generation Dispatch Logic

```
Player presses > at a location:
  1. location_at() → PlacedLocation
  2. Does it have HAS_INTERIOR tag? No → "Nothing to enter." Yes → continue
  3. Look up [interior] from location_types.toml for this location_type
  4. Generator hint present?
     Yes → use that generator ("bsp" or "wfc")
     No → derive from tags:
       UNDERGROUND → BSP
       SETTLEMENT + SURFACE → WFC
       default → BSP
  5. Generate interior using selected generator + scale + seed
  6. Apply [interior].tags to every interior tile entity
  7. Spawn interior entities based on spawn_rules tags
```

### 4.4 Tags Drive Gameplay Differences

| Interior Tag / Environment | Effect |
|---------------------------|--------|
| `BLOCKS_WEATHER` | Weather modifiers don't apply — interior uses its own `environment` scores |
| `INDOORS` | No weather descriptive tags (RAINY, STORMY) applied |
| `environment = { light: 5 }` | Light score 5 → DARK threshold → visibility reduction, DARKVISION matters |
| `environment = { temperature: 90 }` | Temperature score 90 → HOT threshold → fire interaction rules active |
| `HOSTILE_ZONE` | Enemies spawned, loot placed |
| `SETTLEMENT` | Friendly NPCs, shops, no hostile spawns |
| `UNDERGROUND` | Used by generator dispatch, limits weather |

Adding "volcanic_lair" = new TOML entry with tags `HOSTILE_ZONE, HAS_INTERIOR, UNDERGROUND` + `environment = { light: 40, temperature: 90, moisture: 5 }`. Same BSP generator as dungeon, but HOT from the temperature score means fire interaction rules are active throughout — no DARK tag, so DARKVISION isn't needed but creatures take fire damage if FLAMMABLE.

---

## 5. Location Entry/Exit Flow

### 5.1 MapLayer Structure

```rust
pub struct MapLayer {
    pub active_interior: Option<ActiveInterior>,
    pub depth: u32,
}

pub struct ActiveInterior {
    pub location_id: usize,
    pub interior_tags: Vec<TagId>,
    pub overworld_map: WorldMap,
    pub overworld_player_pos: TilePos,
    pub overworld_entities: Vec<Entity>,
}
```

### 5.2 Entering a Location

Triggered by `>` key when player is within a location's zone_radius and the location has `HAS_INTERIOR`:

1. Check `MapLayer.active_interior` — if inside, look for `DeeperStair` (depth progression)
2. `location_at()` finds `PlacedLocation`
3. Check location tags for `HAS_INTERIOR` — skip if absent
4. Look up `[interior]` section for this location type
5. **Save overworld state:** snapshot `WorldMap`, player position, mark all current entities with `OverworldEntity` component
6. **Generate interior:** dispatch to BSP/WFC per template, spawn tile entities with interior tags applied
7. **Replace `WorldMap`:** create new `WorldMap` from interior tiles, insert as resource (old stored in `MapLayer`)
8. **Spawn interior entities:** read `spawn_rules` → spawn enemies, loot, NPCs using cascade/spawner
9. **Move player** to entrance position
10. **Set `MapLayer.active_interior`**

### 5.3 Inside a Location

- Same ECS world, same systems — no new `AppScreen` state
- Renderer renders whatever `WorldMap` provides (already generic)
- Interior tiles have `BLOCKS_WEATHER` → weather pipeline skips them
- `DARK` tiles → visibility modifier, DARKVISION matters
- Moving to `EntranceStair` tile and pressing `<` triggers exit
- `DeeperStair` tile and pressing `>` generates next depth level

### 5.4 Exiting a Location

Triggered by `<` key on an entrance stair tile:

1. Check player is on `ENTRANCE_STAIR` tile
2. **Restore overworld:** swap `WorldMap` back to saved overworld map
3. **Despawn interior entities:** remove all entities without `OverworldEntity` marker
4. **Move player** to saved overworld position
5. **Clear `MapLayer.active_interior`**
6. Weather tags resume applying (no more `BLOCKS_WEATHER`)

### 5.5 Files Changed

| File | Change |
|------|--------|
| `assets/config/location_types.toml` | Add `[interior]` sections to each location type |
| `crates/world/src/dungeon.rs` | Restructure `MapLayer` to `ActiveInterior`, add `BLOCKS_WEATHER`/interior tile tags to `DungeonTile` conversion |
| `crates/world/src/wfc.rs` | Add interior tile tag support to WFC output |
| `src/game/mod.rs` | Wire `>` key to `enter_location()`, add `<` key to `exit_location()`, add location entry/exit functions |
| `src/world_gen.rs` | Add `spawn_interior_entities()` function, interior WorldMap builder |
| `crates/world/src/spawner.rs` | Add `WeatherSensitive` component to spawned creatures |
| `crates/core/src/components.rs` | Add `WeatherSensitive` component, `OverworldEntity` marker |

---

## 6. System Composition

### 6.1 Weather + Locations

Rain weather → `moisture +50` modifier → desert tile base moisture 10 → final moisture 60 → crosses WET threshold → WET tag applied to tile and entities. Player enters dungeon → interior tiles have `environment = { moisture: 20 }` with `BLOCKS_WEATHER` → weather modifier ignored → moisture stays 20 → DAMP, not WET. No code path says "if dungeon, don't apply rain." The score composition handles it.

### 6.2 Weather + Combat

Rain → moisture modifier pushes tile to WET → creature has LIGHTNING tag → self-interaction: `LIGHTNING + WET → STUNNED`. No combat code knows about weather. Fire elemental has `FIREPROOF` → conflicts with BURNING → even if temperature score is high, BURNING tag is rejected by the conflict system.

### 6.3 Weather + Movement

Snow → `temperature: -40` modifier → grassland tile base temp 50 → final temp 10 → crosses FREEZING threshold → FREEZING tag on tiles. `visibility: 0.5` → NPC sense range drops 50%. Future extension: FREEZING tag → `move_cost: 1.5` on TagDef → movement system reads tag metadata.

### 6.4 Location + Economy

City interior spawns shopkeeper NPCs with `CAN_BARTER`. Same `RegionEconomies` pricing applies. Village market is just an NPC with tags, spawned by the `SETTLEMENT` spawn rule.

### 6.5 Adding New Content

| Action | What to do |
|--------|-----------|
| New weather type | Create `assets/config/weather/weather_X.toml` with modifiers + tags. Scores drive environmental tags, interaction rules fire automatically. |
| New location type | Add entry to `location_types.toml` with tags + `[interior]` section including `environment` scores. Tags + scores drive generation and gameplay. |
| New environmental axis | Add tags with `range` thresholds to `tags.toml`. Weather TOMLs and biome_rules gain the new modifier key. Pipeline handles it generically. |
| New biome effects | Add base environmental scores to biome rules in `biome_rules.toml`. Add `move_cost` to terrain tags. Movement system reads it. |
| New creature weather behavior | Add `env_modifiers` in `npc_actions.toml` for new weather tags. |
| New interaction | Add rule to `interactions.toml`. Fires when both tags coexist, regardless of source. |
| New interior effect | Set `environment` scores in location `[interior]`. All systems respond to the derived tags. |

---

## 7. Implementation Order

### Phase 1: Environmental Score Infrastructure
1. Add `light` archetype (DARK, DIM, BRIGHT with range thresholds) and `weather` archetype to `tags.toml`
2. Add `range` threshold fields to temperature and moisture tags in `tags.toml`
3. Add base environmental scores to `biome_rules.toml` entries
4. Create weather TOML template files for existing 8 weather types (modifiers + descriptive tags)
5. Refactor `weather.rs`: generic `WeatherDef` loader, score computation, threshold → tag resolution
6. Implement `apply_environmental_tags()` pipeline in turn loop
7. Add `WeatherSensitive` component to player and creatures
8. Wire `get_sense_range()` to visibility × light_score

### Phase 2: Location Interior Infrastructure
1. Add `[interior]` sections (with `environment` score overrides) to `location_types.toml`
2. Restructure `MapLayer` to `ActiveInterior`
3. Write `spawn_interior_tiles()` — DungeonMap/WfcLocation → ECS tile entities with interior tags + environment scores
4. Write `enter_location()` / `exit_location()` functions
5. Wire `>` and `<` key handlers
6. Wire interior entity spawning via spawn_rules tags
7. Wire depth progression for `UNDERGROUND` locations

### Phase 3: Integration and Polish
1. Convert all string-based weather checks to `TagId` checks
2. Add `env_modifiers` for new weather tags in `npc_actions.toml`
3. Add movement cost reads from tag metadata (future)
4. Add multi-sense detection (HEARING, SMELL bypass visual weather) (future)
5. Add new environmental axes (toxicity, etc.) as needed (future)
