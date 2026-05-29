use bevy::prelude::*;
use game_core::{EventBus, GameEvent};
use game_core::emotion::NpcEmotionalState;
use game_tags::TagRegistry;
use game_world::cascade::CascadeEngine;
use crate::interact::InteractState;

pub fn start_trade(
    ecs_world: &mut World,
    npc_entity: bevy_ecs::entity::Entity,
    _interact_state: &mut InteractState,
) {
    let _registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => { push_message(ecs_world, "Barter not available."); return; }
    };
    let _cascade = match ecs_world.get_resource::<CascadeEngine>() {
        Some(c) => c.clone(),
        None => { push_message(ecs_world, "Barter not available."); return; }
    };

    // Show trade message for now
    push_message(ecs_world, "Trade initiated. (Full barter UI coming soon)");

    // Simple future: NPC offers items from their inventory pool
    // Resolve with faction standing and personality
    if let Some(mut state) = ecs_world.get_mut::<NpcEmotionalState>(npc_entity) {
        state.apply_event(-0.05);
    }
}

fn push_message(ecs_world: &mut World, msg: &str) {
    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        bus.push(GameEvent::Message(msg.to_string()));
    }
}
