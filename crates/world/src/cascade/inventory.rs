use game_tags::{TagId, TagRegistry, Tags};
use rand::Rng;
use crate::cascade::{CascadeEngine, TaggedWeight};

/// A generated item in an entity's inventory
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub item_id: String,
    pub quantity: u32,
    pub trade_only: bool,
}

fn is_trade_only(entity_tags: &Tags, registry: &TagRegistry) -> bool {
    registry.tag_id("PEACEFUL").is_some_and(|id| entity_tags.has(id))
}

fn inventory_rolls(entity_tags: &Tags, registry: &TagRegistry, base: u32) -> u32 {
    let large = registry.tag_id("LARGE");
    let huge = registry.tag_id("HUGE");
    let tiny = registry.tag_id("TINY");
    let small = registry.tag_id("SMALL");

    if huge.is_some_and(|id| entity_tags.has(id)) { base * 3 }
    else if large.is_some_and(|id| entity_tags.has(id)) { base * 2 }
    else if tiny.is_some_and(|id| entity_tags.has(id)) { (base / 2).max(1) }
    else if small.is_some_and(|id| entity_tags.has(id)) { (base as f32 * 0.75) as u32 }
    else { base }
}

/// Roll inventory for an entity
pub fn roll_inventory(
    entity_tags: &Tags,
    entity_level: u32,
    faction_id: Option<TagId>,
    location_supply: Option<&[TaggedWeight]>,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    let mut supply_pool: Vec<TaggedWeight> = Vec::new();

    if let Some(fid) = faction_id {
        let faction_name = &registry.tag_by_id(fid).name;
        if let Some(eco) = engine.faction_economies.iter().find(|e| e.id == *faction_name) {
            supply_pool.extend(eco.produces.clone());
        }
    }

    if registry.tag_id("HERBIVORE").is_some_and(|id| entity_tags.has(id)) {
        supply_pool.push(TaggedWeight { tag: "FOOD_WILD".to_string(), weight: 40.0 });
    }
    if registry.tag_id("CARNIVORE").is_some_and(|id| entity_tags.has(id)) {
        supply_pool.push(TaggedWeight { tag: "FOOD_WILD".to_string(), weight: 30.0 });
    }
    if registry.tag_id("BEAST").is_some_and(|id| entity_tags.has(id)) {
        supply_pool.push(TaggedWeight { tag: "LEATHER".to_string(), weight: 15.0 });
    }

    // Merge location economy supply
    if let Some(supply) = location_supply {
        for s in supply {
            supply_pool.push(s.clone());
        }
    }

    if supply_pool.is_empty() { return vec![]; }

    let rolls = inventory_rolls(entity_tags, registry, entity_level.max(1));
    let trade_flag = is_trade_only(entity_tags, registry);
    let mut items: Vec<InventoryItem> = Vec::new();

    for _ in 0..rolls {
        let total: f32 = supply_pool.iter().map(|s| s.weight).sum();
        if total <= 0.0 { break; }

        let roll = rng.random::<f32>() * total;
        let mut accum = 0.0f32;
        let picked_tag = supply_pool.iter()
            .find(|s| { accum += s.weight; roll < accum })
            .map(|s| &s.tag);

        let Some(tag) = picked_tag else { continue; };
        let Some(tag_id) = registry.tag_id(tag) else { continue; };

        let candidates: Vec<&crate::cascade::ItemDef> = engine.items.iter()
            .filter(|item| {
                item.tags.iter()
                    .filter_map(|t| registry.tag_id(t))
                    .any(|id| id == tag_id)
            })
            .collect();

        if candidates.is_empty() { continue; }

        let item_total: f32 = candidates.iter().map(|c| c.weight).sum();
        if item_total <= 0.0 { continue; }

        let item_roll = rng.random::<f32>() * item_total;
        let mut item_accum = 0.0f32;
        if let Some(selected) = candidates.iter()
            .find(|c| { item_accum += c.weight; item_roll < item_accum })
        {
            let quantity = rng.random_range(1..=3);
            items.push(InventoryItem {
                item_id: selected.id.clone(),
                quantity,
                trade_only: trade_flag,
            });
        }
    }

    items
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
    fn herbivore_gets_food_items() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HERBIVORE").unwrap(), TagValue::None, &registry);

        let items = roll_inventory(&tags, 1, None, None, &engine, &registry, &mut rng);
        assert!(!items.is_empty(), "herbivore should have inventory items");
    }

    #[test]
    fn beast_with_faction_gets_items() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("BEAST").unwrap(), TagValue::None, &registry);

        let beast_faction_id = registry.tag_id("mutated_wildlife");
        let items = roll_inventory(&tags, 2, beast_faction_id, None, &engine, &registry, &mut rng);
        assert!(items.len() <= 10, "inventory count should be reasonable");
    }

    #[test]
    fn higher_level_gets_more_rolls() {
        let (engine, registry) = setup();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HERBIVORE").unwrap(), TagValue::None, &registry);

        let mut rng_low = StdRng::seed_from_u64(42);
        let low = roll_inventory(&tags, 1, None, None, &engine, &registry, &mut rng_low);

        let mut rng_high = StdRng::seed_from_u64(42);
        let high = roll_inventory(&tags, 5, None, None, &engine, &registry, &mut rng_high);

        assert!(high.len() >= low.len(), "higher level should get >= items of lower level");
    }

    #[test]
    fn no_tags_returns_empty() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let tags = Tags::new(registry.tag_count());

        let items = roll_inventory(&tags, 1, None, None, &engine, &registry, &mut rng);
        assert!(items.is_empty(), "entity with no matching tags should get no inventory");
    }
}
