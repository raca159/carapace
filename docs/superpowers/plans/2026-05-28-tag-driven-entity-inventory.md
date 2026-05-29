# Tag-Driven Entity Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace ground-item spawning with tag-driven entity inventories, unify chests/containers under the same system, and add looting interaction for dead entities and containers.

**Architecture:** A `populate_inventories()` function runs after entity spawning, scanning for `HAS_INVENTORY` tag and filling Inventory components based on capacity + content-type tags. Chests lose `LootContainer` in favor of tag-driven Inventory. Dead entities become lootable containers via `CONTAINER` + `INTERACTABLE` tags. A new `InteractMode::Looting` handles the loot UI.

**Tech Stack:** Rust, Bevy ECS, ratatui (UI panels), TOML configs, existing tag/cascade systems.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/world/src/cascade/inventory_populate.rs` | **New.** Populator: scans for HAS_INVENTORY, fills Inventory components |
| `src/interact/loot.rs` | **New.** Loot panel UI (Bevy UI) and update system |
| `assets/config/tags.toml` | Add `inventory` archetype (8 tags) |
| `assets/config/spawn_rules.toml` | Add inventory tags to entity spawn rules |
| `crates/tags/assets/config/tags.toml` | Mirror runtime tags.toml changes |
| `crates/world/src/cascade/mod.rs` | Register `inventory_populate` module |
| `crates/world/src/spawner.rs` | Remove ground-item spawning loops (2 locations) |
| `crates/world/src/loot.rs` | `place_dungeon_chests()` uses tags, no `LootContainer` |
| `crates/world/src/lib.rs` | Export `populate_inventories` |
| `crates/core/src/components.rs` | Remove `LootContainer` struct |
| `crates/core/src/lib.rs` | Remove `LootContainer` from exports |
| `crates/world/src/export.rs` | Remove `LootContainerExport` and related code |
| `src/world_gen.rs` | Call `populate_inventories()` after entity spawning |
| `src/location_entry.rs` | Call `populate_inventories()` after interior spawning |
| `src/interact/mod.rs` | Add `InteractMode::Looting` variant + routing + loot module |
| `src/interact/loot.rs` | **New.** Loot panel resource + update system |
| `src/game/mod.rs` | Combat: tag dead as CONTAINER+INTERACTABLE instead of despawn; add looting check to route_interaction |
| `src/render/mod.rs` | Register `LootPanel` resource |

---

### Task 1: Add Inventory Tags to `tags.toml`

**Files:**
- Modify: `assets/config/tags.toml:894` (end of file)
- Modify: `crates/tags/assets/config/tags.toml` (mirror changes)

- [ ] **Step 1: Add inventory archetype to runtime tags.toml**

Append to `assets/config/tags.toml` after the last archetype:

```toml
[[archetype]]
id = "inventory"
name = "Inventory"
exclusivity = "any"

[[archetype.tags]]
id = "HAS_INVENTORY"

[[archetype.tags]]
id = "INVENTORY_TINY"
default_magnitude = 4.0

[[archetype.tags]]
id = "INVENTORY_SMALL"
default_magnitude = 8.0

[[archetype.tags]]
id = "INVENTORY_MEDIUM"
default_magnitude = 12.0

[[archetype.tags]]
id = "INVENTORY_LARGE"
default_magnitude = 20.0

[[archetype.tags]]
id = "INVENTORY_HUGE"
default_magnitude = 30.0

[[archetype.tags]]
id = "INVENTORY_LOOT"

[[archetype.tags]]
id = "INVENTORY_TRADE"

[[archetype.tags]]
id = "INVENTORY_EQUIPMENT"
```

- [ ] **Step 2: Copy the same archetype to the test tags.toml**

Append the identical `[[archetype]]` block above to `crates/tags/assets/config/tags.toml`.

- [ ] **Step 3: Run tests to verify tags load**

Run: `cargo test --package game-tags`
Expected: All existing tests pass (tags load with new archetype).

- [ ] **Step 4: Commit**

```bash
git add assets/config/tags.toml crates/tags/assets/config/tags.toml
git commit -m "feat: add inventory archetype with capacity and content-type tags"
```

---

### Task 2: Add Inventory Tags to Spawn Rules

**Files:**
- Modify: `assets/config/spawn_rules.toml:319-365`

- [ ] **Step 1: Add inventory tags to City Merchant**

Change the City Merchant spawn rule tags from:
```toml
tags = ["HUMANOID", "MEDIUM", "PEACEFUL", "INTERACTABLE", "CAN_TALK", "CAN_BARTER"]
```
to:
```toml
tags = ["HUMANOID", "MEDIUM", "PEACEFUL", "INTERACTABLE", "CAN_TALK", "CAN_BARTER", "HAS_INVENTORY", "INVENTORY_MEDIUM", "INVENTORY_TRADE"]
```

- [ ] **Step 2: Add inventory tags to Settlement Guard**

Change the Settlement Guard tags from:
```toml
tags = ["HUMANOID", "MEDIUM", "TERRITORIAL", "INTERACTABLE", "CAN_TALK"]
```
to:
```toml
tags = ["HUMANOID", "MEDIUM", "TERRITORIAL", "INTERACTABLE", "CAN_TALK", "HAS_INVENTORY", "INVENTORY_SMALL", "INVENTORY_EQUIPMENT"]
```

- [ ] **Step 3: Add inventory tags to Dungeon Guardian**

Change the Dungeon Guardian tags from:
```toml
tags = ["HUMANOID", "MEDIUM", "AGGRESSIVE", "MINDLESS"]
```
to:
```toml
tags = ["HUMANOID", "MEDIUM", "AGGRESSIVE", "MINDLESS", "HAS_INVENTORY", "INVENTORY_SMALL", "INVENTORY_EQUIPMENT"]
```

- [ ] **Step 4: Add inventory tags to Ruin Scavenger**

Change the Ruin Scavenger tags from:
```toml
tags = ["HUMANOID", "SMALL", "COWARDLY", "CURIOUS"]
```
to:
```toml
tags = ["HUMANOID", "SMALL", "COWARDLY", "CURIOUS", "HAS_INVENTORY", "INVENTORY_TINY"]
```

- [ ] **Step 5: Commit**

```bash
git add assets/config/spawn_rules.toml
git commit -m "feat: add HAS_INVENTORY + capacity/content tags to spawn rules"
```

---

### Task 3: Create `inventory_populate.rs` with Core Helpers

**Files:**
- Create: `crates/world/src/cascade/inventory_populate.rs`
- Modify: `crates/world/src/cascade/mod.rs:5-9`

- [ ] **Step 1: Create the populate module with helpers**

Create `crates/world/src/cascade/inventory_populate.rs`:

```rust
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;

