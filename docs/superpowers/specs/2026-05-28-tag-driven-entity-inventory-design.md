# Tag-Driven Entity Inventory System

> Design spec. Date: 2026-05-28.
>
> Foundation for entity inventory, looting, and future barter/trade UI.
> Items stored in entity Inventory components, driven entirely by tags.

---

## 1. Problem

NPCs and chests in the game have no real inventory. During world generation, `roll_inventory()` creates items but spawns them as separate ground entities near the NPC (spawner.rs lines 279-295). Chests use a `LootContainer` component with a `loot_table_id` but it is never consumed at runtime. The trade system (`barter.rs`) is fully implemented but disconnected because NPCs have nothing to trade from.

## 2. Design

### 2.1 New Tags

Add an `inventory` archetype to `tags.toml` with exclusivity `any`:

**Gate tag:**
- `HAS_INVENTORY` — marks entity/object as having an inventory

**Capacity tags (mutual, carry `default_magnitude` for slot count):**
- `INVENTORY_TINY` (4 slots)
- `INVENTORY_SMALL` (8 slots)
- `INVENTORY_MEDIUM` (12 slots)
- `INVENTORY_LARGE` (20 slots)
- `INVENTORY_HUGE` (30 slots)

**Content-type tags (any number can apply):**
- `INVENTORY_LOOT` — fill from loot tables
- `INVENTORY_TRADE` — fill from faction economy + location supply
- `INVENTORY_EQUIPMENT` — fill with gear filtered by equipment slot tags

Capacity tags are mutual (only one applies per entity). Content tags are composable — a City Merchant can be `INVENTORY_TRADE` + `INVENTORY_EQUIPMENT`.

Extension: adding new content categories (e.g. `INVENTORY_VAMPIRE`, `INVENTORY_SCAVENGE`) requires only a new tag + a new supply pool mapping in the populator.

### 2.2 Tag-to-Supply-Pool Mapping

| Content Tag | Supply Pool Source | Used By |
|---|---|---|
| `INVENTORY_LOOT` | `loot_tables.toml` filtered by entity tags + dungeon type + depth | Chests, crates, dungeon containers |
| `INVENTORY_TRADE` | `faction_economy.toml` produces + `region_biomes.toml` + `PricingContext.location_supply` | Merchants, shopkeepers |
| `INVENTORY_EQUIPMENT` | `items.toml` filtered by `EQUIP_WEAPON`/`EQUIP_ARMOR` tags, quality biased by entity equip_tier | Guards, warriors, dungeon guardians |
| No content tag + `HAS_INVENTORY` | Fallback: current `roll_inventory()` logic (faction supply + diet + location supply) | Creatures carrying random stuff |

Multiple content tags = merge all matching supply pools.

### 2.3 Populator Function

New file: `crates/world/src/cascade/inventory_populate.rs`

Entry point: `populate_inventories(world: &mut World)`

**Algorithm:**
1. Query all entities with `HAS_INVENTORY` tag that do not yet have an `Inventory` component
2. For each entity:
   a. Read capacity tag → resolve slot count via `default_magnitude` on the tag definition. If no capacity tag, default to 8.
   b. Read content tags → determine supply pool(s). If no content tags, use fallback `roll_inventory()`.
   c. Read entity tags for context (faction, creature type, PEACEFUL, etc.)
   d. Read environment context: location economy if available, dungeon type/depth for interior chests
   e. Create Item entities in the ECS world with `Item`, `Name`, `Glyph`, `Tags` components — **no Position component**
   f. Collect item entity IDs into `Inventory { items: Vec<Entity>, capacity }`
   g. Insert `Inventory` component on the entity
3. For entities with `INVENTORY_EQUIPMENT` that also have an `Equipment` component: auto-equip the best item per slot

**Helper functions:**
- `resolve_capacity(tags, registry) -> usize`
- `build_supply_pool(content_tags, entity_tags, context, engine, registry) -> Vec<InventoryItem>`
- `fill_from_loot_table(entity_tags, context, loot_tables, registry, rng) -> Vec<InventoryItem>`
- `fill_from_trade(entity_tags, context, engine, registry, rng) -> Vec<InventoryItem>`
- `fill_from_equipment(entity_tags, context, engine, registry, rng) -> Vec<InventoryItem>`

### 2.4 World Gen Pipeline Integration

```
Stage 1: World Canvas
Stage 2: Locations
Stage 3: Economy + Trade
Stage 4: Entity Spawning          ← spawner creates entities with tags, no inventory items
Stage 4b: populate_inventories()  ← NEW: fills inventories based on tags
Stage 5: Equipment Generation     ← unchanged
```

For dungeon interiors (`location_entry.rs`), `populate_inventories()` runs after `spawn_interior_entities()` and `place_dungeon_chests()`.

### 2.5 Chest Unification

`LootContainer` component is removed. Chests are now regular entities with:
- Tags: `HAS_INVENTORY`, `INVENTORY_SMALL`, `INVENTORY_LOOT`, `CONTAINER`
- No `LootContainer` component
- The populator reads `INVENTORY_LOOT` → selects loot table via `LootTables.tables_for_dungeon(dungeon_type, depth)` (already exists in `loot.rs`) → creates items inside Inventory
- No `loot_table_id` stored on the entity — dungeon type + depth context is available from the `ActiveInterior` or passed to the populator

