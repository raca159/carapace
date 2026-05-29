# Procedural Generation Cascade Engine Design

> **Status:** Spec — pending review
> **Goal:** A staged world generation system where each phase asks "questions" of the prior phase's output, deriving entities, equipment, inventory, trade, and economy from a single item pool + tag-based reasoning

**Architecture:** 6 ordered stages. Each stage reads the output of all prior stages and makes decisions via tag-based probability matching. No explicit entity-to-item lookup tables — everything is derived from what tags an entity has and what items are available in its region.

**Core principle:** Items are a single tag-defined TOML pool. Entity needs are derived from entity tags. The engine matches needs to available items, then modulates quality and quantity by entity importance × local conditions.

---

## 1. The Stages

```
Stage 0: World Canvas          Perlin noise → elevation, moisture, temperature → biomes
       ↓
Stage 1: Location Placement    Iterative: big (cities, dungeons) → small (POIs, caves).
       │                        Each placement creates a "zone of influence" that biases
       │                        subsequent placements. Output: Location graph with tags,
       │                        faction, size, neighbors, zone boundaries.
       ▼
Stage 2: Economy & Routes      Run only for locations tagged `HAS_ECONOMY` (cities,
       │                        settlements, trading posts — NOT dungeons or monster
       │                        lairs). Supply/demand from biome + faction + location
       │                        type. Trade routes between economies → availability
       │                        and price shifts between connected locations.
       ▼
Stage 3: Entity Roster         TWO BRANCHES:
       │                        A — In a location: entity types from location tags ×
       │                            faction × economy (more merchants in trade hubs,
       │                            more guards in military outposts)
       │                        B — In the wild: entity types from biome tags × density
       │                            × neighbor biomes (current spawn_entities logic)
       ▼
Stage 4: Equipment             Entity tags → slot needs × item pool. Quality modulated
       │                        by entity level × location prosperity (from Stage 2).
       │                        Entities with no location use wild default prosperity.
       ▼
Stage 5: Inventory             Entity tags + location supply + faction economy →
       │                        carried goods + trade goods. Wild entities get minimal
       │                        inventory from biome/natural pools only.
       ▼
Stage 6: Lore/Quests           Entity roster + economy + locations → quests, narrative,
                                history, generated at world gen + runtime.
```

### Key architectural shifts from earlier design:

- **Economy before Entities** — economy context determines what entity types appear (a prosperous trade hub has different inhabitants than a poor military outpost)
- **Iterative location placement** — big locations placed first define zones of influence; smaller locations later must respect those zones
- **Two-branch entity spawning** — "in location" path vs. "wild" path, each with different probability tables and config files
- **`HAS_ECONOMY` tag** — gates economic computation per location; dungeons, caves, and monster lairs skip economy
- **Trade routes** — generated between economy locations, affecting item availability and pricing at runtime

Each stage asks a distinct set of **questions** of the world state so far. The answers are probabilistic, seeded deterministically. The output of every stage is a set of tags and resources that feed the next stage's "questions."

---

## 2. What the Engine Reads

### `items.toml` — single source of truth for every item in the game

```toml
[[item]]
id = "iron_sword"
name = "Iron Sword"
glyph = "/"
color = [180, 180, 190]
base_value = 30
tags = ["METAL", "EQUIP_WEAPON", "MELEE", "HOLDABLE"]
weight = 40
quality_bias = "common"

[[item]]
id = "bone_sword"
name = "Bone Sword"
glyph = "/"
color = [220, 220, 200]
base_value = 15
tags = ["BONE", "EQUIP_WEAPON", "MELEE", "HOLDABLE"]
weight = 35
quality_bias = "common"

[[item]]
id = "leather_armor"
name = "Leather Armor"
glyph = "["
color = [160, 100, 50]
base_value = 25
tags = ["LEATHER", "EQUIP_ARMOR", "WEARABLE", "LIGHT_ARMOR"]
weight = 50
quality_bias = "common"

[[item]]
id = "healing_potion"
name = "Healing Potion"
glyph = "!"
color = [255, 50, 50]
base_value = 40
tags = ["DRINKABLE", "EDIBLE", "HERB_MEDICINAL", "GLASS", "HEALING", "CONSUMABLE"]
weight = 30
quality_bias = "common"

[[item]]
id = "dried_rations"
name = "Dried Rations"
glyph = "%"
color = [180, 140, 80]
base_value = 5
tags = ["FOOD_WILD", "EDIBLE", "CONSUMABLE"]
weight = 60
quality_bias = "common"
# etc...
```

Every item has `tags`, `base_value`, `weight` (probability of being rolled), `quality_bias` (optional override). The `items.toml` is the single pool that equipment generation, inventory generation, loot tables, encounter spawns, and barter all draw from.

