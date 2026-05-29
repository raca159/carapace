# Carapace — Engine Audit & System Map

> Living document. Last audited: 2026-05-28.
>
> Complete mapping of all game systems, TOML configs, tag flows, cascade pipeline,
> and their wiring status. Use this as the reference for all future integration work.

---

## Legend

| Status | Meaning |
|--------|---------|
| WIRED | Fully integrated into the Bevy game loop |
| PARTIAL | Code exists and partially connected, but missing key integration |
| STUB | Function signature exists but body is placeholder |
| DEAD | Code exists but has zero call sites in the game loop |
| PHANTOM | Used in TOML data but never registered in the tag registry |

---

## 1. System Inventory

### 1.1 Fully Wired Systems (15)

| # | System | Schedule | Entry Point | Reads | Writes |
|---|--------|----------|-------------|-------|--------|
| 1 | Boot sequence | OnEnter(Boot) | `render/mod.rs:101` | — | Camera, sprite atlas |
| 2 | Player movement | Update (InWorld) | `game/mod.rs:74` | Keys, GameWorld, Tags, Position, Health, Equipment, WorldMap | Position, GameTurnState, EventBus, InteractState |
| 3 | Turn loop (NPC) | Update (InWorld) | `game/mod.rs:464` | GameTurnState, GameWorld | TurnCounter, WeatherState, WeatherContext |
| 4 | Combat | On player bump/F key | `game/mod.rs:512` | Health, Equipment, Tags, Name, Position | Health, Position (drops), EventBus, NpcEmotionalState |
| 5 | NPC AI | Via turn loop | `behavior.rs:320` | TagRegistry, WorldMap, FactionRelationships, BehaviorRules, Position, Tags | Position (creatures) |
| 6 | Pathfinding | Via NPC AI | `pathfinding.rs` | WorldMap, Tags (BLOCKED) | — (returns path) |
| 7 | Status effects | Via turn loop | `status.rs:7` | TagRegistry, InteractionRules, Tags, Position | Tags (tick down, interactions) |
| 8 | Weather | Via turn loop | `weather.rs` | WeatherState | WeatherState, WeatherContext |
| 9 | Talk hub | Update (InWorld) | `interact/talk.rs:46` | NpcEmotionalState, NpcActionWeights, TagRegistry, DialogueLinesResource, FactionRelationships, WeatherContext | MessageLog, NpcEmotionalState, InteractState |
| 10 | Crafting | Update (InWorld) | `interact/craft.rs:14` | CraftingRecipesResource, Inventory, TagRegistry, Position | Inventory, EventBus, spawns items |
| 11 | Throw | Update (InWorld) | `interact/throw.rs:8` | Tags (THROWABLE), Position, WorldMap, Health, Equipment | Removes from inventory, Health, EventBus |
| 12 | Consume | Update (InWorld) | `interact/consume.rs:8` | Tags (EDIBLE, DRINKABLE), Health, Inventory | Health, Tags (POISONED/BURNING), removes item |
| 13 | Event formatting | Update (InWorld) | `event_format.rs:217` | EventBus, EventFormats, TagRegistry | MessageLog |
| 14 | Rendering pipeline | Update | `render/mod.rs` | WorldMap, Tile, Position, Glyph, GameCamera | Sprites, HUD text |
| 15 | Location traversal | Update (InWorld) | `game/mod.rs` | Keys, GameWorld, Tags, MapLayer, LocationMap | MapLayer, WorldMap, OverworldEntity tags, Position |

### 1.2 Partial Systems (7)

