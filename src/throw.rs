use bevy_ecs::prelude::World;
use bevy_ecs::query::With;

use game_core::{Creature, EventBus, GameEvent, Health, Inventory, Name, Player, Position};
#[cfg(test)]
use game_core::MessageLog;
use game_tags::TagRegistry;
use game_world::{Faction, ReputationTracker, PLAYER_FACTION_ID, REP_KILL_PENALTY, TilePos, WorldMap};

#[allow(dead_code)]
const MAX_THROW_RANGE: u32 = 8;

#[allow(dead_code)]
pub fn start_throw(
    ecs_world: &mut World,
    cursor: usize,
) -> Option<bevy_ecs::entity::Entity> {
    let player_entity = {
        let mut pq = ecs_world.query_filtered::<bevy_ecs::entity::Entity, With<Player>>();
        pq.single(ecs_world).ok()
    };
    let player_entity = player_entity?;

    let item_entity = {
        let inv = ecs_world.get::<Inventory>(player_entity);
        match inv {
            Some(inv) if cursor < inv.items.len() => inv.items[cursor],
            _ => return None,
        }
    };

    let registry = ecs_world.resource::<TagRegistry>().clone();
    let throwable_id = registry.tag_id("THROWABLE");

    let item_tags = ecs_world.get::<game_tags::Tags>(item_entity).cloned();
    let item_tags = item_tags?;

    let is_throwable = throwable_id.is_some_and(|id| item_tags.has(id));

    if !is_throwable {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("That item cannot be thrown.".to_string()));
        }
        return None;
    }

    Some(item_entity)
}

