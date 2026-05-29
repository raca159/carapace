# Location Interiors Implementation Plan (Phase 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans.

**Goal:** Wire location entry/exit so pressing `>` on a PlacedLocation generates an interior (BSP dungeon or WFC city), the player enters it, and pressing `<` at the entrance returns them to the overworld.

**Architecture:** Location types in TOML gain optional `[interior]` sections that define generator type, tileset, scale, tags, environment scores, and spawn rules. A new `ActiveInterior` struct tracks the interior state. `spawn_interior_tiles()` converts generated tiles to ECS entities + `WorldMap`. `enter_location()`/`exit_location()` orchestrate the swap. The renderer already works generically with any `WorldMap`.

**Tech Stack:** Bevy ECS, BSP dungeon generator (crates/world/src/dungeon.rs), WFC (crates/world/src/wfc.rs), existing cascade spawner, existing tag system.

---

### Task 1: Add `[interior]` sections to location_types.toml + update LocationType struct

**Files:**
- Modify: `assets/config/location_types.toml` — add `HAS_INTERIOR` tags + `[interior]` sections
- Modify: `crates/world/src/cascade/mod.rs` — add `InteriorDef` struct + `interior` field to `LocationType`

**Details:**
- Add `InteriorDef` struct (optional, parsed from TOML's `[interior]` sections):
```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct InteriorDef {
    pub generator: Option<String>,
    pub tileset: Option<String>,
    pub scale: Option<[u32; 2]>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub environment: Option<HashMap<String, u32>>,
    #[serde(default)]
    pub spawn_rules: Vec<String>,
    pub depth_range: Option<[u32; 2]>,
}
```
- Add `#[serde(default)] pub interior: Option<InteriorDef>` to `LocationType`
- Update `location_types.toml` with `[interior]` sections for dungeon, city, ruin, cave, shrine. Add `HAS_INTERIOR` tag to those types.
- Village and outpost don't get interiors in MVP (they're small overworld locations).
- **Keep all existing fields unchanged** — `cascade.load()` should still work.

**Test:** Run `cargo test -p game-world` — all existing placement tests must pass. The CascadeEngine should still load location types correctly.

**Commit:** `feat: add [interior] sections to location_types.toml, InteriorDef struct`

---

### Task 2: Restructure MapLayer + add ActiveInterior + OverworldEntity

**Files:**
- Modify: `crates/world/src/dungeon.rs` — replace `MapLayer` struct, add `ActiveInterior`
- Modify: `crates/core/src/components.rs` — add `OverworldEntity` marker component
- Modify: `crates/core/src/lib.rs` — export `OverworldEntity`

**Details:**

`crates/world/src/dungeon.rs` — replace existing `MapLayer`:
```rust
#[derive(Debug, Clone)]
pub struct ActiveInterior {
    pub location_id: usize,
    pub interior_tags: Vec<game_tags::TagId>,
    pub environment: Option<std::collections::HashMap<String, u32>>,
    pub saved_world_map: crate::map::WorldMap,
    pub saved_player_pos: (u32, u32),
}

#[derive(Resource, Default)]
pub struct MapLayer {
    pub active_interior: Option<ActiveInterior>,
    pub depth: u32,
}
```

`crates/core/src/components.rs` — add:
```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct OverworldEntity;
```

Export from `lib.rs`: add `OverworldEntity` to the `pub use components::{...}` line.

**Test:** `cargo test -p game-world -p game-core` must pass.

**Commit:** `feat: restructure MapLayer to ActiveInterior, add OverworldEntity component`

---

### Task 3: Write `spawn_interior_tiles()` — DungeonMap/WfcLocation → ECS tiles

**Files:**
- Create: `crates/world/src/interior.rs` — tile spawning from BSP/WFC output
- Modify: `crates/world/src/lib.rs` — add `pub mod interior;`

**Details:**

This function converts a `DungeonMap` (from BSP generation) into ECS tile entities. WFC support is deferred (only BSP for MVP).

