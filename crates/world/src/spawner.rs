use bevy_ecs::prelude::World;
use bevy_ecs::entity::Entity;
use rand::SeedableRng;
use rand::Rng;
use serde::Deserialize;

use game_core::{BehaviorState, Creature, Equipment, Glyph, Health, Item, Name, PersonalityScores, Position, QuestBoard, QuestGiver, NpcEmotionalState, WeatherSensitive, tags_from_personality};
use game_tags::{TagRegistry, TagValue, Tags};

use crate::map::WorldMap;
use crate::tile::TilePos;

#[derive(Debug, Clone, Deserialize)]
pub struct SpawnRule {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub tags: Vec<String>,
    pub biome_tags: Vec<String>,
    pub density: f32,
    pub min_distance_from_player: u32,
    #[serde(default)]
    pub faction: Option<String>,
    #[serde(default)]
    pub is_item: bool,
    #[serde(default)]
    pub quest_giver: bool,
    #[serde(default)]
    pub quest_board: bool,
    #[serde(default)]
    pub equip_chance: f32,
    #[serde(default)]
    pub equip_tier: Option<String>,
    #[serde(default)]
    pub location_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SpawnRulesFile {
    #[serde(rename = "spawn_rule")]
    rules: Vec<SpawnRule>,
}

pub fn load_spawn_rules(toml_str: &str) -> Result<Vec<SpawnRule>, toml::de::Error> {
    let file: SpawnRulesFile = toml::from_str(toml_str)?;
    Ok(file.rules)
}

fn derive_health(tags: &[String]) -> u32 {
    for tag in tags {
        match tag.as_str() {
            "TINY" => return 10,
            "SMALL" => return 25,
            "MEDIUM" => return 50,
            "LARGE" => return 100,
            "HUGE" => return 200,
            _ => continue,
        }
    }
    50
}

fn is_creature(tags: &[String]) -> bool {
    let creature_archetypes = [
        "BEAST", "ELEMENTAL", "INSECT", "UNDEAD", "HUMANOID",
    ];
    tags.iter().any(|t| creature_archetypes.contains(&t.as_str()))
}

const QUALITY_WEIGHTS: &[(&str, f32)] = &[
    ("COMMON", 60.0),
    ("UNCOMMON", 25.0),
    ("RARE", 10.0),
    ("EPIC", 4.0),
    ("LEGENDARY", 1.0),
];

fn roll_quality(rng: &mut impl Rng) -> &'static str {
    let total: f32 = QUALITY_WEIGHTS.iter().map(|(_, w)| w).sum();
    let mut roll = rng.random::<f32>() * total;
    for (name, weight) in QUALITY_WEIGHTS {
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

fn generate_npc_equipment(
    world: &mut World,
    creature_entity: Entity,
    rule_tags: &[String],
    equip_chance: f32,
    equip_tier: Option<&str>,
    registry: &TagRegistry,
    cascade: &crate::cascade::CascadeEngine,
    prosperity: f32,
    rng: &mut impl Rng,
) {
    if equip_chance <= 0.0 || rng.random::<f32>() >= equip_chance {
        return;
    }

    let tag_count = registry.tag_count();
    let mut entity_tags = Tags::new(tag_count);
    for tag_name in rule_tags {
        if let Some(tid) = registry.tag_id(tag_name) {
            entity_tags.add_tag(tid, TagValue::None, registry);
        }
    }

    let entity_level = if let Some(tier) = equip_tier {
        match tier { "UNCOMMON" => 2, "RARE" => 4, "EPIC" => 7, "LEGENDARY" => 10, _ => 1 }
    } else { 1 };

    let rolls = crate::cascade::equipment::generate_entity_equipment(
        &entity_tags, entity_level, prosperity, cascade, registry, rng,
    );
    if rolls.is_empty() { return; }

    let mut equipment = Equipment::default();
    for roll in &rolls {
        let prefix = crate::cascade::equipment::quality_prefix(roll.quality);
        let quality_id = registry.tag_id(roll.quality);
        let mut item_tags = Tags::new(tag_count);

        for tag_name in &roll.item.tags {
            if let Some(tid) = registry.tag_id(tag_name) {
                item_tags.add_tag(tid, TagValue::None, registry);
            }
        }
        if let Some(qid) = quality_id {
            item_tags.add_tag(qid, TagValue::None, registry);
        }

        let item_entity = world.spawn((
            item_tags,
            Name(format!("{}{}", prefix, roll.item.name)),
            Glyph {
                char: roll.item.glyph,
                color: (roll.item.color[0], roll.item.color[1], roll.item.color[2]),
            },
            Item,
        )).id();

        let is_weapon = roll.item.tags.iter().any(|t| t == "EQUIP_WEAPON");
        let is_armor = roll.item.tags.iter().any(|t| t == "EQUIP_ARMOR");

        if is_weapon { equipment.weapon = Some(item_entity); }
        else if is_armor { equipment.armor = Some(item_entity); }
    }

    world.entity_mut(creature_entity).insert(equipment);
}

pub fn spawn_location_entities(
    world: &mut World,
    rules: &[SpawnRule],
    location: &crate::cascade::PlacedLocation,
    player_pos: TilePos,
) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let map = match world.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return,
    };
    let cascade = match world.get_resource::<crate::cascade::CascadeEngine>() {
        Some(c) => c.clone(),
        None => return,
    };
    let region_economies = world.get_resource::<crate::cascade::RegionEconomies>()
        .map(|r| r.economies.clone()).unwrap_or_default();
    let economy = region_economies.get(&location.id);
    let prosperity = economy.map(|e| e.prosperity).unwrap_or(0.0);
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        map.seed.0.wrapping_add(location.id as u64).wrapping_add(0xDEAD)
    );

    for rule in rules {
        if !rule.location_types.is_empty()
            && !rule.location_types.iter().any(|lt| lt == &location.location_type)
        { continue; }

        let creature = is_creature(&rule.tags);
        if !creature { continue; }
        let hp = derive_health(&rule.tags);
        let entity_tag_ids: Vec<game_tags::TagId> = rule.tags.iter()
            .filter_map(|name| registry.tag_id(name))
            .collect();

        let attempts = (location.zone_radius / 3).max(2);
        for _ in 0..attempts {
            let ox = rng.random_range(0..=location.zone_radius * 2);
            let oy = rng.random_range(0..=location.zone_radius * 2);
            let x = (location.x + ox).min(map.width - 1);
            let y = (location.y + oy).min(map.height - 1);

            if rng.random::<f32>() >= rule.density * 2.0 { continue; }

            let dist = (x as i64 - player_pos.x as i64).unsigned_abs()
                + (y as i64 - player_pos.y as i64).unsigned_abs();
            if dist < rule.min_distance_from_player as u64 { continue; }

            let mut entity_tags = Tags::new(registry.tag_count());
            for &tid in &entity_tag_ids {
                entity_tags.add_tag(tid, TagValue::None, &registry);
            }

            let mut personality = PersonalityScores::new_random(&mut rng);
            if let Some(ref faction_name) = rule.faction {
                match faction_name.as_str() {
                    "great_carapace" | "mutated_wildlife" => {
                        personality.aggression = personality.aggression.saturating_add(20).min(100);
                    }
                    "free_humanity" | "the_remnant" => {
                        personality.sociability = personality.sociability.saturating_add(15).min(100);
                    }
                    "sanguine_elite" => {
                        personality.volatility = personality.volatility.saturating_add(15).min(100);
                    }
                    _ => {}
                }
            }
            tags_from_personality(&personality, &mut entity_tags, &registry);

            let pos = Position { x, y, z: 0 };
            let faction_component = rule.faction.as_deref()
                .and_then(|name| {
                    let faction_rels = world.get_resource::<crate::faction::FactionRelationships>().cloned();
                    faction_rels.and_then(|rels| rels.faction_id(name))
                })
                .map(|fid| crate::faction::Faction { faction_id: fid });

            let creature_entity = world.spawn((
                pos,
                Glyph { char: rule.glyph, color: (rule.color[0], rule.color[1], rule.color[2]) },
                Health { current: hp, max: hp },
                entity_tags.clone(),
                Name(rule.name.clone()),
                Creature,
                BehaviorState { home_pos: Some(pos) },
                NpcEmotionalState::default(),
            )).id();

            if let Some(fc) = faction_component {
                world.entity_mut(creature_entity).insert(fc);
            }
            world.entity_mut(creature_entity).insert(personality);

            generate_npc_equipment(
                world, creature_entity, &rule.tags,
                rule.equip_chance, rule.equip_tier.as_deref(),
                &registry, &cascade, prosperity, &mut rng,
            );
        }
    }
}