use game_core::{Glyph, Inventory, Item, Name};
use game_tags::{TagRegistry, Tags, TagValue};

use crate::cascade::{CascadeEngine, TaggedWeight};
use crate::cascade::inventory::{self, InventoryItem};
use crate::loot::{LootTables, roll_loot_for_table, LootDropInstance};

pub struct PopulateContext {
    pub dungeon_type: Option<String>,
    pub depth: Option<u32>,
    pub dungeon_seed: Option<u64>,
}

fn resolve_capacity(tags: &Tags, registry: &TagRegistry) -> usize {
    for tag_name in &["INVENTORY_HUGE", "INVENTORY_LARGE", "INVENTORY_MEDIUM", "INVENTORY_SMALL", "INVENTORY_TINY"] {
        if let Some(tid) = registry.tag_id(tag_name) {
            if tags.has(tid) {
                if let Some(mag) = registry.tag_by_id(tid).default_magnitude {
                    return mag as usize;
                }
            }
        }
    }
    8
}

fn has_content_tag(tags: &Tags, tag_name: &str, registry: &TagRegistry) -> bool {
    registry.tag_id(tag_name).is_some_and(|id| tags.has(id))
}

fn create_item_entity(
    world: &mut World,
    item_def: &crate::cascade::ItemDef,
    registry: &TagRegistry,
) -> Entity {
    let mut item_tags = Tags::new(registry.tag_count());
    for t in &item_def.tags {
        if let Some(tid) = registry.tag_id(t) {
            item_tags.add_tag(tid, TagValue::None, registry);
        }
    }
    world.spawn((
        Glyph { char: item_def.glyph, color: (item_def.color[0], item_def.color[1], item_def.color[2]) },
        item_tags,
        Name(item_def.name.clone()),
        Item,
    )).id()
}

fn fill_trade(
    entity_tags: &Tags,
    faction_id: Option<game_tags::TagId>,
    location_supply: Option<&[TaggedWeight]>,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    inventory::roll_inventory(entity_tags, 3, faction_id, location_supply, engine, registry, rng)
}

fn fill_equipment(
    entity_tags: &Tags,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    let equip_tag_ids: Vec<game_tags::TagId> = ["EQUIP_WEAPON", "EQUIP_ARMOR", "EQUIP_ACCESSORY"]
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();

    let candidates: Vec<&crate::cascade::ItemDef> = engine.items.iter()
        .filter(|item| {
            item.tags.iter()
                .filter_map(|t| registry.tag_id(t))
                .any(|id| equip_tag_ids.contains(&id))
        })
        .collect();

    if candidates.is_empty() { return vec![]; }

    let roll_count = 1 + (rng.random::<f32>() * 2.0) as u32;
    let mut items = Vec::new();
    for _ in 0..roll_count {
        let total: f32 = candidates.iter().map(|c| c.weight).sum();
        if total <= 0.0 { break; }
        let roll = rng.random::<f32>() * total;
        let mut accum = 0.0f32;
        if let Some(selected) = candidates.iter().find(|c| { accum += c.weight; roll < accum }) {
            items.push(InventoryItem {
                item_id: selected.id.clone(),
                quantity: 1,
                trade_only: false,
            });
        }
    }
    items
}

fn fill_fallback(
    entity_tags: &Tags,
    faction_id: Option<game_tags::TagId>,
    location_supply: Option<&[TaggedWeight]>,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    inventory::roll_inventory(entity_tags, 2, faction_id, location_supply, engine, registry, rng)
}

