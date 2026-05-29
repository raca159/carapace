use bevy_ecs::prelude::{Entity, Resource, World};

use game_tags::{TagRegistry, TagValue, Tags};

use crate::biome::{BiomeClassifier, BiomeEnvironment};
use crate::latitude::apply_latitude_modifier;
use crate::loader::WorldGenConfig;
use crate::map::WorldMap;
use crate::noise_gen::NoiseGenerator;
use crate::seed::WorldSeed;
use crate::tile::{Tile, TilePos};

#[derive(Debug, Clone, Resource)]
pub struct WorldConfig {
    pub seed: WorldSeed,
    pub width: u32,
    pub height: u32,
}

impl WorldConfig {
    pub fn default_mvp() -> Self {
        Self {
            seed: WorldSeed::from_value(42),
            width: 200,
            height: 200,
        }
    }
}

#[derive(Resource)]
pub struct WorldGenResources {
    pub gen_config: WorldGenConfig,
    pub biome_classifier: BiomeClassifier,
}

pub fn generate_world(world: &mut World) {
    let (seed, gen_config, biome_classifier) = {
        let config = world.resource::<WorldConfig>();
        let seed = config.seed;
        let width = config.width;
        let height = config.height;

        let gen_res = world.resource::<WorldGenResources>();
        let gen_config = gen_res.gen_config.clone();
        let biome_classifier = gen_res.biome_classifier.clone();

        let mut gen_config = gen_config;
        gen_config.width = width;
        gen_config.height = height;

        (seed, gen_config, biome_classifier)
    };

    let registry = world.resource::<TagRegistry>().clone();
    let width = gen_config.width;
    let height = gen_config.height;

    let noise_gen = NoiseGenerator::new(seed);

    let elevation = noise_gen.generate_layer(&gen_config.elevation, width, height, 0);
    let moisture = noise_gen.generate_layer(&gen_config.moisture, width, height, 1000);
    let mut temperature = noise_gen.generate_layer(&gen_config.temperature, width, height, 2000);

    apply_latitude_modifier(&mut temperature, width, height, gen_config.latitude_weight);

    let tag_count = registry.tag_count();
    let mut tile_entities: Vec<Entity> = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let elev = elevation[idx];
            let moist = moisture[idx];
            let temp = temperature[idx];

            let rule = biome_classifier
                .classify(elev, moist, temp)
                .expect("fallback biome rule should always match");

            let tile = Tile {
                pos: TilePos::new(x, y),
                elevation: elev,
                moisture: moist,
                temperature: temp,
                biome_name: rule.biome.clone(),
                glyph: rule.glyph,
                color: (rule.color[0], rule.color[1], rule.color[2]),
            };

            let mut tags = Tags::new(tag_count);
            for tag_name in &rule.tags {
                if let Some(tag_id) = registry.tag_id(tag_name) {
                    tags.add_tag(tag_id, TagValue::None, &registry);
                }
            }

            let entity = world.spawn((tile, tags, rule.environment)).id();
            tile_entities.push(entity);
        }
    }

    world.insert_resource(WorldMap {
        width,
        height,
        depth: 1,
        current_z: 0,
        seed,
        tiles: tile_entities,
    });
}

pub struct WorldPlugin;