pub fn spawn_wild_entities(
    world: &mut World,
    rules: &[SpawnRule],
    player_pos: TilePos,
) {
    let registry = world.resource::<TagRegistry>().clone();
    let map = world.resource::<WorldMap>().clone();
    let cascade = match world.get_resource::<crate::cascade::CascadeEngine>() {
        Some(c) => c.clone(),
        None => return,
    };
    let faction_rels = world.get_resource::<crate::faction::FactionRelationships>().cloned();
    let seed = map.seed;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.0.wrapping_add(0xBEEF));
    let mut tile_tags_query = world.query::<&Tags>();

    let biome_tag_ids: Vec<(usize, Vec<game_tags::TagId>)> = rules.iter().enumerate()
        .map(|(i, rule)| {
            let ids: Vec<game_tags::TagId> = rule.biome_tags.iter()
                .filter_map(|name| registry.tag_id(name))
                .collect();
            (i, ids)
        }).collect();

    for (rule_idx, rule) in rules.iter().enumerate() {
        if !rule.location_types.is_empty() { continue; }

        let biome_ids = &biome_tag_ids[rule_idx].1;
        if biome_ids.is_empty() { continue; }

        let entity_tag_ids: Vec<game_tags::TagId> = rule.tags.iter()
            .filter_map(|name| registry.tag_id(name)).collect();
        let creature = is_creature(&rule.tags);
        let is_item_rule = rule.is_item;
        let hp = if creature { Some(derive_health(&rule.tags)) } else { None };
        let faction_id: Option<crate::faction::FactionId> = rule.faction.as_deref()
            .and_then(|name| faction_rels.as_ref().and_then(|rels| rels.faction_id(name)));

        for y in 0..map.height {
            for x in 0..map.width {
                if (x as i64 - player_pos.x as i64).unsigned_abs()
                    + (y as i64 - player_pos.y as i64).unsigned_abs()
                    < rule.min_distance_from_player as u64 { continue; }

                let tile_entity = match map.get(TilePos::new(x, y)) { Some(e) => e, None => continue };
                let tile_tags = match tile_tags_query.get(world, tile_entity) { Ok(t) => t, Err(_) => continue };
                if !tile_tags.has_any(biome_ids) { continue; }
                if rng.random::<f32>() >= rule.density { continue; }

                let mut entity_tags = Tags::new(registry.tag_count());
                for &tid in &entity_tag_ids { entity_tags.add_tag(tid, TagValue::None, &registry); }
                let glyph = Glyph { char: rule.glyph, color: (rule.color[0], rule.color[1], rule.color[2]) };
                let position = Position { x, y, z: 0 };

                if let Some(max_hp) = hp {
                    let mut personality = PersonalityScores::new_random(&mut rng);
                    if let Some(faction_name) = rule.faction.as_deref() {
                        match faction_name {
                            "great_carapace" | "mutated_wildlife" => {
                                personality.aggression = personality.aggression.saturating_add(20).min(100);
                            }
                            "free_humanity" | "the_remnant" => {
                                personality.sociability = personality.sociability.saturating_add(15).min(100);
                            }
                            "sanguine_elite" => {
                                personality.volatility = personality.volatility.saturating_add(15).min(100);
                            }
                            _ => {}
                        }
                    }
                    tags_from_personality(&personality, &mut entity_tags, &registry);

                    let fc = faction_id.map(|fid| crate::faction::Faction { faction_id: fid });
                    let creature_entity = match (&fc, rule.quest_giver) {
                        (Some(f), true) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, WeatherSensitive, BehaviorState { home_pos: Some(position) }, *f, QuestGiver, NpcEmotionalState::default())).id(),
                        (Some(f), false) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, WeatherSensitive, BehaviorState { home_pos: Some(position) }, *f, NpcEmotionalState::default())).id(),
                        (None, true) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, WeatherSensitive, BehaviorState { home_pos: Some(position) }, QuestGiver, NpcEmotionalState::default())).id(),
                        (None, false) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, WeatherSensitive, BehaviorState { home_pos: Some(position) }, NpcEmotionalState::default())).id(),
                    };
                    world.entity_mut(creature_entity).insert(personality);
                    generate_npc_equipment(world, creature_entity, &rule.tags, rule.equip_chance, rule.equip_tier.as_deref(), &registry, &cascade, 0.0, &mut rng);

                } else if rule.quest_board {
                    world.spawn((position, glyph, entity_tags, Name(rule.name.clone()), QuestBoard));
                } else if is_item_rule {
                    let quality = roll_quality(&mut rng);
                    if let Some(qid) = registry.tag_id(quality) { entity_tags.add_tag(qid, TagValue::None, &registry); }
                    world.spawn((position, glyph, entity_tags, Name(format!("{}{}", quality_prefix(quality), rule.name)), Item));
                } else if !creature {
                    world.spawn((position, glyph, entity_tags, Name(rule.name.clone()), Item));
                }
            }
        }
    }
}

