use bevy_ecs::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::{Creature, Glyph, Health, Item, Name, Player, Position};
use game_tags::Tags;

#[derive(Debug, Clone, Deserialize)]
pub struct EncounterSpawn {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub tags: Vec<String>,
    pub health: u32,
    #[serde(default = "default_one")]
    pub count: u32,
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EncounterLoot {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub tags: Vec<String>,
    #[serde(default = "default_one")]
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EncounterEffect {
    #[serde(default)]
    pub damage: u32,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EncounterDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default = "default_min_distance")]
    pub min_distance: u32,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub spawn: Vec<EncounterSpawn>,
    #[serde(default)]
    pub loot: Vec<EncounterLoot>,
    #[serde(default)]
    pub effect: Vec<EncounterEffect>,
}

#[derive(Debug, Clone, Deserialize)]
struct EncountersFile {
    #[serde(rename = "encounter")]
    encounters: Vec<EncounterDef>,
}

#[derive(Resource, Debug, Clone)]
pub struct Encounters {
    pub defs: Vec<EncounterDef>,
}

fn default_one() -> u32 { 1 }
fn default_weight() -> u32 { 10 }
fn default_min_distance() -> u32 { 5 }

impl Encounters {
    pub fn new(defs: Vec<EncounterDef>) -> Self {
        Self { defs }
    }

    pub fn def_by_id(&self, id: &str) -> Option<&EncounterDef> {
        self.defs.iter().find(|d| d.id == id)
    }

    pub fn total_weight(&self) -> u32 {
        self.defs.iter().map(|d| d.weight).sum()
    }

    pub fn random_encounter(&self, rng: &mut impl Rng) -> Option<&EncounterDef> {
        let total = self.total_weight();
        if total == 0 { return None; }
        let mut roll = rng.random_range(0..total);
        for def in &self.defs {
            if roll < def.weight {
                return Some(def);
            }
            roll -= def.weight;
        }
        self.defs.last()
    }
}

pub fn load_encounters(toml_str: &str) -> Result<Vec<EncounterDef>, toml::de::Error> {
    let file: EncountersFile = toml::from_str(toml_str)?;
    Ok(file.encounters)
}

pub fn roll_encounter(
    world: &World,
    _pos: &Position,
    biome_tags: &[game_tags::TagId],
    near_location: Option<&str>,
    weather_context: Option<&crate::WeatherContext>,
    rng: &mut impl Rng,
) -> Option<String> {
    let encounters = world.get_resource::<Encounters>()?;

    // Compute chance from context
    let mut chance = 0.08f32;

    // Location proximity reduces chance
    if near_location.is_some() { chance = 0.04; }

    // Weather modifiers — use applied_tags (TagId) for efficient lookups
    if let Some(wc) = weather_context {
        let registry = world.get_resource::<game_tags::TagRegistry>();
        let dark_id = registry.and_then(|r| r.tag_id("DARK"));
        let stormy_id = registry.and_then(|r| r.tag_id("STORMY"));
        let reduced_id = registry.and_then(|r| r.tag_id("REDUCED_VISIBILITY"));
        if dark_id.is_some_and(|id| wc.applied_tags.contains(&id)) { chance += 0.05; }
        if stormy_id.is_some_and(|id| wc.applied_tags.contains(&id)) { chance += 0.03; }
        if reduced_id.is_some_and(|id| wc.applied_tags.contains(&id)) { chance += 0.02; }
    }

    // Biome modifiers — hostile biomes increase encounter chance
    if !biome_tags.is_empty() {
        let registry = world.get_resource::<game_tags::TagRegistry>();
        let hostile_ids: Vec<game_tags::TagId> = [
            "BIOME_SWAMP", "BIOME_VOLCANIC", "BIOME_DESERT", "BIOME_TUNDRA", "BIOME_TAIGA",
        ].iter().filter_map(|name| {
            registry.and_then(|r| r.tag_id(name))
        }).collect();
        for id in hostile_ids {
            if biome_tags.contains(&id) {
                chance += 0.03;
            }
        }
    }

    if rng.random::<f32>() > chance { return None; }

    encounters.random_encounter(rng).map(|d| d.id.clone())
}

