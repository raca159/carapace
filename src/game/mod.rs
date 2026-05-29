use bevy::prelude::*;
use game_core::screen::AppScreen;
use game_core::components::{Player, Position, Health, Inventory, Equipment, Item, Creature, Name};
use game_core::{EventBus, GameEvent, MessageLog, ExamineMode, TurnCounter, PlayerStats, Durability};
use game_core::WeatherState;
use game_core::narrative::{NarrativeCooldowns, check_narrative_events};
use game_core::quest::{self, QuestBoard, check_quest_failures};
use game_world::{WorldMap, WorldSeed, TilePos, MapLayer, Tile, process_npc_turns};
use game_tags::{TagRegistry, Tags};
use crate::interact::{InteractState, InteractMode, SelectionMode};
use game_core::world_overview::WorldOverviewState;
use crate::render::{GameWorld, GameCamera};
use game_core::emotion::NpcEmotionalState;
use crate::reputation_sync::sync_reputation_systems;
use crate::ui::{WorldGenParams, CharacterName, SaveFileList};
use game_core::save::deserialize_to_world;

#[derive(Resource, Default)]
pub struct GameTurnState {
    pub processing_npcs: bool,
}

#[derive(Resource, Default)]
pub struct InventoryState {
    pub open: bool,
    pub cursor: usize,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameTurnState>()
            .init_resource::<InventoryState>()
            .add_systems(OnEnter(AppScreen::InWorld), spawn_world)
            .add_systems(Update, (
                handle_game_input,
                handle_examine_navigation,
                finish_npc_turn,
                check_player_death,
                crate::event_format::format_events,
            ).run_if(in_state(AppScreen::InWorld)));
    }
}

fn spawn_world(
    mut game_world: ResMut<GameWorld>,
    mut game_camera: ResMut<GameCamera>,
    params: Res<WorldGenParams>,
    name: Res<CharacterName>,
    mut saves: ResMut<SaveFileList>,
) {
    let ecs_world = &mut game_world.0;

    if let Some(save) = saves.selected_save.take() {
        let seed = WorldSeed::from_value(save.seed);
        let width = 200u32;
        let height = 200u32;
        crate::world_gen::generate_world(ecs_world, seed, width, height);
        let _ = deserialize_to_world(ecs_world, &save);
        let player_pos = ecs_world
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .single(ecs_world)
            .ok()
            .copied()
            .unwrap_or(Position { x: 100, y: 100, z: 0 });
        game_camera.x = player_pos.x;
        game_camera.y = player_pos.y;

        ecs_world.insert_resource(MessageLog::new(50));
        ecs_world.insert_resource(EventBus::new());
        let events_toml = include_str!("../../assets/config/events.toml");
        if let Ok(formats) = crate::event_format::load_event_formats(events_toml) {
            ecs_world.insert_resource(formats);
        }
        ecs_world.insert_resource(ExamineMode::new());
        ecs_world.insert_resource(NarrativeCooldowns::default());
        return;
    }

    let seed = WorldSeed::from_value(params.seed);
    let width = params.width.max(50);
    let height = params.height.max(50);

    crate::world_gen::generate_world(ecs_world, seed, width, height);
    let mut render_camera = game_render::Camera::new(80, 24);
    let player_pos = crate::world_gen::spawn_player(ecs_world, &mut render_camera);
    crate::world_gen::spawn_game_entities(ecs_world, player_pos);

    let player_entity = ecs_world
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
        .iter(ecs_world)
        .next();
    if let Some(p) = player_entity {
        let character_name = if name.0.is_empty() { "Adventurer".to_string() } else { name.0.clone() };
        if let Some(mut n) = ecs_world.get_mut::<Name>(p) {
            n.0 = character_name;
        } else {
            ecs_world.entity_mut(p).insert(Name(character_name));
        }
    }

    ecs_world.insert_resource(MessageLog::new(50));
    ecs_world.insert_resource(EventBus::new());
    let events_toml = include_str!("../../assets/config/events.toml");
    if let Ok(formats) = crate::event_format::load_event_formats(events_toml) {
        ecs_world.insert_resource(formats);
    }
    ecs_world.insert_resource(TurnCounter::new());
    ecs_world.insert_resource(PlayerStats::default());
    ecs_world.insert_resource(ExamineMode::new());
    ecs_world.insert_resource(NarrativeCooldowns::default());

    game_camera.x = player_pos.x;
    game_camera.y = player_pos.y;
}