```rust
use bevy_ecs::prelude::*;
use game_tags::{TagId, TagRegistry, Tags, TagValue};
use crate::dungeon::{DungeonMap, DungeonTileType};
use crate::tile::{Tile, TilePos};
use crate::map::WorldMap;

pub fn spawn_interior_tiles(
    world: &mut World,
    dungeon: &DungeonMap,
    interior_tags: &[TagId],
    interior_environment: Option<&std::collections::HashMap<String, u32>>,
    registry: &TagRegistry,
) -> WorldMap {
    let mut tile_entities = Vec::with_capacity(dungeon.tiles.len());
    for dt in &dungeon.tiles {
        let mut tags = Tags::new(registry.tag_count());
        // Apply interior template tags (INDOORS, BLOCKS_WEATHER, etc.)
        for &tag_id in interior_tags {
            tags.add_tag(tag_id, TagValue::None, registry);
        }
        // Apply tile-type-specific tags
        match dt.tile_type {
            DungeonTileType::Wall => {
                if let Some(id) = registry.tag_id("BLOCKED") {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            DungeonTileType::Floor | DungeonTileType::Corridor => {
                if let Some(id) = registry.tag_id("WALKABLE") {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            DungeonTileType::EntranceStair => {
                if let Some(id) = registry.tag_id("WALKABLE") {
                    tags.add_tag(id, TagValue::None, registry);
                }
                if let Some(id) = registry.tag_id("ENTRANCE_STAIR") {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            DungeonTileType::DeeperStair => {
                if let Some(id) = registry.tag_id("WALKABLE") {
                    tags.add_tag(id, TagValue::None, registry);
                }
                if let Some(id) = registry.tag_id("DEEPER_STAIR") {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
        }
        let tile = Tile {
            pos: TilePos::new(dt.pos.x, dt.pos.y),
            elevation: 0.0,
            moisture: 0.0,
            temperature: 0.0,
            biome_name: "interior".to_string(),
            glyph: dt.tile_type.glyph(),
            color: match dt.tile_type {
                DungeonTileType::Wall => (80, 70, 90),
                DungeonTileType::Floor => (50, 50, 50),
                DungeonTileType::Corridor => (60, 60, 60),
                DungeonTileType::EntranceStair => (100, 100, 100),
                DungeonTileType::DeeperStair => (120, 120, 80),
            },
        };
        let entity = world.spawn((tile, tags)).id();
        tile_entities.push(entity);
    }
    WorldMap {
        width: dungeon.width,
        height: dungeon.height,
        depth: 1,
        current_z: 0,
        seed: crate::seed::WorldSeed(dungeon.seed),
        tiles: tile_entities,
    }
}
```

Add a `glyph()` method to `DungeonTileType` if it doesn't exist:
```rust
impl DungeonTileType {
    pub fn glyph(&self) -> char {
        match self {
            DungeonTileType::Wall => '#',
            DungeonTileType::Floor => '.',
            DungeonTileType::Corridor => '.',
            DungeonTileType::EntranceStair => '<',
            DungeonTileType::DeeperStair => '>',
        }
    }
}
```

Add tests: generate a small DungeonMap, call `spawn_interior_tiles()`, verify WorldMap dimensions, verify tile entities have correct tags (BLOCKED on walls, WALKABLE on floors, etc.).

**Register tags:** Add `ENTRANCE_STAIR` and `DEEPER_STAIR` to `tags.toml` under the `terrain` archetype.

**Test:** `cargo test -p game-world` must pass. New tests for interior tile spawning must pass.

**Commit:** `feat: add spawn_interior_tiles and ENTRANCE_STAIR/DEEPER_STAIR tags`

---

### Task 4: Write `enter_location()` / `exit_location()` functions

**Files:**
- Create: `src/location_entry.rs` — entry/exit orchestration
- Modify: `src/main.rs` — add `mod location_entry;`

**Details:**

