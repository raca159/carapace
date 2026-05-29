use rand::Rng;

use bevy_ecs::prelude::*;
use serde::Deserialize;

use crate::components::{Adaptation, GeneSplicing, SpliceSlot};
use crate::{Inventory, Position};
use game_tags::{TagId, TagRegistry, TagValue, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct SplicingRecipe {
    pub name: String,
    pub input_sample_tag: String,
    pub output_mutation_tag: String,
    pub output_mutation_name: String,
    pub success_chance: f64,
    pub humanity_cost: u32,
    pub failure_tags: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub slot: SpliceSlot,
}

#[derive(Debug, Clone, Deserialize)]
struct SplicingToml {
    #[serde(rename = "recipe")]
    recipes: Vec<SplicingRecipe>,
}

pub fn load_splicing_recipes(toml_str: &str) -> Vec<SplicingRecipe> {
    toml::from_str::<SplicingToml>(toml_str)
        .map(|f| f.recipes)
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct SpliceRecipeAvailability {
    pub recipe: SplicingRecipe,
    pub available: bool,
    pub reason: String,
    pub success_chance: f64,
    pub humanity_cost: u32,
    pub failure_tags: Vec<String>,
}

pub fn find_available_recipes(
    recipes: &[SplicingRecipe],
    player_entity: Entity,
    world: &mut World,
    registry: &TagRegistry,
) -> Vec<SpliceRecipeAvailability> {
    let _viable_tissue = registry.tag_id("VIABLE_TISSUE");
    let Some(_viable_tissue) = _viable_tissue else {
        return recipes.iter().map(|r| SpliceRecipeAvailability {
            recipe: r.clone(),
            available: false,
            reason: "Tag registry not loaded".to_string(),
            success_chance: 0.0,
            humanity_cost: 0,
            failure_tags: vec![],
        }).collect();
    };

    let player_has_splicer = {
        let mut query = world.query_filtered::<Entity, (With<Position>, With<crate::components::GeneSplicer>)>();
        let player_pos = world.get::<Position>(player_entity).map(|p| (p.x, p.y));
        query.iter(world).any(|e| {
            world.get::<Position>(e).map(|p| {
                let dx = (p.x as i32 - player_pos.unwrap_or((0,0)).0 as i32).unsigned_abs();
                let dy = (p.y as i32 - player_pos.unwrap_or((0,0)).1 as i32).unsigned_abs();
                dx <= 1 && dy <= 1
            }).unwrap_or(false)
        })
    };

    let inventory = world.get::<Inventory>(player_entity);
    let gene_splicing = world.get::<GeneSplicing>(player_entity);

    let total_splices = gene_splicing.map(|g| g.splice_points).unwrap_or(0);
    let humanity = gene_splicing.map(|g| g.humanity).unwrap_or(100);

    let inventory_sample_tags: Vec<TagId> = inventory.map(|inv| {
        inv.items.iter().filter_map(|&item_entity| {
            world.get::<Tags>(item_entity).map(|tags| {
                tags.iter_present().collect::<Vec<_>>()
            })
        }).flatten().collect()
    }).unwrap_or_default();

    recipes.iter().map(|recipe| {
        let sample_tag_id = registry.tag_id(&recipe.input_sample_tag);
        let has_sample = sample_tag_id.map(|id| inventory_sample_tags.contains(&id)).unwrap_or(false);

        if !player_has_splicer {
            return SpliceRecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: "Not near a Gene-Splicing Pod".to_string(),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        if !has_sample {
            return SpliceRecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: format!("Missing tissue: {}", recipe.input_sample_tag),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        if humanity == 0 {
            return SpliceRecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: "Humanity depleted \u{2014} cannot splice further.".to_string(),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        let quality_mult = sample_tag_id.and_then(|id| {
            let inventory = world.get::<Inventory>(player_entity);
            inventory.and_then(|inv| {
                inv.items.iter().find(|&&e| {
                    world.get::<Tags>(e).map(|t| t.has(id)).unwrap_or(false)
                })
            }).and_then(|e| {
                world.get::<Tags>(*e).map(|t| {
                    let legendary = registry.tag_id("LEGENDARY");
                    let epic = registry.tag_id("EPIC");
                    let rare = registry.tag_id("RARE");
                    let uncommon = registry.tag_id("UNCOMMON");
                    if legendary.map_or(false, |lid| t.has(lid)) { 1.6 }
                    else if epic.map_or(false, |eid| t.has(eid)) { 1.4 }
                    else if rare.map_or(false, |rid| t.has(rid)) { 1.2 }
                    else if uncommon.map_or(false, |uid| t.has(uid)) { 1.0 }
                    else { 0.8 }
                })
            })
        }).unwrap_or(1.0);

        let splice_penalty = 0.03 * total_splices as f64;
        let humanity_mod = if humanity < 10 { 0.15 } else if humanity < 30 { 0.1 } else { 0.0 };
        let final_chance = (recipe.success_chance * quality_mult - splice_penalty + humanity_mod)
            .clamp(0.05, 0.95);

        let failure_tag_names: Vec<String> = recipe.failure_tags.iter()
            .filter_map(|t| registry.tag_id(t).map(|id| registry.tag_by_id(id).name.clone()))
            .collect();

        SpliceRecipeAvailability {
            recipe: recipe.clone(),
            available: true,
            reason: format!("{:.0}% chance", final_chance * 100.0),
            success_chance: final_chance,
            humanity_cost: recipe.humanity_cost,
            failure_tags: failure_tag_names,
        }
    }).collect()
}

pub enum SpliceOutcome {
    Success {
        mutation_name: String,
        new_tag: TagId,
    },
    Failure {
        malapty_name: String,
        applied_tags: Vec<TagId>,
    },
}

pub fn execute_splice(
    recipe: &SplicingRecipe,
    player_entity: Entity,
    world: &mut World,
    registry: &TagRegistry,
) -> SpliceOutcome {
    execute_splice_with_rng(recipe, player_entity, world, registry, &mut rand::rng())
}

fn execute_splice_with_rng(
    recipe: &SplicingRecipe,
    player_entity: Entity,
    world: &mut World,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> SpliceOutcome {
    let sample_tag_id = match registry.tag_id(&recipe.input_sample_tag) {
        Some(id) => id,
        None => return SpliceOutcome::Failure {
            malapty_name: "Unknown".to_string(),
            applied_tags: vec![],
        },
    };

    let mutation_tag_id = match registry.tag_id(&recipe.output_mutation_tag) {
        Some(id) => id,
        None => return SpliceOutcome::Failure {
            malapty_name: "Unknown".to_string(),
            applied_tags: vec![],
        },
    };

    let (quality_mult, sample_entity) = {
        let inventory = world.get::<Inventory>(player_entity).cloned();
        match inventory {
            Some(inv) => {
                let found = inv.items.iter().find(|&&e| {
                    world.get::<Tags>(e).map(|t| t.has(sample_tag_id)).unwrap_or(false)
                });
                match found {
                    Some(&e) => {
                        let qual = {
                            let tags = world.get::<Tags>(e);
                            let legendary = registry.tag_id("LEGENDARY");
                            let epic = registry.tag_id("EPIC");
                            let rare = registry.tag_id("RARE");
                            let uncommon = registry.tag_id("UNCOMMON");
                            if let Some(t) = tags {
                                if legendary.map_or(false, |lid| t.has(lid)) { 1.6 }
                                else if epic.map_or(false, |eid| t.has(eid)) { 1.4 }
                                else if rare.map_or(false, |rid| t.has(rid)) { 1.2 }
                                else if uncommon.map_or(false, |uid| t.has(uid)) { 1.0 }
                                else { 0.8 }
                            } else { 1.0 }
                        };
                        (qual, Some(e))
                    }
                    None => (1.0, None),
                }
            }
            None => (1.0, None),
        }
    };

    let total_splices = world.get::<GeneSplicing>(player_entity)
        .map(|g| g.splice_points).unwrap_or(0);
    let humanity = world.get::<GeneSplicing>(player_entity)
        .map(|g| g.humanity).unwrap_or(100);

    let splice_penalty = 0.03 * total_splices as f64;
    let humanity_mod = if humanity < 10 { 0.15 } else if humanity < 30 { 0.1 } else { 0.0 };
    let final_chance = (recipe.success_chance * quality_mult - splice_penalty + humanity_mod)
        .clamp(0.05, 0.95);

    let roll: f64 = rng.random();

    if roll < final_chance {
        if let Some(mut gs) = world.get_mut::<GeneSplicing>(player_entity) {
            let human_cost = recipe.humanity_cost.min(gs.humanity);
            gs.humanity = gs.humanity.saturating_sub(human_cost).min(100);
            gs.splice_points += 1;
            gs.adaptations.push(Adaptation {
                id: recipe.output_mutation_tag.clone(),
                source: recipe.name.clone(),
                tags_granted: vec![mutation_tag_id],
                humanity_cost: recipe.humanity_cost,
                slot: recipe.slot,
            });
        }

        if let Some(mut tags) = world.get_mut::<Tags>(player_entity) {
            tags.add_tag(mutation_tag_id, TagValue::None, registry);
            let spliced_id = registry.tag_id("GENE_SPLICED");
            if let Some(sid) = spliced_id {
                tags.add_tag(sid, TagValue::None, registry);
            }
        }

        if let Some(sample_entity) = sample_entity {
            if let Some(mut inv) = world.get_mut::<Inventory>(player_entity) {
                inv.items.retain(|&e| e != sample_entity);
            }
            let _ = world.despawn(sample_entity);
        }

        SpliceOutcome::Success {
            mutation_name: recipe.output_mutation_name.clone(),
            new_tag: mutation_tag_id,
        }
    } else {
        let failure_ids: Vec<TagId> = recipe.failure_tags.iter()
            .filter_map(|t| registry.tag_id(t))
            .collect();

        if let Some(mut tags) = world.get_mut::<Tags>(player_entity) {
            for &fid in &failure_ids {
                tags.add_tag(fid, TagValue::None, registry);
            }
        }

        if let Some(mut gs) = world.get_mut::<GeneSplicing>(player_entity) {
            gs.splice_points += 1;
        }

        if let Some(sample_entity) = sample_entity {
            if let Some(mut inv) = world.get_mut::<Inventory>(player_entity) {
                inv.items.retain(|&e| e != sample_entity);
            }
            let _ = world.despawn(sample_entity);
        }

        let malapty_name = failure_ids.first()
            .map(|id| registry.tag_by_id(*id).name.clone())
            .unwrap_or_else(|| "Unknown Malapty".to_string());

        SpliceOutcome::Failure {
            malapty_name,
            applied_tags: failure_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::EventBus;

    use rand::RngCore;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");
    const SPLICING_TOML: &str = include_str!("../../../assets/config/gene_splicing.toml");

    struct FixedRng(u64);

    impl RngCore for FixedRng {
        fn next_u32(&mut self) -> u32 { self.0 as u32 }
        fn next_u64(&mut self) -> u64 { self.0 }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let bytes = self.0.to_le_bytes();
            for (i, b) in dest.iter_mut().enumerate() {
                *b = bytes[i % 8];
            }
        }
    }

    fn setup_registry() -> game_tags::TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).expect("tags")
    }

    fn setup_world() -> World {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());
        world.insert_resource(EventBus::new());
        world
    }

    fn registry(world: &World) -> TagRegistry {
        world.resource::<TagRegistry>().clone()
    }

    #[test]
    fn test_load_splicing_recipes() {
        let recipes = load_splicing_recipes(SPLICING_TOML);
        assert_eq!(recipes.len(), 5);
        assert_eq!(recipes[0].name, "Sonic Cavitation");
        assert_eq!(recipes[4].name, "Chromatophoric Shift");
    }

    #[test]
    fn test_load_splicing_recipes_empty() {
        let recipes = load_splicing_recipes("");
        assert!(recipes.is_empty());
    }

    #[test]
    fn test_find_available_recipes_missing_pod() {
        let mut world = setup_world();
        let reg = registry(&world);

        let player = world.spawn((
            Player,
            Position { z: 0, x: 5, y: 5 },
            GeneSplicing::new(),
            Tags::new(reg.tag_count()),
            Inventory { items: vec![], capacity: 20 },
        )).id();

        let recipes = load_splicing_recipes(SPLICING_TOML);
        let avail = find_available_recipes(&recipes, player, &mut world, &reg);
        assert!(!avail.is_empty());
        assert!(!avail[0].available);
        assert!(avail[0].reason.contains("Pod"));
    }

    #[test]
    fn test_find_available_recipes_with_pod_and_sample() {
        let mut world = setup_world();
        let reg = registry(&world);
        let viable_tissue = reg.tag_id("VIABLE_TISSUE").unwrap();
        let sample_tag = reg.tag_id("TISSUE_PISTOL_SHRIMP").unwrap();

        let _pod = world.spawn((
            GeneSplicer,
            Position { z: 0, x: 6, y: 5 },
        )).id();

        let sample = world.spawn((
            Item,
            Name("Pistol Shrimp Gland".to_string()),
            Glyph { char: 't', color: (200, 100, 50) },
            {
                let mut t = Tags::new(reg.tag_count());
                t.add_tag(viable_tissue, TagValue::None, &reg);
                t.add_tag(sample_tag, TagValue::None, &reg);
                t
            },
        )).id();

        let player = world.spawn((
            Player,
            Position { z: 0, x: 5, y: 5 },
            GeneSplicing::new(),
            Tags::new(reg.tag_count()),
            Inventory { items: vec![sample], capacity: 20 },
        )).id();

        let recipes = load_splicing_recipes(SPLICING_TOML);
        let avail = find_available_recipes(&recipes, player, &mut world, &reg);
        let sonic = avail.iter().find(|a| a.recipe.name == "Sonic Cavitation");
        assert!(sonic.is_some());
        assert!(sonic.unwrap().available);
    }

    #[test]
    fn test_execute_splice_success_deterministic() {
        let mut world = setup_world();
        let reg = registry(&world);
        let viable_tissue = reg.tag_id("VIABLE_TISSUE").unwrap();
        let sample_tag = reg.tag_id("TISSUE_PISTOL_SHRIMP").unwrap();
        let mutation_tag = reg.tag_id("SONIC_CAVITATION").unwrap();

        let _pod = world.spawn((
            GeneSplicer,
            Position { z: 0, x: 6, y: 5 },
        )).id();

        let sample = world.spawn((
            Item,
            Name("Pistol Shrimp Gland".to_string()),
            Glyph { char: 't', color: (200, 100, 50) },
            {
                let mut t = Tags::new(reg.tag_count());
                t.add_tag(viable_tissue, TagValue::None, &reg);
                t.add_tag(sample_tag, TagValue::None, &reg);
                t
            },
        )).id();

        let player = world.spawn((
            Player,
            Position { z: 0, x: 5, y: 5 },
            GeneSplicing::new(),
            Tags::new(reg.tag_count()),
            Inventory { items: vec![sample], capacity: 20 },
        )).id();

        let recipes = load_splicing_recipes(SPLICING_TOML);
        let sonic = recipes.iter().find(|r| r.name == "Sonic Cavitation").unwrap();

        // FixedRng yields 0 on first next_u64(), guaranteeing roll < final_chance (0.65)
        let mut rng = FixedRng(0);
        let outcome = execute_splice_with_rng(sonic, player, &mut world, &reg, &mut rng);
        match outcome {
            SpliceOutcome::Success { mutation_name, .. } => {
                assert_eq!(mutation_name, "Sonic Cavitation");
                let gs = world.get::<GeneSplicing>(player).unwrap();
                assert_eq!(gs.splice_points, 1);
                assert!(gs.humanity < 100);
                assert!(!gs.adaptations.is_empty());
                assert_eq!(gs.adaptations[0].slot, SpliceSlot::Arms);
                let tags = world.get::<Tags>(player).unwrap();
                assert!(tags.has(mutation_tag));
            }
            _ => panic!("Expected success outcome"),
        }
    }

    #[test]
    fn test_execute_splice_deterministic_failure() {
        let mut world = setup_world();
        let reg = registry(&world);
        let viable_tissue = reg.tag_id("VIABLE_TISSUE").unwrap();
        let sample_tag = reg.tag_id("TISSUE_PISTOL_SHRIMP").unwrap();
        let failure_tag = reg.tag_id("NEURAL_FRAGILE").unwrap();

        let _pod = world.spawn((
            GeneSplicer,
            Position { z: 0, x: 6, y: 5 },
        )).id();

        let sample = world.spawn((
            Item,
            Name("Pistol Shrimp Gland".to_string()),
            Glyph { char: 't', color: (200, 100, 50) },
            {
                let mut t = Tags::new(reg.tag_count());
                t.add_tag(viable_tissue, TagValue::None, &reg);
                t.add_tag(sample_tag, TagValue::None, &reg);
                t
            },
        )).id();

        let player = world.spawn((
            Player,
            Position { z: 0, x: 5, y: 5 },
            GeneSplicing::new(),
            Tags::new(reg.tag_count()),
            Inventory { items: vec![sample], capacity: 20 },
        )).id();

        let recipes = load_splicing_recipes(SPLICING_TOML);
        let sonic = recipes.iter().find(|r| r.name == "Sonic Cavitation").unwrap();

        // FixedRng yields u64::MAX on first next_u64(), guaranteeing roll >= final_chance (0.65)
        let mut rng = FixedRng(u64::MAX);
        let outcome = execute_splice_with_rng(sonic, player, &mut world, &reg, &mut rng);
        match outcome {
            SpliceOutcome::Failure { malapty_name, applied_tags } => {
                assert!(!malapty_name.is_empty());
                assert!(applied_tags.contains(&failure_tag));
                let gs = world.get::<GeneSplicing>(player).unwrap();
                assert_eq!(gs.splice_points, 1);
                // humanity should not change on failure
                assert_eq!(gs.humanity, 100);
            }
            _ => panic!("Expected failure outcome"),
        }
    }

    #[test]
    fn test_humanity_clamping() {
        let mut gs = GeneSplicing::new();
        assert_eq!(gs.humanity, 100);
        assert_eq!(gs.humanity_rank(), "Full Human");
        gs.humanity = 0;
        assert_eq!(gs.humanity_rank(), "Carapace-Mind");
        gs.humanity = 50;
        assert_eq!(gs.humanity_rank(), "Significant Chitin");
    }

    #[test]
    fn test_humanity_rank_overflow_clamps_to_full_human() {
        let mut gs = GeneSplicing::new();
        gs.humanity = 200;
        assert_eq!(gs.humanity_rank(), "Full Human");
    }
}