pub fn spawn_encounter(world: &mut World, encounter_id: &str, pos: Position) {
    let def = {
        let encounters = match world.get_resource::<Encounters>() {
            Some(e) => e,
            None => return,
        };
        match encounters.def_by_id(encounter_id) {
            Some(d) => d.clone(),
            None => return,
        }
    };

    let registry = match world.get_resource::<game_tags::TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    if def.kind == "hazard" {
        for effect in &def.effect {
            if !effect.message.is_empty() {
                if let Some(mut bus) = world.get_resource_mut::<crate::EventBus>() {
                    bus.push(crate::GameEvent::Message(effect.message.clone()));
                }
            }
            if effect.damage > 0 {
                if let Ok(mut hp) = world
                    .query_filtered::<&mut crate::Health, bevy_ecs::query::With<Player>>()
                    .single_mut(world)
                {
                    hp.current = hp.current.saturating_sub(effect.damage);
                    if let Some(mut bus) = world.get_resource_mut::<crate::EventBus>() {
                        bus.push(crate::GameEvent::Message(
                            format!("You take {} damage!", effect.damage)
                        ));
                    }
                }
            }
        }
    }

    for spawn in &def.spawn {
        for i in 0..spawn.count {
            let sx = (pos.x as i32 + spawn.offset_x + i as i32) as u32;
            let sy = (pos.y as i32 + spawn.offset_y + i as i32) as u32;
            let mut tags = Tags::new(registry.tag_count());
            for t in &spawn.tags {
                if let Some(tid) = registry.tag_id(t) {
                    tags.add_tag(tid, game_tags::TagValue::None, &registry);
                }
            }
            world.spawn((
                Creature,
                Name(spawn.name.clone()),
                Glyph { char: spawn.glyph, color: (spawn.color[0], spawn.color[1], spawn.color[2]) },
                Position { x: sx, y: sy, z: 0 },
                tags,
                Health { current: spawn.health, max: spawn.health },
            ));
        }
    }

    for loot in &def.loot {
        for _ in 0..loot.count {
            let mut tags = Tags::new(registry.tag_count());
            for t in &loot.tags {
                if let Some(tid) = registry.tag_id(t) {
                    tags.add_tag(tid, game_tags::TagValue::None, &registry);
                }
            }
            world.spawn((
                Item,
                Name(loot.name.clone()),
                Glyph { char: loot.glyph, color: (loot.color[0], loot.color[1], loot.color[2]) },
                Position { x: pos.x, y: pos.y, z: 0 },
                tags,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENCOUNTERS_TOML: &str = r#"
[[encounter]]
id = "merchant"
name = "Traveling Merchant"
description = "A weathered merchant with wares."
weight = 8
min_distance = 6

[[encounter.spawn]]
name = "Traveling Merchant"
glyph = "M"
color = [255, 200, 50]
tags = ["HUMAN", "PEACEFUL"]
health = 30

[[encounter.loot]]
name = "Rations"
glyph = "%"
color = [180, 140, 80]
tags = ["EDIBLE"]
count = 2

[[encounter]]
id = "patrol_guards"
name = "Patrol Guards"
weight = 12

[[encounter.spawn]]
name = "Patrol Guard"
glyph = "G"
color = [50, 100, 200]
tags = ["HUMAN", "AGGRESSIVE"]
health = 25
count = 3

[[encounter]]
id = "ambush"
name = "Bandit Ambush"
weight = 15

[[encounter.spawn]]
name = "Bandit"
glyph = "b"
color = [200, 50, 50]
tags = ["HUMAN", "AGGRESSIVE"]
health = 20
count = 4

[[encounter]]
id = "wild_beasts"
name = "Wild Beasts"
weight = 18

[[encounter.spawn]]
name = "Wolf"
glyph = "w"
color = [139, 90, 43]
tags = ["BEAST", "AGGRESSIVE"]
health = 20
count = 3

[[encounter]]
id = "toxic_spill"
name = "Toxic Spill"
kind = "hazard"
weight = 10

[[encounter.effect]]
damage = 10
message = "You stumble into a toxic spill! It burns!"

[[encounter]]
id = "abandoned_cache"
name = "Abandoned Cache"
weight = 10

[[encounter.loot]]
name = "Old Map Fragments"
glyph = "□"
color = [180, 150, 80]
tags = ["ARTIFACT"]
count = 1

[[encounter.loot]]
name = "Ancient Coin"
glyph = "*"
color = [220, 190, 60]
tags = ["VALUABLE"]
count = 3

[[encounter]]
id = "hermit"
name = "Wasteland Hermit"
weight = 7

[[encounter.spawn]]
name = "Hermit"
glyph = "h"
color = [150, 150, 150]
tags = ["HUMAN", "PEACEFUL"]
health = 15

[[encounter]]
id = "traveling_healer"
name = "Traveling Healer"
weight = 5

[[encounter.spawn]]
name = "Traveling Healer"
glyph = "H"
color = [50, 255, 50]
tags = ["HUMAN", "PEACEFUL"]
health = 20
"#;

    #[test]
    fn test_load_encounters() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        assert_eq!(defs.len(), 8);
        assert_eq!(defs[0].id, "merchant");
    }

    #[test]
    fn test_total_weight() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        let encounters = Encounters::new(defs);
        assert!(encounters.total_weight() > 0);
    }

    #[test]
    fn test_random_encounter_returns_some() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        let encounters = Encounters::new(defs);
        let mut rng = rand::rng();
        let result = encounters.random_encounter(&mut rng);
        assert!(result.is_some());
    }

    #[test]
    fn test_random_encounter_all_ids() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        let encounters = Encounters::new(defs);
        let mut rng = rand::rng();
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100 {
            if let Some(def) = encounters.random_encounter(&mut rng) {
                seen.insert(def.id.clone());
            }
        }
        assert!(seen.len() >= 6, "expected most encounter ids to appear, got {}", seen.len());
    }

    #[test]
    fn test_def_by_id() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        let encounters = Encounters::new(defs);
        assert!(encounters.def_by_id("merchant").is_some());
        assert!(encounters.def_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_spawn_encounter_merchant_creates_creature() {
        let mut world = World::new();
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        world.insert_resource(Encounters::new(defs));
        world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        world.insert_resource(crate::components::MessageLog::new(10));
        world.insert_resource(crate::turn::TurnCounter(1));
        spawn_encounter(&mut world, "merchant", Position { x: 5, y: 5, z: 0 });
        let count = world.query::<&Creature>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_spawn_encounter_patrol_guards_spawns_three() {
        let mut world = World::new();
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        world.insert_resource(Encounters::new(defs));
        world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        world.insert_resource(crate::components::MessageLog::new(10));
        world.insert_resource(crate::turn::TurnCounter(1));
        spawn_encounter(&mut world, "patrol_guards", Position { x: 0, y: 0, z: 0 });
        let count = world.query::<&Creature>().iter(&world).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_spawn_hazard_no_crash() {
        let mut world = World::new();
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        world.insert_resource(Encounters::new(defs));
        world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        world.insert_resource(crate::components::MessageLog::new(10));
        world.insert_resource(crate::turn::TurnCounter(1));
        spawn_encounter(&mut world, "toxic_spill", Position { x: 0, y: 0, z: 0 });
    }

    #[test]
    fn test_spawn_all_encounter_types() {
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        let ids: Vec<String> = defs.iter().map(|d| d.id.clone()).collect();
        for id in &ids {
            let mut world = World::new();
            world.insert_resource(Encounters::new(defs.clone()));
            world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
            world.insert_resource(crate::components::MessageLog::new(10));
            world.insert_resource(crate::turn::TurnCounter(1));
            spawn_encounter(&mut world, id, Position { x: 0, y: 0, z: 0 });
        }
    }

    #[test]
    fn test_roll_encounter_returns_sometimes() {
        let mut world = World::new();
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        world.insert_resource(Encounters::new(defs));
        let pos = Position { x: 0, y: 0, z: 0 };
        let mut rng = rand::rng();
        let result = roll_encounter(&world, &pos, &[], None, None, &mut rng);
        if let Some(id) = result {
            assert!(!id.is_empty());
        }
    }

    #[test]
    fn test_encounter_without_resource_returns_none() {
        let world = World::new();
        let pos = Position { x: 0, y: 0, z: 0 };
        let mut rng = rand::rng();
        let result = roll_encounter(&world, &pos, &[], None, None, &mut rng);
    assert!(result.is_none());
    }

    #[test]
    fn test_encounter_without_resource_no_crash() {
        let mut world = World::new();
        world.insert_resource(crate::components::MessageLog::new(10));
        world.insert_resource(crate::turn::TurnCounter(1));
        spawn_encounter(&mut world, "merchant", Position { x: 0, y: 0, z: 0 });
    }

    #[test]
    fn test_abandoned_cache_spawns_loot() {
        let mut world = World::new();
        let defs = load_encounters(ENCOUNTERS_TOML).unwrap();
        world.insert_resource(Encounters::new(defs));
        world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        world.insert_resource(crate::components::MessageLog::new(10));
        world.insert_resource(crate::turn::TurnCounter(1));
        spawn_encounter(&mut world, "abandoned_cache", Position { x: 0, y: 0, z: 0 });
        let count = world.query::<&Item>().iter(&world).count();
        assert_eq!(count, 4, "expected 4 items (1 map + 3 coins), got {}", count);
    }
}
