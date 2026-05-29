use bevy_ecs::prelude::World;

use game_core::{EventBus, GameEvent, Health, Inventory, Player};

use crate::equipment::{get_item_display_name, get_quality_multiplier};

pub fn handle_consume(ecs_world: &mut World, cursor: usize) {
    use bevy_ecs::query::With;

    let player_entity = {
        let mut pq = ecs_world.query_filtered::<bevy_ecs::entity::Entity, With<Player>>();
        pq.single(ecs_world).ok()
    };
    let Some(player_entity) = player_entity else { return };

    let item_entity = {
        let inv = ecs_world.get::<Inventory>(player_entity);
        match inv {
            Some(inv) if cursor < inv.items.len() => inv.items[cursor],
            _ => return,
        }
    };

    let registry = ecs_world.resource::<game_tags::TagRegistry>().clone();

    let item_tags = ecs_world.get::<game_tags::Tags>(item_entity).cloned();
    let Some(item_tags) = item_tags else { return };

    let edible_id = registry.tag_id("EDIBLE");
    let drinkable_id = registry.tag_id("DRINKABLE");

    let is_consumable = edible_id.is_some_and(|id| item_tags.has(id))
        || drinkable_id.is_some_and(|id| item_tags.has(id));

    if !is_consumable {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("That item cannot be consumed.".to_string()));
        }
        return;
    }

    let item_name = get_item_display_name(ecs_world, item_entity, &registry);

    let herb_med_id = registry.tag_id("HERB_MEDICINAL");
    let herb_pois_id = registry.tag_id("HERB_POISONOUS");
    let food_wild_id = registry.tag_id("FOOD_WILD");
    let water_fresh_id = registry.tag_id("WATER_FRESH");
    let burning_id = registry.tag_id("BURNING");
    let poisoned_id = registry.tag_id("POISONED");

    let mut healed: u32 = 0;
    let mut poisoned = false;
    let mut extinguished = false;

    let quality_mult = get_quality_multiplier(&item_tags, &registry) as f32;

    if herb_med_id.is_some_and(|id| item_tags.has(id)) && edible_id.is_some_and(|id| item_tags.has(id)) {
        let base_heal: u32 = 22;
        healed = ((base_heal as f32) * quality_mult) as u32;
    }

    if herb_pois_id.is_some_and(|id| item_tags.has(id)) && edible_id.is_some_and(|id| item_tags.has(id)) {
        poisoned = true;
    }

    if food_wild_id.is_some_and(|id| item_tags.has(id)) && edible_id.is_some_and(|id| item_tags.has(id)) {
        healed += 5;
    }

    if drinkable_id.is_some_and(|id| item_tags.has(id)) && water_fresh_id.is_some_and(|id| item_tags.has(id)) {
        let mut player_tags = ecs_world.get_mut::<game_tags::Tags>(player_entity).unwrap();
        if let Some(bid) = burning_id
            && player_tags.has(bid) {
                player_tags.remove_tag(bid, &registry);
                extinguished = true;
            }
    }

    if healed > 0 {
        let mut hp = ecs_world.get_mut::<Health>(player_entity).unwrap();
        let before = hp.current;
        hp.current = hp.max.min(hp.current + healed);
        healed = hp.current - before;
    }

    if poisoned
        && let Some(pid) = poisoned_id {
            let mut player_tags = ecs_world.get_mut::<game_tags::Tags>(player_entity).unwrap();
            player_tags.add_tag(pid, game_tags::TagValue::Ticks { remaining: 12, max: 15 }, &registry);
        }

    {
        let mut inv = ecs_world.get_mut::<Inventory>(player_entity).unwrap();
        inv.items.retain(|&e| e != item_entity);
    }
    ecs_world.entity_mut(item_entity).despawn();

    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        bus.push(GameEvent::ItemConsumed {
            item_name,
            healed,
            poisoned,
            extinguished,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
    const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");

    use game_core::{Equipment, EventBus, Name, Position};

    fn setup_world() -> World {
        let mut world = World::new();
        let registry = game_tags::load_tag_registry(TAGS_TOML).expect("tags");
        let rules = game_tags::load_interaction_rules(INTERACTIONS_TOML, &registry).expect("rules");
        world.insert_resource(registry);
        world.insert_resource(rules);
        world
    }

    fn registry(world: &World) -> game_tags::TagRegistry {
        world.resource::<game_tags::TagRegistry>().clone()
    }

    #[test]
    fn consume_medicinal_item_heals_hp() {
        let mut world = setup_world();
        let reg = registry(&world);
        let edible_id = reg.tag_id("EDIBLE").unwrap();
        let herb_med_id = reg.tag_id("HERB_MEDICINAL").unwrap();

        let herb = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Medicinal Herb".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(herb).unwrap();
            tags.add_tag(edible_id, game_tags::TagValue::None, &reg);
            tags.add_tag(herb_med_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 50, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![herb], capacity: 20 },
            Equipment::default(),
        )).id();

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let hp = world.get::<Health>(player).unwrap();
        assert!(hp.current > 50, "player should be healed after consuming medicinal herb");
        assert!(hp.current <= 100, "HP should not exceed max");

        let inv = world.get::<Inventory>(player).unwrap();
        assert!(!inv.items.contains(&herb), "consumed item should be removed from inventory");

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(bus.events.iter().any(|e| matches!(e, GameEvent::ItemConsumed { healed, .. } if *healed > 0)), "message should mention healing");
    }

    #[test]
    fn consume_poisonous_item_applies_poisoned() {
        let mut world = setup_world();
        let reg = registry(&world);
        let edible_id = reg.tag_id("EDIBLE").unwrap();
        let herb_pois_id = reg.tag_id("HERB_POISONOUS").unwrap();
        let poisoned_id = reg.tag_id("POISONED").unwrap();

        let herb = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Poison Berry".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(herb).unwrap();
            tags.add_tag(edible_id, game_tags::TagValue::None, &reg);
            tags.add_tag(herb_pois_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![herb], capacity: 20 },
            Equipment::default(),
        )).id();

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let player_tags = world.get::<game_tags::Tags>(player).unwrap();
        assert!(player_tags.has(poisoned_id), "player should have POISONED status after consuming poisonous item");

        let inv = world.get::<Inventory>(player).unwrap();
        assert!(!inv.items.contains(&herb), "consumed item should be removed from inventory");

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(bus.events.iter().any(|e| matches!(e, GameEvent::ItemConsumed { poisoned: true, .. })), "message should mention feeling sick");
    }

    #[test]
    fn consume_wild_food_heals_small_amount() {
        let mut world = setup_world();
        let reg = registry(&world);
        let edible_id = reg.tag_id("EDIBLE").unwrap();
        let food_wild_id = reg.tag_id("FOOD_WILD").unwrap();

        let food = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Wild Berry".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(food).unwrap();
            tags.add_tag(edible_id, game_tags::TagValue::None, &reg);
            tags.add_tag(food_wild_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 80, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![food], capacity: 20 },
            Equipment::default(),
        )).id();

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 85, "wild food should heal 5 HP");
    }

    #[test]
    fn consume_water_removes_burning() {
        let mut world = setup_world();
        let reg = registry(&world);
        let drinkable_id = reg.tag_id("DRINKABLE").unwrap();
        let water_fresh_id = reg.tag_id("WATER_FRESH").unwrap();
        let burning_id = reg.tag_id("BURNING").unwrap();

        let water = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Fresh Water".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(water).unwrap();
            tags.add_tag(drinkable_id, game_tags::TagValue::None, &reg);
            tags.add_tag(water_fresh_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![water], capacity: 20 },
            Equipment::default(),
        )).id();
        {
            let mut player_tags = world.get_mut::<game_tags::Tags>(player).unwrap();
            player_tags.add_tag(burning_id, game_tags::TagValue::Ticks { remaining: 5, max: 10 }, &reg);
        }

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let player_tags = world.get::<game_tags::Tags>(player).unwrap();
        assert!(!player_tags.has(burning_id), "BURNING should be removed after drinking fresh water");

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(bus.events.iter().any(|e| matches!(e, GameEvent::ItemConsumed { extinguished: true, .. })), "message should mention flames");
    }

    #[test]
    fn consume_non_consumable_shows_message() {
        let mut world = setup_world();
        let reg = registry(&world);
        let metal_id = reg.tag_id("METAL").unwrap();

        let ore = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Iron Ore".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(ore).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![ore], capacity: 20 },
            Equipment::default(),
        )).id();

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 100, "HP should not change for non-consumable");

        let inv = world.get::<Inventory>(player).unwrap();
        assert!(inv.items.contains(&ore), "non-consumable should remain in inventory");

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(bus.events.iter().any(|e| matches!(e, GameEvent::Message(m) if m.contains("cannot be consumed"))), "message should say item cannot be consumed");
    }

    #[test]
    fn consume_healing_cannot_exceed_max_hp() {
        let mut world = setup_world();
        let reg = registry(&world);
        let edible_id = reg.tag_id("EDIBLE").unwrap();
        let herb_med_id = reg.tag_id("HERB_MEDICINAL").unwrap();

        let herb = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Medicinal Herb".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(herb).unwrap();
            tags.add_tag(edible_id, game_tags::TagValue::None, &reg);
            tags.add_tag(herb_med_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 99, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![herb], capacity: 20 },
            Equipment::default(),
        )).id();

        world.insert_resource(EventBus::new());

        handle_consume(&mut world, 0);

        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 100, "HP should not exceed max");
    }
}
