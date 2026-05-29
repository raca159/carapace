use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::Deserialize;

use crate::{Glyph, Inventory, Item, Name, Position};
use game_tags::{TagId, TagRegistry, TagValue, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeOutput {
    pub tags: Vec<String>,
    pub name: String,
    pub glyph: char,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Deserialize)]
pub struct CraftingRecipe {
    pub name: String,
    pub inputs: Vec<String>,
    #[serde(default)]
    pub requires_env: Vec<String>,
    pub outputs: Vec<RecipeOutput>,
}

#[derive(Debug, Clone, Deserialize)]
struct CraftingToml {
    #[serde(rename = "recipe")]
    recipes: Vec<CraftingRecipe>,
}

pub fn load_crafting_recipes(toml_str: &str) -> Result<Vec<CraftingRecipe>, toml::de::Error> {
    let file: CraftingToml = toml::from_str(toml_str)?;
    Ok(file.recipes)
}

#[derive(Debug, Clone)]
pub struct RecipeAvailability {
    pub recipe: CraftingRecipe,
    pub available: bool,
    pub missing_inputs: Vec<String>,
    pub env_met: bool,
}

pub fn find_available_recipes(
    recipes: &[CraftingRecipe],
    inventory: &Inventory,
    world: &mut World,
    player_pos: (u32, u32),
    registry: &TagRegistry,
) -> Vec<RecipeAvailability> {
    let mut inventory_tag_counts: HashMap<TagId, usize> = HashMap::new();

    for &item_entity in &inventory.items {
        if let Some(item_tags) = world.get::<Tags>(item_entity) {
            for tag_id in item_tags.iter_present() {
                *inventory_tag_counts.entry(tag_id).or_insert(0) += 1;
            }
        }
    }

    let adjacent_tags = collect_adjacent_tags(world, player_pos);

    recipes
        .iter()
        .map(|recipe| {
            let input_tag_ids: Vec<TagId> = recipe
                .inputs
                .iter()
                .filter_map(|name| registry.tag_id(name))
                .collect();

            let mut needed: HashMap<TagId, usize> = HashMap::new();
            for &id in &input_tag_ids {
                *needed.entry(id).or_insert(0) += 1;
            }

            let mut missing_inputs = Vec::new();
            let mut available = true;

            for (&tag_id, &count_needed) in &needed {
                let have = inventory_tag_counts.get(&tag_id).copied().unwrap_or(0);
                if have < count_needed {
                    available = false;
                    let def = registry.tag_by_id(tag_id);
                    for _ in 0..(count_needed - have) {
                        missing_inputs.push(def.name.clone());
                    }
                }
            }

            let env_met = if recipe.requires_env.is_empty() {
                true
            } else {
                recipe
                    .requires_env
                    .iter()
                    .all(|tag_name| {
                        registry
                            .tag_id(tag_name)
                            .is_some_and(|id| adjacent_tags.contains_key(&id))
                    })
            };

            if !env_met {
                available = false;
            }

            RecipeAvailability {
                recipe: recipe.clone(),
                available,
                missing_inputs,
                env_met,
            }
        })
        .collect()
}

fn collect_adjacent_tags(
    world: &mut World,
    player_pos: (u32, u32),
) -> HashMap<TagId, usize> {
    let mut adjacent_tags: HashMap<TagId, usize> = HashMap::new();

    let offsets: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),           (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];

    let mut entity_query = world.query::<(Entity, &Position, &Tags)>();

    for &(dx, dy) in &offsets {
        let nx = (player_pos.0 as i32 + dx) as u32;
        let ny = (player_pos.1 as i32 + dy) as u32;

        for (_, pos, tags) in entity_query.iter(world) {
            if pos.x == nx && pos.y == ny {
                for tag_id in tags.iter_present() {
                    *adjacent_tags.entry(tag_id).or_insert(0) += 1;
                }
            }
        }
    }

    adjacent_tags
}

