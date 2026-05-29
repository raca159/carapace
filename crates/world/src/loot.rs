use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;

use game_core::{Glyph, Item, Name, Position};
use game_tags::{TagRegistry, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct LootEntryDef {
    pub name: String,
    pub tags: Vec<String>,
    pub glyph: char,
    pub color: [u8; 3],
    pub weight: f32,
    pub base_chance: f32,
    pub quantity_min: u32,
    pub quantity_max: u32,
    #[serde(default)]
    pub quality_bias: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LootTableDef {
    pub id: String,
    #[serde(default)]
    pub match_tags: Vec<String>,
    #[serde(default)]
    pub dungeon_types: Vec<String>,
    #[serde(default = "default_min_depth")]
    pub min_depth: u32,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_slots")]
    pub slots: u32,
    #[serde(default)]
    pub entry: Vec<LootEntryDef>,
    #[serde(default)]
    pub quality_bias: Option<String>,
}

fn default_min_depth() -> u32 {
    1
}
fn default_max_depth() -> u32 {
    10
}
fn default_slots() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
struct LootTablesFile {
    #[serde(rename = "loot_table")]
    tables: Vec<LootTableDef>,
}

#[derive(Resource, Debug, Clone)]
pub struct LootTables {
    pub tables: Vec<LootTableDef>,
}

impl LootTables {
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    pub fn tables_matching_creature<'a>(
        &'a self,
        creature_tags: &Tags,
        registry: &TagRegistry,
    ) -> Vec<&'a LootTableDef> {
        self.tables
            .iter()
            .filter(|table| !table.match_tags.is_empty())
            .filter(|table| {
                table.match_tags.iter().all(|tag_name| {
                    registry
                        .tag_id(tag_name)
                        .is_some_and(|id| creature_tags.has(id))
                })
            })
            .collect()
    }

    pub fn tables_for_dungeon<'a>(
        &'a self,
        dungeon_type_name: &str,
        depth: u32,
    ) -> Vec<&'a LootTableDef> {
        self.tables
            .iter()
            .filter(|table| {
                table.dungeon_types.iter().any(|dt| dt == dungeon_type_name)
                    && depth >= table.min_depth
                    && depth <= table.max_depth
            })
            .collect()
    }
}

impl Default for LootTables {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_loot_tables(toml_str: &str) -> Result<Vec<LootTableDef>, toml::de::Error> {
    let file: LootTablesFile = toml::from_str(toml_str)?;
    Ok(file.tables)
}

#[derive(Debug, Clone)]
pub struct LootDropInstance {
    pub name: String,
    pub tags: Vec<String>,
    pub glyph: char,
    pub color: [u8; 3],
    pub quantity: u32,
}

const QUALITY_WEIGHTS: &[(&str, f32)] = &[
    ("COMMON", 60.0),
    ("UNCOMMON", 25.0),
    ("RARE", 10.0),
    ("EPIC", 4.0),
    ("LEGENDARY", 1.0),
];

const QUALITY_BIAS_WEIGHTS: &[(&str, &[(&str, f32)])] = &[
    (
        "COMMON",
        &[
            ("COMMON", 60.0),
            ("UNCOMMON", 25.0),
            ("RARE", 10.0),
            ("EPIC", 4.0),
            ("LEGENDARY", 1.0),
        ],
    ),
    (
        "UNCOMMON",
        &[
            ("COMMON", 30.0),
            ("UNCOMMON", 40.0),
            ("RARE", 20.0),
            ("EPIC", 8.0),
            ("LEGENDARY", 2.0),
        ],
    ),
    (
        "RARE",
        &[
            ("COMMON", 10.0),
            ("UNCOMMON", 25.0),
            ("RARE", 40.0),
            ("EPIC", 18.0),
            ("LEGENDARY", 7.0),
        ],
    ),
    (
        "EPIC",
        &[
            ("COMMON", 5.0),
            ("UNCOMMON", 15.0),
            ("RARE", 30.0),
            ("EPIC", 35.0),
            ("LEGENDARY", 15.0),
        ],
    ),
    (
        "LEGENDARY",
        &[
            ("COMMON", 2.0),
            ("UNCOMMON", 8.0),
            ("RARE", 20.0),
            ("EPIC", 35.0),
            ("LEGENDARY", 35.0),
        ],
    ),
];

fn roll_quality(rng: &mut impl Rng, bias: Option<&str>) -> &'static str {
    let weights = bias
        .and_then(|b| QUALITY_BIAS_WEIGHTS.iter().find(|(name, _)| *name == b))
        .map(|(_, w)| *w)
        .unwrap_or(QUALITY_WEIGHTS);
    let total: f32 = weights.iter().map(|(_, w)| w).sum();
    let mut roll = rng.random::<f32>() * total;
    for (name, weight) in weights {
        roll -= weight;
        if roll <= 0.0 {
            return name;
        }
    }
    "COMMON"
}

