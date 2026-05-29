use std::time::Instant;

use bevy_ecs::prelude::*;
use rand::SeedableRng;
use game_core::{
    Camera, Creature, Glyph, Health, Inventory, Item, Name, Player, Position,
    ColorTheme,
};
use game_world::{Tile, TilePos, WorldMap, WorldSeed};

fn build_test_world(width: u32, height: u32, entity_count: usize) -> World {
    let mut world = World::new();

    let seed = WorldSeed::from_value(12345);
    let map = WorldMap {
        width,
        height,
        depth: 1,
        current_z: 0,
        seed,
        tiles: Vec::new(),
    };

    world.insert_resource(map);
    world.insert_resource(ColorTheme::default());

    let w = width as usize;
    let total = w * height as usize;
    let mut tile_entities = Vec::with_capacity(total);
    for i in 0..total {
        let x = (i % w) as u32;
        let y = (i / w) as u32;
        let entity = world.spawn((
            Tile {
                pos: TilePos::new(x, y),
                glyph: '.',
                color: (100, 100, 100),
                elevation: 0.5,
                moisture: 0.3,
                temperature: 0.5,
                biome_name: "test".to_string(),
            },
            Position { x, y, z: 0 },
        )).id();
        tile_entities.push(entity);
    }

    world.resource_mut::<WorldMap>().tiles = tile_entities;

    use rand::Rng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..entity_count {
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        world.spawn((
            Position { x, y, z: 0 },
            Glyph { char: 'e', color: (200, 50, 50) },
            Creature,
            Name(format!("Entity_{}", rng.random_range(0..1000))),
            Health { current: 10, max: 10 },
        ));
    }

    let player_x = width / 2;
    let player_y = height / 2;
    world.spawn((
        Player,
        Position { x: player_x, y: player_y, z: 0 },
        Glyph { char: '@', color: (255, 255, 0) },
        Name("Player".to_string()),
        Health { current: 100, max: 100 },
        Inventory { items: Vec::new(), capacity: 20 },
    ));

    world
}

// TODO(phase3): Restore with rewritten snapshot module
// fn project_tiles_bench(ecs_world: &mut World, camera: &Camera, screen_width: u16, screen_height: u16) -> Vec<TileSnapshot> { ... }
// fn project_entities_bench(ecs_world: &mut World, camera: &Camera, screen_width: u16, screen_height: u16) -> Vec<EntitySnapshot> { ... }
// fn full_projection_bench(world: &mut World, camera: &Camera, width: u16, height: u16) -> WorldRenderData { ... }
// fn bench_projection(...) { ... }
// #[test] fn bench_viewport_80x25() { ... }
// #[test] fn bench_viewport_120x40() { ... }
// #[test] fn bench_viewport_240x80() { ... }
// #[test] fn bench_world_500_entities() { ... }
// #[test] fn bench_tile_cache_hit() { ... }