`enter_location()`:
1. Takes `&mut World` (game sub-world), `&PlacedLocation`, location's `InteriorDef`
2. Saves overworld state: clone `WorldMap`, find player position, mark all entities with `OverworldEntity`
3. Generates interior: calls BSP `generate_dungeon()` with scale from interior def
4. Applies interior template tags + environment scores to each tile
5. Calls `spawn_interior_tiles()` to build interior `WorldMap`
6. Replaces the `WorldMap` resource with the interior map
7. Moves player to entrance position (`dungeon.entrance`)
8. Sets `MapLayer.active_interior`
9. Spawns interior entities based on spawn_rules (see Task 6)

`exit_location()`:
1. Checks `MapLayer.active_interior` exists
2. Restores saved `WorldMap` from `ActiveInterior.saved_world_map`
3. Moves player to `ActiveInterior.saved_player_pos`
4. Despawns all entities WITHOUT `OverworldEntity` marker
5. Clears `MapLayer.active_interior`

Add the `ENTRANCE_STAIR` and `DEEPER_STAIR` tag lookups inline (they're already used in Task 3's tag application).

**Test:** `cargo build` must pass (no runtime test possible without game loop). Verify by examining function signatures.

**Commit:** `feat: add enter_location/exit_location orchestration functions`

---

### Task 5: Wire `>` and `<` key handlers

**Files:**
- Modify: `src/game/mod.rs` — update the `>` (Period) key handler, add `<` (Comma) key handler

**Details:**

Replace the current `>` handler (lines 182-230) which just prints a message:

```rust
// Location entry (> key)
if keyboard.just_pressed(KeyCode::Period) || keyboard.just_pressed(KeyCode::NumpadDecimal) {
    let map_layer = game_world.0.get_resource::<game_world::dungeon::MapLayer>();
    
    // If inside an interior, check for DeeperStair
    if let Some(ref ml) = map_layer {
        if ml.active_interior.is_some() {
            // Check if player is on a DEEPER_STAIR tile
            let player_pos = match game_world.0
                .query_filtered::<&game_core::Position, bevy_ecs::query::With<game_core::Player>>()
                .single(&game_world.0)
            {
                Ok(p) => (p.x, p.y),
                Err(_) => return,
            };
            // Check tile tags for DEEPER_STAIR
            let deeper_tag = game_world.0.get_resource::<game_tags::TagRegistry>()
                .and_then(|r| r.tag_id("DEEPER_STAIR"));
            let entrance_tag = game_world.0.get_resource::<game_tags::TagRegistry>()
                .and_then(|r| r.tag_id("ENTRANCE_STAIR"));
            // On DeeperStair -> depth progression (future)
            // On EntranceStair -> exit
            // Otherwise print message
            return;
        }
    }

    // On overworld -> check for location
    let location_map = match game_world.0.get_resource::<game_world::cascade::LocationMap>() {
        Some(lm) => lm.clone(),
        None => { return; }
    };
    // ... existing player_pos lookup ...
    // ... existing location_at() check ...

    if let Some(location) = loc {
        // Check HAS_INTERIOR
        if !location.tags.iter().any(|t| t == "HAS_INTERIOR") {
            // Show flavor text and return
            return;
        }
        // Look up InteriorDef from CascadeEngine
        let cascade = match game_world.0.get_resource::<game_world::cascade::CascadeEngine>() {
            Some(c) => c,
            None => return,
        };
        let interiors = game_world::cascade::location_entry::load_interiors();
        // Find interior def matching this location type
        // Call crate::location_entry::enter_location(...)
        return;
    }
}
```

Add `<` (Comma) key handler:
```rust
if keyboard.just_pressed(KeyCode::Comma) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
    // Check if on EntranceStair
    let player_pos = match game_world.0
        .query_filtered::<&game_core::Position, bevy_ecs::query::With<game_core::Player>>()
        .single(&game_world.0)
    {
        Ok(p) => (p.x, p.y),
        Err(_) => return,
    };
    // Check tile at player position for ENTRANCE_STAIR tag
    // If found, call crate::location_entry::exit_location(...)
}
```

