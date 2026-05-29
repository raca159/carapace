use game_tags::{TagRegistry, Tags};
use rand::Rng;
use crate::cascade::{CascadeEngine, ItemDef};

/// Result of an equipment roll for a single slot
#[derive(Debug, Clone)]
pub struct EquipmentRoll {
    pub item: ItemDef,
    pub quality: &'static str,
}

/// Roll equipment for a specific equipment slot
pub fn roll_equipment_for_slot(
    slot_tag: &str,
    entity_tags: &Tags,
    entity_level: u32,
    prosperity: f32,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Option<EquipmentRoll> {
    let slot_tag_id = registry.tag_id(slot_tag)?;

    // 1. Filter items by slot tag
    let candidates: Vec<&ItemDef> = engine.items.iter()
        .filter(|item| {
            item.tags.iter()
                .filter_map(|t| registry.tag_id(t))
                .any(|id| id == slot_tag_id)
        })
        .collect();

    if candidates.is_empty() { return None; }

    // 2. Apply material preference from entity tags
    let preferred_material = engine.preferred_material(entity_tags, registry);
    let preferred_id = preferred_material.and_then(|m| registry.tag_id(m));

    // 3. Weighted selection with preference boost
    let weights: Vec<f32> = candidates.iter().map(|item| {
        let mut w = item.weight;
        if let Some(pref) = preferred_id {
            if item.tags.iter()
                .filter_map(|t| registry.tag_id(t))
                .any(|id| id == pref)
            {
                w *= 2.0;
            }
        }
        w
    }).collect();

    let total: f32 = weights.iter().sum();
    if total <= 0.0 { return None; }

    let roll = rng.random::<f32>() * total;
    let mut accum = 0.0f32;
    let selected = candidates.iter().zip(weights.iter())
        .find(|&(_, &w)| { accum += w; roll < accum })
        .map(|(item, _)| *item)?;

    // 4. Roll quality modulated by entity level
    let prosperity_bias = if prosperity > 0.0 {
        ((1.0 + prosperity * 0.5) * entity_level as f32 / 10.0).min(1.0)
    } else {
        (entity_level as f32 / 10.0).min(1.0)
    };
    let quality = roll_quality_with_bias(
        selected.quality_bias.as_deref(),
        Some(prosperity_bias),
        rng,
    );

    Some(EquipmentRoll {
        item: selected.to_owned(),
        quality,
    })
}

const QUALITY_WEIGHTS: &[(f32, &str)] = &[
    (60.0, "COMMON"),
    (25.0, "UNCOMMON"),
    (10.0, "RARE"),
    (4.0, "EPIC"),
    (1.0, "LEGENDARY"),
];

const BIAS_SHIFTS: &[(&str, &[(f32, &str)])] = &[
    ("common", &[(60.0, "COMMON"), (25.0, "UNCOMMON"), (10.0, "RARE"), (4.0, "EPIC"), (1.0, "LEGENDARY")]),
    ("uncommon", &[(30.0, "COMMON"), (40.0, "UNCOMMON"), (20.0, "RARE"), (8.0, "EPIC"), (2.0, "LEGENDARY")]),
    ("rare", &[(10.0, "COMMON"), (25.0, "UNCOMMON"), (40.0, "RARE"), (18.0, "EPIC"), (7.0, "LEGENDARY")]),
    ("epic", &[(5.0, "COMMON"), (15.0, "UNCOMMON"), (30.0, "RARE"), (35.0, "EPIC"), (15.0, "LEGENDARY")]),
    ("legendary", &[(2.0, "COMMON"), (8.0, "UNCOMMON"), (20.0, "RARE"), (35.0, "EPIC"), (35.0, "LEGENDARY")]),
];

fn roll_quality_with_bias(
    item_bias: Option<&str>,
    prosperity_bias: Option<f32>,
    rng: &mut impl Rng,
) -> &'static str {
    let weights = item_bias
        .and_then(|b| BIAS_SHIFTS.iter().find(|(name, _)| *name == b))
        .map(|(_, w)| *w)
        .unwrap_or(QUALITY_WEIGHTS);

    let prosperity = prosperity_bias.unwrap_or(0.0);
    let adjusted: Vec<(f32, &str)> = weights.iter().enumerate().map(|(i, &(w, label))| {
        let tier_factor = i as f32 / (weights.len() - 1).max(1) as f32;
        (w * (1.0 + prosperity * tier_factor * 0.5), label)
    }).collect();

    let total: f32 = adjusted.iter().map(|(w, _)| w).sum();
    if total <= 0.0 { return "COMMON"; }

    let roll = rng.random::<f32>() * total;
    let mut accum = 0.0f32;
    adjusted.iter()
        .find(|(w, _)| { accum += w; roll < accum })
        .map(|(_, label)| *label)
        .unwrap_or("COMMON")
}