pub fn populate_inventories(
    world: &mut World,
    ctx: Option<&PopulateContext>,
) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let cascade = match world.get_resource::<CascadeEngine>() {
        Some(c) => c.clone(),
        None => return,
    };
    let loot_tables = world.get_resource::<LootTables>().cloned();

    let has_inv_id = match registry.tag_id("HAS_INVENTORY") {
        Some(id) => id,
        None => return,
    };

    let entities: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Tags)>();
        q.iter(world)
            .filter(|(_, tags)| tags.has(has_inv_id))
            .map(|(e, _)| e)
            .collect()
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(
        world.get_resource::<crate::map::WorldMap>()
            .map(|m| m.seed.0.wrapping_add(0x1NV))
            .unwrap_or(0xDEAD)
    );

    for entity in entities {
        if world.get::<Inventory>(entity).is_some() {
            continue;
        }

        let entity_tags = match world.get::<Tags>(entity) {
            Some(t) => t.clone(),
            None => continue,
        };

        let capacity = resolve_capacity(&entity_tags, &registry);
        let is_loot = has_content_tag(&entity_tags, "INVENTORY_LOOT", &registry);
        let is_trade = has_content_tag(&entity_tags, "INVENTORY_TRADE", &registry);
        let is_equipment = has_content_tag(&entity_tags, "INVENTORY_EQUIPMENT", &registry);

        let faction_id = world.get::<crate::faction::Faction>(entity)
            .and_then(|f| {
                let faction_rels = world.get_resource::<crate::faction::FactionRelationships>()?;
                let name = faction_rels.name_for(f.faction_id)?;
                registry.tag_id(&name)
            });

        let location_supply = {
            let pos = world.get::<game_core::Position>(entity);
            let loc_map = world.get_resource::<crate::cascade::LocationMap>();
            let economies = world.get_resource::<crate::cascade::RegionEconomies>();
            match (pos, loc_map, economies) {
                (Some(p), Some(lm), Some(econ)) => {
                    let loc = crate::cascade::locations::location_at(&lm.locations, p.x, p.y);
                    loc.and_then(|l| econ.economies.get(&l.id))
                        .map(|pc| pc.location_supply.as_slice())
                }
                _ => None,
            }
        };

        let mut all_items: Vec<InventoryItem> = Vec::new();

        if is_loot {
            if let (Some(ref tables), Some(ref dctx)) = (loot_tables.as_ref(), ctx) {
                let dtype = dctx.dungeon_type.as_deref().unwrap_or("standard");
                let depth = dctx.depth.unwrap_or(1);
                let seed = dctx.dungeon_seed.unwrap_or(0);
                let matching = tables.tables_for_dungeon(dtype, depth);
                for table in matching {
                    let drops = roll_loot_for_table(table, seed, depth, &mut rng);
                    for drop in drops {
                        all_items.push(InventoryItem {
                            item_id: drop.name.clone(),
                            quantity: drop.quantity.unwrap_or(1),
                            trade_only: false,
                        });
                    }
                }
            }
        }

        if is_trade {
            let trade_items = fill_trade(
                &entity_tags, faction_id, location_supply,
                &cascade, &registry, &mut rng,
            );
            all_items.extend(trade_items);
        }

        if is_equipment {
            let equip_items = fill_equipment(
                &entity_tags, &cascade, &registry, &mut rng,
            );
            all_items.extend(equip_items);
        }

        if !is_loot && !is_trade && !is_equipment {
            let fallback_items = fill_fallback(
                &entity_tags, faction_id, location_supply,
                &cascade, &registry, &mut rng,
            );
            all_items.extend(fallback_items);
        }

        let mut item_entities: Vec<Entity> = Vec::new();
        for inv_item in all_items.iter().take(capacity) {
            if let Some(item_def) = cascade.item_by_id.get(&inv_item.item_id) {
                for _ in 0..inv_item.quantity {
                    if item_entities.len() >= capacity { break; }
                    let ie = create_item_entity(world, item_def, &registry);
                    item_entities.push(ie);
                }
            }
        }

        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(Inventory {
                items: item_entities,
                capacity,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../../../../assets/config/tags.toml");
    const ITEMS_TOML: &str = include_str!("../../../../assets/config/items.toml");
    const BIOMES_TOML: &str = include_str!("../../../../assets/config/region_biomes.toml");
    const FACTIONS_TOML: &str = include_str!("../../../../assets/config/faction_economy.toml");
    const LOCATIONS_TOML: &str = include_str!("../../../../assets/config/location_types.toml");

    fn setup() -> (World, TagRegistry, CascadeEngine) {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let cascade = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        let mut world = World::new();
        world.insert_resource(registry.clone());
        world.insert_resource(cascade.clone());
        world.insert_resource(crate::map::WorldMap::default());
        world
    }

    #[test]
    fn test_resolve_capacity_tiny() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("INVENTORY_TINY").unwrap(), TagValue::None, &registry);
        assert_eq!(resolve_capacity(&tags, &registry), 4);
    }

    #[test]
    fn test_resolve_capacity_medium() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("INVENTORY_MEDIUM").unwrap(), TagValue::None, &registry);
        assert_eq!(resolve_capacity(&tags, &registry), 12);
    }

    #[test]
    fn test_resolve_capacity_default() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tags = Tags::new(registry.tag_count());
        assert_eq!(resolve_capacity(&tags, &registry), 8);
    }

    #[test]
    fn test_has_inventory_gets_inventory_component() {
        let (mut world, registry, cascade) = {
            let reg = load_tag_registry(TAGS_TOML).unwrap();
            let eng = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
            let mut w = World::new();
            w.insert_resource(reg.clone());
            w.insert_resource(eng.clone());
            w.insert_resource(crate::map::WorldMap::default());
            (w, reg, eng)
        };

        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HAS_INVENTORY").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("INVENTORY_TINY").unwrap(), TagValue::None, &registry);

        let entity = world.spawn((
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Glyph { char: 'M', color: (255, 200, 50) },
            tags,
            game_core::Name("Test Merchant".to_string()),
            game_core::Creature,
        )).id();

        populate_inventories(&mut world, None);

        let inv = world.get::<Inventory>(entity);
        assert!(inv.is_some(), "Entity with HAS_INVENTORY should get Inventory component");
        let inv = inv.unwrap();
        assert_eq!(inv.capacity, 4, "INVENTORY_TINY should give 4 slots");
    }

    #[test]
    fn test_no_has_inventory_no_component() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let cascade = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        let mut world = World::new();
        world.insert_resource(registry.clone());
        world.insert_resource(cascade);
        world.insert_resource(crate::map::WorldMap::default());

        let tags = Tags::new(registry.tag_count());
        let entity = world.spawn((
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Glyph { char: 'x', color: (100, 100, 100) },
            tags,
            game_core::Name("No Inventory Entity".to_string()),
            game_core::Creature,
        )).id();

        populate_inventories(&mut world, None);

        let inv = world.get::<Inventory>(entity);
        assert!(inv.is_none(), "Entity without HAS_INVENTORY should not get Inventory component");
    }
}
```

- [ ] **Step 2: Register the module in cascade/mod.rs**

Add `pub mod inventory_populate;` to `crates/world/src/cascade/mod.rs` after the existing module declarations (after line 9):

```rust
pub mod inventory_populate;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --package game-world`
Expected: Tests pass, including new `test_resolve_capacity_*` and `test_has_inventory_*` tests.

- [ ] **Step 4: Commit**

```bash
git add crates/world/src/cascade/inventory_populate.rs crates/world/src/cascade/mod.rs
git commit -m "feat: add inventory populator with tag-driven capacity and fill logic"
```

---

### Task 4: Export `populate_inventories` from game-world

**Files:**
- Modify: `crates/world/src/lib.rs:45`

- [ ] **Step 1: Add export to lib.rs**

Add to the `pub use loot::` line in `crates/world/src/lib.rs` (line 45) — append `inventory_populate::populate_inventories`:

Change:
```rust
pub use loot::{LootDropInstance, LootEntryDef, LootTableDef, LootTables, load_loot_tables, place_dungeon_chests, roll_loot_for_creature, roll_loot_for_table, spawn_dungeon_floor_loot, spawn_loot_drop};
```
to:
```rust
pub use loot::{LootDropInstance, LootEntryDef, LootTableDef, LootTables, load_loot_tables, place_dungeon_chests, roll_loot_for_creature, roll_loot_for_table, spawn_dungeon_floor_loot, spawn_loot_drop};
pub use cascade::inventory_populate::{populate_inventories, PopulateContext};
```

- [ ] **Step 2: Verify build**

Run: `cargo build --package game-world`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add crates/world/src/lib.rs
git commit -m "feat: export populate_inventories and PopulateContext from game-world"
```

