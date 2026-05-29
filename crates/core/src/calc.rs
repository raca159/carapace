use bevy_ecs::prelude::World;

use crate::components::Equipment;
use game_tags::{TagRegistry, Tags};

pub fn calc_weapon_damage(equipment: &Equipment, world: &World, registry: &TagRegistry) -> u32 {
    let weapon_entity = match equipment.weapon {
        Some(e) => e,
        None => return 0,
    };

    let tags = match world.get::<Tags>(weapon_entity) {
        Some(t) => t.clone(),
        None => return 5,
    };

    let base: u32 = 5;
    let material_bonus: u32 = if registry.tag_id("METAL").is_some_and(|id| tags.has(id)) {
        3
    } else if registry.tag_id("STONE").is_some_and(|id| tags.has(id)) {
        1
    } else {
        0
    };

    let quality_mult = get_quality_multiplier(&tags, registry);

    (base + material_bonus).saturating_mul(quality_mult)
}

pub fn calc_armor_protection(equipment: &Equipment, world: &World, registry: &TagRegistry) -> u32 {
    let armor_entity = match equipment.armor {
        Some(e) => e,
        None => return 0,
    };

    let tags = match world.get::<Tags>(armor_entity) {
        Some(t) => t.clone(),
        None => return 0,
    };

    let material_bonus: u32 = if registry.tag_id("METAL").is_some_and(|id| tags.has(id)) {
        5
    } else if registry.tag_id("LEATHER").is_some_and(|id| tags.has(id)) {
        3
    } else if registry.tag_id("CLOTH").is_some_and(|id| tags.has(id)) {
        1
    } else {
        0
    };

    let quality_mult = get_quality_multiplier(&tags, registry);

    material_bonus.saturating_mul(quality_mult)
}

pub fn get_quality_multiplier(tags: &Tags, registry: &TagRegistry) -> u32 {
    let quality_ids = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"];
    for qname in &quality_ids {
        if let Some(qid) = registry.tag_id(qname)
            && tags.has(qid)
            && let Some(mult) = registry.tag_by_id(qid).multiplier
        {
            return mult as u32;
        }
    }
    1
}

pub fn get_quality_prefix(tags: &Tags, registry: &TagRegistry) -> String {
    let quality_ids: Vec<game_tags::TagId> = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"]
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();
    for qid in &quality_ids {
        if tags.has(*qid) {
            let name = &registry.tag_by_id(*qid).name;
            return match name.as_str() {
                "COMMON" => String::new(),
                other => format!("{} ", other.to_lowercase()),
            };
        }
    }
    String::new()
}