#[allow(dead_code)]
pub fn execute_throw(
    ecs_world: &mut World,
    item_entity: bevy_ecs::entity::Entity,
    target_x: u32,
    target_y: u32,
) -> bool {
    use bevy_ecs::entity::Entity;
    use rand::Rng;

    let player_entity = {
        let mut pq = ecs_world.query_filtered::<Entity, With<Player>>();
        match pq.single(ecs_world).ok() {
            Some(e) => e,
            None => return false,
        }
    };

    let player_pos = match ecs_world.get::<Position>(player_entity) {
        Some(p) => *p,
        None => return false,
    };

    if player_pos.x == target_x && player_pos.y == target_y {
        return false;
    }

    let dist = ((target_x as i32 - player_pos.x as i32).unsigned_abs())
        .max((target_y as i32 - player_pos.y as i32).unsigned_abs());

    if dist > MAX_THROW_RANGE {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("That is too far to throw.".to_string()));
        }
        return false;
    }

    let registry = ecs_world.resource::<TagRegistry>().clone();
    let blocked_id = registry.tag_id("BLOCKED");

    let item_name = ecs_world
        .get::<Name>(item_entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "?".to_string());

    let map = match ecs_world.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return false,
    };

    let width = map.width;
    let height = map.height;

    if width == 0 || height == 0 {
        return false;
    }

    // Trace path from player to target
    let dx = (target_x as i32 - player_pos.x as i32).signum();
    let dy = (target_y as i32 - player_pos.y as i32).signum();

    let mut cur_x = player_pos.x as i32;
    let mut cur_y = player_pos.y as i32;
    let mut hit_entity: Option<Entity> = None;
    let mut final_x: u32 = player_pos.x;
    let mut final_y: u32 = player_pos.y;

    for _step in 0..MAX_THROW_RANGE {
        let next_x = cur_x + dx;
        let next_y = cur_y + dy;

        if next_x < 0 || next_y < 0 || next_x >= width as i32 || next_y >= height as i32 {
            break;
        }

        let nx = next_x as u32;
        let ny = next_y as u32;

        let target_pos = TilePos::new(nx, ny);
        if let Some(tile_entity) = map.get(target_pos)
            && let Some(blocked) = blocked_id
        {
            let mut tags_query = ecs_world.query::<&game_tags::Tags>();
            if let Ok(tags) = tags_query.get(ecs_world, tile_entity)
                && tags.has(blocked)
            {
                break;
            }
        }

        let creatures_at_target: Vec<Entity> = {
            let mut creature_query = ecs_world.query_filtered::<(Entity, &Position), With<Creature>>();
            creature_query
                .iter(ecs_world)
                .filter(|(_, pos)| pos.x == nx && pos.y == ny)
                .map(|(e, _)| e)
                .collect()
        };

        if !creatures_at_target.is_empty() {
            hit_entity = Some(creatures_at_target[0]);
            final_x = nx;
            final_y = ny;
            break;
        }

        if nx == target_x && ny == target_y {
            final_x = nx;
            final_y = ny;
            break;
        }

        cur_x = next_x;
        cur_y = next_y;
        final_x = nx;
        final_y = ny;
    }

    {
        let mut inv = ecs_world.get_mut::<Inventory>(player_entity).unwrap();
        inv.items.retain(|&e| e != item_entity);
    }

    let hit_chance = (0.70f64 - 0.05f64 * dist as f64).clamp(0.05, 0.95);
    let roll: f64 = rand::rng().random();

    // Check if we actually hit the target creature
    if let Some(hit) = hit_entity {
        if roll > hit_chance {
            // Miss - scatter to adjacent tile
            let scatter_dir: u32 = rand::rng().random_range(0..8);
            let (sx, sy) = match scatter_dir {
                0 => (target_x.wrapping_sub(1), target_y.wrapping_sub(1)),
                1 => (target_x, target_y.wrapping_sub(1)),
                2 => (target_x.wrapping_add(1), target_y.wrapping_sub(1)),
                3 => (target_x.wrapping_sub(1), target_y),
                4 => (target_x.wrapping_add(1), target_y),
                5 => (target_x.wrapping_sub(1), target_y.wrapping_add(1)),
                6 => (target_x, target_y.wrapping_add(1)),
                _ => (target_x.wrapping_add(1), target_y.wrapping_add(1)),
            };
            final_x = sx.min(width - 1);
            final_y = sy.min(height - 1);

            let miss_target_name = ecs_world
                .get::<Name>(hit)
                .map(|n| n.0.clone())
                .unwrap_or_default();
            if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message(format!(
                    "You throw the {} at {} but miss! It scatters to ({}, {}).",
                    item_name, miss_target_name, final_x, final_y,
                )));
            }

            ecs_world.entity_mut(item_entity).insert(Position { x: final_x, y: final_y, z: 0 });
            return true;
        }

        let throw_damage = calc_throw_damage(ecs_world, item_entity, &registry);
        let creature_name = ecs_world
            .get::<Name>(hit)
            .map(|n| n.0.clone())
            .unwrap_or_default();

        // Factor in creature armor
        let armor_protection = ecs_world
            .get::<game_core::Equipment>(hit)
            .map(|eq| crate::equipment::calc_armor_protection(eq, ecs_world, &registry))
            .unwrap_or(0);

        let final_damage = throw_damage.saturating_sub(armor_protection).max(1);

        if let Some(mut hp) = ecs_world.get_mut::<Health>(hit) {
            let new_hp = hp.current.saturating_sub(final_damage);

            if new_hp == 0 {
                let killed_faction_id = ecs_world.get::<Faction>(hit).map(|f| f.faction_id);
                if let Some(fid) = killed_faction_id
                    && let Some(mut rep) = ecs_world.get_resource_mut::<ReputationTracker>() {
                        rep.modify(PLAYER_FACTION_ID, fid, REP_KILL_PENALTY);
                    }
                crate::reputation_sync::sync_reputation_systems(ecs_world);

                let drop_items: Vec<Entity> = ecs_world
                    .get::<game_core::Equipment>(hit)
                    .map(|eq| {
                        [eq.weapon, eq.armor, eq.accessory]
                            .into_iter()
                            .flatten()
                            .collect()
                    })
                    .unwrap_or_default();

                for item_entity in &drop_items {
                    if let Some(mut pos) = ecs_world.get_mut::<Position>(*item_entity) {
                        pos.x = final_x;
                        pos.y = final_y;
                    } else {
                        ecs_world
                            .entity_mut(*item_entity)
                            .insert(Position { x: final_x, y: final_y, z: 0 });
                    }
                }

                ecs_world.entity_mut(hit).despawn();

                if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                    bus.push(GameEvent::ItemThrown {
                        item_name: item_name.clone(),
                        hit_entity: true,
                        hit_name: Some(creature_name),
                        damage: final_damage,
                        target_died: true,
                    });
                }
            } else {
                hp.current = new_hp;
                if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                    bus.push(GameEvent::ItemThrown {
                        item_name: item_name.clone(),
                        hit_entity: true,
                        hit_name: Some(creature_name),
                        damage: final_damage,
                        target_died: false,
                    });
                }
            }
        }
    } else {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::ItemThrown {
                item_name: item_name.clone(),
                hit_entity: false,
                hit_name: None,
                damage: 0,
                target_died: false,
            });
        }
    }

    ecs_world.entity_mut(item_entity).insert(Position { x: final_x, y: final_y, z: 0 });

    true
}

