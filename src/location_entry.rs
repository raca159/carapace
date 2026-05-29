use bevy_ecs::world::World;
use game_core::components::{Position, Player, OverworldEntity, Health, Creature, Name, WeatherSensitive};
use game_core::emotion::NpcEmotionalState;
use game_core::turn::BehaviorState;
use game_core::Glyph;
use game_tags::{TagId, TagRegistry, Tags};
use game_world::cascade::{CascadeEngine, PlacedLocation};
use game_world::dungeon::{DungeonConfig, DungeonType, generate_dungeon, dungeon_spawn_positions, MapLayer, ActiveInterior, DungeonMap};
use game_world::interior::spawn_interior_tiles;
use game_world::WorldMap;
use rand::SeedableRng;

pub fn enter_location(world: &mut World, location: &PlacedLocation) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    let interior_def = {
        let cascade = match world.get_resource::<CascadeEngine>() {
            Some(c) => c,
            None => return,
        };
        let loc_type = cascade.location_types.iter()
            .find(|lt| lt.id == location.location_type)
            .and_then(|lt| lt.interior.as_ref())
            .cloned();
        match loc_type {
            Some(d) => d,
            None => return,
        }
    };

    let (width, height) = match interior_def.scale {
        Some([w, h]) => (w, h),
        None => (30, 40),
    };

    let dungeon_type = match location.location_type.as_str() {
        "dungeon" => DungeonType::Crypt,
        "cave" => DungeonType::CaveSystem,
        "ruin" => DungeonType::Sewer,
        _ => DungeonType::Crypt,
    };

    let config = DungeonConfig {
        width,
        height,
        min_room_size: 3,
        max_room_size: 6,
        target_room_count: 8,
        ..Default::default()
    };
    let seed = world.get_resource::<WorldMap>()
        .map(|wm| wm.seed.0)
        .unwrap_or(42);
    let dungeon = generate_dungeon(&config, dungeon_type, seed.wrapping_add(1));

    let interior_tag_ids: Vec<TagId> = interior_def.tags.iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();

    let saved_world_map = world.get_resource::<WorldMap>().cloned();
    let saved_player_pos = {
        let mut q = world.query_filtered::<&Position, bevy_ecs::query::With<Player>>();
        q.iter(world).next().map(|p| (p.x, p.y)).unwrap_or((0, 0))
    };

    let current_entities: Vec<bevy_ecs::entity::Entity> = {
        let mut q = world.query::<bevy_ecs::entity::Entity>();
        q.iter(world).collect()
    };
    for entity in &current_entities {
        world.entity_mut(*entity).insert(OverworldEntity);
    }

    let interior_map = spawn_interior_tiles(world, &dungeon, &interior_tag_ids, Some(&interior_def.environment), &registry);

    place_dungeon_traps(world, &dungeon, &registry);

    spawn_interior_entities(world, &dungeon, &interior_def.spawn_rules, &registry);

    if let Some(owm) = saved_world_map {
        if let Some(mut ml) = world.get_resource_mut::<MapLayer>() {
            ml.active_interior = Some(ActiveInterior {
                location_id: location.id,
                location_type: location.location_type.clone(),
                interior_tags: interior_tag_ids,
                environment: interior_def.environment,
                depth_range: interior_def.depth_range,
                saved_world_map: owm,
                saved_player_pos,
            });
        }
    }
    world.insert_resource(interior_map);

    let entrance = dungeon.entrance;
    for mut pos in world.query_filtered::<&mut Position, bevy_ecs::query::With<Player>>().iter_mut(world) {
        pos.x = entrance.0;
        pos.y = entrance.1;
    }
}