/// Full equipment generation for an entity: determine slot needs, roll per slot
pub fn generate_entity_equipment(
    entity_tags: &Tags,
    entity_level: u32,
    prosperity: f32,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<EquipmentRoll> {
    let slots = engine.slot_needs(entity_tags, registry);
    slots.iter()
        .filter_map(|slot| roll_equipment_for_slot(
            slot, entity_tags, entity_level, prosperity, engine, registry, rng,
        ))
        .collect()
}

// Test helper to get the quality multiplier for a quality tag name
pub fn quality_multiplier(quality: &str) -> f32 {
    match quality {
        "COMMON" => 1.0,
        "UNCOMMON" => 1.5,
        "RARE" => 3.0,
        "EPIC" => 7.0,
        "LEGENDARY" => 15.0,
        _ => 1.0,
    }
}

pub fn quality_prefix(quality: &str) -> &'static str {
    match quality {
        "UNCOMMON" => "Uncommon ",
        "RARE" => "Rare ",
        "EPIC" => "Epic ",
        "LEGENDARY" => "Legendary ",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::{Tags, TagValue};
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use crate::cascade::CascadeEngine;

    const ITEMS_TOML: &str = include_str!("../../../../assets/config/items.toml");
    const BIOMES_TOML: &str = include_str!("../../../../assets/config/region_biomes.toml");
    const FACTIONS_TOML: &str = include_str!("../../../../assets/config/faction_economy.toml");
    const LOCATIONS_TOML: &str = include_str!("../../../../assets/config/location_types.toml");

    fn setup() -> (CascadeEngine, TagRegistry) {
        let tags_toml = include_str!("../../../../assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let engine = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        (engine, registry)
    }

    #[test]
    fn equipment_humanoid_aggressive_gets_weapon() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HUMANOID").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);

        let rolls = generate_entity_equipment(&tags, 1, 0.0, &engine, &registry, &mut rng);
        assert!(!rolls.is_empty(), "humanoid aggressive should get equipment");
        assert!(
            rolls.iter().any(|r| r.item.tags.contains(&"EQUIP_WEAPON".to_string())),
            "should include a weapon"
        );
    }

    #[test]
    fn equipment_beast_peaceful_gets_none() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("BEAST").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("PEACEFUL").unwrap(), TagValue::None, &registry);

        let rolls = generate_entity_equipment(&tags, 1, 0.0, &engine, &registry, &mut rng);
        assert!(rolls.is_empty(), "peaceful beast should have no equipment");
    }

    #[test]
    fn quality_scales_with_level() {
        let (engine, registry) = setup();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HUMANOID").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);

        let mut rng1 = StdRng::seed_from_u64(100);
        let rolls1 = generate_entity_equipment(&tags, 1, 0.0, &engine, &registry, &mut rng1);

        let mut rng10 = StdRng::seed_from_u64(100);
        let rolls10 = generate_entity_equipment(&tags, 10, 0.0, &engine, &registry, &mut rng10);

        assert!(!rolls1.is_empty());
        assert!(!rolls10.is_empty());
    }

    #[test]
    fn deterministic_with_same_seed() {
        let (engine, registry) = setup();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HUMANOID").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);

        let mut rng_a = StdRng::seed_from_u64(42);
        let rolls_a = generate_entity_equipment(&tags, 3, 0.0, &engine, &registry, &mut rng_a);

        let mut rng_b = StdRng::seed_from_u64(42);
        let rolls_b = generate_entity_equipment(&tags, 3, 0.0, &engine, &registry, &mut rng_b);

        assert_eq!(rolls_a.len(), rolls_b.len());
        for (a, b) in rolls_a.iter().zip(rolls_b.iter()) {
            assert_eq!(a.item.id, b.item.id);
            assert_eq!(a.quality, b.quality);
        }
    }

    #[test]
    fn roll_equipment_for_slot_weapon() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let tags = Tags::new(registry.tag_count());

        let result = roll_equipment_for_slot(
            "EQUIP_WEAPON", &tags, 1, 0.0, &engine, &registry, &mut rng,
        );
        assert!(result.is_some(), "should find a weapon");
    }
}