| # | System | What Works | What's Missing |
|---|--------|-----------|----------------|
| 1 | Barter/Trade | B key routes to `start_trade()`, `resolve_barter_with_haggle()` fully implemented in core | Trade UI is a stub ("Full barter UI coming soon"), `RegionEconomies` pricing never displayed, no inventory exchange |
| 2 | Equip/Unequip | `handle_equip()`/`handle_unequip()` fully implemented with slot assignment, tag transfer, old equipment return | No key binding triggers them — no way to change equipment in-game |
| 3 | Quest system | Quest board display + acceptance (1-9 keys) works | `check_quest_completion`, `track_kill`, `track_collect`, `handle_quest_turn_in` all dead — quests never complete |
| 4 | NPC Action Engine | `score_action()` used in talk panel for display scoring | `NpcAction` enum (Speak, OfferTrade, AttackTarget, GiveQuest, EndConversation) never dispatched — scoring is cosmetic |
| 5 | Personality | `PersonalityScores` queried by `process_npc_turns`, `NpcPersonalitiesResource` loaded | `PersonalityScores` never inserted on spawned entities — personality multiplier always 1.0 |
| 6 | Encounters | `roll_encounter()` called on player movement, weather modulates chance | `biome_tags` parameter always `Vec::new()` — no biome-based encounter filtering |
| 7 | Location traversal | Previously cosmetic | **Now WIRED** — Phase 2 interiors: `>` enters locations with BSP dungeon gen, `<` exits, depth progression on deeper stairs, dungeon creatures + chests |

### 1.3 Dead Systems (10+)

| # | System | File | Lines | Notes |
|---|--------|------|-------|-------|
| 1 | Save/Load | `core/save.rs` | ~200 | Full implementation, never called, no save points |
| 2 | Traps | `core/traps.rs` | ~150 | Detection/disarm/trigger, never spawned or triggered |
| 3 | Durability | `core/durability.rs` | ~100 | Degrade/repair, items never get Durability component |
| 4 | Narrative events | `core/narrative.rs` | ~100 | Resources loaded, `check_narrative_events` never called |
| 5 | Legacy input | `core/input.rs` | ~300 | Crossterm terminal backend, replaced by Bevy `ButtonInput` |
| 6 | Gene splicing | `core/gene_splicing.rs` | ~557 | Module not declared in lib.rs, recipes loaded in tests only |
| 7 | Genetics | `core/genetics.rs` | ~150 | Module not declared in lib.rs |
| 8 | Artifacts | `core/artifacts.rs` | ~206 | Module not declared in lib.rs |
| 9 | Game endings | `core/game_endings.rs` | ~137 | Module not declared in lib.rs |
| 10 | TurnState/TurnPhase | `core/turn.rs` | ~50 | Superseded by `GameTurnState`, never inserted |
| 11 | Spatial hash grid | `crates/world/src/spatial.rs` | ~262 | Module not declared, optimization unused |

---

## 2. Cascade Pipeline — Data Flow Map

The cascade pipeline executes in `src/world_gen.rs` during world generation:

```
Stage 1: World Canvas  →  Stage 2: Locations  →  Stage 3: Economy + Trade
  WorldMap + tiles       LocationMap              RegionEconomies + TradeRoutes

Stage 4: Entity Spawning        ← spawn_entities() with tags, no ground items
  │                              generate_npc_equipment() called during spawn
  │
  └──→ Stage 4b: populate_inventories()  ← NEW: fills Inventory from HAS_INVENTORY tags
       Uses: INVENTORY_LOOT → loot tables
             INVENTORY_TRADE → faction economy + location supply
             INVENTORY_EQUIPMENT → filtered items.toml
             (none) → fallback roll_inventory()
```

### Stage-by-Stage Detail

#### Stage 1: World Canvas
- **Config**: `world.toml` (noise params), `biome_rules.toml` (elevation/moisture/temp ranges + tags)
- **Processing**: 3 noise layers (elevation/moisture/temperature) → latitude blend → `BiomeClassifier::classify()` per tile
- **Outputs**: `WorldMap` resource + tile entities with `Tile {pos, elevation, moisture, temperature, biome_name, glyph, color}` and `Tags` (biome + terrain tags)
- **Consumers**: Stages 2, 4; runtime movement, rendering, overview

#### Stage 2: Location Placement
- **Config**: `location_types.toml` (7 types: seed/city/dungeon/village/outpost/cave/ruin/shrine)
- **Processing**: 3-pass iterative (seeds→villages→POIs), habitability scoring, zone-of-influence, biome/faction affinity
- **Outputs**: `LocationMap` with `Vec<PlacedLocation>` (id, type, name, x, y, zone_radius, tags, faction)
- **Consumers**: Stage 3 economy loop; Stage 4 spawning; runtime location traversal; world overview