---

### Task 5: Remove Ground-Item Spawning from Spawner

**Files:**
- Modify: `crates/world/src/spawner.rs:272-295` (location entities)
- Modify: `crates/world/src/spawner.rs:382-403` (wild entities)

- [ ] **Step 1: Remove ground-item loop from `spawn_location_entities`**

In `crates/world/src/spawner.rs`, delete lines 272-295 (the comment `// Roll inventory from location supply` through the closing brace of the `for inv_item` loop). This is the block starting at:
```rust
            // Roll inventory from location supply
            let faction_id = ...
```
through:
```rust
                }
            }
```
(Remove the entire block after `generate_npc_equipment(...)` call ending at line ~270.)

- [ ] **Step 2: Remove ground-item loop from `spawn_wild_entities`**

In the same file, delete lines 382-403 (the comment `// Roll wild inventory` through the closing brace of the `for inv_item` loop). This is the block starting at:
```rust
                    // Roll wild inventory (no location supply)
```
through:
```rust
                        }
                    }
```
(Remove the entire block after `generate_npc_equipment(...)` call ending at line ~380.)

- [ ] **Step 3: Run tests**

Run: `cargo test --package game-world`
Expected: All existing tests pass. Spawner no longer creates ground items.

- [ ] **Step 4: Commit**

```bash
git add crates/world/src/spawner.rs
git commit -m "refactor: remove ground-item spawning from spawner (moved to populator)"
```

---

### Task 6: Update `place_dungeon_chests` to Use Tags Instead of `LootContainer`

**Files:**
- Modify: `crates/world/src/loot.rs:337-400`

- [ ] **Step 1: Update `place_dungeon_chests` to spawn tag-based entities**

Replace the `place_dungeon_chests` function body (lines 337-400) with:

```rust
pub fn place_dungeon_chests(world: &mut World, dungeon: &DungeonMap, depth: u32) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let tag_count = registry.tag_count();
    let chest_seed = dungeon.seed.wrapping_add(depth as u64).wrapping_add(0xCAB5);
    let mut rng = rand::rngs::StdRng::seed_from_u64(chest_seed);

    let entrance_pos = dungeon.rooms.first().map(|r| (r.x, r.y, r.w, r.h));

    for room in &dungeon.rooms {
        if entrance_pos.is_some_and(|(ex, ey, ew, eh)| {
            room.x == ex && room.y == ey && room.w == ew && room.h == eh
        }) {
            continue;
        }

        if rng.random::<f32>() >= 0.5 {
            continue;
        }

        let mut candidates: Vec<(u32, u32)> = Vec::new();
        for dy in 0..room.h {
            for dx in 0..room.w {
                let x = room.x + dx;
                let y = room.y + dy;
                let idx = (y * dungeon.width + x) as usize;
                if let Some(tile) = dungeon.tiles.get(idx)
                    && tile.tile_type == DungeonTileType::Floor
                {
                    candidates.push((x, y));
                }
            }
        }

        if candidates.is_empty() {
            continue;
        }

        let (cx, cy) = candidates[rng.random_range(0..candidates.len())];

        let mut entity_tags = Tags::new(tag_count);
        if let Some(tag_id) = registry.tag_id("CONTAINER") {
            entity_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }
        if let Some(tag_id) = registry.tag_id("HAS_INVENTORY") {
            entity_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }
        if let Some(tag_id) = registry.tag_id("INVENTORY_SMALL") {
            entity_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }
        if let Some(tag_id) = registry.tag_id("INVENTORY_LOOT") {
            entity_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }

        world.spawn((
            Position { x: cx, y: cy, z: 0 },
            Glyph {
                char: '=',
                color: (255, 215, 0),
            },
            entity_tags,
            Name("Container".to_string()),
        ));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --package game-world`
Expected: All loot tests pass. `test_place_dungeon_chests_creates_entities` should still pass (it queries for entities with Position/Glyph at chest positions).

- [ ] **Step 3: Commit**

```bash
git add crates/world/src/loot.rs
git commit -m "refactor: place_dungeon_chests uses inventory tags instead of LootContainer"
```

---

### Task 7: Remove `LootContainer` Component

**Files:**
- Modify: `crates/core/src/components.rs:57-62`
- Modify: `crates/core/src/lib.rs:38`
- Modify: `crates/world/src/export.rs:47-52, 74, 150-157, 178, 249, 613, 628`

- [ ] **Step 1: Remove `LootContainer` from components.rs**

Delete lines 57-62 in `crates/core/src/components.rs`:
```rust
#[derive(Component, Debug, Clone)]
pub struct LootContainer {
    pub loot_table_id: String,
    pub opened: bool,
    pub dungeon_seed: u64,
    pub depth: u32,
}
```

- [ ] **Step 2: Remove `LootContainer` from lib.rs exports**

In `crates/core/src/lib.rs` line 38, change:
```rust
    ArmorProtection, Creature, Equipment, EquipmentSlot, Glyph, Health, Inventory, Item,
    ItemEffects, LootContainer, MessageLog, Name, OverworldEntity, Player, Position, WeaponDamage, WeatherSensitive,
```
to:
```rust
    ArmorProtection, Creature, Equipment, EquipmentSlot, Glyph, Health, Inventory, Item,
    ItemEffects, MessageLog, Name, OverworldEntity, Player, Position, WeaponDamage, WeatherSensitive,
```

- [ ] **Step 3: Remove `LootContainerExport` and references from export.rs**

In `crates/world/src/export.rs`:
1. Remove the `LootContainerExport` struct (lines 47-52)
2. Remove `loot_container: Option<LootContainerExport>` from `EntityExport` (line 74)
3. Remove the `loot_container` field construction (lines 150-157)
4. Remove `loot_container` from the `EntityExport` construction (line 178)
5. Remove `LootContainer` from the import line (line 5)
6. Remove `LootContainer` from import line in `import_entity` function (~line 249)
7. Remove `LootContainer { ... }` construction in import code (~line 613)
8. Remove `LootContainer` test assertion (~line 628)

For each removal, also fix the surrounding code so the struct/expression remains valid. The `EntityExport` struct and its construction site need `loot_container` removed from both definition and instantiation.

- [ ] **Step 4: Run build**

Run: `cargo build`
Expected: Clean build (may have unused import warnings for `game_core::LootContainer` in other files — fix those too).

- [ ] **Step 5: Fix any remaining `LootContainer` references**

Search for `LootContainer` across the codebase. Fix any remaining import references to remove them.

Run: `cargo build`
Expected: Clean build, no errors.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/components.rs crates/core/src/lib.rs crates/world/src/export.rs
git commit -m "refactor: remove LootContainer component, replaced by tag-driven Inventory"
```

---

### Task 8: Wire `populate_inventories` into World Gen

**Files:**
- Modify: `src/world_gen.rs:207-217`

- [ ] **Step 1: Add `populate_inventories` call after `spawn_entities`**

In `src/world_gen.rs`, update the `spawn_game_entities` function to call the populator after spawning:

Change:
```rust
pub fn spawn_game_entities(ecs_world: &mut World, player_pos: TilePos) {
    let spawn_rules_toml = include_str!("../assets/config/spawn_rules.toml");
    let rules = match load_spawn_rules(spawn_rules_toml) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load spawn rules: {}", e);
            return;
        }
    };
    spawn_entities(ecs_world, &rules, player_pos);
}
```
to:
```rust
pub fn spawn_game_entities(ecs_world: &mut World, player_pos: TilePos) {
    let spawn_rules_toml = include_str!("../assets/config/spawn_rules.toml");
    let rules = match load_spawn_rules(spawn_rules_toml) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load spawn rules: {}", e);
            return;
        }
    };
    spawn_entities(ecs_world, &rules, player_pos);
    game_world::populate_inventories(ecs_world, None);
}
```

- [ ] **Step 2: Run build**

Run: `cargo build`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add src/world_gen.rs
git commit -m "feat: wire populate_inventories into world gen pipeline"
```