fn spawn_interior_entities(world: &mut World, dungeon: &game_world::dungeon::DungeonMap, spawn_rules: &[String], registry: &TagRegistry) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(dungeon.seed);
    let spawn_positions = dungeon_spawn_positions(dungeon, &mut rng);
    if spawn_positions.is_empty() { return; }

    for rule_name in spawn_rules {
        match rule_name.as_str() {
            "hostile" => {
                for &pos in &spawn_positions {
                    let mut tags = Tags::new(registry.tag_count());
                    if let Some(id) = registry.tag_id("AGGRESSIVE") {
                        tags.add_tag(id, game_tags::TagValue::None, registry);
                    }
                    world.spawn((
                        Position { x: pos.0, y: pos.1, z: 0 },
                        Glyph { char: 'g', color: (200, 50, 50) },
                        Health { current: 20, max: 20 },
                        tags,
                        Name("Dungeon Creature".to_string()),
                        Creature,
                        WeatherSensitive,
                        BehaviorState { home_pos: Some(Position { x: pos.0, y: pos.1, z: 0 }) },
                        NpcEmotionalState::default(),
                    ));
                }
            }
            "loot" => {
                game_world::loot::place_dungeon_chests(world, dungeon, 0);
            }
            _ => {}
        }
    }

    let dungeon_type = dungeon.dungeon_type.name().to_string();
    let ctx = game_world::PopulateContext {
        dungeon_type: Some(dungeon_type),
        depth: Some(0),
        dungeon_seed: Some(dungeon.seed),
    };
    game_world::populate_inventories(world, Some(&ctx));
}

pub fn exit_location(world: &mut World) {
    let (saved_map, saved_pos) = {
        let map_layer = match world.get_resource::<MapLayer>() {
            Some(ml) => ml,
            None => return,
        };
        let interior = match &map_layer.active_interior {
            Some(i) => i,
            None => return,
        };
        (interior.saved_world_map.clone(), interior.saved_player_pos)
    };

    for mut pos in world.query_filtered::<&mut Position, bevy_ecs::query::With<Player>>().iter_mut(world) {
        pos.x = saved_pos.0;
        pos.y = saved_pos.1;
    }

    let interior_entities: Vec<bevy_ecs::entity::Entity> = {
        let mut q = world.query::<bevy_ecs::entity::Entity>();
        q.iter(world).filter(|e| world.get::<OverworldEntity>(*e).is_none()).collect()
    };
    for entity in interior_entities {
        world.despawn(entity);
    }

    world.insert_resource(saved_map);

    if let Some(mut ml) = world.get_resource_mut::<MapLayer>() {
        ml.active_interior = None;
    }
}