#### Stage 3: Economy + Trade
- **Config**: `region_biomes.toml` (biome production), `faction_economy.toml` (faction produce/consume)
- **Processing**: `HAS_ECONOMY` gate → supply = biome + faction production → demand = faction consumption → price_multipliers = clamp(demand/supply, 0.3, 3.0) → prosperity = min(supply/200, 1.0). Trade routes: nearest-3 economies, distance-blended prices.
- **Outputs**: `RegionEconomies` (HashMap<usize, PricingContext>), `TradeRoutes`
- **Consumers**: Stage 4 spawner (prosperity + location_supply). **DISCONNECTED at runtime** — trade UI is a stub

#### Stage 4: Entity Spawning
- **Config**: `spawn_rules.toml` (tags, biome_tags, density, faction, equip_chance, equip_tier, location_types)
- **Processing**: Two branches — location entities (within zone_radius, gated by density) and wild entities (every tile, biome match). Both call `generate_npc_equipment()`. No inventory items created — entities have `HAS_INVENTORY` tags if they should get an Inventory component.
- **Outputs**: Creatures with (Position, Glyph, Health, Tags, Name, Creature, BehaviorState, NpcEmotionalState, Equipment, [Faction], possibly PersonalityScores). No Item entities on ground.
- **Consumers**: Stage 4b populator (reads tags, fills Inventory)

#### Stage 4b: Inventory Population (NEW)
- **Config**: `items.toml`, `loot_tables.toml`, `faction_economy.toml`, `tags.toml` (inventory archetype)
- **Processing**: `populate_inventories()` scans for `HAS_INVENTORY` tag. Reads capacity tags (INVENTORY_TINY→HUGE) for slot count. Reads content tags (INVENTORY_LOOT/TRADE/EQUIPMENT) to dispatch fill strategy. Creates Item entities with no Position. Inserts Inventory component.
- **Outputs**: Inventory component on entities with item Entity references
- **Consumers**: Combat (death drops), trade (future), looting (runtime)

#### Stage 5: Equipment Generation
- **Config**: `items.toml` (filtered by slot tag), quality_bias per item
- **Processing**: `slot_needs()` → HUMANOID/AGGRESSIVE → EQUIP_WEAPON; HUMANOID/TERRITORIAL → EQUIP_ARMOR. Per slot: filter candidates → material preference (HUMANOID→METAL×2) → weighted pick → quality roll (60/25/10/4/1 base, shifted by bias + prosperity).
- **Outputs**: `Equipment` component on creature, spawned item entities with quality tags
- **Consumers**: `resolve_combat()` damage calc, death drops

### Pipeline Disconnections

| # | Output | Produced By | Problem |
|---|--------|-------------|---------|
| D1 | `TradeRoutes` resource | Stage 3 | Never read after insertion. Dead resource. |
| D2 | `PricingContext.price_multipliers` | Stage 3 | Trade UI is a stub — pricing data computed but never displayed or used. |
| D3 | `InventoryItem.trade_only` | Stage 4b | Set during `INVENTORY_TRADE` fill, preserved in InventoryItem. Trade UI (future) must read it from item tags or supply context. |
| D4 | WFC locations | `crates/world/src/wfc.rs` | Entire WFC module with 6 tilesets exists but is never called from cascade pipeline. |

### Missing Connections

| # | Gap | Impact |
|---|-----|--------|
| M1 | Trade system has no price integration | `resolve_barter_with_haggle()` exists but is never called; pricing data unused |
| M2 | Creature inventory not stored on creature | **RESOLVED** — items stored in NPC `Inventory` component via tag-driven populator |
| M3 | No runtime economy dynamics | Economy computed once at world gen; supply/demand/prices never change |
| M4 | Faction economy disconnected from runtime faction system | Faction produce/consume only used during generation; no runtime connection to reputation |
| M5 | `equip_tier` is statistical nudge, not floor | EPIC-tier creature can still roll COMMON equipment |

---

## 3. TOML Config Audit

### 3.1 Active Runtime Configs (21)

All loaded via `include_str!()` in `src/world_gen.rs`:

| # | File | Parsed Into | Primary Consumer |
|---|------|-------------|------------------|
| 1 | `assets/config/tags.toml` | `TagRegistry` | Tag system (all modules) |
| 2 | `assets/config/interactions.toml` | `InteractionRules` | `status.rs` |
| 3 | `assets/config/items.toml` | `CascadeEngine` | Equipment + inventory generation |
| 4 | `assets/config/region_biomes.toml` | `CascadeEngine` | Economy computation |
| 5 | `assets/config/faction_economy.toml` | `CascadeEngine` | Economy computation |
| 6 | `assets/config/location_types.toml` | `CascadeEngine` | Location placement |
| 7 | `assets/config/world.toml` | `WorldGenConfig` | World canvas generation |
| 8 | `assets/config/biome_rules.toml` | `BiomeClassifier` | Biome classification |
| 9 | `assets/config/factions.toml` | `FactionRelationships` | Faction hostility checks |
| 10 | `assets/config/behavior_rules.toml` | `BehaviorRules` | NPC chase/flee/guard/approach/wander |
| 11 | `assets/config/narrative_events.toml` | `NarrativeEvents` | Loaded but never queried |
| 12 | `assets/config/quests.toml` | `QuestTemplates` | Quest board display |
| 13 | `assets/config/lore_fragments.toml` | `LoreFragmentsResource` | Loaded but never queried |
| 14 | `assets/config/npc_personalities.toml` | `NpcPersonalitiesResource` | Loaded but never queried |
| 15 | `assets/config/npc_actions.toml` | `NpcActionWeights` | Talk panel action scoring |
| 16 | `assets/config/dialogue.toml` | `DialogueLinesResource` | Talk panel dialogue selection |
| 17 | `assets/config/crafting.toml` | `CraftingRecipesResource` | Crafting panel |
| 18 | `assets/config/loot_tables.toml` | `LootTables` | Loaded but never queried |
| 19 | `assets/config/encounters.toml` | `Encounters` | Encounter rolling |
| 20 | `assets/config/spawn_rules.toml` | `SpawnRulesFile` | Entity spawning |
| 21 | `assets/config/events.toml` | `EventFormats` | Event message formatting |

Additional active configs:
- `assets/sprites/atlas.toml` → sprite atlas (glyph-based rendering)
- `assets/config/wfc_tilesets/*.toml` (6 tilesets) → WFC generation (loaded via file I/O)
- `crates/llm/assets/config/llm.toml` → LLM config (crate-level)

### 3.2 Dead Configs (11 files)

| File | Why Dead |
|------|----------|
| `assets/config/entities/merchant.toml` | Never loaded by any `include_str!` or file I/O |
| `assets/config/entities/vampire_lord.toml` | Never loaded |
| `assets/config/entities/guard.toml` | Never loaded |
| `assets/config/entities/wandering_merchant.toml` | Never loaded |
| `assets/config/entities/the_anomaly.toml` | Never loaded |
| `assets/config/entities/cryo_vault_overseer.toml` | Never loaded |
| `assets/config/world/world.toml` | Duplicate of top-level `world.toml` |
| `assets/config/world/biome_rules.toml` | Older version with simpler biome IDs |
| `assets/sprites/ui_icons.toml` | Never loaded (no texture atlas loader) |
| `assets/sprites/creatures_carapace.toml` | Never loaded |
| `assets/sprites/tiles_terrain.toml` | Never loaded |

### 3.3 Schema Drift

| Pair | Issue |
|------|-------|
| `assets/config/tags.toml` (830 lines) vs `crates/tags/assets/config/tags.toml` (363 lines) | Runtime has `conflicts`, `craft_complexity` archetype, 6 extra trait tags. Tests validate against smaller file. |
| `assets/config/interactions.toml` vs `crates/tags/assets/config/interactions.toml` | 4 interaction rules produce different outputs (ACID+METAL, FREEZING+LIQUID, HOT+ICE_MATERIAL, ARCANE+ARCANE). Tests validate different behavior than production. |

### 3.4 Resources Loaded But Never Consumed