---

### Task 9: Wire `populate_inventories` into Interior Entry

**Files:**
- Modify: `src/location_entry.rs:103-135`

- [ ] **Step 1: Add `populate_inventories` call after interior entity spawning**

In `src/location_entry.rs`, update the `spawn_interior_entities` function. After the entity/chest spawning loop, add the populator call:

After the closing `}` of the `for rule_name in spawn_rules` loop (line ~134), add:

```rust
    let ctx = game_world::PopulateContext {
        dungeon_type: Some(dungeon.dungeon_type.name().to_string()),
        depth: Some(0),
        dungeon_seed: Some(dungeon.seed),
    };
    game_world::populate_inventories(world, Some(&ctx));
```

- [ ] **Step 2: Check what `DungeonType::name()` is called — verify the method exists**

If `DungeonType` uses a different method to get the name string, adjust accordingly. The method may be `name()` or the type may have a `Display` impl or a string field.

- [ ] **Step 3: Run build**

Run: `cargo build`
Expected: Clean build.

- [ ] **Step 4: Commit**

```bash
git add src/location_entry.rs
git commit -m "feat: wire populate_inventories into interior dungeon entry"
```

---

### Task 10: Add `InteractMode::Looting` and Routing

**Files:**
- Modify: `src/interact/mod.rs:20-38` (InteractMode enum)
- Modify: `src/interact/mod.rs:275-293` (disambiguation routing)
- Modify: `src/game/mod.rs:793-822` (route_interaction)

- [ ] **Step 1: Add `Looting` variant to `InteractMode`**

In `src/interact/mod.rs`, add to the `InteractMode` enum (after the `ThrowTargeting` variant):

```rust
    Looting {
        container_entity: bevy_ecs::entity::Entity,
        cursor: usize,
    },
```

- [ ] **Step 2: Add looting check to disambiguation routing**

In `src/interact/mod.rs`, in the `handle_interact_input` function's disambiguation section (around line 275), add a `HAS_INVENTORY` + `CONTAINER` check. Change:

```rust
            let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
            let is_quest_board = game_world.0.get::<game_core::quest::QuestBoard>(target).is_some();
            let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));

            if can_talk {
                interact.active = Some(InteractMode::Talk { npc_entity: target });
            } else if is_quest_board {
                interact.active = Some(InteractMode::QuestBoard);
            } else if can_craft {
                interact.active = Some(InteractMode::Crafting);
            } else {
                interact.active = None;
                if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    msg.messages.push("Nothing to do here.".to_string());
                }
            }
```

to:

```rust
            let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
            let is_quest_board = game_world.0.get::<game_core::quest::QuestBoard>(target).is_some();
            let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));
            let has_inventory = registry.tag_id("HAS_INVENTORY").is_some_and(|id| tags.has(id));
            let is_container = registry.tag_id("CONTAINER").is_some_and(|id| tags.has(id));

            if can_talk {
                interact.active = Some(InteractMode::Talk { npc_entity: target });
            } else if has_inventory && is_container {
                interact.active = Some(InteractMode::Looting { container_entity: target, cursor: 0 });
            } else if is_quest_board {
                interact.active = Some(InteractMode::QuestBoard);
            } else if can_craft {
                interact.active = Some(InteractMode::Crafting);
            } else {
                interact.active = None;
                if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    msg.messages.push("Nothing to do here.".to_string());
                }
            }
```

- [ ] **Step 3: Add looting check to `route_interaction` in game/mod.rs**

In `src/game/mod.rs`, update the `route_interaction` function (line 793-822) to check for HAS_INVENTORY + CONTAINER:

Change:
```rust
    let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
    let is_quest_board = ecs_world.get::<QuestBoard>(target).is_some();
    let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));

    if can_talk {
        interact_state.active = Some(InteractMode::Talk { npc_entity: target });
    } else if is_quest_board {
        interact_state.active = Some(InteractMode::QuestBoard);
    } else if can_craft {
        interact_state.active = Some(InteractMode::Crafting);
    } else {
        if let Some(mut msg) = ecs_world.get_resource_mut::<MessageLog>() {
            msg.messages.push("Nothing to do here.".to_string());
        }
    }
```

to:

```rust
    let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
    let is_quest_board = ecs_world.get::<QuestBoard>(target).is_some();
    let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));
    let has_inventory = registry.tag_id("HAS_INVENTORY").is_some_and(|id| tags.has(id));
    let is_container = registry.tag_id("CONTAINER").is_some_and(|id| tags.has(id));

    if can_talk {
        interact_state.active = Some(InteractMode::Talk { npc_entity: target });
    } else if has_inventory && is_container {
        interact_state.active = Some(InteractMode::Looting { container_entity: target, cursor: 0 });
    } else if is_quest_board {
        interact_state.active = Some(InteractMode::QuestBoard);
    } else if can_craft {
        interact_state.active = Some(InteractMode::Crafting);
    } else {
        if let Some(mut msg) = ecs_world.get_resource_mut::<MessageLog>() {
            msg.messages.push("Nothing to do here.".to_string());
        }
    }
```

