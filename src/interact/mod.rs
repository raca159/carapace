use bevy::prelude::*;
use game_core::quest::{QuestBoardState, handle_quest_turn_in};
use game_core::screen::AppScreen;
use game_core::{EventBus, GameEvent, Inventory, Player};
use game_tags::TagRegistry;

pub mod trade;
pub mod talk;
pub mod craft;
pub mod quest_board;
pub mod consume;
pub mod throw;
pub mod loot;
pub mod overview;

#[derive(Resource, Default)]
pub struct InteractState {
    pub active: Option<InteractMode>,
}

#[derive(Clone)]
pub enum InteractMode {
    Disambiguating(Vec<bevy_ecs::entity::Entity>),
    Talk {
        npc_entity: bevy_ecs::entity::Entity,
    },
    Crafting,
    QuestBoard,
    ItemSelection {
        mode: SelectionMode,
        items: Vec<bevy_ecs::entity::Entity>,
        cursor: usize,
    },
    ThrowTargeting {
        item_entity: bevy_ecs::entity::Entity,
        cursor_x: u32,
        cursor_y: u32,
    },
    Looting {
        container_entity: bevy_ecs::entity::Entity,
        cursor: usize,
    },
}

#[derive(Clone)]
pub enum SelectionMode {
    Consume,
    Throw,
}

pub struct InteractPlugin;

impl Plugin for InteractPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InteractState>()
            .add_systems(Update, (
                handle_interact_input,
                talk::handle_talk_input,
                talk::update_talk_panel,
                craft::update_craft_panel,
                quest_board::update_quest_board_panel,
                consume::update_consume_overlay,
                throw::update_throw_overlay,
                loot::update_loot_panel,
            ).run_if(in_state(AppScreen::InWorld)));
    }
}

