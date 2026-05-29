use bevy::prelude::*;
use game_core::{EventBus, GameEvent, Inventory, Name};
use game_core::barter::{BarterItem, BarterOffer, resolve_barter_with_haggle};
use game_core::emotion::NpcEmotionalState;
use game_tags::{TagRegistry, Tags};
use game_world::cascade::{CascadeEngine, RegionEconomies, LocationMap};
use game_world::cascade::locations::location_at;
use crate::interact::InteractState;

pub fn start_trade(
    ecs_world: &mut World,
    npc_entity: bevy_ecs::entity::Entity,
    interact_state: &mut InteractState,
) {
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => { push_message(ecs_world, "Barter not available."); return; }
    };
    let cascade = match ecs_world.get_resource::<CascadeEngine>() {
        Some(c) => c.clone(),
        None => { push_message(ecs_world, "Barter not available."); return; }
    };
    let economies = match ecs_world.get_resource::<RegionEconomies>() {
        Some(e) => e.clone(),
        None => { push_message(ecs_world, "No local economy data."); return; }
    };
    let location_map = ecs_world.get_resource::<LocationMap>()
        .map(|lm| lm.locations.clone());

    let npc_tags = match ecs_world.get::<Tags>(npc_entity) {
        Some(t) => t.clone(),
        None => { push_message(ecs_world, "Cannot assess merchandise."); return; }
    };

    let npc_inventory = match ecs_world.get::<Inventory>(npc_entity) {
        Some(i) => i.items.clone(),
        None => { push_message(ecs_world, "Nothing to trade."); return; }
    };

    if npc_inventory.is_empty() {
        push_message(ecs_world, "They have nothing to trade.");
        return;
    }

    let player_pos = match ecs_world
        .query_filtered::<&game_core::Position, bevy_ecs::query::With<game_core::Player>>()
        .single(ecs_world)
    {
        Ok(p) => (p.x, p.y),
        Err(_) => { push_message(ecs_world, "Cannot find you."); return; }
    };

    // Find nearest location economy for pricing
    let econ_id = location_map.as_ref().and_then(|locs| {
        location_at(locs, player_pos.0, player_pos.1).map(|loc| loc.id)
    });

    let pricing = econ_id.and_then(|id| economies.economies.get(&id)).cloned();

    // Build tradeable items from NPC inventory with pricing
    let mut npc_items: Vec<BarterItem> = Vec::new();

    for &item_entity in &npc_inventory {
        let base_value = item_base_value(item_entity, ecs_world, &registry, &cascade);
        let name = ecs_world.get::<Name>(item_entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| "item".to_string());

        let price_mult = pricing.as_ref().map(|p| {
            let mut mult = 1.0f32;
            if let Some(item_tags) = ecs_world.get::<Tags>(item_entity) {
                for tag_id in item_tags.iter_present() {
                    let tag_name = &registry.tag_by_id(tag_id).name;
                    if let Some(tag_mult) = p.price_multipliers.get(tag_name) {
                        mult *= tag_mult;
                    }
                }
            }
            mult
        }).unwrap_or(1.0);

        let adjusted_value = (base_value as f32 * price_mult).round() as u32;
        if adjusted_value > 0 {
            npc_items.push(BarterItem {
                name,
                quantity: 1,
                base_value: adjusted_value,
            });
        }
    }

    if npc_items.is_empty() {
        push_message(ecs_world, "No tradeable items available.");
        return;
    }

    // Get player inventory for potential exchange
    let player_items: Vec<BarterItem> = {
        let player = match ecs_world
            .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<game_core::Player>>()
            .single(ecs_world)
        {
            Ok(e) => e,
            Err(_) => return,
        };
        let inv = match ecs_world.get::<Inventory>(player) {
            Some(i) => i.items.clone(),
            None => return,
        };
        inv.iter().map(|&item_entity| {
            let base_value = item_base_value(item_entity, ecs_world, &registry, &cascade);
            let name = ecs_world.get::<Name>(item_entity)
                .map(|n| n.0.clone())
                .unwrap_or_else(|| "item".to_string());
            BarterItem { name, quantity: 1, base_value }
        }).collect()
    };

    // Build first available offer: offer NPC's priciest item, ask for player's best item
    let offer = BarterOffer {
        offered: vec![npc_items[0].clone()],
        requested: if !player_items.is_empty() {
            vec![player_items[0].clone()]
        } else {
            vec![]
        },
    };

    let mut rng = rand::rng();
    let result = resolve_barter_with_haggle(
        &offer,
        Some(&npc_tags),
        Some(&npc_tags),
        Some(&registry),
        &mut rng,
    );

    let prosperity_note = pricing.as_ref().map(|p| {
        if p.prosperity > 0.7 { " (prosperous)" }
        else if p.prosperity < 0.3 { " (poor)" }
        else { "" }
    }).unwrap_or("");

    if result.accepted {
        push_message(ecs_world, &format!(
            "Trade accepted! {} for {}. Economy{}",
            offer.offered[0].name, offer.requested[0].name, prosperity_note,
        ));
    } else {
        push_message(ecs_world, &format!(
            "Trade rejected: need better offer. Economy{}",
            prosperity_note,
        ));
    }

    if let Some(mut state) = ecs_world.get_mut::<NpcEmotionalState>(npc_entity) {
        state.apply_event(-0.05);
    }

    *interact_state = InteractState::default();
}

fn item_base_value(
    item_entity: bevy_ecs::entity::Entity,
    ecs_world: &World,
    registry: &TagRegistry,
    cascade: &CascadeEngine,
) -> u32 {
    let tags = match ecs_world.get::<Tags>(item_entity) {
        Some(t) => t,
        None => return 5,
    };

    let name = match ecs_world.get::<Name>(item_entity) {
        Some(n) => &n.0,
        None => return 5,
    };

    if let Some(item_def) = cascade.item_by_id.get(name) {
        let base_val = item_def.base_value;
        let quality_mult: u32 = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"]
            .iter()
            .filter_map(|q| registry.tag_id(q))
            .find(|&qid| tags.has(qid))
            .and_then(|qid| registry.tag_by_id(qid).multiplier.map(|m| m as u32))
            .unwrap_or(1);
        return base_val.saturating_mul(quality_mult);
    }

    5
}

fn push_message(ecs_world: &mut World, msg: &str) {
    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        bus.push(GameEvent::Message(msg.to_string()));
    }
}