| Resource | Inserted At | Status |
|----------|-------------|--------|
| `PlayerStats` | `world_gen.rs:66` | Kept — consumed by render overlays |
| `NpcPersonalitiesResource` | `world_gen.rs:115` | **Now wired** — consumed by spawner.rs for personality generation |
| `NarrativeEvents` | Removed | Insertion deleted (TOML kept) |
| `NarrativeCooldowns` | Removed | Insertion deleted |
| `LoreFragmentsResource` | Removed | Insertion deleted (TOML kept) |
| `LootTables` | Removed | Insertion deleted (TOML kept) |

---

## 4. Tag System Audit

### 4.1 Architecture

- **Central registry**: `assets/config/tags.toml` — 23 archetypes, ~160 tags
- **Tag flow**: TOML configs assign tags → Rust code consumes them
- **Dynamic derivation**: `tags_from_personality()` (6 output tags), `weather_tags_for_context()` (11 output tags)
- **Interaction rules**: `interactions.toml` — 30+ rules producing/consuming tag pairs

### 4.2 Tag Categories

| Archetype | Count | With Consumers | Without Consumers |
|-----------|-------|---------------|-------------------|
| element | 7 | 5 | EARTH, AIR |
| material | 12 | 9 | SAND, PAPER |
| state | 4 | 1 (LIQUID) | SOLID, GAS, PLASMA |
| quality | 5 | 5 | — |
| temperature | 5 | 3 | NEUTRAL, WARM |
| moisture | 4 | 2 | DAMP, SOAKED |
| creature_type | 9 | 7 | DEMON, DRAGON |
| size | 6 | 5 | COLOSSAL |
| diet | 7 | 2 (HERBIVORE, CARNIVORE in TOML only) | 5 never assigned |
| trait | 24 | 14 | 10 including FEARLESS, NOCTURNAL, STATIONARY, LLM_CAPABLE, CRAFT_ANYWHERE |
| status | 13 | 7 | 6 with no producer (PARALYZED, BLEEDING, BLINDED, INVISIBLE, ENRAGED, MELTING) |
| terrain | 11 | 3 | 8 including SLOW, DANGEROUS, OPAQUE, ARABLE, MINERAL_RICH |
| biome | 18 | 16 | BIOME_MOUNTAIN_PEAK, BIOME_RIVER, BIOME_LAKE |
| magic | 6 | 6 | — |
| interaction | 20 | 10 | 10 including WATERPROOF, FRAGILE, HEAT_RESISTANT, FLOATS, SPREADS |
| resource | 14 | 11 | GEM_RUBY, GEM_DIAMOND, WATER_FRESH, WATER_SALT, FISH |
| damage_type | 8 | 4 | LIGHTNING_DAMAGE, POISON_DAMAGE |
| sense | 6 | 1 (SIGHT) | HEARING, SMELL, TREMOR_SENSE, DARKVISION, THERMAL_SENSE |
| item_slot | 4 | 3 | EQUIP_NONE |
| item_function | 6 | 5 | USABLE |
| weapon_type | 2 | 0 | MELEE, RANGED (no combat differentiation) |
| armor_type | 3 | 0 | LIGHT_ARMOR, HEAVY_ARMOR; SHIELD never assigned |
| craft_complexity | 5 | 0 | All 5 (magnitude metadata never read) |
| inventory | 9 | 4 (HAS_INVENTORY, 5 capacity, 3 content) | INVENTORY_LOOT, INVENTORY_TRADE, INVENTORY_EQUIPMENT, CONTAINER all consumed by populator; capacity tags consumed by populator |

### 4.3 Phantom Tags (~80 not in registry)

Tags used in `items.toml` and `entity_templates.toml` but **not registered** in `tags.toml`. `registry.tag_id(name)` returns `None` — silently dropped:

**Key missing registrations**: CHITIN, ARTIFACT, CONSUMABLE, CARAPACE, SANGUINE, FAMILIAR, MUTANT, HUMAN, TOOL, VALUABLE, CURRENCY, MEDICAL, PLANT, FOOD, CHEMICAL, CORROSIVE, POISON, ICE (damage), SHOCK, SONIC, REGENERATIVE, SWARM, KEY, STRUCTURE, BLOOD, SANGUIS, TAINTED, ALCHEMICAL, MUTAGENIC, ELECTRONIC, OPTICAL, CRAFTING, ARMOR_CRAFTING, COMPOUND_EYES, DELICATE, THICK, HARDENED, PRESSURE_RESISTANT, CONCEALING, CHROMATOPHORIC, BIO_ELECTRIC, REINFORCED, CONSUMABLE, PERISHABLE, LIGHT, ENERGY, ELECTRIC, CONCEALED, CEREMONIAL, BLOOD_BONDED, PRECISION, SALVAGE, CRUSHING