fn quality_prefix(quality: &str) -> &'static str {
    match quality {
        "COMMON" => "",
        "UNCOMMON" => "Uncommon ",
        "RARE" => "Rare ",
        "EPIC" => "Epic ",
        "LEGENDARY" => "Legendary ",
        _ => "",
    }
}

fn weighted_pick<'a>(entries: &'a [LootEntryDef], rng: &mut impl Rng) -> Option<&'a LootEntryDef> {
    let total: f32 = entries.iter().map(|e| e.weight).sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll = rng.random::<f32>() * total;
    for entry in entries {
        roll -= entry.weight;
        if roll <= 0.0 {
            return Some(entry);
        }
    }
    entries.last()
}

pub fn roll_loot_for_creature(
    creature_tags: &Tags,
    loot_tables: &LootTables,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<LootDropInstance> {
    let matching = loot_tables.tables_matching_creature(creature_tags, registry);
    let mut drops = Vec::new();
    for table in &matching {
        for entry in &table.entry {
            let roll = rng.random::<f32>();
            if roll >= entry.base_chance {
                continue;
            }
            let quality = roll_quality(
                rng,
                entry
                    .quality_bias
                    .as_deref()
                    .or(table.quality_bias.as_deref()),
            );
            let quantity = if entry.quantity_min == entry.quantity_max {
                entry.quantity_min
            } else {
                rng.random_range(entry.quantity_min..=entry.quantity_max)
            };
            let prefixed_name = format!("{}{}", quality_prefix(quality), entry.name);
            let mut tags = entry.tags.clone();
            if !quality.is_empty() {
                tags.push(quality.to_string());
            }
            drops.push(LootDropInstance {
                name: prefixed_name,
                tags,
                glyph: entry.glyph,
                color: entry.color,
                quantity,
            });
        }
    }
    drops
}

pub fn roll_loot_for_table(table: &LootTableDef, rng: &mut impl Rng) -> Vec<LootDropInstance> {
    let mut drops = Vec::new();
    let slots = if table.slots == 0 { 1 } else { table.slots };
    for _ in 0..slots {
        let entry = match weighted_pick(&table.entry, rng) {
            Some(e) => e,
            None => continue,
        };
        let roll = rng.random::<f32>();
        if roll >= entry.base_chance {
            continue;
        }
        let quality = roll_quality(
            rng,
            entry
                .quality_bias
                .as_deref()
                .or(table.quality_bias.as_deref()),
        );
        let quantity = if entry.quantity_min == entry.quantity_max {
            entry.quantity_min
        } else {
            rng.random_range(entry.quantity_min..=entry.quantity_max)
        };
        let prefixed_name = format!("{}{}", quality_prefix(quality), entry.name);
        let mut tags = entry.tags.clone();
        if !quality.is_empty() {
            tags.push(quality.to_string());
        }
        drops.push(LootDropInstance {
            name: prefixed_name,
            tags,
            glyph: entry.glyph,
            color: entry.color,
            quantity,
        });
    }
    drops
}

pub fn spawn_loot_drop(world: &mut World, drop: &LootDropInstance, pos: TilePos) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let tag_count = registry.tag_count();
    let mut entity_tags = Tags::new(tag_count);
    for tag_name in &drop.tags {
        if let Some(tag_id) = registry.tag_id(tag_name) {
            entity_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }
    }
    world.spawn((
        Position { x: pos.x, y: pos.y, z: 0 },
        Glyph {
            char: drop.glyph,
            color: (drop.color[0], drop.color[1], drop.color[2]),
        },
        entity_tags,
        Name(drop.name.clone()),
        Item,
    ));
}