pub fn spawn_entities(
    world: &mut World,
    rules: &[SpawnRule],
    player_pos: TilePos,
) {
    let location_map = world.get_resource::<crate::cascade::LocationMap>()
        .cloned().unwrap_or_default();
    for loc in &location_map.locations {
        spawn_location_entities(world, rules, loc, player_pos);
    }

    spawn_wild_entities(world, rules, player_pos);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::WorldSeed;
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");
    const ITEMS_TOML: &str = include_str!("../../../assets/config/items.toml");
    const BIOMES_TOML: &str = include_str!("../../../assets/config/region_biomes.toml");
    const FACTIONS_TOML: &str = include_str!("../../../assets/config/faction_economy.toml");
    const LOCATIONS_TOML: &str = include_str!("../../../assets/config/location_types.toml");

    fn make_rule(
        name: &str,
        glyph: char,
        color: [u8; 3],
        tags: Vec<&str>,
        biome_tags: Vec<&str>,
        density: f32,
        min_dist: u32,
    ) -> SpawnRule {
        SpawnRule {
            name: name.to_string(),
            glyph,
            color,
            tags: tags.into_iter().map(String::from).collect(),
            biome_tags: biome_tags.into_iter().map(String::from).collect(),
            density,
            min_distance_from_player: min_dist,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 0.0,
            equip_tier: None,
            location_types: vec![],
        }
    }

    #[test]
    fn test_load_spawn_rules() {
        let toml_str = r#"
[[spawn_rule]]
name = "Test Creature"
glyph = "t"
color = [255, 0, 0]
tags = ["BEAST", "MEDIUM"]
biome_tags = ["BIOME_DESERT"]
density = 0.5
min_distance_from_player = 5
"#;
        let rules = load_spawn_rules(toml_str).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Test Creature");
        assert_eq!(rules[0].glyph, 't');
        assert_eq!(rules[0].density, 0.5);
    }

    #[test]
    fn test_derive_health() {
        assert_eq!(derive_health(&["TINY".to_string()]), 10);
        assert_eq!(derive_health(&["SMALL".to_string()]), 25);
        assert_eq!(derive_health(&["MEDIUM".to_string()]), 50);
        assert_eq!(derive_health(&["LARGE".to_string()]), 100);
        assert_eq!(derive_health(&["HUGE".to_string()]), 200);
        assert_eq!(derive_health(&["UNKNOWN".to_string()]), 50);
    }

    #[test]
    fn test_is_creature() {
        assert!(is_creature(&["BEAST".to_string()]));
        assert!(is_creature(&["UNDEAD".to_string()]));
        assert!(is_creature(&["ELEMENTAL".to_string()]));
        assert!(!is_creature(&["STONE".to_string()]));
        assert!(!is_creature(&["PLANT".to_string()]));
    }

    #[test]
    fn test_spawn_entities_respects_biome_tags() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let mut tiles = Vec::with_capacity((width * height) as usize);

        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
        let forest_id = registry.tag_id("BIOME_TEMPERATE_FOREST").unwrap();

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: if x < 5 { "desert".to_string() } else { "forest".to_string() },
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                if x < 5 {
                    tags.add_tag(desert_id, TagValue::None, &registry);
                } else {
                    tags.add_tag(forest_id, TagValue::None, &registry);
                }
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let rules = vec![make_rule(
            "Scorpion",
            's',
            [139, 90, 43],
            vec!["BEAST", "SMALL", "CARNIVORE"],
            vec!["BIOME_DESERT"],
            1.0,
            0,
        )];

        let player_pos = TilePos::new(0, 0);
        spawn_entities(&mut world, &rules, player_pos);

        let mut pos_query = world.query::<&Position>();
        let spawned_positions: Vec<(u32, u32)> = pos_query
            .iter(&world)
            .map(|p| (p.x, p.y))
            .collect();

        for (x, _) in &spawned_positions {
            assert!(
                *x < 5,
                "scorpion spawned in forest (x={}), should only be in desert (x<5)",
                x
            );
        }
    }

    #[test]
    fn test_spawn_entities_respects_min_distance() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 20u32;
        let height = 20u32;
        let mut tiles = Vec::with_capacity((width * height) as usize);

        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "desert".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(desert_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let rules = vec![make_rule(
            "Scorpion",
            's',
            [139, 90, 43],
            vec!["BEAST", "SMALL"],
            vec!["BIOME_DESERT"],
            1.0,
            5,
        )];

        let player_pos = TilePos::new(10, 10);
        spawn_entities(&mut world, &rules, player_pos);

        let mut pos_query = world.query::<&Position>();
        for pos in pos_query.iter(&world) {
            let dist = (pos.x as i64 - 10).unsigned_abs() + (pos.y as i64 - 10).unsigned_abs();
            assert!(
                dist >= 5,
                "entity at ({},{}) is within min distance from player",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn test_spawn_entities_deterministic() {
        let make_world = || -> World {
            let mut world = World::new();
            let registry = load_tag_registry(TAGS_TOML).unwrap();
            let tag_count = registry.tag_count();
            world.insert_resource(registry.clone());

            let width = 20u32;
            let height = 20u32;
            let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
            let mut tiles = Vec::with_capacity((width * height) as usize);

            for y in 0..height {
                for x in 0..width {
                    let tile = crate::tile::Tile {
                        pos: TilePos::new(x, y),
                        elevation: 0.5,
                        moisture: 0.5,
                        temperature: 0.5,
                        biome_name: "desert".to_string(),
                        glyph: '.',
                        color: (200, 200, 200),
                    };
                    let mut tags = Tags::new(tag_count);
                    tags.add_tag(desert_id, TagValue::None, &registry);
                    let entity = world.spawn((tile, tags)).id();
                    tiles.push(entity);
                }
            }

            world.insert_resource(WorldMap {
                width,
                height,
                depth: 1,
                current_z: 0,
                seed: WorldSeed::from_value(777),
                tiles,
            });
            world
        };

        let rules = vec![make_rule(
            "Test",
            't',
            [100, 100, 100],
            vec!["BEAST", "SMALL"],
            vec!["BIOME_DESERT"],
            0.5,
            0,
        )];

        let mut world1 = make_world();
        let mut world2 = make_world();

        let player_pos = TilePos::new(0, 0);
        spawn_entities(&mut world1, &rules, player_pos);
        spawn_entities(&mut world2, &rules, player_pos);

        let mut q1 = world1.query::<&Position>();
        let mut q2 = world2.query::<&Position>();

        let p1: Vec<_> = q1.iter(&world1).collect();
        let p2: Vec<_> = q2.iter(&world2).collect();

        assert_eq!(p1.len(), p2.len(), "same seed should produce same entity count");
        for (a, b) in p1.iter().zip(p2.iter()) {
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
        }
    }

    #[test]
    fn test_spawn_entities_creatures_get_health() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "desert".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(desert_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let rules = vec![make_rule(
            "Scorpion",
            's',
            [139, 90, 43],
            vec!["BEAST", "SMALL"],
            vec!["BIOME_DESERT"],
            1.0,
            0,
        )];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut health_query = world.query::<&Health>();
        for health in health_query.iter(&world) {
            assert_eq!(health.current, 25, "SMALL creatures should have 25 hp");
            assert_eq!(health.max, 25);
        }
    }

    #[test]
    fn test_spawn_entities_items_no_health() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let mountain_id = registry.tag_id("BIOME_MOUNTAIN").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "mountain".to_string(),
                    glyph: '^',
                    color: (128, 128, 128),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(mountain_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let cascade = crate::cascade::CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        world.insert_resource(cascade);

        let rules = vec![make_rule(
            "Iron Ore",
            '*',
            [150, 150, 150],
            vec!["STONE", "HARD", "ORE_IRON"],
            vec!["BIOME_MOUNTAIN"],
            1.0,
            0,
        )];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut health_query = world.query::<&Health>();
        assert_eq!(
            health_query.iter(&world).count(),
            0,
            "non-creature entities should not have Health component"
        );

        let mut pos_query = world.query::<&Position>();
        assert!(pos_query.iter(&world).count() > 0, "items should still be spawned");
    }

    #[test]
    fn test_is_item_field_deserialization() {
        let toml_str = r#"
[[spawn_rule]]
name = "Metal Blade"
glyph = "/"
color = [180, 180, 190]
tags = ["METAL", "HARD", "EQUIP_WEAPON"]
biome_tags = ["BIOME_MOUNTAIN"]
density = 0.01
min_distance_from_player = 5
is_item = true

[[spawn_rule]]
name = "Creature"
glyph = "w"
color = [128, 128, 128]
tags = ["BEAST", "MEDIUM"]
biome_tags = ["BIOME_DESERT"]
density = 0.5
min_distance_from_player = 5
"#;
        let rules = load_spawn_rules(toml_str).unwrap();
        assert_eq!(rules.len(), 2);
        assert!(rules[0].is_item, "Metal Blade should have is_item = true");
        assert!(!rules[1].is_item, "Creature should default is_item = false");
    }

    #[test]
    fn test_spawn_entities_is_item_produces_item_entities() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let mountain_id = registry.tag_id("BIOME_MOUNTAIN").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "mountain".to_string(),
                    glyph: '^',
                    color: (128, 128, 128),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(mountain_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let rules = vec![SpawnRule {
            name: "Metal Blade".to_string(),
            glyph: '/',
            color: [180, 180, 190],
            tags: vec!["METAL".into(), "HARD".into(), "EQUIP_WEAPON".into()],
            biome_tags: vec!["BIOME_MOUNTAIN".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: true,
            quest_giver: false,
            quest_board: false,
            equip_chance: 0.0,
            equip_tier: None,
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut name_query = world.query::<&Name>();
        for name in name_query.iter(&world) {
            assert!(
                name.0.contains("Metal Blade"),
                "item name should contain base name, got '{}'",
                name.0
            );
        }
    }

    #[test]
    fn test_aggressive_creature_spawns_with_weapon() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "desert".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(desert_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let cascade = crate::cascade::CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        world.insert_resource(cascade);

        let rules = vec![SpawnRule {
            name: "Wolf".to_string(),
            glyph: 'w',
            color: [128, 128, 128],
            tags: vec!["BEAST".into(), "MEDIUM".into(), "CARNIVORE".into(), "AGGRESSIVE".into()],
            biome_tags: vec!["BIOME_DESERT".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 1.0,
            equip_tier: None,
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut equip_query = world.query::<&Equipment>();
        let equipped: Vec<_> = equip_query.iter(&world).collect();
        assert!(!equipped.is_empty(), "aggressive creature should have Equipment");

        let has_weapons: Vec<_> = equipped.iter().filter(|e| e.weapon.is_some()).collect();
        assert!(!has_weapons.is_empty(), "aggressive creature should have a weapon equipped");
    }

    #[test]
    fn test_territorial_creature_spawns_with_armor() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let forest_id = registry.tag_id("BIOME_TEMPERATE_FOREST").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "forest".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(forest_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let cascade = crate::cascade::CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        world.insert_resource(cascade);

        let rules = vec![SpawnRule {
            name: "Guard".to_string(),
            glyph: 'G',
            color: [0, 200, 100],
            tags: vec!["HUMANOID".into(), "MEDIUM".into(), "TERRITORIAL".into()],
            biome_tags: vec!["BIOME_TEMPERATE_FOREST".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 1.0,
            equip_tier: None,
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut equip_query = world.query::<&Equipment>();
        let equipped: Vec<_> = equip_query.iter(&world).collect();
        assert!(!equipped.is_empty(), "territorial creature should have Equipment");

        let has_armor: Vec<_> = equipped.iter().filter(|e| e.armor.is_some()).collect();
        assert!(!has_armor.is_empty(), "territorial creature should have armor equipped");
    }

    #[test]
    fn test_humanoid_creature_spawns_with_weapon_and_armor() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let forest_id = registry.tag_id("BIOME_TEMPERATE_FOREST").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "forest".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(forest_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let cascade = crate::cascade::CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        world.insert_resource(cascade);

        let rules = vec![SpawnRule {
            name: "Bandit".to_string(),
            glyph: 'b',
            color: [180, 100, 50],
            tags: vec!["HUMANOID".into(), "MEDIUM".into(), "AGGRESSIVE".into()],
            biome_tags: vec!["BIOME_TEMPERATE_FOREST".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 1.0,
            equip_tier: None,
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut equip_query = world.query::<&Equipment>();
        let equipped: Vec<_> = equip_query.iter(&world).collect();
        assert!(!equipped.is_empty(), "humanoid creature should have Equipment");

        let has_both: Vec<_> = equipped.iter()
            .filter(|e| e.weapon.is_some() && e.armor.is_some())
            .collect();
        assert!(!has_both.is_empty(), "humanoid should have both weapon and armor");
    }

    #[test]
    fn test_equip_chance_zero_produces_no_equipment() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "desert".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(desert_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let rules = vec![SpawnRule {
            name: "Wolf".to_string(),
            glyph: 'w',
            color: [128, 128, 128],
            tags: vec!["BEAST".into(), "MEDIUM".into(), "AGGRESSIVE".into()],
            biome_tags: vec!["BIOME_DESERT".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 0.0,
            equip_tier: None,
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut equip_query = world.query::<&Equipment>();
        let equipped_count = equip_query.iter(&world).count();
        assert_eq!(equipped_count, 0, "zero equip_chance should produce no equipment");
    }

    #[test]
    fn test_equip_tier_minimum_quality() {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tag_count = registry.tag_count();
        world.insert_resource(registry.clone());

        let width = 10u32;
        let height = 10u32;
        let desert_id = registry.tag_id("BIOME_DESERT").unwrap();
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "desert".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let mut tags = Tags::new(tag_count);
                tags.add_tag(desert_id, TagValue::None, &registry);
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }

        world.insert_resource(WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        });

        let cascade = crate::cascade::CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        world.insert_resource(cascade);

        let rules = vec![SpawnRule {
            name: "Elite Guard".to_string(),
            glyph: 'G',
            color: [255, 200, 0],
            tags: vec!["HUMANOID".into(), "MEDIUM".into(), "AGGRESSIVE".into(), "TERRITORIAL".into()],
            biome_tags: vec!["BIOME_DESERT".into()],
            density: 1.0,
            min_distance_from_player: 0,
            faction: None,
            is_item: false,
            quest_giver: false,
            quest_board: false,
            equip_chance: 1.0,
            equip_tier: Some("EPIC".to_string()),
            location_types: vec![],
        }];

        spawn_entities(&mut world, &rules, TilePos::new(0, 0));

        let mut equip_query = world.query::<&Equipment>();
        let mut found_weapon = false;
        for equip in equip_query.iter(&world) {
            if let Some(weapon_entity) = equip.weapon {
                found_weapon = true;
                let weapon_tags = world.get::<Tags>(weapon_entity).unwrap();
                let quality_names = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"];
                let qualities: Vec<_> = quality_names.iter()
                    .filter_map(|name| registry.tag_id(name))
                    .filter(|id| weapon_tags.has(*id))
                    .map(|id| registry.tag_by_id(id).name.clone())
                    .collect();
                assert!(
                    !qualities.is_empty(),
                    "weapon should have a quality tag, got: {:?}",
                    qualities
                );
            }
        }
        assert!(found_weapon, "should have at least one creature with a weapon");
    }

    #[test]
    fn test_equip_chance_deserialization() {
        let toml_str = r#"
[[spawn_rule]]
name = "Armed Bandit"
glyph = "b"
color = [180, 100, 50]
tags = ["HUMANOID", "MEDIUM", "AGGRESSIVE"]
biome_tags = ["BIOME_TEMPERATE_FOREST"]
density = 0.02
min_distance_from_player = 10
equip_chance = 0.5
equip_tier = "UNCOMMON"
"#;
        let rules = load_spawn_rules(toml_str).unwrap();
        assert_eq!(rules.len(), 1);
        assert!((rules[0].equip_chance - 0.5).abs() < 0.001);
        assert_eq!(rules[0].equip_tier, Some("UNCOMMON".to_string()));
    }

    #[test]
    fn test_equip_chance_defaults_to_zero() {
        let toml_str = r#"
[[spawn_rule]]
name = "Plain Wolf"
glyph = "w"
color = [128, 128, 128]
tags = ["BEAST", "MEDIUM", "AGGRESSIVE"]
biome_tags = ["BIOME_DESERT"]
density = 0.02
min_distance_from_player = 10
"#;
        let rules = load_spawn_rules(toml_str).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].equip_chance, 0.0);
        assert_eq!(rules[0].equip_tier, None);
    }
}