**From other configs**: BIOME_DUNGEON, BIOME_CAVE, BIOME_ANCIENT_VAULT (spawn_rules), HOSTILE_ZONE, HAS_LOOT, SACRED (location_types), SANGUINE_GOODS, FAMILIAR_GOODS, TECH_GOODS, MUTATED_GOODS (faction_economy), QUEST_GIVER (npc_actions)

### 4.4 Tag Flow Map

```
tags.toml (central registry)
    │
    ├── items.toml ──────────→ spawner.rs → Tags component on entities
    │                          equipment.rs → slot routing, damage/armor calc
    │                          economy.rs → price multiplier lookup
    │
    ├── location_types.toml ─→ locations.rs → PlacedLocation.tags
    │                          trade.rs → filters HAS_ECONOMY
    │                          economy.rs → checks HAS_ECONOMY
    │
    ├── spawn_rules.toml ────→ spawner.rs → Tags on spawned entities
    │                          **3 phantom biome tags dropped**
    │
    ├── region_biomes.toml ──→ economy.rs → supply weights
    │
    ├── faction_economy.toml → economy.rs → supply/demand weights
    │                          **4 phantom goods tags dropped**
    │
    ├── behavior_rules.toml ─→ behavior.rs → trait_tag → chase/flee/guard/approach/wander
    │
    ├── interactions.toml ───→ InteractionRules resource
    │                          status.rs → self/cross interactions per tick
    │
    ├── npc_actions.toml ────→ npc_action.rs → requires_tags gates, env_modifiers
    │
    ├── tags_from_personality() → Tags component (AGGRESSIVE, PEACEFUL, COWARDLY, CURIOUS, TERRITORIAL, FEARLESS)
    │
    └── weather_tags_for_context() → WeatherContext.tags (Vec<String>, NOT TagIds!)
                                      encounters.rs → encounter chance
                                      npc_action.rs → env_modifiers
                                      interact/talk.rs → is_night
```

### 4.5 Critical Tag Gaps

| # | Gap | Impact |
|---|-----|--------|
| G1 | ~80 phantom tags silently dropped | Entities have incomplete tag sets; new systems checking these tags find nothing |
| G2 | Weather tags are `Vec<String>`, not `TagId` | Cannot trigger interaction rules; rain can't make entities WET |
| G3 | 30+ tags defined but never consumed | HEAT_RESISTANT, COLD_RESISTANT, FRAGILE, SLIPPERY, etc. represent unrealized gameplay |
| G4 | 6 status effects have no producer | PARALYZED, BLEEDING, BLINDED, INVISIBLE, ENRAGED, MELTING never applied |
| G5 | 5 of 6 sense tags unused | Only SIGHT consumed; HEARING, SMELL, etc. are dead |
| G6 | Equipment material tags incomplete | BONE weapons get no material bonus; CHITIN not registered |
| G7 | Terrain tags with move_cost never read | SLOW, DANGEROUS, OPAQUE defined but movement is binary (blocked/not) |
| G8 | Size tag metadata never read | `tile_occupancy`/`hp_mult` on TagDef unused; health derived from string matching instead |

---

## 5. Critical Gaps — Priority Fix List

### Done (2026-05-28)

| # | Gap | Resolution |
|---|-----|------------|
| 1 | PersonalityScores never inserted on entities | `spawner.rs` now generates random scores with faction biases + calls `tags_from_personality()` on both spawn branches |
| 2 | ~80 phantom tags silently dropped | 80 tags registered in `tags.toml` across 9 archetypes |
| 3 | Test/runtime TOML drift | Runtime TOMLs copied to `crates/tags/assets/config/`, test assertions updated |
| 4 | Equip/Unequip has no key binding | E key in inventory mode calls `handle_equip()` |
| 5 | Dead code cleanup | `core/input.rs` deleted, crossterm/ratatui deps removed, unused resource insertions removed |
| 6 | Pre-existing compile errors | Fixed: `WorldOverviewState` missing `#[derive(Resource)]`, `player_pos` tuple field access, borrow-after-move in talk.rs, double borrow in game/mod.rs overview |
| 7 | **Environmental Score System** | Full weather/location deep integration (Phase 1): threshold fields on tags, base biome scores, WeatherDef TOML templates, score computation, tag pipeline via `apply_environmental_tags()`, `WeatherSensitive` component, NPC sense range modulated by weather visibility, DARKVISION/THERMAL_SENSE support |