- [ ] **Step 4: Run build**

Run: `cargo build`
Expected: Clean build (looting mode is defined but no input handler for it yet — that comes in Task 12).

- [ ] **Step 5: Commit**

```bash
git add src/interact/mod.rs src/game/mod.rs
git commit -m "feat: add InteractMode::Looting with container routing"
```

---

### Task 11: Update Combat Resolution — Dead Entities Become Lootable

**Files:**
- Modify: `src/game/mod.rs:628-645`

- [ ] **Step 1: Change death handling to add tags instead of despawning**

In `src/game/mod.rs`, the `resolve_combat` function, replace the death block (lines 628-645). Change:

```rust
        if let Some(equip) = ecs_world.get::<Equipment>(creature_entity).cloned() {
            let pos = *ecs_world.get::<Position>(creature_entity).unwrap();
            if let Some(wpn) = equip.weapon { ecs_world.entity_mut(wpn).insert(pos); }
            if let Some(arm) = equip.armor { ecs_world.entity_mut(arm).insert(pos); }
        }
        ecs_world.entity_mut(creature_entity).despawn();
```

to:

```rust
        if let Some(equip) = ecs_world.get::<Equipment>(creature_entity).cloned() {
            let pos = *ecs_world.get::<Position>(creature_entity).unwrap();
            if let Some(wpn) = equip.weapon {
                let has_inv = ecs_world.get::<Inventory>(creature_entity).is_some();
                if has_inv {
                    if let Some(mut inv) = ecs_world.get_mut::<Inventory>(creature_entity) {
                        inv.items.push(wpn);
                    }
                } else {
                    ecs_world.entity_mut(wpn).insert(pos);
                }
            }
            if let Some(arm) = equip.armor {
                let has_inv = ecs_world.get::<Inventory>(creature_entity).is_some();
                if has_inv {
                    if let Some(mut inv) = ecs_world.get_mut::<Inventory>(creature_entity) {
                        inv.items.push(arm);
                    }
                } else {
                    ecs_world.entity_mut(arm).insert(pos);
                }
            }
        }
        {
            let mut entity = ecs_world.entity_mut(creature_entity);
            entity.remove::<Creature>();
            if let Some(tid) = registry.tag_id("CONTAINER") {
                let mut tags = entity.get_mut::<game_tags::Tags>().unwrap();
                if !tags.has(tid) { tags.add_tag(tid, game_tags::TagValue::None, &registry); }
            }
            if let Some(tid) = registry.tag_id("INTERACTABLE") {
                let mut tags = entity.get_mut::<game_tags::Tags>().unwrap();
                if !tags.has(tid) { tags.add_tag(tid, game_tags::TagValue::None, &registry); }
            }
        }
```

- [ ] **Step 2: Run build**

Run: `cargo build`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add src/game/mod.rs
git commit -m "feat: dead entities become lootable containers instead of despawning"
```

---

### Task 12: Create Loot Panel UI (Bevy UI)

**Files:**
- Modify: `src/interact/mod.rs` (add `LootPanel` resource and `update_loot_panel` system)
- Modify: `src/render/mod.rs` (register `LootPanel` resource)

The game uses Bevy UI (`commands.spawn(Text(...))`) for panels, not ratatui Frame rendering. Follow the same pattern as `TalkPanel` in `src/interact/talk.rs`.

- [ ] **Step 1: Add `LootPanel` resource and `update_loot_panel` system**

Add to `src/interact/mod.rs` (or create a new `src/interact/loot.rs` module):

```rust
pub mod loot;
```

Create `src/interact/loot.rs`:

```rust
use bevy::prelude::*;
use game_core::{Inventory, Player, Name, Glyph};
use game_tags::TagRegistry;
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct LootPanel(pub Option<Entity>);