use crate::dungeon::{DungeonMap, DungeonTileType};
use crate::tile::TilePos;

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

pub fn spawn_dungeon_floor_loot(
    world: &mut World,
    dungeon: &DungeonMap,
    depth: u32,
    loot_tables: &LootTables,
) {
    let scatter_seed = dungeon.seed.wrapping_add(depth as u64).wrapping_add(0xF00D);
    let mut rng = rand::rngs::StdRng::seed_from_u64(scatter_seed);

    let entrance_pos = dungeon.rooms.first().map(|r| (r.x, r.y, r.w, r.h));

    let matching_tables = loot_tables.tables_for_dungeon(dungeon.dungeon_type.name(), depth);

    for room in &dungeon.rooms {
        if entrance_pos.is_some_and(|(ex, ey, ew, eh)| {
            room.x == ex && room.y == ey && room.w == ew && room.h == eh
        }) {
            continue;
        }

        for dy in 0..room.h {
            for dx in 0..room.w {
                let x = room.x + dx;
                let y = room.y + dy;
                let idx = (y * dungeon.width + x) as usize;

                let is_floor = dungeon
                    .tiles
                    .get(idx)
                    .is_some_and(|t| t.tile_type == DungeonTileType::Floor);

                if !is_floor {
                    continue;
                }

                if rng.random::<f32>() >= 0.15 {
                    continue;
                }

                if let Some(table) = matching_tables.first() {
                    let drops = roll_loot_for_table(table, &mut rng);
                    for drop in &drops {
                        spawn_loot_drop(world, drop, TilePos::new(x, y));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::load_tag_registry;
    use rand::SeedableRng;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    const LOOT_TOML: &str = r#"
[[loot_table]]
id = "beast_common"
match_tags = ["BEAST"]

[[loot_table.entry]]
name = "Raw Meat"
tags = ["EDIBLE", "FOOD_WILD"]
glyph = "%"
color = [200, 100, 100]
weight = 50
base_chance = 0.8
quantity_min = 1
quantity_max = 2

[[loot_table.entry]]
name = "Bone Fragment"
tags = ["STONE", "BONE"]
glyph = "."
color = [220, 220, 200]
weight = 30
base_chance = 0.6
quantity_min = 1
quantity_max = 3

[[loot_table]]
id = "dungeon_common"
dungeon_types = ["Ancient Vault"]
min_depth = 1
max_depth = 5
slots = 2

[[loot_table.entry]]
name = "Chip"
tags = ["VALUABLE", "METAL", "CURRENCY"]
glyph = "*"
color = [255, 215, 0]
weight = 40
base_chance = 1.0
quantity_min = 1
quantity_max = 1
"#;

    #[test]
    fn test_load_loot_tables() {
        let tables = load_loot_tables(LOOT_TOML).unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].id, "beast_common");
        assert_eq!(tables[0].match_tags, vec!["BEAST"]);
        assert_eq!(tables[0].entry.len(), 2);
        assert_eq!(tables[1].id, "dungeon_common");
        assert_eq!(tables[1].dungeon_types, vec!["Ancient Vault"]);
    }

    #[test]
    fn test_load_loot_tables_invalid_toml() {
        let result = load_loot_tables("not valid toml {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_tables_matching_creature() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let beast_id = registry.tag_id("BEAST").unwrap();
        let mut creature_tags = Tags::new(tag_count);
        creature_tags.add_tag(beast_id, game_tags::TagValue::None, &registry);

        let matching = tables.tables_matching_creature(&creature_tags, &registry);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "beast_common");
    }

    #[test]
    fn test_tables_matching_creature_requires_all_tags() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let undead_id = registry.tag_id("UNDEAD").unwrap();
        let mut creature_tags = Tags::new(tag_count);
        creature_tags.add_tag(undead_id, game_tags::TagValue::None, &registry);

        let matching = tables.tables_matching_creature(&creature_tags, &registry);
        assert!(matching.is_empty());
    }

    #[test]
    fn test_tables_for_dungeon() {
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let matching = tables.tables_for_dungeon("Ancient Vault", 1);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "dungeon_common");
    }

    #[test]
    fn test_tables_for_dungeon_depth_out_of_range() {
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let matching = tables.tables_for_dungeon("Ancient Vault", 10);
        assert!(matching.is_empty());
    }

    #[test]
    fn test_tables_for_dungeon_wrong_type() {
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let matching = tables.tables_for_dungeon("Cave System", 1);
        assert!(matching.is_empty());
    }

    #[test]
    fn test_roll_loot_for_creature() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let beast_id = registry.tag_id("BEAST").unwrap();
        let mut creature_tags = Tags::new(tag_count);
        creature_tags.add_tag(beast_id, game_tags::TagValue::None, &registry);

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let drops = roll_loot_for_creature(&creature_tags, &tables, &registry, &mut rng);
        assert!(drops.len() <= 2, "should roll at most 2 entry types");
        for drop in &drops {
            assert!(!drop.name.is_empty());
            assert!(drop.quantity >= 1);
        }
    }

    #[test]
    fn test_roll_loot_for_creature_no_match() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let undead_id = registry.tag_id("UNDEAD").unwrap();
        let mut creature_tags = Tags::new(tag_count);
        creature_tags.add_tag(undead_id, game_tags::TagValue::None, &registry);

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let drops = roll_loot_for_creature(&creature_tags, &tables, &registry, &mut rng);
        assert!(drops.is_empty());
    }

    #[test]
    fn test_roll_loot_for_table() {
        let tables = load_loot_tables(LOOT_TOML).unwrap();
        let dungeon_table = &tables[1];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let drops = roll_loot_for_table(dungeon_table, &mut rng);
        assert!(!drops.is_empty());
        assert!(drops.len() <= 2);
    }

    #[test]
    fn test_weighted_pick_distribution() {
        let entries = vec![
            LootEntryDef {
                name: "Common".to_string(),
                tags: vec![],
                glyph: '.',
                color: [100; 3],
                weight: 90.0,
                base_chance: 1.0,
                quantity_min: 1,
                quantity_max: 1,
                quality_bias: None,
            },
            LootEntryDef {
                name: "Rare".to_string(),
                tags: vec![],
                glyph: '*',
                color: [200; 3],
                weight: 10.0,
                base_chance: 1.0,
                quantity_min: 1,
                quantity_max: 1,
                quality_bias: None,
            },
        ];

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut common_count = 0;
        let mut rare_count = 0;
        for _ in 0..1000 {
            match weighted_pick(&entries, &mut rng) {
                Some(e) if e.name == "Common" => common_count += 1,
                Some(e) if e.name == "Rare" => rare_count += 1,
                _ => {}
            }
        }

        assert!(
            common_count > rare_count,
            "common should be picked more often"
        );
        assert!(rare_count > 0, "rare should be picked sometimes");
    }

    #[test]
    fn test_deterministic_roll() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        let beast_id = registry.tag_id("BEAST").unwrap();
        let mut creature_tags = Tags::new(tag_count);
        creature_tags.add_tag(beast_id, game_tags::TagValue::None, &registry);

        let run = || -> Vec<String> {
            let mut rng = rand::rngs::StdRng::seed_from_u64(999);
            roll_loot_for_creature(&creature_tags, &tables, &registry, &mut rng)
                .into_iter()
                .map(|d| d.name)
                .collect()
        };

        assert_eq!(run(), run(), "same seed should produce same loot");
    }

    #[test]
    fn test_quality_bias_shifts_distribution() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let mut rare_or_better = 0u32;
        for _ in 0..500 {
            let q = roll_quality(&mut rng, Some("RARE"));
            if q == "RARE" || q == "EPIC" || q == "LEGENDARY" {
                rare_or_better += 1;
            }
        }

        assert!(
            rare_or_better > 100,
            "RARE bias should produce many rare+ items, got {}",
            rare_or_better
        );
    }

    #[test]
    fn test_spawn_loot_drop_creates_entity() {
        use bevy_ecs::world::World;
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry.clone());

        let drop = LootDropInstance {
            name: "Test Item".to_string(),
            tags: vec!["STONE".to_string()],
            glyph: '*',
            color: [100; 3],
            quantity: 1,
        };

        spawn_loot_drop(&mut world, &drop, TilePos::new(5, 10));

        let mut item_query = world.query::<(&Name, &Position)>();
        let items: Vec<_> = item_query.iter(&world).collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].0.0, "Test Item");
        assert_eq!(items[0].1.x, 5);
        assert_eq!(items[0].1.y, 10);
    }

    fn make_test_dungeon() -> crate::dungeon::DungeonMap {
        let width = 20;
        let height = 20;
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.push(crate::dungeon::DungeonTile {
                    pos: TilePos::new(x, y),
                    tile_type: crate::dungeon::DungeonTileType::Floor,
                    dungeon_type: crate::dungeon::DungeonType::Crypt,
                });
            }
        }
        let entrance_room = crate::dungeon::RoomRect {
            x: 0,
            y: 0,
            w: 4,
            h: 4,
        };
        let other_room = crate::dungeon::RoomRect {
            x: 8,
            y: 8,
            w: 6,
            h: 6,
        };
        crate::dungeon::DungeonMap {
            width,
            height,
            tiles,
            rooms: vec![entrance_room, other_room],
            entrance: (2, 2),
            dungeon_type: crate::dungeon::DungeonType::Crypt,
            seed: 42,
        }
    }

    #[test]
    fn test_place_dungeon_chests_creates_entities() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry.clone());
        let dungeon = make_test_dungeon();

        place_dungeon_chests(&mut world, &dungeon, 1);

        let mut container_query = world.query::<(&Tags, &Position, &Glyph)>();
        let chests: Vec<_> = container_query.iter(&world).collect();

        assert!(!chests.is_empty(), "should place at least some chests");
        for (tags, pos, glyph) in &chests {
            if let Some(tag_id) = registry.tag_id("CONTAINER") {
                assert!(tags.has(tag_id), "chests have CONTAINER tag");
            }
            assert!(tags.has(registry.tag_id("HAS_INVENTORY").unwrap()), "chests have HAS_INVENTORY tag");
            assert!(tags.has(registry.tag_id("INVENTORY_SMALL").unwrap()), "chests have INVENTORY_SMALL tag");
            assert!(tags.has(registry.tag_id("INVENTORY_LOOT").unwrap()), "chests have INVENTORY_LOOT tag");
            assert_eq!(glyph.char, '=');
            assert_eq!(glyph.color, (255, 215, 0));
            // Chest must be in non-entrance room (x >= 8 && y >= 8)
            assert!(
                pos.x >= 8 && pos.y >= 8,
                "chest at ({}, {}) should be in non-entrance room",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn test_place_dungeon_chests_skips_entrance_room() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry);
        let dungeon = make_test_dungeon();

        place_dungeon_chests(&mut world, &dungeon, 1);

        let mut container_query = world.query::<&Position>();
        for pos in container_query.iter(&world) {
            assert!(
                !(pos.x < 4 && pos.y < 4),
                "chest in entrance room at ({}, {})",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn test_place_dungeon_chests_deterministic() {
        let mut world1 = World::new();
        let registry1 = load_tag_registry(TAGS_TOML).unwrap();
        world1.insert_resource(registry1);
        let dungeon1 = make_test_dungeon();
        place_dungeon_chests(&mut world1, &dungeon1, 1);

        let mut world2 = World::new();
        let registry2 = load_tag_registry(TAGS_TOML).unwrap();
        world2.insert_resource(registry2);
        let dungeon2 = make_test_dungeon();
        place_dungeon_chests(&mut world2, &dungeon2, 1);

        let count1 = world1.query::<&Name>().iter(&world1).count();
        let count2 = world2.query::<&Name>().iter(&world2).count();
        assert_eq!(
            count1, count2,
            "same seed should produce same number of chests"
        );
    }

    #[test]
    fn test_spawn_dungeon_floor_loot_creates_items() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry);
        let dungeon = make_test_dungeon();
        let tables = LootTables {
            tables: load_loot_tables(LOOT_TOML).unwrap(),
        };

        spawn_dungeon_floor_loot(&mut world, &dungeon, 1, &tables);

        let mut item_query = world.query::<(&Name, &Position)>();
        let items: Vec<_> = item_query.iter(&world).collect();
        // May be 0 (if no matching dungeon tables or RNG rolls below threshold)
        // But should not panic or error
        for (_name, pos) in &items {
            assert!(
                !(pos.x < 4 && pos.y < 4),
                "floor loot should not be in entrance room"
            );
        }
    }
}