pub fn execute_recipe(
    recipe: &CraftingRecipe,
    inventory: &mut Inventory,
    world: &mut World,
    registry: &TagRegistry,
) -> Vec<Entity> {
    let input_tag_ids: Vec<TagId> = recipe
        .inputs
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();

    let mut to_consume: Vec<Entity> = Vec::new();
    let mut needed: HashMap<TagId, usize> = HashMap::new();
    for &id in &input_tag_ids {
        *needed.entry(id).or_insert(0) += 1;
    }

    for &item_entity in &inventory.items {
        if needed.is_empty() {
            break;
        }
        if let Some(item_tags) = world.get::<Tags>(item_entity) {
            let mut consumed_this = false;
            let tag_ids: Vec<TagId> = item_tags.iter_present().collect();
            for tag_id in &tag_ids {
                if let Some(count) = needed.get_mut(tag_id)
                    && *count > 0 {
                        *count -= 1;
                        consumed_this = true;
                        if *count == 0 {
                            needed.remove(tag_id);
                        }
                    }
            }
            if consumed_this {
                to_consume.push(item_entity);
            }
        }
    }

    for entity in &to_consume {
        inventory.items.retain(|&e| e != *entity);
        world.entity_mut(*entity).despawn();
    }

    let mut created: Vec<Entity> = Vec::new();
    for output in &recipe.outputs {
        let output_tag_ids: Vec<TagId> = output
            .tags
            .iter()
            .filter_map(|name| registry.tag_id(name))
            .collect();

        let mut entity_tags = Tags::new(registry.tag_count());
        for &tag_id in &output_tag_ids {
            entity_tags.add_tag(tag_id, TagValue::None, registry);
        }

        let entity = world
            .spawn((
                Item,
                Name(output.name.clone()),
                Glyph {
                    char: output.glyph,
                    color: (output.color[0], output.color[1], output.color[2]),
                },
                entity_tags,
            ))
            .id();

        created.push(entity);
    }

    for &entity in &created {
        inventory.items.push(entity);
    }

    created
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    fn setup_registry() -> TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).expect("tags")
    }

    #[test]
    fn test_load_crafting_recipes() {
        let toml = r#"
[[recipe]]
name = "Torch"
inputs = ["WOOD", "CLOTH"]
outputs = [{ tags = ["WOOD", "CLOTH", "FLAMMABLE", "HOLDABLE", "LUMINESCENT"], name = "Torch", glyph = "t", color = [255, 200, 50] }]
"#;
        let recipes = load_crafting_recipes(toml).unwrap();
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].name, "Torch");
        assert_eq!(recipes[0].inputs, vec!["WOOD", "CLOTH"]);
    }

    #[test]
    fn test_find_available_recipes_with_matching_inventory() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let wood_id = registry.tag_id("WOOD").unwrap();
        let cloth_id = registry.tag_id("CLOTH").unwrap();

        let wood_entity = world.spawn((
            Item,
            Name("Wood Piece".to_string()),
            Glyph { char: 'w', color: (139, 90, 43) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(wood_id, TagValue::None, &registry);
                t
            },
        )).id();

        let cloth_entity = world.spawn((
            Item,
            Name("Cloth Scrap".to_string()),
            Glyph { char: 'c', color: (200, 200, 200) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(cloth_id, TagValue::None, &registry);
                t
            },
        )).id();

        let player_entity = world.spawn((
            Player,
            Position { x: 50, y: 50, z: 0 },
            Inventory {
                items: vec![wood_entity, cloth_entity],
                capacity: 20,
            },
        )).id();

        let recipe = CraftingRecipe {
            name: "Torch".to_string(),
            inputs: vec!["WOOD".to_string(), "CLOTH".to_string()],
            requires_env: vec![],
            outputs: vec![RecipeOutput {
                tags: vec!["WOOD".to_string(), "CLOTH".to_string(), "FLAMMABLE".to_string(), "HOLDABLE".to_string(), "LUMINESCENT".to_string()],
                name: "Torch".to_string(),
                glyph: 't',
                color: [255, 200, 50],
            }],
        };

        let inventory = world.get::<Inventory>(player_entity).unwrap().clone();
        let results = find_available_recipes(
            &[recipe],
            &inventory,
            &mut world,
            (50, 50),
            &registry,
        );

        assert_eq!(results.len(), 1);
        assert!(results[0].available);
        assert!(results[0].missing_inputs.is_empty());
    }

    #[test]
    fn test_find_available_recipes_missing_input() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let wood_id = registry.tag_id("WOOD").unwrap();

        let wood_entity = world.spawn((
            Item,
            Name("Wood Piece".to_string()),
            Glyph { char: 'w', color: (139, 90, 43) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(wood_id, TagValue::None, &registry);
                t
            },
        )).id();

        let player_entity = world.spawn((
            Player,
            Position { x: 50, y: 50, z: 0 },
            Inventory {
                items: vec![wood_entity],
                capacity: 20,
            },
        )).id();

        let recipe = CraftingRecipe {
            name: "Torch".to_string(),
            inputs: vec!["WOOD".to_string(), "CLOTH".to_string()],
            requires_env: vec![],
            outputs: vec![RecipeOutput {
                tags: vec!["WOOD".to_string(), "CLOTH".to_string()],
                name: "Torch".to_string(),
                glyph: 't',
                color: [255, 200, 50],
            }],
        };

        let inventory = world.get::<Inventory>(player_entity).unwrap().clone();
        let results = find_available_recipes(
            &[recipe],
            &inventory,
            &mut world,
            (50, 50),
            &registry,
        );

        assert_eq!(results.len(), 1);
        assert!(!results[0].available);
        assert!(results[0].missing_inputs.contains(&"CLOTH".to_string()));
    }

    #[test]
    fn test_execute_recipe_consumes_and_produces() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let wood_id = registry.tag_id("WOOD").unwrap();
        let cloth_id = registry.tag_id("CLOTH").unwrap();

        let wood_entity = world.spawn((
            Item,
            Name("Wood Piece".to_string()),
            Glyph { char: 'w', color: (139, 90, 43) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(wood_id, TagValue::None, &registry);
                t
            },
        )).id();

        let cloth_entity = world.spawn((
            Item,
            Name("Cloth Scrap".to_string()),
            Glyph { char: 'c', color: (200, 200, 200) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(cloth_id, TagValue::None, &registry);
                t
            },
        )).id();

        let player_entity = world.spawn((
            Player,
            Position { x: 50, y: 50, z: 0 },
            Inventory {
                items: vec![wood_entity, cloth_entity],
                capacity: 20,
            },
        )).id();

        let recipe = CraftingRecipe {
            name: "Torch".to_string(),
            inputs: vec!["WOOD".to_string(), "CLOTH".to_string()],
            requires_env: vec![],
            outputs: vec![RecipeOutput {
                tags: vec!["WOOD".to_string(), "CLOTH".to_string(), "FLAMMABLE".to_string(), "HOLDABLE".to_string(), "LUMINESCENT".to_string()],
                name: "Torch".to_string(),
                glyph: 't',
                color: [255, 200, 50],
            }],
        };

        let mut inventory = world.get::<Inventory>(player_entity).unwrap().clone();
        let created = execute_recipe(&recipe, &mut inventory, &mut world, &registry);

        assert_eq!(created.len(), 1);
        assert_eq!(inventory.items.len(), 1);

        let torch_name = world.get::<Name>(created[0]).unwrap();
        assert_eq!(torch_name.0, "Torch");

        let torch_glyph = world.get::<Glyph>(created[0]).unwrap();
        assert_eq!(torch_glyph.char, 't');
    }

    #[test]
    fn test_execute_recipe_with_duplicate_inputs() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let metal_id = registry.tag_id("METAL").unwrap();

        let metal1 = world.spawn((
            Item,
            Name("Metal 1".to_string()),
            Glyph { char: 'm', color: (180, 180, 180) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(metal_id, TagValue::None, &registry);
                t
            },
        )).id();

        let metal2 = world.spawn((
            Item,
            Name("Metal 2".to_string()),
            Glyph { char: 'm', color: (180, 180, 180) },
            {
                let mut t = Tags::new(registry.tag_count());
                t.add_tag(metal_id, TagValue::None, &registry);
                t
            },
        )).id();

        let player_entity = world.spawn((
            Player,
            Position { x: 50, y: 50, z: 0 },
            Inventory {
                items: vec![metal1, metal2],
                capacity: 20,
            },
        )).id();

        let recipe = CraftingRecipe {
            name: "Forge Blade".to_string(),
            inputs: vec!["METAL".to_string(), "METAL".to_string()],
            requires_env: vec![],
            outputs: vec![RecipeOutput {
                tags: vec!["METAL".to_string(), "HARD".to_string(), "EQUIP_WEAPON".to_string(), "MELEE".to_string()],
                name: "Forged Blade".to_string(),
                glyph: '/',
                color: [200, 200, 210],
            }],
        };

        let mut inventory = world.get::<Inventory>(player_entity).unwrap().clone();
        let created = execute_recipe(&recipe, &mut inventory, &mut world, &registry);

        assert_eq!(created.len(), 1);
        assert_eq!(inventory.items.len(), 1);

        let sword_name = world.get::<Name>(created[0]).unwrap();
        assert_eq!(sword_name.0, "Forged Blade");
    }

    #[test]
    fn test_load_actual_crafting_toml() {
        let crafting_toml = include_str!("../../../assets/config/crafting.toml");
        let recipes = load_crafting_recipes(crafting_toml).unwrap();
        assert_eq!(recipes.len(), 6);
        assert_eq!(recipes[0].name, "Smelt Iron");
        assert_eq!(recipes[1].name, "Forge Sword");
    }
}