`place_dungeon_chests()` in `loot.rs` updated to spawn tag-based chest entities without `LootContainer`.

### 2.6 Death and Looting

**On entity death (combat resolution):**
- Entity keeps its `Inventory` component (items are NOT dropped to ground)
- Entity gets `CONTAINER` tag added (if not already present)
- Entity gets `INTERACTABLE` tag added
- Entity's `Glyph` changes to a corpse glyph or remains as-is
- Entity loses `Creature` component (optional — marks as dead)

**Loot interaction:**
- New `InteractMode::Looting { container_entity: Entity }` variant in `interact/mod.rs`
- Player presses `E` near entity with `HAS_INVENTORY` + `CONTAINER` → enters Looting mode
- Loot panel shows container's Inventory items
- Player selects items to take → item moves from container Inventory to player Inventory
- "Take all" option moves all items
- When container Inventory is empty, entity is despawned (or kept as decorative corpse)

**Interaction routing priority** (in `interact/mod.rs` and `game/mod.rs`):
1. `CAN_TALK` → Talk mode
2. `HAS_INVENTORY` + `CONTAINER` → Looting mode
3. `QuestBoard` → QuestBoard mode
4. `CAN_CRAFT` → Crafting mode

### 2.7 Spawn Rules Updates

`spawn_rules.toml` entity entries get inventory tags:

| Entity | New Tags |
|---|---|
| City Merchant | `HAS_INVENTORY`, `INVENTORY_MEDIUM`, `INVENTORY_TRADE` |
| Settlement Guard | `HAS_INVENTORY`, `INVENTORY_SMALL`, `INVENTORY_EQUIPMENT` |
| Dungeon Guardian | `HAS_INVENTORY`, `INVENTORY_SMALL`, `INVENTORY_EQUIPMENT` |
| Ruin Scavenger | `HAS_INVENTORY`, `INVENTORY_TINY` |
| Creatures with PEACEFUL | `HAS_INVENTORY`, `INVENTORY_TINY` (fallback fill) |

### 2.8 Spawner Changes

`spawner.rs`: Remove the ground-item spawning loop (lines 279-295). The spawner's job is creating entities with tags. The populator handles inventory. The `roll_inventory()` function stays as a supply pool builder called by the populator, not by the spawner.

## 3. Component Changes

| Component | Action |
|---|---|
| `LootContainer` | **Removed** — replaced by tag-driven Inventory |
| `Inventory` | Unchanged — `{ items: Vec<Entity>, capacity: usize }` |
| `Equipment` | Unchanged — `{ weapon, armor, accessory: Option<Entity> }` |

No new components.

## 4. File Impact

| File | Change |
|---|---|
| `assets/config/tags.toml` | Add `inventory` archetype with 8 tags |
| `assets/config/spawn_rules.toml` | Add inventory tags to relevant spawn rules |
| `crates/world/src/cascade/inventory_populate.rs` | **New file** — populator + helpers |
| `crates/world/src/cascade/mod.rs` | Register `inventory_populate` module |
| `crates/world/src/spawner.rs` | Remove ground-item spawning loop |
| `crates/world/src/loot.rs` | `place_dungeon_chests()` uses tags instead of `LootContainer` |
| `src/world_gen.rs` | Call `populate_inventories()` after Stage 4 |
| `src/location_entry.rs` | Call `populate_inventories()` after interior entity spawning |
| `src/interact/mod.rs` | Add `InteractMode::Looting` variant + routing |
| `src/game/mod.rs` | Combat: on kill, add `CONTAINER` + `INTERACTABLE` tags |
| `crates/core/src/components.rs` | Remove `LootContainer` struct |
| `crates/world/src/export.rs` | Remove `LootContainerExport`, update serialization |
| `crates/render/src/panels/` | New loot panel for Looting mode |
| `crates/core/src/lib.rs` | Remove `LootContainer` from exports |

## 5. Test Plan

- `inventory_populate.rs` unit tests: tag → capacity resolution, tag → supply pool selection, multi-content-tag merging, fallback behavior
- Updated `loot.rs` tests: tag-based chests instead of `LootContainer`
- Integration test: spawn entity with inventory tags → run populator → verify Inventory contents match tag-driven rules
- Death/loot test: entity with Inventory dies → gains CONTAINER + INTERACTABLE → player can loot items → items transfer correctly
- Capacity enforcement test: inventory respects slot limit

## 6. Out of Scope (Future Work)

- Barter/trade UI wiring (uses the Inventory foundation this spec builds)
- `resolve_barter_with_haggle()` integration into trade flow
- `RegionEconomies` pricing display in trade UI
- `value_of_item()` and `apply_faction_price_modifier()` connection to runtime pricing
- Inventory regeneration for merchants over time
- NPC-initiated trades (`NpcAction::OfferTrade` dispatch)
- Runtime economy dynamics (prices changing based on player actions)
- `trade_only` flag on items (deferred until trade UI)
