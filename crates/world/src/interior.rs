use bevy_ecs::prelude::*;
use game_tags::{TagId, TagRegistry, Tags, TagValue};
use std::collections::HashMap;
use crate::tile::{Tile, TilePos};
use crate::map::WorldMap;
use crate::biome::BiomeEnvironment;

/// Convert a generated DungeonMap into ECS tile entities with interior tags + environment.
/// Returns a WorldMap that can be inserted as a resource (swapping the overworld).
pub fn spawn_interior_tiles(
    world: &mut World,
    dungeon: &crate::dungeon::DungeonMap,
    interior_tags: &[TagId],
    environment: Option<&HashMap<String, u32>>,
    registry: &TagRegistry,
) -> WorldMap {
    let blocked_id = registry.tag_id("BLOCKED");
    let walkable_id = registry.tag_id("WALKABLE");
    let entrance_stair_id = registry.tag_id("ENTRANCE_STAIR");
    let deeper_stair_id = registry.tag_id("DEEPER_STAIR");

    // Build environment component from interior config
    let biome_env = environment.map(|env| BiomeEnvironment {
        light: env.get("light").copied().unwrap_or(50),
        temperature: env.get("temperature").copied().unwrap_or(50),
        moisture: env.get("moisture").copied().unwrap_or(50),
    }).unwrap_or(BiomeEnvironment { light: 50, temperature: 50, moisture: 50 });

    let mut tile_entities = Vec::with_capacity(dungeon.tiles.len());

    for dt in &dungeon.tiles {
        let mut tags = Tags::new(registry.tag_count());

        for &tag_id in interior_tags {
            tags.add_tag(tag_id, TagValue::None, registry);
        }

        match dt.tile_type {
            crate::dungeon::DungeonTileType::Wall => {
                if let Some(id) = blocked_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            crate::dungeon::DungeonTileType::Floor | crate::dungeon::DungeonTileType::Corridor => {
                if let Some(id) = walkable_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            crate::dungeon::DungeonTileType::EntranceStair => {
                if let Some(id) = walkable_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
                if let Some(id) = entrance_stair_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
            crate::dungeon::DungeonTileType::DeeperStair => {
                if let Some(id) = walkable_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
                if let Some(id) = deeper_stair_id {
                    tags.add_tag(id, TagValue::None, registry);
                }
            }
        }

        let tile = Tile {
            pos: TilePos::new(dt.pos.x, dt.pos.y),
            elevation: 0.0,
            moisture: 0.0,
            temperature: 0.0,
            biome_name: "interior".to_string(),
            glyph: dt.tile_type.glyph(),
            color: match dt.tile_type {
                crate::dungeon::DungeonTileType::Wall => (80, 70, 90),
                crate::dungeon::DungeonTileType::Floor => (50, 50, 50),
                crate::dungeon::DungeonTileType::Corridor => (60, 60, 60),
                crate::dungeon::DungeonTileType::EntranceStair => (100, 100, 100),
                crate::dungeon::DungeonTileType::DeeperStair => (120, 120, 80),
            },
        };

        let entity = world.spawn((tile, tags, biome_env)).id();
        tile_entities.push(entity);
    }

    WorldMap {
        width: dungeon.width,
        height: dungeon.height,
        depth: 1,
        current_z: 0,
        seed: crate::seed::WorldSeed(dungeon.seed),
        tiles: tile_entities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DungeonConfig, DungeonType, generate_dungeon};
    use game_tags::{TagRegistryBuilder, Exclusivity};

    fn make_registry() -> TagRegistry {
        let mut builder = TagRegistryBuilder::new();
        let terrain = builder.add_archetype("terrain", "Terrain", Exclusivity::Any);
        let trait_arch = builder.add_archetype("trait", "Trait", Exclusivity::Any);
        builder.add_tag(terrain, "BLOCKED", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(terrain, "WALKABLE", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(terrain, "ENTRANCE_STAIR", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(terrain, "DEEPER_STAIR", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(trait_arch, "INDOORS", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(trait_arch, "BLOCKS_WEATHER", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn test_spawn_interior_tiles_creates_world_map() {
        let config = DungeonConfig { width: 10, height: 10, min_room_size: 3, max_room_size: 6, target_room_count: 5, corridor_width: 1, enemy_density: 0.3 };
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let registry = make_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let indoors_id = registry.tag_id("INDOORS").unwrap();
        let blocks_weather_id = registry.tag_id("BLOCKS_WEATHER").unwrap();
        let interior_tags = vec![indoors_id, blocks_weather_id];

        let wm = spawn_interior_tiles(&mut world, &dungeon, &interior_tags, None, &registry);

        assert_eq!(wm.width, 10);
        assert_eq!(wm.height, 10);
        assert_eq!(wm.tiles.len(), 100);
    }

    #[test]
    fn test_interior_tiles_have_correct_tags() {
        let config = DungeonConfig { width: 10, height: 10, min_room_size: 3, max_room_size: 6, target_room_count: 5, corridor_width: 1, enemy_density: 0.3 };
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let registry = make_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let indoors_id = registry.tag_id("INDOORS").unwrap();
        let blocked_id = registry.tag_id("BLOCKED").unwrap();
        let walkable_id = registry.tag_id("WALKABLE").unwrap();

        let interior_tags = vec![indoors_id];
        let wm = spawn_interior_tiles(&mut world, &dungeon, &interior_tags, None, &registry);

        let entrance_idx = dungeon.entrance.1 as usize * wm.width as usize + dungeon.entrance.0 as usize;
        let entrance_entity = wm.tiles[entrance_idx];
        if let Some(tags) = world.get::<Tags>(entrance_entity) {
            assert!(tags.has(indoors_id), "entrance should have INDOORS tag");
        }

        let walked_wall = world.query::<(&Tile, &Tags)>().iter(&world)
            .filter(|(tile, tags)| {
                tile.glyph == '#' && tags.has(blocked_id)
            })
            .count();
        assert!(walked_wall > 0, "at least some walls should be blocked");
    }
}