fn handle_game_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_world: ResMut<GameWorld>,
    mut turn_state: ResMut<GameTurnState>,
    mut inv_state: ResMut<InventoryState>,
    mut interact_state: ResMut<InteractState>,
    mut next_screen: ResMut<NextState<AppScreen>>,
) {
    if turn_state.processing_npcs {
        return;
    }

    // Don't process game input while interaction panel is active
    if interact_state.active.is_some() {
        return;
    }

    // Toggle examine
    if keyboard.just_pressed(KeyCode::KeyX) {
        let pos = game_world.0
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .iter(&game_world.0)
            .next()
            .copied();
        let active = game_world.0.get_resource::<ExamineMode>().map(|e| e.active).unwrap_or(false);
        if let Some(mut em) = game_world.0.get_resource_mut::<ExamineMode>() {
            em.active = !active;
            if !active {
                if let Some(p) = pos {
                    em.cursor.x = p.x;
                    em.cursor.y = p.y;
                }
            }
        }
        return;
    }

    // Toggle inventory
    if keyboard.just_pressed(KeyCode::KeyI) {
        inv_state.open = !inv_state.open;
        inv_state.cursor = 0;
        return;
    }

    // Pause
    if keyboard.just_pressed(KeyCode::Escape) {
        if AppScreen::transition_allowed(&AppScreen::InWorld, &AppScreen::PauseMenu) {
            next_screen.set(AppScreen::PauseMenu);
        }
        return;
    }

    // Unified interact key — collect INTERACTABLE entities adjacent to player
    if keyboard.just_pressed(KeyCode::KeyE) {
        let registry = match game_world.0.get_resource::<TagRegistry>() {
            Some(r) => r.clone(),
            None => return,
        };
        let interactable_id = match registry.tag_id("INTERACTABLE") {
            Some(id) => id,
            None => return,
        };

        let player_pos = match game_world.0
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .single(&game_world.0)
        {
            Ok(p) => (p.x, p.y),
            Err(_) => return,
        };

        let offsets: [(i32, i32); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];

        let mut targets: Vec<Entity> = Vec::new();
        for (entity, pos, tags) in game_world.0
            .query::<(Entity, &Position, &game_tags::Tags)>()
            .iter(&game_world.0)
        {
            if !tags.has(interactable_id) { continue; }
            let in_range = offsets.iter().any(|(dx, dy)| {
                let nx = player_pos.0 as i32 + dx;
                let ny = player_pos.1 as i32 + dy;
                nx >= 0 && ny >= 0 && pos.x == nx as u32 && pos.y == ny as u32
            });
            if in_range {
                targets.push(entity);
            }
        }

        if targets.is_empty() {
            if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                msg.messages.push("Nothing to interact with.".to_string());
            }
            return;
        }

        if targets.len() == 1 {
            route_interaction(&mut game_world.0, targets[0], &mut interact_state);
        } else {
            interact_state.active = Some(InteractMode::Disambiguating(targets));
        }
        return;
    }

    // Unified location traversal — > or Enter
    if keyboard.just_pressed(KeyCode::Period) || keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadDecimal) {
        // Check if inside an interior first
        if game_world.0.get_resource::<MapLayer>()
            .is_some_and(|ml| ml.active_interior.is_some())
        {
            let player_pos = match game_world.0
                .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
                .single(&game_world.0)
            {
                Ok(p) => (p.x, p.y),
                Err(_) => return,
            };
            let deeper_id = match game_world.0.get_resource::<TagRegistry>()
                .and_then(|r| r.tag_id("DEEPER_STAIR"))
            {
                Some(id) => id,
                None => return,
            };
            let entrance_id = match game_world.0.get_resource::<TagRegistry>()
                .and_then(|r| r.tag_id("ENTRANCE_STAIR"))
            {
                Some(id) => id,
                None => return,
            };

            let (on_deeper, on_entrance) = {
                let world_map = match game_world.0.get_resource::<WorldMap>() {
                    Some(wm) => wm,
                    None => return,
                };
                let idx = player_pos.1 as usize * world_map.width as usize + player_pos.0 as usize;
                let tags = world_map.tiles.get(idx)
                    .and_then(|tile_entity| game_world.0.get::<Tags>(*tile_entity));

                (
                    tags.is_some_and(|t| t.has(deeper_id)),
                    tags.is_some_and(|t| t.has(entrance_id)),
                )
            };

            if on_deeper {
                crate::location_entry::enter_next_depth(&mut game_world.0);
                return;
            }

            if on_entrance {
                crate::location_entry::exit_location(&mut game_world.0);
                return;
            }

            if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                msg.messages.push("Nothing to enter here.".to_string());
            }
            return;
        }

        // On overworld — check if player position has a location with interior
        let location_map = match game_world.0.get_resource::<game_world::cascade::LocationMap>() {
            Some(lm) => lm.clone(),
            None => {
                if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    msg.messages.push("Nothing to enter here.".to_string());
                }
                return;
            }
        };
        let player_pos = match game_world.0
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .single(&game_world.0)
        {
            Ok(p) => (p.x, p.y),
            Err(_) => return,
        };

        let loc = game_world::cascade::locations::location_at(&location_map.locations, player_pos.0, player_pos.1);
        if let Some(location) = loc {
            if location.tags.iter().any(|t| t == "HAS_INTERIOR") {
                crate::location_entry::enter_location(&mut game_world.0, &location);
            } else {
                let msg = match location.location_type.as_str() {
                    "dungeon" | "cave" => format!("Entering {}...", location.name),
                    "city" | "village" | "outpost" => format!("You arrive at {}.", location.name),
                    "ruin" | "shrine" => format!("You explore the {}.", location.name),
                    _ => format!("You approach {}.", location.name),
                };
                if let Some(mut log) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    log.messages.push(msg);
                }
            }
        } else {
            if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                msg.messages.push("Nothing to enter here.".to_string());
            }
        }
        return;
    }

    // Exit interior — , (comma)
    if keyboard.just_pressed(KeyCode::Comma) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        let map_layer = game_world.0.get_resource::<MapLayer>().cloned();
        if let Some(ml) = map_layer {
            if ml.active_interior.is_some() {
                let player_pos = match game_world.0
                    .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    Ok(p) => (p.x, p.y),
                    Err(_) => return,
                };
                let registry = match game_world.0.get_resource::<TagRegistry>() {
                    Some(r) => r.clone(),
                    None => return,
                };
                let entrance_id = match registry.tag_id("ENTRANCE_STAIR") {
                    Some(id) => id,
                    None => return,
                };

                let on_entrance = {
                    let world_map = match game_world.0.get_resource::<WorldMap>() {
                        Some(wm) => wm,
                        None => return,
                    };
                    let idx = player_pos.1 as usize * world_map.width as usize + player_pos.0 as usize;
                    world_map.tiles.get(idx)
                        .and_then(|tile_entity| game_world.0.get::<Tags>(*tile_entity))
                        .is_some_and(|tags| tags.has(entrance_id))
                };

                if on_entrance {
                    crate::location_entry::exit_location(&mut game_world.0);
                    return;
                }
                if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    msg.messages.push("Not near an exit.".to_string());
                }
            }
        }
        return;
    }

    // World overview map
    if keyboard.just_pressed(KeyCode::KeyM) {
        let player_pos = game_world.0
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .single(&game_world.0)
            .ok()
            .map(|p| (p.x, p.y));
        if let Some(mut ov) = game_world.0.get_resource_mut::<WorldOverviewState>() {
            ov.active = !ov.active;
            if ov.active {
                if let Some((px, py)) = player_pos {
                    ov.player_x = px;
                    ov.player_y = py;
                    ov.cursor_x = px;
                    ov.cursor_y = py;
                }
            }
        }
        return;
    }

    // Crafting key
    if keyboard.just_pressed(KeyCode::KeyC) {
        interact_state.active = Some(InteractMode::Crafting);
        return;
    }

    // Consume / Use key
    if keyboard.just_pressed(KeyCode::KeyU) {
        let items = get_consumable_items(&mut game_world.0);
        if items.is_empty() {
            if let Some(mut bus) = game_world.0.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message("Nothing to consume.".to_string()));
            }
        } else {
            interact_state.active = Some(InteractMode::ItemSelection {
                mode: SelectionMode::Consume,
                items,
                cursor: 0,
            });
        }
        return;
    }

    // Throw key
    if keyboard.just_pressed(KeyCode::KeyT) {
        let items = get_throwable_items(&mut game_world.0);
        if items.is_empty() {
            if let Some(mut bus) = game_world.0.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message("Nothing to throw.".to_string()));
            }
        } else {
            interact_state.active = Some(InteractMode::ItemSelection {
                mode: SelectionMode::Throw,
                items,
                cursor: 0,
            });
        }
        return;
    }

    if inv_state.open {
        let inv_len = get_inventory_len(&mut game_world.0);
        if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyI) {
            inv_state.open = false;
        }
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            inv_state.cursor = inv_state.cursor.saturating_sub(1);
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) && inv_state.cursor + 1 < inv_len {
            inv_state.cursor += 1;
        }
        if keyboard.just_pressed(KeyCode::KeyE) {
            crate::equipment::handle_equip(&mut game_world.0, inv_state.cursor);
        }
        return;
    }

    // Pickup key
    if keyboard.just_pressed(KeyCode::KeyG) {
        auto_pickup_at_player(&mut game_world.0);
        turn_state.processing_npcs = true;
        return;
    }

    // Movement
    let (dx, dy) = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        (0i32, -1i32)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        (0i32, 1i32)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        (-1i32, 0i32)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        (1i32, 0i32)
    } else {
        return;
    };

    let (cur_x, cur_y) = match game_world.0
        .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
        .single(&game_world.0)
    {
        Ok(p) => (p.x, p.y),
        Err(_) => return,
    };

    let (map_width, map_height) = match game_world.0.get_resource::<WorldMap>() {
        Some(m) => (m.width, m.height),
        None => return,
    };

    let nx = (cur_x as i32 + dx) as u32;
    let ny = (cur_y as i32 + dy) as u32;

    if nx >= map_width || ny >= map_height {
        return;
    }

    // Blocked tile check
    let registry = game_world.0.resource::<TagRegistry>().clone();
    let blocked_id = match registry.tag_id("BLOCKED") {
        Some(id) => id,
        None => return,
    };
    let tile_entity = game_world.0.get_resource::<WorldMap>()
        .and_then(|map| map.get(TilePos::new(nx, ny)));
    if let Some(tile_entity) = tile_entity {
        if let Ok(tile_tags) = game_world.0.query::<&Tags>().get(&game_world.0, tile_entity) {
            if tile_tags.has(blocked_id) {
                return;
            }
        }
    }

    // Build position lookup map
    let mut positions: std::collections::HashMap<Entity, (u32, u32)> = std::collections::HashMap::new();
    for (entity, pos) in game_world.0.query::<(Entity, &Position)>().iter(&game_world.0) {
        positions.insert(entity, (pos.x, pos.y));
    }

    // Combat check: aggressive creature at target
    let aggressive_id = registry.tag_id("AGGRESSIVE");
    let mut creatures = Vec::new();
    for (entity, tags, name, health) in game_world.0
        .query_filtered::<(Entity, &Tags, &Name, &Health), bevy_ecs::query::With<Creature>>()
        .iter(&game_world.0)
    {
        if let Some(&(px, py)) = positions.get(&entity) {
            if px == nx && py == ny {
                creatures.push((entity, tags.clone(), name.0.clone(), *health));
            }
        }
    }

    for (creature_entity, creature_tags, creature_name, creature_health) in &creatures {
        if aggressive_id.is_some_and(|id| creature_tags.has(id)) {
            resolve_combat(&mut game_world.0, *creature_entity, creature_name.clone(), *creature_health);
            turn_state.processing_npcs = true;
            return;
        }
    }

    // Auto-pickup items at target
    auto_pickup_items(&mut game_world.0, nx, ny);

    // Move player
    if let Ok(mut pos) = game_world.0
        .query_filtered::<&mut Position, bevy_ecs::query::With<Player>>()
        .single_mut(&mut game_world.0)
    {
        pos.x = nx;
        pos.y = ny;
    }

    // Trap detection and triggering at new position
    {
        let player = game_world.0
            .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
            .single(&game_world.0).ok();
        if let Some(player_entity) = player {
            let traps: Vec<(Entity, game_core::traps::Trap)> = game_world.0
                .query::<(Entity, &game_core::traps::Trap, &Position)>()
                .iter(&game_world.0)
                .filter(|(_, _, p)| p.x == nx && p.y == ny)
                .map(|(e, t, _)| (e, t.clone()))
                .collect();
            for (trap_entity, trap) in &traps {
                if trap.triggered { continue; }
                if !trap.detected {
                    // Detect the trap
                    if let Some(mut t) = game_world.0.get_mut::<game_core::traps::Trap>(*trap_entity) {
                        t.detected = true;
                    }
                    if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                        bus.push(game_core::GameEvent::Message("You spot a trap!".to_string()));
                    }
                } else {
                    // Step on a known trap — trigger it
                    game_core::traps::trigger_trap(&mut game_world.0, *trap_entity, player_entity);
                }
            }
        }
    }

    // Track quest progress for reaching new biome
    if let Some(tile_entity) = game_world.0.get_resource::<WorldMap>()
        .and_then(|map| map.get(TilePos::new(nx, ny)))
    {
        if let Some(tile) = game_world.0.get::<Tile>(tile_entity) {
            let biome_clean = tile.biome_name.trim_start_matches("BIOME_").replace("_", " ").to_lowercase();
            quest::track_reach(&mut game_world.0, &biome_clean);
        }
    }

    // Roll encounter at new position
    let encounter = {
        let pos = Position { x: nx, y: ny, z: 0 };
        let wc = game_world.0.get_resource::<game_core::WeatherContext>();
        let location_map = game_world.0.get_resource::<game_world::cascade::LocationMap>()
            .map(|lm| lm.locations.clone()).unwrap_or_default();
        let near_loc = game_world::cascade::locations::location_at(&location_map, nx, ny)
            .map(|l| l.location_type.as_str());
        let biome_tags: Vec<game_tags::TagId> = game_world.0.get_resource::<WorldMap>()
            .and_then(|map| map.get(TilePos::new(nx, ny)))
            .and_then(|tile_entity| {
                let tags = game_world.0.get::<game_tags::Tags>(tile_entity)?;
                let registry = game_world.0.get_resource::<game_tags::TagRegistry>()?;
                Some(tags.iter_present().filter(|tid| {
                    registry.tag_by_id(*tid).name.starts_with("BIOME_")
                }).collect::<Vec<_>>())
            })
            .unwrap_or_default();
        game_core::encounters::roll_encounter(
            &game_world.0, &pos, &biome_tags, near_loc, wc, &mut rand::rng(),
        )
    };
    if let Some(encounter_id) = encounter {
        let pos = Position { x: nx, y: ny, z: 0 };
        game_core::encounters::spawn_encounter(&mut game_world.0, &encounter_id, pos);
    }

    turn_state.processing_npcs = true;
}

