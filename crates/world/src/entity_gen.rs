use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;

use game_core::{
    BehaviorState, Creature, Equipment, Glyph, Health, Inventory, Item, Name, Position,
};
use game_tags::{TagRegistry, TagValue, Tags};

use crate::faction::{Faction, FactionRelationships};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatRange {
    #[serde(default)]
    pub health_min: u32,
    #[serde(default)]
    pub health_max: u32,
    #[serde(default)]
    pub inventory_capacity: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentEntry {
    pub slot: String,
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub tags: Vec<String>,
    pub chance: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactionEntry {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntityTemplate {
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub stats: StatRange,
    #[serde(default)]
    pub equipment: Vec<EquipmentEntry>,
    #[serde(default)]
    pub faction: Option<FactionEntry>,
    #[serde(default)]
    pub quest_giver: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct EntityTemplatesFile {
    #[serde(rename = "template")]
    templates: Vec<EntityTemplate>,
}

pub fn load_entity_templates(toml_str: &str) -> Result<Vec<EntityTemplate>, toml::de::Error> {
    let file: EntityTemplatesFile = toml::from_str(toml_str)?;
    Ok(file.templates)
}

pub fn generate_entity(
    world: &mut World,
    template: &EntityTemplate,
    seed: u64,
    pos: Position,
    registry: &TagRegistry,
    faction_rels: Option<&FactionRelationships>,
) -> Entity {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    template.name.hash(&mut hasher);
    seed.hash(&mut hasher);
    let combined_seed = hasher.finish();

    let mut rng = rand::rngs::StdRng::seed_from_u64(combined_seed);

    let mut tags = Tags::new(registry.tag_count());
    for tag_name in &template.tags {
        if let Some(tag_id) = registry.tag_id(tag_name) {
            tags.add_tag(tag_id, TagValue::None, registry);
        }
    }

    use game_core::personality::{PersonalityScores, tags_from_personality};
    let mut scores = PersonalityScores::new_random(&mut rng);
    if let Some(ref faction_entry) = template.faction {
        match faction_entry.name.as_str() {
            "great_carapace" | "mutated_wildlife" => {
                scores.aggression = scores.aggression.saturating_add(20).min(100);
            }
            "free_humanity" | "the_remnant" => {
                scores.sociability = scores.sociability.saturating_add(15).min(100);
            }
            "sanguine_elite" => {
                scores.volatility = scores.volatility.saturating_add(15).min(100);
            }
            _ => {}
        }
    }
    tags_from_personality(&scores, &mut tags, registry);

    let health_val = if template.stats.health_max > template.stats.health_min {
        rng.random_range(template.stats.health_min..=template.stats.health_max)
    } else {
        template.stats.health_min
    };

    let glyph = Glyph {
        char: template.glyph,
        color: (template.color[0], template.color[1], template.color[2]),
    };

    let mut entity_cmds = world.spawn((
        pos,
        glyph,
        Health {
            current: health_val,
            max: health_val,
        },
        tags,
        scores,
        Name(template.name.clone()),
        Creature,
        BehaviorState {
            home_pos: Some(pos),
        },
    ));

    if template.stats.inventory_capacity > 0 {
        entity_cmds.insert(Inventory {
            items: Vec::new(),
            capacity: template.stats.inventory_capacity,
        });
    }

    if let Some(ref faction_entry) = template.faction
        && let Some(rels) = faction_rels
        && let Some(fid) = rels.faction_id(&faction_entry.name)
    {
        entity_cmds.insert(Faction { faction_id: fid });
    }

    if template.quest_giver {
        entity_cmds.insert(game_core::quest::QuestGiver);
    }

    let entity = entity_cmds.id();

    if !template.equipment.is_empty() {
        let mut equipment = Equipment::default();

        for entry in &template.equipment {
            if rng.random::<f32>() > entry.chance {
                continue;
            }

            let mut eq_tags = Tags::new(registry.tag_count());
            for tag_name in &entry.tags {
                if let Some(tag_id) = registry.tag_id(tag_name) {
                    eq_tags.add_tag(tag_id, TagValue::None, registry);
                }
            }

            let eq_glyph = Glyph {
                char: entry.glyph,
                color: (entry.color[0], entry.color[1], entry.color[2]),
            };

            let eq_entity = world
                .spawn((eq_glyph, Name(entry.name.clone()), Item, eq_tags))
                .id();

            match entry.slot.to_lowercase().as_str() {
                "weapon" => equipment.weapon = Some(eq_entity),
                "armor" => equipment.armor = Some(eq_entity),
                "accessory" => equipment.accessory = Some(eq_entity),
                _ => {}
            }
        }

        world.entity_mut(entity).insert(equipment);
    }

    entity
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    const TEMPLATES_TOML: &str = r#"
[[template]]
name = "Test Human"
glyph = "@"
color = [255, 200, 150]
tags = ["HUMANOID", "MEDIUM"]

[template.stats]
health_min = 40
health_max = 60
inventory_capacity = 10

[[template.equipment]]
slot = "weapon"
name = "Metal Blade"
glyph = "/"
color = [180, 180, 180]
tags = ["METAL", "COMMON"]
chance = 1.0

[template.faction]
name = "free_settlements"
"#;

    fn setup_world() -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).expect("tags should load");
        world.insert_resource(registry);
        world
    }

    #[test]
    fn test_load_entity_templates() {
        let templates = load_entity_templates(TEMPLATES_TOML).unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Test Human");
        assert_eq!(templates[0].glyph, '@');
        assert_eq!(templates[0].color, [255, 200, 150]);
        assert_eq!(templates[0].tags, vec!["HUMANOID", "MEDIUM"]);
        assert_eq!(templates[0].stats.health_min, 40);
        assert_eq!(templates[0].stats.health_max, 60);
        assert_eq!(templates[0].stats.inventory_capacity, 10);
        assert_eq!(templates[0].equipment.len(), 1);
        assert_eq!(templates[0].equipment[0].name, "Metal Blade");
        assert_eq!(templates[0].faction.as_ref().unwrap().name, "free_settlements");
        assert!(!templates[0].quest_giver);
    }

    #[test]
    fn test_generate_entity_has_components() {
        let mut world = setup_world();
        let registry = world.resource::<TagRegistry>().clone();
        let templates = load_entity_templates(TEMPLATES_TOML).unwrap();
        let template = &templates[0];

        let pos = Position { x: 5, y: 10, z: 0 };
        let entity = generate_entity(&mut world, template, 42, pos, &registry, None);

        assert!(world.get::<Position>(entity).is_some());
        assert_eq!(world.get::<Position>(entity).unwrap().x, 5);
        assert_eq!(world.get::<Position>(entity).unwrap().y, 10);

        assert!(world.get::<Glyph>(entity).is_some());
        assert_eq!(world.get::<Glyph>(entity).unwrap().char, '@');

        assert!(world.get::<Health>(entity).is_some());
        assert!(world.get::<Creature>(entity).is_some());
        assert!(world.get::<Name>(entity).is_some());
        assert_eq!(world.get::<Name>(entity).unwrap().0, "Test Human");

        assert!(world.get::<Tags>(entity).is_some());
        assert!(world.get::<BehaviorState>(entity).is_some());
        assert!(world.get::<Inventory>(entity).is_some());
        assert!(world.get::<Equipment>(entity).is_some());
    }

    #[test]
    fn test_generate_entity_tags_assigned() {
        let mut world = setup_world();
        let registry = world.resource::<TagRegistry>().clone();
        let templates = load_entity_templates(TEMPLATES_TOML).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };
        let entity = generate_entity(&mut world, template, 42, pos, &registry, None);

        let tags = world.get::<Tags>(entity).unwrap();
        let humanoid = registry.tag_id("HUMANOID").unwrap();
        let medium = registry.tag_id("MEDIUM").unwrap();
        assert!(tags.has(humanoid));
        assert!(tags.has(medium));
    }

    #[test]
    fn test_generate_entity_health_in_range() {
        let mut world = setup_world();
        let registry = world.resource::<TagRegistry>().clone();
        let templates = load_entity_templates(TEMPLATES_TOML).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };

        for seed in 0..50 {
            let entity = generate_entity(&mut world, template, seed, pos, &registry, None);
            let health = world.get::<Health>(entity).unwrap();
            assert!(
                health.max >= template.stats.health_min,
                "seed {}: health {} < min {}",
                seed,
                health.max,
                template.stats.health_min
            );
            assert!(
                health.max <= template.stats.health_max,
                "seed {}: health {} > max {}",
                seed,
                health.max,
                template.stats.health_max
            );
        }
    }

    #[test]
    fn test_generate_entity_quest_giver() {
        let toml = r#"
[[template]]
name = "Quest NPC"
glyph = "Q"
color = [255, 255, 0]
tags = ["HUMANOID", "MEDIUM"]
quest_giver = true
"#;
        let mut world = setup_world();
        let registry = world.resource::<TagRegistry>().clone();
        let templates = load_entity_templates(toml).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };
        let entity = generate_entity(&mut world, template, 42, pos, &registry, None);

        assert!(world.get::<game_core::quest::QuestGiver>(entity).is_some());
    }

    #[test]
    fn test_generate_entity_no_faction_by_default() {
        let toml = r#"
[[template]]
name = "No Faction"
glyph = "N"
color = [128, 128, 128]
tags = ["HUMANOID", "MEDIUM"]
"#;
        let mut world = setup_world();
        let registry = world.resource::<TagRegistry>().clone();
        let templates = load_entity_templates(toml).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };
        let entity = generate_entity(&mut world, template, 42, pos, &registry, None);

        assert!(world.get::<Faction>(entity).is_none());
    }

    #[test]
    fn test_generate_entity_deterministic() {
        let toml = r#"
[[template]]
name = "Deterministic Test"
glyph = "D"
color = [100, 100, 100]
tags = ["HUMANOID", "MEDIUM"]

[template.stats]
health_min = 50
health_max = 100
"#;
        let templates = load_entity_templates(toml).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };

        let mut world1 = setup_world();
        let reg1 = world1.resource::<TagRegistry>().clone();
        let e1 = generate_entity(&mut world1, template, 42, pos, &reg1, None);
        let h1 = world1.get::<Health>(e1).unwrap().max;

        let mut world2 = setup_world();
        let reg2 = world2.resource::<TagRegistry>().clone();
        let e2 = generate_entity(&mut world2, template, 42, pos, &reg2, None);
        let h2 = world2.get::<Health>(e2).unwrap().max;

        assert_eq!(
            h1, h2,
            "same seed on same template should produce same health"
        );
    }

    #[test]
    fn test_generate_entity_different_seeds_different_stats() {
        let toml = r#"
[[template]]
name = "Variety Test"
glyph = "V"
color = [100, 100, 100]
tags = ["HUMANOID", "MEDIUM"]

[template.stats]
health_min = 10
health_max = 200
"#;
        let templates = load_entity_templates(toml).unwrap();
        let template = &templates[0];

        let pos = Position { x: 0, y: 0, z: 0 };

        let mut health_values = Vec::new();
        for seed in 0..20 {
            let mut world = setup_world();
            let reg = world.resource::<TagRegistry>().clone();
            let entity = generate_entity(&mut world, template, seed, pos, &reg, None);
            health_values.push(world.get::<Health>(entity).unwrap().max);
        }

        let unique: std::collections::HashSet<u32> = health_values.iter().copied().collect();
        assert!(
            unique.len() > 1,
            "expected varied health values across seeds, got {} unique",
            unique.len()
        );
    }

    #[test]
    fn test_load_all_templates_from_file() {
        let toml = include_str!("../../../assets/config/entity_templates.toml");
        let templates = load_entity_templates(toml).unwrap();
        assert!(
            templates.len() >= 5,
            "expected at least 5 templates, got {}",
            templates.len()
        );
    }

    #[test]
    fn test_generate_each_template() {
        let toml = include_str!("../../../assets/config/entity_templates.toml");
        let templates = load_entity_templates(toml).unwrap();

        for template in &templates {
            let mut world = setup_world();
            let registry = world.resource::<TagRegistry>().clone();
            let pos = Position { x: 0, y: 0, z: 0 };

            let entity = generate_entity(&mut world, template, 42, pos, &registry, None);

            assert!(
                world.get::<Position>(entity).is_some(),
                "{} missing Position",
                template.name
            );
            assert!(
                world.get::<Creature>(entity).is_some(),
                "{} missing Creature",
                template.name
            );
            assert!(
                world.get::<Health>(entity).is_some(),
                "{} missing Health",
                template.name
            );
            assert!(
                world.get::<Name>(entity).is_some(),
                "{} missing Name",
                template.name
            );
            assert!(
                world.get::<Tags>(entity).is_some(),
                "{} missing Tags",
                template.name
            );
        }
    }
}