fn handle_interact_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interact: ResMut<InteractState>,
    mut game_world: ResMut<crate::render::GameWorld>,
) {
    let targets = match &interact.active {
        Some(InteractMode::Disambiguating(t)) => t.clone(),
        Some(InteractMode::ItemSelection { mode, items, cursor }) => {
            let mode = mode.clone();
            let items = items.clone();
            let cur = *cursor;

            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
            } else if keyboard.just_pressed(KeyCode::ArrowUp) && cur > 0 {
                if let Some(InteractMode::ItemSelection { cursor, .. }) = &mut interact.active {
                    *cursor = cur - 1;
                }
            } else if keyboard.just_pressed(KeyCode::ArrowDown) && cur + 1 < items.len() {
                if let Some(InteractMode::ItemSelection { cursor, .. }) = &mut interact.active {
                    *cursor = cur + 1;
                }
            } else if keyboard.just_pressed(KeyCode::Enter) {
                let is_throw = matches!(mode, crate::interact::SelectionMode::Throw);
                interact.active = None;
                if is_throw {
                    let item_entity = match crate::throw::start_throw(&mut game_world.0, cur) {
                        Some(e) => e,
                        None => return,
                    };
                    let player_pos = match game_world.0
                        .query_filtered::<&game_core::Position, bevy_ecs::query::With<game_core::Player>>()
                        .single(&game_world.0)
                    {
                        Ok(p) => (p.x, p.y),
                        Err(_) => return,
                    };
                    interact.active = Some(InteractMode::ThrowTargeting {
                        item_entity,
                        cursor_x: player_pos.0,
                        cursor_y: player_pos.1,
                    });
                } else {
                    crate::consume::handle_consume(&mut game_world.0, cur);
                }
            }
            return;
        }
        Some(InteractMode::ThrowTargeting { item_entity, cursor_x, cursor_y }) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
                return;
            }

            let (dx, dy) = if keyboard.just_pressed(KeyCode::ArrowUp) { (0i32, -1i32) }
            else if keyboard.just_pressed(KeyCode::ArrowDown) { (0i32, 1i32) }
            else if keyboard.just_pressed(KeyCode::ArrowLeft) { (-1i32, 0i32) }
            else if keyboard.just_pressed(KeyCode::ArrowRight) { (1i32, 0i32) }
            else { (0i32, 0i32) };

            if dx != 0 || dy != 0 {
                let map = match game_world.0.get_resource::<game_world::WorldMap>() {
                    Some(m) => m.clone(),
                    None => return,
                };
                let new_x = (*cursor_x as i32 + dx).max(0).min(map.width as i32 - 1) as u32;
                let new_y = (*cursor_y as i32 + dy).max(0).min(map.height as i32 - 1) as u32;
                if let Some(InteractMode::ThrowTargeting { cursor_x, cursor_y, .. }) = &mut interact.active {
                    *cursor_x = new_x;
                    *cursor_y = new_y;
                }
                return;
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let item = *item_entity;
                let (tx, ty) = (*cursor_x, *cursor_y);
                interact.active = None;
                crate::throw::execute_throw(&mut game_world.0, item, tx, ty);
                return;
            }
            return;
        }
        Some(InteractMode::Crafting) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
                return;
            }
            let recipes = match game_world.0.get_resource::<crate::interact::craft::CraftingRecipesResource>() {
                Some(r) => r.recipes.clone(),
                None => return,
            };
            let registry = match game_world.0.get_resource::<TagRegistry>() {
                Some(r) => r.clone(),
                None => return,
            };
            let player = match game_world.0
                .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
                .single(&game_world.0)
            {
                Ok(e) => e,
                Err(_) => return,
            };
            let inventory = match game_world.0.get::<Inventory>(player) {
                Some(i) => i.clone(),
                None => return,
            };
            let player_pos = match game_world.0
                .query_filtered::<&game_core::Position, bevy_ecs::query::With<Player>>()
                .single(&game_world.0)
            {
                Ok(p) => (p.x, p.y),
                Err(_) => return,
            };
            let available = game_core::crafting::find_available_recipes(
                &recipes, &inventory, &mut game_world.0, player_pos, &registry,
            );

            for i in 0..available.len().min(9) {
                let key = match i {
                    0 => KeyCode::Digit1, 1 => KeyCode::Digit2, 2 => KeyCode::Digit3,
                    3 => KeyCode::Digit4, 4 => KeyCode::Digit5, 5 => KeyCode::Digit6,
                    6 => KeyCode::Digit7, 7 => KeyCode::Digit8, 8 => KeyCode::Digit9,
                    _ => continue,
                };
                if keyboard.just_pressed(key) && available[i].available {
                    let recipe = available[i].recipe.clone();
                    let reg = registry.clone();
                    let mut inv = inventory.clone();
                    let created = game_core::crafting::execute_recipe(&recipe, &mut inv, &mut game_world.0, &reg);
                    if let Some(mut p_inv) = game_world.0.get_mut::<Inventory>(player) {
                        *p_inv = inv;
                    }
                    let names: Vec<String> = created.iter().map(|item| {
                        game_world.0.get::<game_core::Name>(*item)
                            .map(|n| n.0.clone())
                            .unwrap_or_default()
                    }).collect();
                    if let Some(mut bus) = game_world.0.get_resource_mut::<EventBus>() {
                        for name in names {
                            bus.push(GameEvent::ItemCrafted { item_name: name });
                        }
                    }
                    interact.active = None;
                    return;
                }
            }
            return;
        }
        Some(InteractMode::QuestBoard) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
                return;
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                handle_quest_turn_in(&mut game_world.0, 0);
                return;
            }

            let board_state = match game_world.0.get_resource::<QuestBoardState>() {
                Some(s) => s.clone(),
                None => return,
            };
            for i in 0..board_state.available_quests.len().min(9) {
                let key = match i {
                    0 => KeyCode::Digit1, 1 => KeyCode::Digit2, 2 => KeyCode::Digit3,
                    3 => KeyCode::Digit4, 4 => KeyCode::Digit5, 5 => KeyCode::Digit6,
                    6 => KeyCode::Digit7, 7 => KeyCode::Digit8, 8 => KeyCode::Digit9,
                    _ => continue,
                };
                if keyboard.just_pressed(key) {
                    let entry = board_state.available_quests[i].clone();
                    let accepted = game_core::quest::accept_board_quest(
                        &mut game_world.0, &entry,
                    );
                    if accepted.is_some() {
                        if let Some(mut bus) = game_world.0.get_resource_mut::<EventBus>() {
                            bus.push(GameEvent::Message("Quest accepted!".to_string()));
                        }
                    }
                    interact.active = None;
                    return;
                }
            }
            return;
        }
        Some(InteractMode::Looting { container_entity, cursor }) => {
            if keyboard.just_pressed(KeyCode::Escape) {
                interact.active = None;
                return;
            }

            let container = *container_entity;
            let cur = *cursor;

            if keyboard.just_pressed(KeyCode::ArrowUp) && cur > 0 {
                if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                    *cursor = cur - 1;
                }
                return;
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                let item_count = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if cur + 1 < item_count {
                    if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                        *cursor = cur + 1;
                    }
                }
                return;
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                let player = match game_world.0
                    .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    Ok(e) => e,
                    Err(_) => { interact.active = None; return; }
                };

                let inv = match game_world.0.get::<game_core::Inventory>(container) {
                    Some(i) => i.clone(),
                    None => { interact.active = None; return; }
                };

                if cur >= inv.items.len() {
                    interact.active = None;
                    return;
                }

                let item = inv.items[cur];
                let item_name = game_world.0.get::<game_core::Name>(item)
                    .map(|n| n.0.clone()).unwrap_or_default();

                if let Some(mut player_inv) = game_world.0.get_mut::<game_core::Inventory>(player) {
                    if player_inv.items.len() >= player_inv.capacity {
                        if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                            bus.push(game_core::GameEvent::Message("Inventory full.".to_string()));
                        }
                        return;
                    }
                    player_inv.items.push(item);
                }

                if let Some(mut container_inv) = game_world.0.get_mut::<game_core::Inventory>(container) {
                    container_inv.items.retain(|&e| e != item);
                }

                if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                    bus.push(game_core::GameEvent::Message(format!("Took {}.", item_name)));
                }

                let remaining = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if remaining == 0 {
                    game_world.0.entity_mut(container).despawn();
                    interact.active = None;
                } else if cur >= remaining {
                    if let Some(InteractMode::Looting { cursor, .. }) = &mut interact.active {
                        *cursor = remaining.saturating_sub(1);
                    }
                }
                return;
            }

            if keyboard.just_pressed(KeyCode::KeyT) {
                let player = match game_world.0
                    .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    Ok(e) => e,
                    Err(_) => { interact.active = None; return; }
                };

                let inv = match game_world.0.get::<game_core::Inventory>(container) {
                    Some(i) => i.clone(),
                    None => { interact.active = None; return; }
                };

                let mut taken = 0;
                for &item in &inv.items {
                    if let Some(mut player_inv) = game_world.0.get_mut::<game_core::Inventory>(player) {
                        if player_inv.items.len() >= player_inv.capacity { break; }
                        player_inv.items.push(item);
                        taken += 1;
                    }
                }

                if taken > 0 {
                    if let Some(mut bus) = game_world.0.get_resource_mut::<game_core::EventBus>() {
                        bus.push(game_core::GameEvent::Message(format!("Took {} item(s).", taken)));
                    }
                }

                let remaining = game_world.0.get::<game_core::Inventory>(container)
                    .map(|i| i.items.len()).unwrap_or(0);
                if remaining == 0 {
                    game_world.0.entity_mut(container).despawn();
                }
                interact.active = None;
                return;
            }
            return;
        }
        Some(_) => {
            return;
        }
        None => return,
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        interact.active = None;
        return;
    }

    for i in 0..targets.len().min(9) {
        let key = match i {
            0 => KeyCode::Digit1, 1 => KeyCode::Digit2, 2 => KeyCode::Digit3,
            3 => KeyCode::Digit4, 4 => KeyCode::Digit5, 5 => KeyCode::Digit6,
            6 => KeyCode::Digit7, 7 => KeyCode::Digit8, 8 => KeyCode::Digit9,
            _ => continue,
        };
        if keyboard.just_pressed(key) {
            let target = targets[i];
            let registry = match game_world.0.get_resource::<game_tags::TagRegistry>() {
                Some(r) => r.clone(),
                None => { interact.active = None; return; }
            };
            let tags = match game_world.0.get::<game_tags::Tags>(target) {
                Some(t) => t.clone(),
                None => { interact.active = None; return; }
            };

            let can_talk = registry.tag_id("CAN_TALK").is_some_and(|id| tags.has(id));
            let is_quest_board = game_world.0.get::<game_core::quest::QuestBoard>(target).is_some();
            let can_craft = registry.tag_id("CAN_CRAFT").is_some_and(|id| tags.has(id));
            let has_inventory = registry.tag_id("HAS_INVENTORY").is_some_and(|id| tags.has(id));
            let is_container = registry.tag_id("CONTAINER").is_some_and(|id| tags.has(id));

            if can_talk {
                interact.active = Some(InteractMode::Talk { npc_entity: target });
            } else if has_inventory && is_container {
                interact.active = Some(InteractMode::Looting { container_entity: target, cursor: 0 });
            } else if is_quest_board {
                interact.active = Some(InteractMode::QuestBoard);
            } else if can_craft {
                interact.active = Some(InteractMode::Crafting);
            } else {
                interact.active = None;
                if let Some(mut msg) = game_world.0.get_resource_mut::<game_core::MessageLog>() {
                    msg.messages.push("Nothing to do here.".to_string());
                }
            }
            return;
        }
    }
}