pub fn handle_examine_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_world: ResMut<GameWorld>,
) {
    let ecs_world = &mut game_world.0;

    let (active, cursor) = {
        let em = match ecs_world.get_resource::<ExamineMode>() {
            Some(e) => e,
            None => return,
        };
        (em.active, em.cursor)
    };
    if !active {
        return;
    }

    let (dx, dy) = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        (0i32, -1i32)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        (0i32, 1i32)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        (-1i32, 0i32)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        (1i32, 0i32)
    } else {
        return;
    };

    let map = match ecs_world.get_resource::<WorldMap>() {
        Some(m) => m,
        None => return,
    };

    let new_x = (cursor.x as i32 + dx).max(0).min(map.width as i32 - 1) as u32;
    let new_y = (cursor.y as i32 + dy).max(0).min(map.height as i32 - 1) as u32;

    if let Some(mut em) = ecs_world.get_resource_mut::<ExamineMode>() {
        em.set_cursor(new_x, new_y);
    }
}

fn finish_npc_turn(
    mut game_world: ResMut<GameWorld>,
    mut turn_state: ResMut<GameTurnState>,
) {
    if !turn_state.processing_npcs {
        return;
    }
    process_npc_turns(&mut game_world.0);

    // Advance weather and apply environmental tags BEFORE status effects
    // so interaction rules see the new tags this turn
    if let Some(mut ws) = game_world.0.get_resource_mut::<WeatherState>() {
        let dummy = World::new();
        ws.advance_time(&dummy);
    }
    crate::weather_pipeline::apply_environmental_tags(&mut game_world.0);
    crate::status::process_status_effects(&mut game_world.0);
    game_core::traps::process_trapped_status(&mut game_world.0);

    check_narrative_events(&mut game_world.0);
    sync_reputation_systems(&mut game_world.0);
    check_quest_failures(&mut game_world.0, None);

    turn_state.processing_npcs = false;
    if let Some(mut tc) = game_world.0.get_resource_mut::<TurnCounter>() {
        tc.increment();
        if game_core::save::should_auto_save(&game_world.0) {
            let seed = game_world.0.get_resource::<game_world::WorldSeed>()
                .map(|s| s.0)
                .unwrap_or(0);
            let _ = game_core::save::save_game(&mut game_world.0, seed);
        }
    }
}