### `region_biomes.toml` — what biomes produce

```toml
[[biome]]
tags = ["BIOME_TEMPERATE_FOREST"]
produces = [
  { tag = "WOOD_TIMBER",   weight = 60 },
  { tag = "HERB_MEDICINAL", weight = 30 },
  { tag = "FOOD_WILD",     weight = 40 },
]
```

### `faction_economy.toml` — what factions produce, consume, and have surplus of

```toml
[[faction]]
id = "free_humanity"
name = "Free Humanity"
# What this faction naturally produces in their settlements
produces = [
  { tag = "FOOD_WILD",     weight = 50 },
  { tag = "WOOD_TIMBER",   weight = 40 },
  { tag = "HERB_MEDICINAL", weight = 20 },
]
# What this faction consumes (creates demand)
consumes = [
  { tag = "METAL",    weight = 40 },
  { tag = "BONE",     weight = 10 },
]
```

---

## 3. How Stages Work

### Stage 0: World Canvas (existing — unchanged)

Perlin noise → elevation, moisture, temperature → biome tags per tile. Same as current `generate_world()`.

### Stage 1: Location Generation (enhanced)

**Questions asked of Stage 0:**
- "What biomes are adjacent?" → determines settlement type
- "Are there resources nearby?" → determines economy (mining town, farming village, trade hub)
- "Is there a faction present?" → determines culture/alignment
- "Is this far from existing settlements?" → determines isolation level → entity behavior modifiers

Output: Settlement entities placed on the map with tags: `["SETTLEMENT", "TRADE_HUB", "FREE_HUMANITY", "PROSPEROUS"]`

Example: A grassland tile group near a river with temperate forest → `human_settlement` type. A mountain pass between two plains → `military_outpost`.

### Stage 2: Entity Roster (enhanced from spawn_rules.toml)

**Questions asked of Stage 0+1:**
- "What entities live in this biome?" → filter spawn_rules by biome_tags
- "What entities live in this settlement?" → filter by settlement_type + faction
- "What density makes sense?" → density × location size modifier
- "What faction alignment?" → entity faction from location faction

Output: Entity positions with types, factions, levels. Same as current `spawn_entities()` but with faction and context tags from Stage 1.

### Stage 3: Equipment Loadout (replaces hardcoded spawner.rs logic)

**Questions asked of entity tags:**
- "Does this entity need a weapon?" → tags match `["HUMANOID", "AGGRESSIVE", "TERRITORIAL"]` (any)
- "Does this entity need armor?" → tags match `["HUMANOID", "TERRITORIAL"]`
- "Does this entity need an accessory?" → tags match `["HUMANOID", "CURIOUS", "LEADER"]`

**Questions asked of entity type + region:**
- "What material should preferred?" → tags hierarchy: `HUMANOID→METAL`, `UNDEAD→BONE`, `BEAST→BONE`, else random
- "What quality tier?" → base entity level + location prosperity bonus → quality bias level
- "What's available here?" → if region produces METAL, METAL items are more likely

**Algorithm:**
1. Determine slot needs from entity tags
2. For each needed slot:
   - Query `items.toml` for items with matching slot tag + material preference
   - Roll item by weight
   - Roll quality by (base_weight × quality_bias_from_level_and_prosperity)
   - Apply region material abundance as probability shift
3. Attach items as equipment components + insert into entity `Equipment`

```rust
fn roll_equipment_for_slot(
    slot_tag: TagId,          // EQUIP_WEAPON, EQUIP_ARMOR, EQUIP_ACCESSORY
    entity_tags: &Tags,
    available_items: &[ItemDef],  // pre-filtered from items.toml
    prosperity: f32,              // from location economy (0.5-2.0)
    rng: &mut impl Rng,
    registry: &TagRegistry,
) -> Option<ItemDef> {
    // 1. Filter items by slot tag
    let candidates: Vec<_> = available_items.iter()
        .filter(|item| item.tags.has(slot_tag))
        .collect();
    
    // 2. Apply material preference from entity tags
    let preferred_material = derive_preferred_material(entity_tags, registry);
    let weighted = candidates.iter().map(|item| {
        let weight = item.weight;
        // Boost weight if item matches preferred material
        let pref_boost = if preferred_material.is_some_and(|mat| item.tags.has(mat)) {
            2.0  // double chance
        } else { 1.0 };
        weight * pref_boost
    });
    
    // 3. Roll item by weight
    let selected = weighted_pick(&candidates, &weighted, rng);
    
    // 4. Roll quality modulated by prosperity
    let quality = roll_quality_with_bias(
        selected.quality_bias.as_deref(),
        prosperity_level_to_bias(prosperity),
        rng,
    );
    
    Some(selected.with_quality(quality))
}
```