pub fn update_loot_panel(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut panel: ResMut<LootPanel>,
    game_world: ResMut<crate::render::GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    let (container_entity, cursor) = match &interact.active {
        Some(InteractMode::Looting { container_entity, cursor }) => (*container_entity, *cursor),
        _ => return,
    };

    let ecs_world = &game_world.0;

    let container_name = ecs_world.get::<Name>(container_entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "Container".to_string());

    let inv = ecs_world.get::<Inventory>(container_entity).cloned();

    let mut lines = vec![format!("╭─ {} ─╮", container_name)];

    match inv {
        Some(ref i) if !i.items.is_empty() => {
            for (idx, &item_entity) in i.items.iter().enumerate() {
                let name = ecs_world.get::<Name>(item_entity)
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| "?".to_string());
                let glyph_char = ecs_world.get::<Glyph>(item_entity)
                    .map(|g| g.char)
                    .unwrap_or('?');
                let marker = if idx == cursor { ">" } else { " " };
                lines.push(format!("│ {} {} {}", marker, glyph_char, name));
            }
        }
        _ => {
            lines.push("│  (empty)".to_string());
        }
    }

    lines.push("├──────────────────────────────────┤".to_string());
    lines.push("│  [Enter] take  [T] take all".to_string());
    lines.push("│  [Esc] close".to_string());
    lines.push("╰──────────────────────────────────╯".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(28.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(root);
}
```

- [ ] **Step 2: Register the loot module and system**

In `src/interact/mod.rs`:
1. Add `pub mod loot;` to the module declarations (line 8 area)
2. Add `update_loot_panel` to the InteractPlugin systems:

```rust
pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InteractState>()
            .add_systems(Update, (
                handle_interact_input,
                talk::handle_talk_input,
                talk::update_talk_panel,
                craft::update_craft_panel,
                quest_board::update_quest_board_panel,
                consume::update_consume_overlay,
                throw::update_throw_overlay,
                overview::update_world_overview,
                loot::update_loot_panel,
            ).run_if(in_state(AppScreen::InWorld)));
    }
}
```

- [ ] **Step 3: Register `LootPanel` resource in RenderPlugin**

In `src/render/mod.rs`, add to the `RenderPlugin::build` method (after line 71):
```rust
.init_resource::<crate::interact::loot::LootPanel>()
```

- [ ] **Step 4: Run build**

Run: `cargo build`
Expected: Clean build.

- [ ] **Step 5: Commit**

```bash
git add src/interact/loot.rs src/interact/mod.rs src/render/mod.rs
git commit -m "feat: add loot panel UI with Bevy UI system"
```

---

### Task 13: Wire Looting Input Handler

**Files:**
- Modify: `src/interact/mod.rs` (add Looting handler in `handle_interact_input`)

- [ ] **Step 1: Add Looting mode handler**

In `src/interact/mod.rs`, inside `handle_interact_input`, add a new arm for `InteractMode::Looting` before the `Some(_) =>` catch-all (line ~246). Insert after the `QuestBoard` handler:

```rust
        Some(InteractMode::Looting { container_entity, cursor }) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
                return;
            }

            let container = *container_entity;
            let cur = *cursor;

            if keyboard.just_pressed(KeyCode::ArrowUp) && cur > 0 {
                if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                    *cursor = cur - 1;
                }
                return;
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                let item_count = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if cur + 1 < item_count {
                    if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                        *cursor = cur + 1;
                    }
                }
                return;
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let player = match game_world.0
                    .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    Ok(e) => e,
                    Err(_) => { interact.active = None; return; }
                };

                let inv = match game_world.0.get::<game_core::Inventory>(container) {
                    Some(i) => i.clone(),
                    None => { interact.active = None; return; }
                };

                if cur >= inv.items.len() {
                    interact.active = None;
                    return;
                }

                let item = inv.items[cur];
                let item_name = game_world.0.get::<game_core::Name>(item)
                    .map(|n| n.0.clone()).unwrap_or_default();

                if let Some(mut player_inv) = game_world.0.get_mut::<game_core::Inventory>(player) {
                    if player_inv.items.len() >= player_inv.capacity {
                        if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                            bus.push(game_core::GameEvent::Message("Inventory full.".to_string()));
                        }
                        return;
                    }
                    player_inv.items.push(item);
                }

                if let Some(mut container_inv) = game_world.0.get_mut::<game_core::Inventory>(container) {
                    container_inv.items.retain(|&e| e != item);
                }

                if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                    bus.push(game_core::GameEvent::Message(format!("Took {}.", item_name)));
                }

                let remaining = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if remaining == 0 {
                    game_world.0.entity_mut(container).despawn();
                    interact.active = None;
                } else if cur >= remaining {
                    if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                        *cursor = remaining.saturating_sub(1);
                    }
                }
                return;
            }

            if keyboard.just_pressed(KeyCode::KeyT) {
                let player = match game_world.0
                    .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    Ok(e) => e,
                    Err(_) => { interact.active = None; return; }
                };

                let inv = match game_world.0.get::<game_core::Inventory>(container) {
                    Some(i) => i.clone(),
                    None => { interact.active = None; return; }
                };

                let mut taken = 0;
                for &item in &inv.items {
                    if let Some(mut player_inv) = game_world.0.get_mut::<game_core::Inventory>(player) {
                        if player_inv.items.len() >= player_inv.capacity { break; }
                        player_inv.items.push(item);
                        taken += 1;
                    }
                }

                if taken > 0 {
                    if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                        bus.push(game_core::GameEvent::Message(format!("Took {} item(s).", taken)));
                    }
                }

                let remaining = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if remaining == 0 {
                    game_world.0.entity_mut(container).despawn();
                }
                interact.active = None;
                return;
            }
            return;
        }
```

- [ ] **Step 2: Run build**

Run: `cargo build`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add src/interact/mod.rs
git commit -m "feat: add looting input handler with take/take-all/cursor navigation"
```

---

### Task 14: Full Integration Test and Cleanup

**Files:**
- Various

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass (478+ tests).

- [ ] **Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No new warnings.

- [ ] **Step 3: Fix any remaining issues**

Address any test failures or clippy warnings.

- [ ] **Step 4: Update TODO.md**

In `TODO.md`, move the completed items from "Remaining — Next Steps" to "Done":
- Item 10 (Location traversal) is already done
- Add new completed item for entity inventory system

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete tag-driven entity inventory system"
```