fn check_player_death(
    mut game_world: ResMut<GameWorld>,
    mut next_screen: ResMut<NextState<AppScreen>>,
) {
    let hp = game_world.0
        .query_filtered::<&Health, bevy_ecs::query::With<Player>>()
        .iter(&game_world.0)
        .next()
        .copied();
    if let Some(hp) = hp && hp.current == 0 {
        if AppScreen::transition_allowed(&AppScreen::InWorld, &AppScreen::Dead) {
            next_screen.set(AppScreen::Dead);
        }
    }
}

pub fn resolve_combat(
    ecs_world: &mut World,
    creature_entity: Entity,
    creature_name: String,
    creature_health: Health,
) {
    let player_entity = match ecs_world
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(e) => e,
        Err(_) => return,
    };

    let player_health = ecs_world.get::<Health>(player_entity).copied().unwrap_or(Health { current: 1, max: 1 });
    let player_equip = ecs_world.get::<Equipment>(player_entity).cloned().unwrap_or_default();
    let creature_equip = ecs_world.get::<Equipment>(creature_entity).cloned().unwrap_or_default();
    let registry = ecs_world.resource::<TagRegistry>().clone();

    let player_damage = crate::equipment::calc_weapon_damage(&player_equip, ecs_world, &registry).max(1);
    let creature_damage = crate::equipment::calc_weapon_damage(&creature_equip, ecs_world, &registry).max(1);

    let new_creature_hp = creature_health.current.saturating_sub(player_damage);
    let new_player_hp = player_health.current.saturating_sub(creature_damage);

    let mut events = Vec::new();

    if new_creature_hp == 0 {
        quest::track_kill(ecs_world, &creature_name);
        let creature_pos = ecs_world.get::<Position>(creature_entity).copied();
        if let Some(pos) = creature_pos {
            if let Some(tile_entity) = ecs_world.get_resource::<WorldMap>()
                .and_then(|map| map.get(TilePos::new(pos.x, pos.y)))
            {
                if let Some(tile) = ecs_world.get::<game_world::Tile>(tile_entity) {
                    let biome_clean = tile.biome_name.trim_start_matches("BIOME_").replace("_", " ").to_lowercase();
                    quest::track_kill_area(ecs_world, &biome_clean);
                }
            }
        }
        events.push(GameEvent::Combat {
            attacker: Some(player_entity),
            target: Some(creature_entity),
            attacker_name: "You".to_string(),
            target_name: creature_name.clone(),
            damage_dealt: player_damage,
            target_hp_remaining: 0,
            target_died: true,
            is_player_attacker: true,
            is_player_target: false,
        });
        if let Some(equip) = ecs_world.get::<Equipment>(creature_entity).cloned() {
            let pos = *ecs_world.get::<Position>(creature_entity).unwrap();
            if let Some(wpn) = equip.weapon {
                let has_inv = ecs_world.get::<Inventory>(creature_entity).is_some();
                if has_inv {
                    if let Some(mut inv) = ecs_world.get_mut::<Inventory>(creature_entity) {
                        inv.items.push(wpn);
                    }
                } else {
                    ecs_world.entity_mut(wpn).insert(pos);
                }
            }
            if let Some(arm) = equip.armor {
                let has_inv = ecs_world.get::<Inventory>(creature_entity).is_some();
                if has_inv {
                    if let Some(mut inv) = ecs_world.get_mut::<Inventory>(creature_entity) {
                        inv.items.push(arm);
                    }
                } else {
                    ecs_world.entity_mut(arm).insert(pos);
                }
            }
        }
        {
            let mut entity = ecs_world.entity_mut(creature_entity);
            entity.remove::<Creature>();
            if let Some(tid) = registry.tag_id("CONTAINER") {
                let mut tags = entity.get_mut::<game_tags::Tags>().unwrap();
                if !tags.has(tid) { tags.add_tag(tid, game_tags::TagValue::None, &registry); }
            }
            if let Some(tid) = registry.tag_id("INTERACTABLE") {
                let mut tags = entity.get_mut::<game_tags::Tags>().unwrap();
                if !tags.has(tid) { tags.add_tag(tid, game_tags::TagValue::None, &registry); }
            }
        }
    } else {
        if let Some(mut hp) = ecs_world.get_mut::<Health>(creature_entity) { hp.current = new_creature_hp; }
        events.push(GameEvent::Combat {
            attacker: Some(player_entity),
            target: Some(creature_entity),
            attacker_name: "You".to_string(),
            target_name: creature_name.clone(),
            damage_dealt: player_damage,
            target_hp_remaining: new_creature_hp,
            target_died: false,
            is_player_attacker: true,
            is_player_target: false,
        });
    }

    if let Some(mut hp) = ecs_world.get_mut::<Health>(player_entity) { hp.current = new_player_hp; }

    events.push(GameEvent::Combat {
        attacker: Some(creature_entity),
        target: Some(player_entity),
        attacker_name: creature_name.clone(),
        target_name: "you".to_string(),
        damage_dealt: creature_damage,
        target_hp_remaining: new_player_hp,
        target_died: false,
        is_player_attacker: false,
        is_player_target: true,
    });

    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        for event in events {
            bus.push(event);
        }
    }

    // Degrade player's weapon and armor from combat use
    {
        let player_eq = ecs_world.get::<Equipment>(player_entity).cloned();
        if let Some(eq) = player_eq {
            if let Some(wpn) = eq.weapon {
                if ecs_world.get::<Durability>(wpn).is_some() {
                    game_core::durability::degrade_weapon(ecs_world, wpn, 1);
                }
            }
            if let Some(arm) = eq.armor {
                if ecs_world.get::<Durability>(arm).is_some() {
                    game_core::durability::degrade_armor(ecs_world, arm, 1);
                }
            }
        }
    }

    if ecs_world.get_entity(creature_entity).is_ok() {
        if let Some(mut state) = ecs_world.get_mut::<NpcEmotionalState>(creature_entity) {
            state.apply_event(0.5);
        }
    }
}