### Done (2026-05-28)

| # | Gap | Resolution |
|---|-----|------------|
| 10 | Location traversal cosmetic (Phase 2 interiors) | Full enter/exit/depth-progression with BSP dungeon gen for 5 location types |
| 11 | Entity inventory system missing | **Tag-driven entity inventory system** — see full architecture entry below |

---

## Entity Inventory System — Architecture & Impact

### What was built

A tag-driven system where entities (NPCs, chests, containers) get an `Inventory` component filled by a standalone `populate_inventories()` function that scans for `HAS_INVENTORY` tag, reads capacity tags for slot count, and dispatches to fill strategies based on content tags.

**Core architecture:**
- **Spawner's job** is just to create entities with tags — it no longer spawns ground items
- **Populator** (`crates/world/src/cascade/inventory_populate.rs`) is a standalone function called from `world_gen.rs` after entity spawning, and from `location_entry.rs` after interior/depth spawning
- **Inventory fill strategies** are tag-dispatched:
  | Tag | Strategy |
  |-----|----------|
  | `INVENTORY_LOOT` | Loot tables (dungeon type + depth) |
  | `INVENTORY_TRADE` | Faction economy supply + location supply |
  | `INVENTORY_EQUIPMENT` | Items with EQUIP_WEAPON/ARMOR/ACCESSORY tags |
  | (no content tags) | Fallback `roll_inventory()` (faction + diet + location) |
- **Tags added**: `HAS_INVENTORY`, `INVENTORY_TINY(4)`, `INVENTORY_SMALL(8)`, `INVENTORY_MEDIUM(12)`, `INVENTORY_LARGE(20)`, `INVENTORY_HUGE(30)`, `INVENTORY_LOOT`, `INVENTORY_TRADE`, `INVENTORY_EQUIPMENT`, `CONTAINER`

### Impact on trade

The barter/trade foundation is now in place:
- **City Merchants** get `HAS_INVENTORY + INVENTORY_MEDIUM + INVENTORY_TRADE` — their inventory is filled from faction economy + location supply directly into the NPC's `Inventory` component, NOT on the ground
- When `start_trade()` is wired (B key in talk panel), the NPC's `Inventory` is directly readable to build `BarterOffer` items
- The `trade_only` flag on `InventoryItem` is set for PEACEFUL entities, ready for trade filtering
- Previously blocking gap D5 (creature inventory ownership) is **resolved**: items are stored in NPC `Inventory`, not scattered on ground. M2 also resolved.

### Impact on loot

- **Chests/containers** are regular entities with `HAS_INVENTORY + INVENTORY_SMALL + INVENTORY_LOOT + CONTAINER` tags. No `LootContainer` component.
- **Dead creatures** keep their `Inventory` and gain `CONTAINER + INTERACTABLE` tags — lootable corpses. Equipment absorbed into inventory if present.
- **Loot panel UI** (`src/interact/loot.rs`) provides take (Enter) and take-all (T) with cursor navigation
- `LootContainer` component removed entirely across codebase

### Pipeline change

New Stage 4b between entity spawning and equipment generation:
```
Stage 4: Entity Spawning          ← spawner creates entities with tags, no inventory
Stage 4b: populate_inventories() ← fills Inventory based on HAS_INVENTORY tags
Stage 5: Equipment Generation     ← unchanged
```

### Files created (2)
- `crates/world/src/cascade/inventory_populate.rs` — Populator: PopulateContext, populate_inventories(), fill strategy helpers, 4 unit tests
- `src/interact/loot.rs` — Loot panel: LootPanel resource, update_loot_panel system

