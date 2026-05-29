use std::collections::HashMap;
use game_tags::{TagId, TagRegistry, Tags};
use crate::cascade::CascadeEngine;

/// Price multiplier context for a location
#[derive(Debug, Clone)]
pub struct PricingContext {
    /// Per-tag price multipliers (1.0 = base price)
    pub price_multipliers: HashMap<String, f32>,
    /// Overall prosperity of the location (0.0-1.0)
    pub prosperity: f32,
    /// Supply pool (tag + weight) for inventory generation
    pub location_supply: Vec<crate::cascade::TaggedWeight>,
}

/// Compute supply/demand economy for a location
pub fn compute_location_economy(
    biome_tags: &[TagId],
    faction_id: Option<&str>,
    location_tags: &[String],
    engine: &CascadeEngine,
    registry: &TagRegistry,
) -> PricingContext {
    // Check HAS_ECONOMY tag — skip economy if absent
    let has_economy = registry.tag_id("HAS_ECONOMY")
        .is_some_and(|id| location_tags.iter().any(|t| {
            registry.tag_id(t).is_some_and(|lt| lt == id)
        }));
    if !has_economy {
        return PricingContext {
            price_multipliers: HashMap::new(),
            prosperity: 0.0,
            location_supply: Vec::new(),
        };
    }

    let mut supply: HashMap<String, f32> = HashMap::new();

    // Biome supply
    for biome in &engine.region_biomes {
        let matches = biome.tags.iter()
            .filter_map(|t| registry.tag_id(t))
            .any(|id| biome_tags.contains(&id));
        if !matches { continue; }
        for prod in &biome.produces {
            *supply.entry(prod.tag.clone()).or_insert(0.0) += prod.weight;
        }
    }

    // Faction supply
    if let Some(fid) = faction_id {
        if let Some(eco) = engine.faction_economies.iter().find(|e| e.id == fid) {
            for prod in &eco.produces {
                *supply.entry(prod.tag.clone()).or_insert(0.0) += prod.weight;
            }
        }
    }

    // Prosperity from total supply
    let total_supply: f32 = supply.values().sum();
    let prosperity = (total_supply / 200.0).min(1.0);

    // Faction demand
    let mut demand: HashMap<String, f32> = HashMap::new();
    if let Some(fid) = faction_id {
        if let Some(eco) = engine.faction_economies.iter().find(|e| e.id == fid) {
            for cons in &eco.consumes {
                *demand.entry(cons.tag.clone()).or_insert(0.0) += cons.weight;
            }
        }
    }

    // Price multipliers: demand / supply
    let mut price_multipliers: HashMap<String, f32> = HashMap::new();
    let all_tags: std::collections::HashSet<&str> = supply.keys()
        .chain(demand.keys())
        .map(|s| s.as_str())
        .collect();

    for tag in all_tags {
        let sup = supply.get(tag).copied().unwrap_or(0.0);
        let dem = demand.get(tag).copied().unwrap_or(0.0);
        let mult = if sup > 0.0 {
            (dem / sup).clamp(0.3, 3.0)
        } else if dem > 0.0 {
            3.0
        } else {
            1.0
        };
        price_multipliers.insert(tag.to_string(), mult);
    }

    let location_supply: Vec<crate::cascade::TaggedWeight> = supply.into_iter()
        .map(|(tag, weight)| crate::cascade::TaggedWeight { tag, weight })
        .collect();

    PricingContext { price_multipliers, prosperity, location_supply }
}

/// Get the price multiplier for a specific item in a pricing context
pub fn item_price_multiplier(
    item_tags: &Tags,
    pricing: &PricingContext,
    registry: &TagRegistry,
) -> f32 {
    let mut count = 0u32;
    let mut total = 0.0f32;
    for (tag, m) in &pricing.price_multipliers {
        if let Some(tid) = registry.tag_id(tag) {
            if item_tags.has(tid) {
                total += m;
                count += 1;
            }
        }
    }
    if count > 0 { total / count as f32 } else { 1.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::{Tags, TagValue};
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
    fn grassland_free_humanity_has_food_supply() {
        let (engine, registry) = setup();
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();

        let pricing = compute_location_economy(
            &[grassland_id],
            Some("free_humanity"),
            &["HAS_ECONOMY".to_string()],
            &engine, &registry,
        );
        assert!(pricing.prosperity > 0.0, "settlement should have prosperity");
        assert!(
            pricing.price_multipliers.contains_key("FOOD_WILD"),
            "grassland should supply food"
        );
    }

    #[test]
    fn mountain_metal_is_cheap() {
        let (engine, registry) = setup();
        let mountain_id = registry.tag_id("BIOME_MOUNTAIN").unwrap();
        let metal_id = registry.tag_id("METAL").unwrap();

        let pricing = compute_location_economy(
            &[mountain_id],
            None,
            &["HAS_ECONOMY".to_string()],
            &engine, &registry,
        );

        let mut metal_tags = Tags::new(registry.tag_count());
        metal_tags.add_tag(metal_id, TagValue::None, &registry);
        let mult = item_price_multiplier(&metal_tags, &pricing, &registry);
        assert!(mult <= 1.5, "metal should not be expensive in mountains (mult={})", mult);
    }

    #[test]
    fn free_humanity_demands_metal() {
        let (engine, registry) = setup();
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();
        let metal_id = registry.tag_id("METAL").unwrap();

        let pricing = compute_location_economy(
            &[grassland_id],
            Some("free_humanity"),
            &["HAS_ECONOMY".to_string()],
            &engine, &registry,
        );

        let mut metal_tags = Tags::new(registry.tag_count());
        metal_tags.add_tag(metal_id, TagValue::None, &registry);
        let mult = item_price_multiplier(&metal_tags, &pricing, &registry);
        assert!(mult > 0.5, "metal should have a price determined by supply/demand");
    }

    #[test]
    fn item_with_no_matching_tags_gets_default_multiplier() {
        let (engine, registry) = setup();
        let pricing = compute_location_economy(&[], None, &[], &engine, &registry);
        let tags = Tags::new(registry.tag_count());
        let mult = item_price_multiplier(&tags, &pricing, &registry);
        assert!((mult - 1.0).abs() < 0.001, "no tags should give 1.0 multiplier");
    }

    #[test]
    fn no_economy_tag_returns_zero_prosperity() {
        let (engine, registry) = setup();
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();

        // Without HAS_ECONOMY tag, prosperity should be 0
        let pricing = compute_location_economy(
            &[grassland_id], Some("free_humanity"), &[], &engine, &registry,
        );
        assert!((pricing.prosperity - 0.0).abs() < 0.001, "no HAS_ECONOMY should give 0 prosperity");
        assert!(pricing.price_multipliers.is_empty(), "no pricing without economy");
    }

    #[test]
    fn has_economy_tag_enables_pricing() {
        let (engine, registry) = setup();
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();

        let pricing = compute_location_economy(
            &[grassland_id], Some("free_humanity"),
            &["HAS_ECONOMY".to_string()], &engine, &registry,
        );
        assert!(pricing.prosperity > 0.0, "HAS_ECONOMY should enable prosperity");
        assert!(!pricing.price_multipliers.is_empty(), "should have pricing");
    }
}