impl WorldPlugin {
    pub fn generate_and_spawn(
        world: &mut World,
        gen_config: WorldGenConfig,
        biome_classifier: BiomeClassifier,
    ) {
        world.insert_resource(WorldGenResources {
            gen_config,
            biome_classifier,
        });
        generate_world(world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{load_biome_rules, load_world_config_str};
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    const WORLD_TOML: &str = r#"
width = 50
height = 50
latitude_weight = 0.4

[elevation]
name = "elevation"
frequency = 0.02
octaves = 4
persistence = 0.5
lacunarity = 2.0

[moisture]
name = "moisture"
frequency = 0.03
octaves = 4
persistence = 0.5
lacunarity = 2.0

[temperature]
name = "temperature"
frequency = 0.01
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
biome = "OCEAN_SHALLOW"
glyph = "~"
color = [0, 0, 255]
elevation = [0.20, 0.30]
priority = 90
tags = ["BIOME_OCEAN"]

[[rule]]
biome = "BEACH"
glyph = "."
color = [255, 215, 0]
elevation = [0.30, 0.35]
priority = 80
tags = ["BIOME_BEACH"]

[[rule]]
biome = "MOUNTAIN_PEAK"
glyph = "^"
color = [64, 64, 64]
elevation = [0.85, 1.0]
priority = 70
tags = ["BIOME_MOUNTAIN_PEAK"]

[[rule]]
biome = "MOUNTAIN"
glyph = "^"
color = [128, 128, 128]
elevation = [0.70, 0.85]
priority = 60
tags = ["BIOME_MOUNTAIN"]

[[rule]]
biome = "DESERT"
glyph = "."
color = [237, 201, 175]
elevation = [0.35, 0.70]
temperature = [0.7, 1.0]
moisture = [0.0, 0.3]
priority = 50
tags = ["BIOME_DESERT"]

[[rule]]
biome = "TROPICAL_FOREST"
glyph = "T"
color = [0, 100, 0]
elevation = [0.35, 0.70]
temperature = [0.7, 1.0]
moisture = [0.6, 1.0]
priority = 48
tags = ["BIOME_TROPICAL_FOREST"]

[[rule]]
biome = "GRASSLAND"
glyph = "."
color = [144, 238, 144]
elevation = [0.35, 0.70]
priority = 1
tags = ["BIOME_GRASSLAND"]

[[rule]]
biome = "SWAMP"
glyph = "~"
color = [85, 107, 47]
elevation = [0.30, 0.40]
moisture = [0.7, 1.0]
temperature = [0.5, 1.0]
priority = 55
tags = ["BIOME_SWAMP"]
"#;

    fn setup_world(seed: u64) -> World {
        let mut world = World::new();

        let registry = load_tag_registry(TAGS_TOML).expect("tags should load");
        world.insert_resource(registry);

        let config = WorldConfig {
            seed: WorldSeed::from_value(seed),
            width: 50,
            height: 50,
        };
        world.insert_resource(config);

        let gen_config = load_world_config_str(WORLD_TOML).unwrap();
        let biome_classifier = load_biome_rules(BIOME_TOML).unwrap();

        WorldPlugin::generate_and_spawn(&mut world, gen_config, biome_classifier);
        world
    }

    #[test]
    fn test_generates_map() {
        let world = setup_world(42);
        let map = world.resource::<WorldMap>();
        assert_eq!(map.width, 50);
        assert_eq!(map.height, 50);
        assert_eq!(map.tiles.len(), 2500);
    }

    #[test]
    fn test_tile_components() {
        let mut world = setup_world(42);
        let map = world.resource::<WorldMap>().clone();

        let mut query = world.query::<&Tile>();
        for &entity in &map.tiles {
            let tile = query.get(&world, entity).unwrap();
            assert!(tile.elevation >= 0.0 && tile.elevation <= 1.0);
            assert!(tile.moisture >= 0.0 && tile.moisture <= 1.0);
            assert!(tile.temperature >= 0.0 && tile.temperature <= 1.0);
        }
    }

    #[test]
    fn test_determinism() {
        let mut world1 = setup_world(99999);
        let mut world2 = setup_world(99999);

        let map1 = world1.resource::<WorldMap>().clone();
        let map2 = world2.resource::<WorldMap>().clone();

        let mut q1 = world1.query::<&Tile>();
        let mut q2 = world2.query::<&Tile>();

        for i in 0..map1.tiles.len() {
            let t1 = q1.get(&world1, map1.tiles[i]).unwrap();
            let t2 = q2.get(&world2, map2.tiles[i]).unwrap();
            assert_eq!(t1.elevation, t2.elevation, "tile {} elevation mismatch", i);
            assert_eq!(t1.glyph, t2.glyph, "tile {} glyph mismatch", i);
        }
    }

    #[test]
    fn test_different_seeds_different_worlds() {
        let mut world1 = setup_world(1);
        let mut world2 = setup_world(2);

        let map1 = world1.resource::<WorldMap>().clone();
        let map2 = world2.resource::<WorldMap>().clone();

        let mut q1 = world1.query::<&Tile>();
        let mut q2 = world2.query::<&Tile>();

        let mut different = 0;
        for i in 0..map1.tiles.len() {
            let t1 = q1.get(&world1, map1.tiles[i]).unwrap();
            let t2 = q2.get(&world2, map2.tiles[i]).unwrap();
            if (t1.elevation - t2.elevation).abs() > 0.001 {
                different += 1;
            }
        }
        assert!(
            different > map1.tiles.len() / 2,
            "expected >50% different elevations, got {}/{}",
            different,
            map1.tiles.len()
        );
    }

    #[test]
    fn test_tags_assigned() {
        let mut world = setup_world(42);
        let map = world.resource::<WorldMap>().clone();

        let mut query = world.query::<&Tags>();
        let mut tiles_with_biome = 0;
        for &entity in &map.tiles {
            let tags = query.get(&world, entity).unwrap();
            if tags.count() > 0 {
                tiles_with_biome += 1;
            }
        }
        assert_eq!(tiles_with_biome, map.tiles.len(), "all tiles should have tags");
    }
}