#[allow(dead_code)]
fn calc_throw_damage(
    ecs_world: &World,
    item_entity: bevy_ecs::entity::Entity,
    registry: &TagRegistry,
) -> u32 {
    let tags = match ecs_world.get::<game_tags::Tags>(item_entity) {
        Some(t) => t.clone(),
        None => return 2,
    };

    let base: u32 = 4;
    let material_bonus: u32 = if registry.tag_id("METAL").is_some_and(|id| tags.has(id)) {
        4
    } else if registry.tag_id("STONE").is_some_and(|id| tags.has(id)) {
        2
    } else {
        0
    };

    let size_bonus: u32 = if registry.tag_id("LARGE").is_some_and(|id| tags.has(id)) {
        3
    } else if registry.tag_id("HUGE").is_some_and(|id| tags.has(id)) {
        5
    } else if registry.tag_id("SMALL").is_some_and(|id| tags.has(id)) {
        0
    } else {
        1
    };

    let quality_mult = game_core::calc::get_quality_multiplier(&tags, registry);

    (base + material_bonus + size_bonus).saturating_mul(quality_mult)
}

#[cfg(test)]
use game_core::Glyph;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::World;
    use game_core::{Equipment, Item};
    use game_tags::{Tags, load_tag_registry, TagValue};

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
    const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");

    fn setup_world() -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).expect("tags");
        let rules = game_tags::load_interaction_rules(INTERACTIONS_TOML, &registry).expect("rules");
        world.insert_resource(registry);
        world.insert_resource(rules);
        world
    }

    fn registry(world: &World) -> TagRegistry {
        world.resource::<TagRegistry>().clone()
    }

    #[test]
    fn throw_non_throwable_item_shows_message() {
        let mut world = setup_world();
        let reg = registry(&world);
        let metal_id = reg.tag_id("METAL").unwrap();

        let ore = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Iron Ore".to_string()),
                Item,
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(ore).unwrap();
            tags.add_tag(metal_id, TagValue::None, &reg);
        }

        let _player = world
            .spawn((
                Player,
                Position { x: 5, y: 5, z: 0 },
                game_core::Health {
                    current: 100,
                    max: 100,
                },
                Tags::new(reg.tag_count()),
                Inventory {
                    items: vec![ore],
                    capacity: 20,
                },
                Equipment::default(),
            ))
            .id();

        world.insert_resource(EventBus::new());

        let result = start_throw(&mut world, 0);
        assert!(result.is_none(), "non-throwable item should not start throw");

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(
            bus.events.iter().any(|e| matches!(e, GameEvent::Message(m) if m.contains("cannot be thrown"))),
            "message should say item cannot be thrown"
        );
    }

    #[test]
    fn throw_throwable_item_starts_throw() {
        let mut world = setup_world();
        let reg = registry(&world);
        let throwable_id = reg.tag_id("THROWABLE").unwrap();
        let metal_id = reg.tag_id("METAL").unwrap();

        let rock = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Rock".to_string()),
                Item,
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(rock).unwrap();
            tags.add_tag(throwable_id, TagValue::None, &reg);
            tags.add_tag(metal_id, TagValue::None, &reg);
        }

        let _player = world
            .spawn((
                Player,
                Position { x: 5, y: 5, z: 0 },
                game_core::Health {
                    current: 100,
                    max: 100,
                },
                Tags::new(reg.tag_count()),
                Inventory {
                    items: vec![rock],
                    capacity: 20,
                },
                Equipment::default(),
            ))
            .id();

        world.insert_resource(MessageLog::new(50));

        let result = start_throw(&mut world, 0);
        assert!(result.is_some(), "throwable item should start throw");
    }

    #[test]
    fn calc_throw_damage_metal() {
        let mut world = setup_world();
        let reg = registry(&world);
        let throwable_id = reg.tag_id("THROWABLE").unwrap();
        let metal_id = reg.tag_id("METAL").unwrap();

        let rock = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Metal Rock".to_string()),
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(rock).unwrap();
            tags.add_tag(throwable_id, TagValue::None, &reg);
            tags.add_tag(metal_id, TagValue::None, &reg);
        }

        let damage = calc_throw_damage(&world, rock, &reg);
        assert!(damage >= 8, "metal throwable should do at least 8 damage, got {}", damage);
    }

    #[test]
    fn calc_throw_damage_no_material() {
        let mut world = setup_world();
        let reg = registry(&world);
        let throwable_id = reg.tag_id("THROWABLE").unwrap();

        let rock = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Plain Rock".to_string()),
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(rock).unwrap();
            tags.add_tag(throwable_id, TagValue::None, &reg);
        }

        let damage = calc_throw_damage(&world, rock, &reg);
        assert_eq!(damage, 5, "plain throwable with MEDIUM size should do 5 damage, got {}", damage);
    }

    #[test]
    fn execute_throw_removes_from_inventory() {
        let mut world = setup_world();
        let reg = registry(&world);

        let map = {
            let width = 20u32;
            let height = 20u32;
            let mut tiles = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let tile_entity = world
                        .spawn((game_world::Tile {
                            pos: game_world::TilePos::new(x, y),
                            elevation: 0.5,
                            moisture: 0.5,
                            temperature: 0.5,
                            biome_name: "plains".to_string(),
                            glyph: '.',
                            color: (200, 200, 200),
                        },))
                        .id();
                    tiles.push(tile_entity);
                }
            }
            game_world::WorldMap {
                width,
                height,
                depth: 1,
                current_z: 0,
                seed: game_world::WorldSeed::from_value(42),
                tiles,
            }
        };
        world.insert_resource(map);

        let throwable_id = reg.tag_id("THROWABLE").unwrap();

        let rock = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Rock".to_string()),
                Item,
                Glyph { char: '*', color: (128, 128, 128) },
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(rock).unwrap();
            tags.add_tag(throwable_id, TagValue::None, &reg);
        }

        let player = world
            .spawn((
                Player,
                Position { x: 5, y: 5, z: 0 },
                game_core::Health {
                    current: 100,
                    max: 100,
                },
                Tags::new(reg.tag_count()),
                Inventory {
                    items: vec![rock],
                    capacity: 20,
                },
                Equipment::default(),
            ))
            .id();

        world.insert_resource(MessageLog::new(50));

        let result = execute_throw(&mut world, rock, 5 + MAX_THROW_RANGE, 5);
        assert!(result, "throw should succeed");

        let inv = world.get::<Inventory>(player).unwrap();
        assert!(!inv.items.contains(&rock), "thrown item should be removed from inventory");

        let item_pos = world.get::<Position>(rock);
        assert!(item_pos.is_some(), "thrown item should have a position");
        if let Some(pos) = item_pos {
            assert_eq!(pos.x, 5 + MAX_THROW_RANGE, "item should end up at max range in x direction");
            assert_eq!(pos.y, 5, "item y should remain unchanged");
        }
    }

    #[test]
    fn execute_throw_hits_creature() {
        let mut world = setup_world();
        let reg = registry(&world);

        let map = {
            let width = 20u32;
            let height = 20u32;
            let mut tiles = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let tile_entity = world
                        .spawn((game_world::Tile {
                            pos: game_world::TilePos::new(x, y),
                            elevation: 0.5,
                            moisture: 0.5,
                            temperature: 0.5,
                            biome_name: "plains".to_string(),
                            glyph: '.',
                            color: (200, 200, 200),
                        },))
                        .id();
                    tiles.push(tile_entity);
                }
            }
            game_world::WorldMap {
                width,
                height,
                depth: 1,
                current_z: 0,
                seed: game_world::WorldSeed::from_value(42),
                tiles,
            }
        };
        world.insert_resource(map);

        let throwable_id = reg.tag_id("THROWABLE").unwrap();

        // Place creature at distance 1 (adjacent) for high hit chance
        let creature = world
            .spawn((
                Creature,
                Position { x: 6, y: 5, z: 0 },
                game_core::Health {
                    current: 20,
                    max: 20,
                },
                Name("Creature".to_string()),
                Tags::new(reg.tag_count()),
            ))
            .id();

        let rock = world
            .spawn((
                Tags::new(reg.tag_count()),
                Name("Rock".to_string()),
                Item,
                Glyph { char: '*', color: (128, 128, 128) },
            ))
            .id();
        {
            let mut tags = world.get_mut::<Tags>(rock).unwrap();
            tags.add_tag(throwable_id, TagValue::None, &reg);
        }

        let _player = world
            .spawn((
                Player,
                Position { x: 5, y: 5, z: 0 },
                game_core::Health {
                    current: 100,
                    max: 100,
                },
                Tags::new(reg.tag_count()),
                Inventory {
                    items: vec![rock],
                    capacity: 20,
                },
                Equipment::default(),
            ))
            .id();

        world.insert_resource(MessageLog::new(50));

        // Run multiple throws (re-spawning rock each time) to account for RNG
        let mut hit = false;
        let mut miss = false;
        for _ in 0..20 {
            let r = world.spawn((
                Tags::new(reg.tag_count()),
                Name("Rock".to_string()),
                Item,
                Glyph { char: '*', color: (128, 128, 128) },
            )).id();
            {
                let mut tags = world.get_mut::<Tags>(r).unwrap();
                tags.add_tag(throwable_id, TagValue::None, &reg);
            }

            // Reset creature HP
            if let Some(mut hp) = world.get_mut::<Health>(creature) {
                hp.current = 20;
            }

            let result = execute_throw(&mut world, r, 6, 5);
            assert!(result, "throw should succeed");

            if let Some(hp) = world.get::<Health>(creature)
                && hp.current < 20
            {
                hit = true;
            }

            // Check if we got a miss (rock doesn't land on creature tile)
            if let Some(pos) = world.get::<Position>(r)
                && (pos.x != 6 || pos.y != 5)
            {
                miss = true;
            }
        }

        assert!(hit, "creature should have been hit at least once in 20 attempts");
        assert!(miss, "miss should have occurred at least once in 20 attempts (hit chance is not 100%)");
    }
}