### Files modified (14)
`assets/config/tags.toml`, `crates/tags/assets/config/tags.toml`, `assets/config/spawn_rules.toml`, `crates/world/src/cascade/mod.rs`, `crates/world/src/lib.rs`, `crates/world/src/spawner.rs`, `crates/world/src/loot.rs`, `crates/core/src/components.rs`, `crates/core/src/lib.rs`, `crates/world/src/export.rs`, `src/world_gen.rs`, `src/location_entry.rs`, `src/interact/mod.rs`, `src/game/mod.rs`, `src/render/mod.rs`

### Tasks (16 commits across 14 tasks)

1-4: Tags, spawn rules, populator, exports  
5-7: Spawner cleanup, chest tags, LootContainer removal  
8-9: World gen + interior pipeline wiring  
10-13: Looting routing, death handling, loot panel, input handler  
14: Integration test + cleanup  
Fixes: enter_next_depth population, unused _seed variable

### Remaining — Next Steps

| # | Gap | Fix | Effort |
|---|-----|-----|--------|
| 8 | Barter backend exists, UI is stub | Wire `resolve_barter_with_haggle` into `start_trade()` with `RegionEconomies` pricing | Medium |
| 9 | Quest tracking dead | Wire `check_quest_completion`/`track_kill`/etc. into turn loop | Medium |

### Disabled Reference Files

The following files contain implemented gameplay logic that is not yet integrated. They are kept as `.rs.disabled` for future reference — rename back to `.rs` and declare the module when ready to integrate.

| File | Lines | Description |
|------|-------|-------------|
| `crates/core/src/_artifacts.rs.disabled` | 206 | 7 artifact types (NanoForge, PhaseShifter, StasisField, etc.) with charges, activation, message logging |
| `crates/core/src/_gene_splicing.rs.disabled` | 557 | Full recipe system from `gene_splicing.toml`, success/failure rolls, humanity cost, mutation tags, `execute_splice()` |
| `crates/core/src/_genetics.rs.disabled` | 211 | DNA/Allele component with 16 gene traits (Strength, Speed, NightVision, etc.), splicing, mutation |
| `crates/core/src/_game_endings.rs.disabled` | 137 | Ending tracker: Ascension (5 artifacts) and Transformation endings, uses TurnCounter |
| `crates/world/src/_spatial.rs.disabled` | 262 | SpatialHashGrid for O(1) entity queries by position range, performance optimization |

---

## 6. App Architecture Reference

### State Machine
```
Boot → MainMenu → CreateWorld → (WorldGenProgress) → NewCharacter → InWorld
InWorld ↔ PauseMenu
InWorld → Dead → MainMenu
```
Note: `WorldGenProgress` defined but has no systems — skipped entirely.

### Plugin Registration Order
`src/main.rs:16` → `CarapacePlugin` (`src/plugin.rs:9`):
1. `UiPlugin` — menu screens, input
2. `RenderPlugin` — sprites, camera, HUD
3. `GamePlugin` — world gen, game loop, combat, encounters
4. `InteractPlugin` — talk, craft, quest, consume, throw, overview

### GameWorld Pattern
- `GameWorld(bevy_ecs::World)` wraps game ECS separately from Bevy's render world
- Accessed via `&mut game_world.0` with short-lived borrows
- All game entities live in the sub-world; Bevy world handles rendering

### Key Bindings (active)
| Key | Action | Handler |
|-----|--------|---------|
| WASD/Arrows | Move | `game/mod.rs` |
| E | Interact | `game/mod.rs` → `interact/mod.rs` |
| G | Pick up item | `game/mod.rs` |
| X | Examine mode | `game/mod.rs` |
| I | Inventory | `game/mod.rs` |
| C | Craft | `game/mod.rs` |
| U | Consume | `game/mod.rs` |
| T | Throw | `game/mod.rs` |
| M | World overview | `game/mod.rs` |
| > (Period) | Enter location | `game/mod.rs` |
| ESC | Pause menu | `ui/input.rs` |
| Enter | Confirm/accept | Context-dependent |
| 1-9 | Select option | Context-dependent |

### All config files use `include_str!()` at compile time
### `GameWorld.0` = `bevy_ecs::World`; accessed via `&mut game_world.0`
