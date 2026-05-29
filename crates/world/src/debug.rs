use bevy_ecs::prelude::World;

use crate::map::WorldMap;
use crate::tile::Tile;

pub fn render_map_ascii(world: &mut World) -> String {
    let map = world.resource::<WorldMap>().clone();
    let mut query = world.query::<&Tile>();

    let mut output = String::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = crate::tile::TilePos::new(x, y);
            let entity = map.get_unchecked(pos);
            if let Ok(tile) = query.get(world, entity) {
                let (r, g, b) = tile.color;
                output.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                output.push(tile.glyph);
            } else {
                output.push(' ');
            }
        }
        output.push_str("\x1b[0m\n");
    }
    output
}

pub fn render_map_plain(world: &mut World) -> String {
    let map = world.resource::<WorldMap>().clone();
    let mut query = world.query::<&Tile>();

    let mut output = String::new();
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = crate::tile::TilePos::new(x, y);
            let entity = map.get_unchecked(pos);
            if let Ok(tile) = query.get(world, entity) {
                output.push(tile.glyph);
            } else {
                output.push(' ');
            }
        }
        output.push('\n');
    }
    output
}

pub fn biome_summary(world: &mut World) -> String {
    let map = world.resource::<WorldMap>().clone();
    let mut query = world.query::<&Tile>();
    let mut counts = std::collections::HashMap::new();

    for &entity in &map.tiles {
        if let Ok(tile) = query.get(world, entity) {
            *counts.entry(tile.biome_name.clone()).or_insert(0usize) += 1;
        }
    }

    let mut entries: Vec<_> = counts.into_iter().collect();
    entries.sort_by_key(|b| std::cmp::Reverse(b.1));

    let mut output = format!("Map: {}x{} ({} tiles)\n", map.width, map.height, map.tiles.len());
    output.push_str("Biome distribution:\n");
    for (biome, count) in &entries {
        let pct = (*count as f64 / map.tiles.len() as f64) * 100.0;
        output.push_str(&format!("  {:<25} {:>5} ({:.1}%)\n", biome, count, pct));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::{WorldConfig, WorldPlugin};
    use crate::loader::{load_biome_rules, load_world_config_str};
    use crate::seed::WorldSeed;
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");
    const WORLD_TOML: &str = r#"
width = 10
height = 10
latitude_weight = 0.4

[elevation]
name = "elevation"
frequency = 0.05
octaves = 4
persistence = 0.5
lacunarity = 2.0

[moisture]
name = "moisture"
frequency = 0.05
octaves = 4
persistence = 0.5
lacunarity = 2.0

[temperature]
name = "temperature"
frequency = 0.05
octaves = 4
persistence = 0.6
lacunarity = 2.0
"#;
    const BIOME_TOML: &str = r#"
[[rule]]
biome = "OCEAN_DEEP"
glyph = "~"
color = [0, 0, 128]
elevation = [0.0, 0.20]
priority = 100
tags = ["BIOME_DEEP_OCEAN"]

[[rule]]
biome = "GRASSLAND"
glyph = "."
color = [144, 238, 144]
elevation = [0.20, 1.0]
priority = 1
tags = ["BIOME_GRASSLAND"]
"#;

    fn make_test_world() -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry);
        world.insert_resource(WorldConfig {
            seed: WorldSeed::from_value(42),
            width: 10,
            height: 10,
        });
        let gen_config = load_world_config_str(WORLD_TOML).unwrap();
        let classifier = load_biome_rules(BIOME_TOML).unwrap();
        WorldPlugin::generate_and_spawn(&mut world, gen_config, classifier);
        world
    }

    #[test]
    fn test_render_plain() {
        let mut world = make_test_world();
        let output = render_map_plain(&mut world);
        assert_eq!(output.lines().count(), 10);
    }

    #[test]
    fn test_biome_summary() {
        let mut world = make_test_world();
        let summary = biome_summary(&mut world);
        assert!(summary.contains("Map: 10x10"));
        assert!(summary.contains("OCEAN_DEEP") || summary.contains("GRASSLAND"));
    }
}