pub fn enter_next_depth(world: &mut World) {
    let map_layer = match world.get_resource::<MapLayer>() {
        Some(ml) => ml.clone(),
        None => return,
    };
    let interior = match &map_layer.active_interior {
        Some(i) => i,
        None => return,
    };

    // Check depth limit from interior def
    if let Some([_min, max]) = interior.depth_range {
        if map_layer.depth >= max {
            if let Some(mut msg) = world.get_resource_mut::<game_core::MessageLog>() {
                msg.messages.push("You have reached the deepest level.".to_string());
            }
            return;
        }
    }

    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    // Use same scale as current interior map
    let (width, height) = world.get_resource::<WorldMap>()
        .map(|wm| (wm.width, wm.height))
        .unwrap_or((30, 40));

    // Derive DungeonType from location type
    let dungeon_type = match interior.location_type.as_str() {
        "dungeon" => DungeonType::Crypt,
        "cave" => DungeonType::CaveSystem,
        "ruin" => DungeonType::Sewer,
        _ => DungeonType::Crypt,
    };

    let config = DungeonConfig {
        width,
        height,
        min_room_size: 3,
        max_room_size: 6,
        target_room_count: 8,
        ..Default::default()
    };
    let seed = world.get_resource::<WorldMap>()
        .map(|wm| wm.seed.0.wrapping_add(map_layer.depth as u64))
        .unwrap_or(42);
    let dungeon = generate_dungeon(&config, dungeon_type, seed.wrapping_add(1));

    let interior_entities: Vec<bevy_ecs::entity::Entity> = {
        let mut q = world.query::<bevy_ecs::entity::Entity>();
        q.iter(world).filter(|e| world.get::<OverworldEntity>(*e).is_none()).collect()
    };
    for entity in interior_entities {
        world.despawn(entity);
    }

    let interior_map = spawn_interior_tiles(world, &dungeon, &interior.interior_tags, Some(&interior.environment), &registry);

    place_dungeon_traps(world, &dungeon, &registry);

    let mut rng = rand::rngs::StdRng::seed_from_u64(dungeon.seed);
    let spawn_positions = dungeon_spawn_positions(&dungeon, &mut rng);
    for &pos in &spawn_positions {
        let mut tags = Tags::new(registry.tag_count());
        if let Some(id) = registry.tag_id("AGGRESSIVE") {
            tags.add_tag(id, game_tags::TagValue::None, &registry);
        }
        world.spawn((
            Position { x: pos.0, y: pos.1, z: 0 },
            Glyph { char: 'g', color: (200, 50, 50) },
            Health { current: 20, max: 20 },
            tags,
            Name("Dungeon Creature".to_string()),
            Creature,
            WeatherSensitive,
            BehaviorState { home_pos: Some(Position { x: pos.0, y: pos.1, z: 0 }) },
            NpcEmotionalState::default(),
        ));
    }

    // Place chests and populate inventories for the new depth
    game_world::loot::place_dungeon_chests(world, &dungeon, map_layer.depth + 1);
    let pop_ctx = game_world::PopulateContext {
        dungeon_type: Some(dungeon.dungeon_type.name().to_string()),
        depth: Some(map_layer.depth + 1),
        dungeon_seed: Some(dungeon.seed),
    };
    game_world::populate_inventories(world, Some(&pop_ctx));

    world.insert_resource(interior_map);

    let entrance = dungeon.entrance;
    for mut pos in world.query_filtered::<&mut Position, bevy_ecs::query::With<Player>>().iter_mut(world) {
        pos.x = entrance.0;
        pos.y = entrance.1;
    }

    if let Some(mut ml) = world.get_resource_mut::<MapLayer>() {
        ml.depth += 1;
    }
}

/// Place traps on a subset of floor tiles in dungeon rooms.
/// Uses dungeon seed for deterministic placement.
fn place_dungeon_traps(world: &mut World, dungeon: &DungeonMap, _registry: &TagRegistry) {
    use rand::Rng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(dungeon.seed.wrapping_add(0xDEAD));
    let trap_types = [
        game_core::traps::TrapType::PoisonDart,
        game_core::traps::TrapType::ExplosiveRune,
        game_core::traps::TrapType::SummonTrap,
        game_core::traps::TrapType::AlarmTrap,
    ];
    for room in &dungeon.rooms {
        let area = room.area() as f32;
        let trap_count = (area * 0.08 * rng.random::<f32>()).ceil() as u32;
        for _ in 0..trap_count {
            let x = room.x + rng.random_range(0..room.w);
            let y = room.y + rng.random_range(0..room.h);
            let idx = (y * dungeon.width + x) as usize;
            if let Some(tile) = dungeon.tiles.get(idx)
                && tile.tile_type == game_world::dungeon::DungeonTileType::Floor
            {
                // Skip entrance tile
                if x == dungeon.entrance.0 && y == dungeon.entrance.1 {
                    continue;
                }
                let trap_type = trap_types[rng.random_range(0..trap_types.len())];
                let trap = game_core::traps::Trap::new(trap_type)
                    .with_damage(if trap_type == game_core::traps::TrapType::ExplosiveRune { 15 } else if trap_type == game_core::traps::TrapType::PoisonDart { 8 } else { 0 });
                world.spawn((trap, game_core::components::Position { x, y, z: 0 }));
            }
        }
    }
}