The key insight: keep the handler logic minimal. The heavy lifting (generation, map swap, entity management) stays in `location_entry.rs`. The key handler just checks preconditions and dispatches.

**Test:** `cargo build` must pass.

**Commit:** `feat: wire > and < key handlers for location entry/exit`

---

### Task 6: Wire interior entity spawning via spawn_rules

**Files:**
- Modify: `src/location_entry.rs` — add entity spawning logic
- Or modify: `crates/world/src/spawner.rs` — add interior spawning function

**Details:**

Inside `enter_location()`, after spawning tiles, spawn entities based on `spawn_rules`:

```rust
fn spawn_interior_entities(
    world: &mut World,
    dungeon: &DungeonMap,
    spawn_rules: &[String],
    registry: &TagRegistry,
    cascade: &CascadeEngine,
) {
    let spawn_positions = dungeon::dungeon_spawn_positions(dungeon, 0);
    for rule_name in spawn_rules {
        match rule_name.as_str() {
            "hostile" => {
                // Use existing spawner logic to create hostile creatures
                for &pos in &spawn_positions {
                    // Reuse spawner's creature spawning with aggressive tags
                    let tags = Tags::new(registry.tag_count());
                    let creature = world.spawn((
                        game_core::Position { x: pos.0, y: pos.1, z: 0 },
                        game_core::Glyph { char: 'g', color: (200, 0, 0) },
                        game_core::Health { current: 20, max: 20 },
                        tags,
                        game_core::Name("Dungeon Creature".to_string()),
                        game_core::Creature,
                        game_core::WeatherSensitive,
                        game_core::BehaviorState { home_pos: Some(game_core::Position { x: pos.0, y: pos.1, z: 0 }) },
                        game_core::NpcEmotionalState::default(),
                    )).id();
                }
            }
            "loot" => {
                // Place chests using loot system
                crate::loot::place_dungeon_chests(world, dungeon, 0);
            }
            _ => {}
        }
    }
}
```

For simplicity in MVP, use basic creature/item spawning. The full cascade-integrated spawning is deferred.

**Test:** `cargo build` must pass.

**Commit:** `feat: add interior entity spawning via spawn_rules`

---

### Task 7: Wire depth progression for UNDERGROUND locations

**Files:**
- Modify: `src/game/mod.rs` — handle `>` on DeeperStair
- Modify: `src/location_entry.rs` — add `enter_next_depth()`

**Details:**

When player presses `>` on a `DEEPER_STAIR` tile:
1. Check `MapLayer.depth < depth_range.max` (from interior def)
2. Generate a new dungeon level (increment seed or use new seed)
3. Spawn new tiles (reuse `spawn_interior_tiles`)
4. Replace `WorldMap`
5. Move player to entrance of new level
6. Increment `MapLayer.depth`
7. Despawn old interior entities

```rust
pub fn enter_next_depth(world: &mut World) {
    let map_layer = world.get_resource::<MapLayer>().unwrap();
    let interior = map_layer.active_interior.as_ref().unwrap();
    // ... generate next level, swap map, etc.
}
```

**Test:** `cargo build` must pass.

**Commit:** `feat: wire dungeon depth progression on DeeperStair tiles`

---

## Spec Coverage Check

| Spec Requirement | Task |
|-----------------|------|
| Add `[interior]` sections to TOML | Task 1 |
| Restructure `MapLayer` to `ActiveInterior` | Task 2 |
| Write `spawn_interior_tiles()` | Task 3 |
| Write `enter_location()` / `exit_location()` | Task 4 |
| Wire `>` and `<` key handlers | Task 5 |
| Wire interior entity spawning via `spawn_rules` | Task 6 |
| Wire depth progression | Task 7 |
| `OverworldEntity` marker component | Task 2 |
| Environment score overrides for interior tiles | Built into spawn_interior_tiles via `interior_environment` param |
| `BLOCKS_WEATHER` / `INDOORS` tags | Applied from interior.template.tags | 
