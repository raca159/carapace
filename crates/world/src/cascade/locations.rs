use game_tags::{TagId, TagRegistry};
use rand::Rng;
use super::{CascadeEngine, PlacedLocation};

pub fn evaluate_tile_habitability(
    tile_biome_tags: &[TagId],
    registry: &TagRegistry,
) -> f32 {
    let habitable = registry.tag_id("HABITABLE");
    let arable = registry.tag_id("ARABLE");
    let mineral = registry.tag_id("MINERAL_RICH");
    let water = registry.tag_id("WATER_FRESH");
    let walkable = registry.tag_id("WALKABLE");

    let fertile_biomes: &[&str] = &[
        "BIOME_GRASSLAND", "BIOME_TEMPERATE_FOREST", "BIOME_SAVANNA",
        "BIOME_TROPICAL_FOREST", "BIOME_SHRUBLAND",
    ];

    let mut score = 0.0f32;
    for tag_id in tile_biome_tags {
        let name = &registry.tag_by_id(*tag_id).name;
        if fertile_biomes.contains(&name.as_str()) { score += 0.3; }
        if walkable.is_some_and(|w| *tag_id == w) { score += 0.2; }
        if water.is_some_and(|w| *tag_id == w) { score += 0.2; }
        if habitable.is_some_and(|h| *tag_id == h) { score += 0.2; }
        if arable.is_some_and(|a| *tag_id == a) { score += 0.2; }
        if mineral.is_some_and(|m| *tag_id == m) { score += 0.15; }
    }
    if score > 1.0 { 1.0 } else { score }
}

fn in_any_zone(x: u32, y: u32, placed: &[PlacedLocation]) -> bool {
    for loc in placed {
        let dx = (x as i32 - loc.x as i32).unsigned_abs();
        let dy = (y as i32 - loc.y as i32).unsigned_abs();
        let dist_sq = dx * dx + dy * dy;
        let zone_sq = loc.zone_radius * loc.zone_radius;
        if dist_sq <= zone_sq { return true; }
    }
    false
}

fn respects_min_distance(
    x: u32, y: u32, type_id: &str, min_dist: u32, placed: &[PlacedLocation],
) -> bool {
    for loc in placed {
        if loc.location_type != type_id { continue; }
        let dx = (x as i32 - loc.x as i32).unsigned_abs();
        let dy = (y as i32 - loc.y as i32).unsigned_abs();
        let dist_sq = dx * dx + dy * dy;
        let min_sq = min_dist * min_dist;
        if dist_sq < min_sq { return false; }
    }
    true
}

fn matches_biome_affinity(
    tile_tags: &[TagId], affinity: &[String], registry: &TagRegistry,
) -> bool {
    if affinity.is_empty() { return true; }
    for tag_id in tile_tags {
        let name = &registry.tag_by_id(*tag_id).name;
        if affinity.iter().any(|a| a == name) { return true; }
    }
    false
}

pub fn place_locations(
    engine: &CascadeEngine,
    registry: &TagRegistry,
    map_width: u32,
    map_height: u32,
    tile_biomes: &[Vec<TagId>],
    rng: &mut impl Rng,
) -> Vec<PlacedLocation> {
    let mut placed: Vec<PlacedLocation> = Vec::new();
    let mut next_id = 1usize;

    for pass in 1..=3 {
        let pass_types: Vec<&super::LocationType> = engine.location_types.iter()
            .filter(|lt| lt.pass == pass)
            .collect();

        let attempts = ((map_width * map_height) / 100).max(50) as u32;

        for lt in &pass_types {
            for _ in 0..attempts {
                let x = rng.random_range(5..map_width.saturating_sub(5).max(6));
                let y = rng.random_range(5..map_height.saturating_sub(5).max(6));
                let idx = (y * map_width + x) as usize;
                if idx >= tile_biomes.len() { continue; }

                let habitability = evaluate_tile_habitability(&tile_biomes[idx], registry);
                if habitability < lt.habitability_threshold { continue; }

                if !matches_biome_affinity(&tile_biomes[idx], &lt.biome_affinity, registry) {
                    continue;
                }

                if !respects_min_distance(x, y, &lt.id, lt.min_distance_from_same, &placed) {
                    continue;
                }

                if pass > 1 && !in_any_zone(x, y, &placed) { continue; }

                if rng.random::<f32>() > lt.weight / 100.0 { continue; }

                let faction = if lt.faction_affinity.is_empty() { None }
                else {
                    let idx = rng.random_range(0..lt.faction_affinity.len());
                    Some(lt.faction_affinity[idx].clone())
                };

                placed.push(PlacedLocation {
                    id: next_id,
                    location_type: lt.id.clone(),
                    name: lt.name.clone(),
                    x, y,
                    zone_radius: lt.zone_radius,
                    tags: lt.tags.clone(),
                    faction,
                });
                next_id += 1;
            }
        }
    }

    placed
}