fn auto_pickup_items(ecs_world: &mut World, nx: u32, ny: u32) {
    let player = match ecs_world
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(e) => e,
        Err(_) => return,
    };

    let items: Vec<Entity> = ecs_world
        .query_filtered::<(Entity, &Position), bevy_ecs::query::With<Item>>()
        .iter(ecs_world)
        .filter(|(_, pos)| pos.x == nx && pos.y == ny)
        .map(|(e, _)| e)
        .collect();

    for item in items {
        let name = ecs_world.get::<Name>(item).map(|n| n.0.clone()).unwrap_or_else(|| "?".to_string());

        // Track collection quest progress for item tags
        let collect_tags: Vec<String> = {
            let registry = ecs_world.get_resource::<TagRegistry>();
            ecs_world.get::<Tags>(item).map(|item_tags| {
                item_tags.iter_present().filter_map(|tag_id| {
                    registry.map(|reg| reg.tag_by_id(tag_id).name.clone())
                }).collect()
            }).unwrap_or_default()
        };
        for tag_name in &collect_tags {
            quest::track_collect(ecs_world, tag_name);
        }

        let inv = ecs_world.get_mut::<Inventory>(player).unwrap();
        if inv.items.len() >= inv.capacity {
            if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message("Inventory full.".to_string()));
            }
            break;
        }
        drop(inv);

        ecs_world.entity_mut(item).remove::<Position>();
        let mut inv = ecs_world.get_mut::<Inventory>(player).unwrap();
        inv.items.push(item);
        drop(inv);

        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::ItemPickedUp { item_name: name });
        }
    }
}