### Stage 4: Inventory & Trade Goods (NEW)

**Questions asked of entity tags + location:**
- "What would a HUMANOID merchant carry?" → items with CONSUMABLE, VALUABLE tags, filtered by faction economy
- "What would a BEAST have on it?" → items from edible/natural pools
- "What goods are traded here?" → location supply items, faction produces
- "What's trade-only and what's carried for personal use?" → CONSUMABLE items are carried; VALUABLE items are trade-only

**Algorithm:**
1. Determine entity role from tags (MERCHANT, GUARD, HERBIVORE, CARNIVORE, PEACEFUL)
2. Look up faction's `produces` list — these items are abundant
3. Look up region supply — modulate item availability
4. Roll N items from the available pool (N = entity importance)
5. Tag items as `trade_only` based on role (merchants have trade goods; beasts have loot)

```rust
fn roll_inventory(
    entity_tags: &Tags,
    entity_level: u32,
    faction_supply: &[ItemPreference],  // from faction_economy.toml
    region_supply: &[ItemPreference],   // from region_biomes.toml
    item_pool: &[ItemDef],
    rng: &mut impl Rng,
) -> Vec<GeneratedItem> {
    // 1. Determine number of inventory rolls (level-based with role modifier)
    let roll_count = entity_level.min(10) as usize;
    
    // 2. Combine supply pools into weighted item availability
    let supply = merge_supply_pools(faction_supply, region_supply);
    
    // 3. For each roll, pick an item from items.toml whose tags intersect with available supply
    let mut items = Vec::new();
    for _ in 0..roll_count {
        let tag = weighted_pick_supply_tag(&supply, rng);
        let candidates = item_pool.iter()
            .filter(|item| item.tags.has(tag));
        if let Some(item) = weighted_pick_filtered(candidates, rng) {
            items.push(item);
        }
    }
    items
}
```

### Stage 2: Economy & Routes (moved before Entity Roster)

**Questions asked of location type + biome + faction:**
- "Does this location have an economy?" → check `HAS_ECONOMY` tag on location
- "What's abundant here?" → supply from biome + faction produces
- "What's scarce here?" → demand from faction consumes - supply overlap
- "What trade routes exist?" → connect nearby economy locations
- "How does trade affect pricing?" → connected locations share supply surpluses

**Algorithm:**
1. Only run for locations with `HAS_ECONOMY` tag (cities, settlements — NOT dungeons, caves)
2. Compute `supply_score[tag]` = biome produces + faction produces
3. Compute `demand_score[tag]` = faction consumes (filtered by supply overlap)
4. Price multiplier = `demand_score / max(supply_score, 1)` × base_value
5. Trade routes: nearest 2-3 economy locations share supply at reduced multiplier (0.5×)
6. Store per-location in `RegionEconomies` resource for runtime barter

This makes prices emerge naturally: a mining town produces METAL → METAL is cheap there. A farming village far from the mine needs METAL → METAL is expensive there. A trade route between them makes metal slightly cheaper in the village.

### Stage 6: Lore/Quests (mostly exists in core)

- Quests generated from entity roster + economy tensions (e.g., "this town needs METAL" → gather resource quest)
- Lore fragments from location history (which faction founded it, what events happened)
- Narrative events from entity interactions in the region

---

## 4. Implementation Order (each stage independently testable)

| Step | Stage | What | Key Files |
|------|-------|------|-----------|
| 1 | Foundation | `items.toml`, `CascadeEngine`, `region_biomes.toml`, `faction_economy.toml` | `cascade/mod.rs` |
| 2 | Stage 4 | Equipment derivation | `cascade/equipment.rs` |
| 3 | Stage 5 | Inventory derivation | `cascade/inventory.rs` |
| 4 | Stage 2 | Economy pricing | `cascade/economy.rs` |
| 5 | Pipeline fix | Feed economy → equipment quality + inventory; wire `roll_inventory` in spawner; store `RegionEconomies` | `spawner.rs`, `cascade/` |
| 6 | Stage 1 | Iterative location placement (big→small, zones of influence) | `cascade/locations.rs`, new TOML |
| 7 | Stage 3A | Location-specific entity rosters per type/faction/economy | `spawn_rules.toml` location rules |
| 8 | Stage 3B | Wild entity refinement (neighbor biomes, density) | `spawner.rs` |
| 9 | Trade routes | Economy→economy connections, runtime price shifts | `cascade/trade.rs` |
| 10 | Barter | Wire cascade inventory + economy into talk hub | `talk.rs`, `barter.rs` |
| 11 | Encounters | Wire cascade into encounter spawns | `encounters.rs` |
| 12 | Dungeon Entry | Wire cascade depth-based quality into WFC dungeons | `dungeon.rs`, `wfc.rs` |