pub fn location_at(locations: &[PlacedLocation], x: u32, y: u32) -> Option<&PlacedLocation> {
    for loc in locations.iter().rev() {
        let dx = (x as i32 - loc.x as i32).unsigned_abs();
        let dy = (y as i32 - loc.y as i32).unsigned_abs();
        let dist_sq = dx * dx + dy * dy;
        let zone_sq = loc.zone_radius * loc.zone_radius;
        if dist_sq <= zone_sq { return Some(loc); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn place_locations_returns_reasonable_count() {
        let (engine, registry) = setup();
        let mut rng = StdRng::seed_from_u64(42);
        let map_w = 100u32;
        let map_h = 100u32;
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();
        let walkable_id = registry.tag_id("WALKABLE").unwrap();
        let habitable_id = registry.tag_id("HABITABLE").unwrap();
        let tile_biomes: Vec<Vec<TagId>> = (0..(map_w * map_h) as usize)
            .map(|_| vec![grassland_id, walkable_id, habitable_id])
            .collect();

        let locations = place_locations(&engine, &registry, map_w, map_h, &tile_biomes, &mut rng);
        assert!(!locations.is_empty(), "should place at least some locations on fertile map");
        assert!(locations.len() < 50, "should not over-place on 100x100 map");
    }

    #[test]
    fn location_at_finds_nearby() {
        let locations = vec![PlacedLocation {
            id: 1, location_type: "city".to_string(), name: "Test City".to_string(),
            x: 50, y: 50, zone_radius: 20,
            tags: vec!["SETTLEMENT".to_string()], faction: Some("free_humanity".to_string()),
        }];
        assert!(location_at(&locations, 55, 55).is_some(), "should find location near center");
        assert!(location_at(&locations, 5, 5).is_none(), "should not find location far away");
    }

    #[test]
    fn evaluate_grassland_high_habitability() {
        let (_, registry) = setup();
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();
        let walkable_id = registry.tag_id("WALKABLE").unwrap();
        let score = evaluate_tile_habitability(&[grassland_id, walkable_id], &registry);
        assert!(score > 0.3, "grassland should have good habitability");
    }

    #[test]
    fn different_seeds_different_locations() {
        let (engine, registry) = setup();
        let map_w = 100u32;
        let map_h = 100u32;
        let grassland_id = registry.tag_id("BIOME_GRASSLAND").unwrap();
        let walkable_id = registry.tag_id("WALKABLE").unwrap();
        let habitable_id = registry.tag_id("HABITABLE").unwrap();
        let tile_biomes: Vec<Vec<TagId>> = (0..(map_w * map_h) as usize)
            .map(|_| vec![grassland_id, walkable_id, habitable_id])
            .collect();

        let mut rng1 = StdRng::seed_from_u64(42);
        let locs1 = place_locations(&engine, &registry, map_w, map_h, &tile_biomes, &mut rng1);
        let mut rng2 = StdRng::seed_from_u64(99);
        let locs2 = place_locations(&engine, &registry, map_w, map_h, &tile_biomes, &mut rng2);

        let any_different = locs1.iter().zip(locs2.iter())
            .any(|(a, b)| a.x != b.x || a.y != b.y || a.location_type != b.location_type);
        assert!(any_different || locs1.len() != locs2.len(), "seeds should produce different placements");
    }
}
