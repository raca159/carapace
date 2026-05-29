use bevy::prelude::*;
use game_core::{EventBus, GameEvent, MessageLog, Player, Inventory, Name, Position};
use game_core::barter::{BarterItem, BarterOffer, BarterResult, resolve_barter_with_haggle};
use game_core::emotion::NpcEmotionalState;
use game_tags::{TagRegistry, Tags};
use game_world::cascade::{CascadeEngine, RegionEconomies};
use game_world::cascade::economy::item_price_multiplier;
use crate::interact::{InteractState, InteractMode};

pub fn start_trade(
    ecs_world: &mut World,
    npc_entity: bevy_ecs::entity::Entity,
    interact_state: &mut InteractState,
) {
    // Build a simple barter offer using the NPC's inventory
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => { push_message(ecs_world, "Barter not available."); return; }
    };
    let cascade = match ecs_world.get_resource::<CascadeEngine>() {
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