fn auto_pickup_at_player(ecs_world: &mut World) {
    let pos = match ecs_world
        .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(p) => (p.x, p.y),
        Err(_) => return,
    };
    auto_pickup_items(ecs_world, pos.0, pos.1);
}

fn get_inventory_len(ecs_world: &mut World) -> usize {
    ecs_world
        .query_filtered::<&Inventory, bevy_ecs::query::With<Player>>()
        .iter(ecs_world)
        .next()
        .map(|inv| inv.items.len())
        .unwrap_or(0)
}

fn get_consumable_items(ecs_world: &mut World) -> Vec<Entity> {
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return vec![],
    };
    let edible_id = match registry.tag_id("EDIBLE") { Some(id) => id, None => return vec![] };
    let drinkable_id = match registry.tag_id("DRINKABLE") { Some(id) => id, None => return vec![] };
    let player = match ecs_world
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let inv = match ecs_world.get::<Inventory>(player) {
        Some(i) => i.items.clone(),
        None => return vec![],
    };
    inv.into_iter().filter(|&item| {
        let tags = match ecs_world.get::<game_tags::Tags>(item) { Some(t) => t, None => return false };
        tags.has(edible_id) || tags.has(drinkable_id)
    }).collect()
}

fn get_throwable_items(ecs_world: &mut World) -> Vec<Entity> {
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return vec![],
    };
    let throwable_id = match registry.tag_id("THROWABLE") { Some(id) => id, None => return vec![] };
    let player = match ecs_world
        .query_filtered::<Entity, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    let inv = match ecs_world.get::<Inventory>(player) {
        Some(i) => i.items.clone(),
        None => return vec![],
    };
    inv.into_iter().filter(|&item| {
        ecs_world.get::<game_tags::Tags>(item).is_some_and(|tags| tags.has(throwable_id))
    }).collect()
}

fn route_interaction(
    ecs_world: &mut World,
    target: Entity,
    interact_state: &mut InteractState,
) {
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let tags = match ecs_world.get::<game_tags::Tags>(target) {
        Some(t) => t.clone(),
        None => return,
    };

    let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
    let is_quest_board = ecs_world.get::<QuestBoard>(target).is_some();
    let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));
    let has_inventory = registry.tag_id("HAS_INVENTORY").is_some_and(|id| tags.has(id));
    let is_container = registry.tag_id("CONTAINER").is_some_and(|id| tags.has(id));

    if can_talk {
        interact_state.active = Some(InteractMode::Talk { npc_entity: target });
    } else if has_inventory && is_container {
        interact_state.active = Some(InteractMode::Looting { container_entity: target, cursor: 0 });
    } else if is_quest_board {
        interact_state.active = Some(InteractMode::QuestBoard);
    } else if can_craft {
        interact_state.active = Some(InteractMode::Crafting);
    } else {
        if let Some(mut msg) = ecs_world.get_resource_mut::<MessageLog>() {
            msg.messages.push("Nothing to do here.".to_string());
        }
    }
}
