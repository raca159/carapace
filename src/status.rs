use bevy_ecs::prelude::World;

use game_core::Position;

use crate::event_format;

pub fn process_status_effects(world: &mut World) {
    let registry = match world.get_resource::<game_tags::TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let rules = match world.get_resource::<game_tags::InteractionRules>() {
        Some(r) => r.clone(),
        None => return,
    };

    // Phase 1: tick + self-interactions
    let entities: Vec<(bevy_ecs::entity::Entity, game_tags::Tags)> = {
        let mut query = world.query::<(bevy_ecs::entity::Entity, &game_tags::Tags)>();
        query.iter(world).map(|(e, t)| (e, t.clone())).collect()
    };

    for (entity, old_tags) in &entities {
        let mut updated_tags = old_tags.clone();
        updated_tags.tick_status(&registry);

        let self_matches = rules.check_self_interactions(&updated_tags);
        for rule in &self_matches {
            for &produced in &rule.produces {
                updated_tags.add_tag(produced, game_tags::TagValue::None, &registry);
            }
            for &consumed in &rule.consumes {
                updated_tags.remove_tag(consumed, &registry);
            }
        }

        // Write updated tags back
        if let Some(mut tags) = world.get_mut::<game_tags::Tags>(*entity) {
            *tags = updated_tags;
        }

        // Emit events for this entity's self-interactions
        event_format::emit_tag_diff_events(world, *entity, old_tags);
        event_format::emit_self_interaction_events(world, *entity, &self_matches, old_tags);
    }

    // Phase 2: cross-interactions between adjacent entities
    let positioned: Vec<(bevy_ecs::entity::Entity, game_tags::Tags, Position)> = {
        let mut query = world.query::<(bevy_ecs::entity::Entity, &game_tags::Tags, &Position)>();
        query.iter(world).map(|(e, t, p)| (e, t.clone(), *p)).collect()
    };

    let mut already_updated = std::collections::HashSet::new();

    for i in 0..positioned.len() {
        let (entity_a, tags_a, pos_a) = &positioned[i];
        for (entity_b, tags_b, pos_b) in positioned.iter().skip(i + 1) {

            let dx = (pos_a.x as i32 - pos_b.x as i32).unsigned_abs();
            let dy = (pos_a.y as i32 - pos_b.y as i32).unsigned_abs();

            if dx > 1 || dy > 1 {
                continue;
            }

            let matched = rules.check_cross_interactions(tags_a, tags_b);
            if matched.is_empty() {
                continue;
            }

            // Emit cross-interaction events
            event_format::emit_cross_interaction_events(world, *entity_a, *entity_b, &matched);

            let mut new_a = tags_a.clone();
            let mut new_b = tags_b.clone();

            for (rule, reversed) in &matched {
                let tag_b_holder = if *reversed { &mut new_a } else { &mut new_b };

                for &produced in &rule.produces {
                    tag_b_holder.add_tag(produced, game_tags::TagValue::None, &registry);
                }
                for &consumed in &rule.consumes {
                    tag_b_holder.remove_tag(consumed, &registry);
                }
            }

            if !already_updated.contains(entity_a) {
                event_format::emit_tag_diff_events(world, *entity_a, tags_a);
                if let Some(mut tags) = world.get_mut::<game_tags::Tags>(*entity_a) {
                    *tags = new_a;
                }
                already_updated.insert(*entity_a);
            }

            if !already_updated.contains(entity_b) {
                event_format::emit_tag_diff_events(world, *entity_b, tags_b);
                if let Some(mut tags) = world.get_mut::<game_tags::Tags>(*entity_b) {
                    *tags = new_b;
                }
                already_updated.insert(*entity_b);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::World;

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
    const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");

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
    fn process_status_effects_ticks_down_burning() {
        let mut world = setup_world();
        let reg = registry(&world);
        let burning = reg.tag_id("BURNING").unwrap();

        world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 5, y: 5, z: 0 },
        ));

        let entity = world.query::<(bevy_ecs::entity::Entity, &game_tags::Tags)>()
            .iter(&world)
            .next()
            .unwrap().0;

        {
            let mut tags = world.get_mut::<game_tags::Tags>(entity).unwrap();
            tags.add_tag(burning, game_tags::TagValue::Ticks { remaining: 3, max: 5 }, &reg);
        }

        assert!(world.get::<game_tags::Tags>(entity).unwrap().has(burning));

        process_status_effects(&mut world);
        assert!(world.get::<game_tags::Tags>(entity).unwrap().has(burning), "still present after tick 1");

        process_status_effects(&mut world);
        assert!(world.get::<game_tags::Tags>(entity).unwrap().has(burning), "still present after tick 2");

        process_status_effects(&mut world);
        assert!(!world.get::<game_tags::Tags>(entity).unwrap().has(burning), "expired after tick 3");
    }

    #[test]
    fn process_status_effects_self_interaction_fire_flammable_produces_burning() {
        let mut world = setup_world();
        let reg = registry(&world);
        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();
        let burning = reg.tag_id("BURNING").unwrap();

        let entity = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 10, y: 10, z: 0 },
        )).id();

        {
            let mut tags = world.get_mut::<game_tags::Tags>(entity).unwrap();
            tags.add_tag(fire, game_tags::TagValue::None, &reg);
            tags.add_tag(flammable, game_tags::TagValue::None, &reg);
        }

        process_status_effects(&mut world);

        let tags = world.get::<game_tags::Tags>(entity).unwrap();
        assert!(tags.has(burning), "FIRE + FLAMMABLE should produce BURNING via self-interaction");
    }

    #[test]
    fn process_status_effects_cross_interaction_water_extinguishes_fire() {
        let mut world = setup_world();
        let reg = registry(&world);
        let fire = reg.tag_id("FIRE").unwrap();
        let water = reg.tag_id("WATER").unwrap();

        let entity_a = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 5, y: 5, z: 0 },
        )).id();

        let entity_b = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 6, y: 5, z: 0 },
        )).id();

        {
            let mut tags_a = world.get_mut::<game_tags::Tags>(entity_a).unwrap();
            tags_a.add_tag(water, game_tags::TagValue::None, &reg);
            let mut tags_b = world.get_mut::<game_tags::Tags>(entity_b).unwrap();
            tags_b.add_tag(fire, game_tags::TagValue::None, &reg);
        }

        assert!(world.get::<game_tags::Tags>(entity_b).unwrap().has(fire));

        process_status_effects(&mut world);

        assert!(!world.get::<game_tags::Tags>(entity_b).unwrap().has(fire),
            "WATER adjacent to FIRE should consume FIRE via cross-interaction");
    }

    #[test]
    fn process_status_effects_no_resources_no_panic() {
        let mut world = World::new();
        process_status_effects(&mut world);

        world.insert_resource(game_tags::load_tag_registry(TAGS_TOML).unwrap());
        process_status_effects(&mut world);
    }

    #[test]
    fn process_status_effects_multiple_entities_tick() {
        let mut world = setup_world();
        let reg = registry(&world);
        let burning = reg.tag_id("BURNING").unwrap();

        let e1 = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 1, y: 1, z: 0 },
        )).id();

        let e2 = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 2, y: 2, z: 0 },
        )).id();

        {
            let mut t1 = world.get_mut::<game_tags::Tags>(e1).unwrap();
            t1.add_tag(burning, game_tags::TagValue::Ticks { remaining: 1, max: 1 }, &reg);
            let mut t2 = world.get_mut::<game_tags::Tags>(e2).unwrap();
            t2.add_tag(burning, game_tags::TagValue::Ticks { remaining: 2, max: 2 }, &reg);
        }

        process_status_effects(&mut world);

        assert!(!world.get::<game_tags::Tags>(e1).unwrap().has(burning), "e1 BURNING expired");
        assert!(world.get::<game_tags::Tags>(e2).unwrap().has(burning), "e2 BURNING still present");
    }

    #[test]
    fn process_status_effects_cross_interaction_non_adjacent_ignored() {
        let mut world = setup_world();
        let reg = registry(&world);
        let fire = reg.tag_id("FIRE").unwrap();
        let water = reg.tag_id("WATER").unwrap();

        let entity_a = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 5, y: 5, z: 0 },
        )).id();

        let entity_b = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Position { x: 20, y: 20, z: 0 },
        )).id();

        {
            let mut tags_a = world.get_mut::<game_tags::Tags>(entity_a).unwrap();
            tags_a.add_tag(water, game_tags::TagValue::None, &reg);
            let mut tags_b = world.get_mut::<game_tags::Tags>(entity_b).unwrap();
            tags_b.add_tag(fire, game_tags::TagValue::None, &reg);
        }

        assert!(world.get::<game_tags::Tags>(entity_b).unwrap().has(fire));

        process_status_effects(&mut world);

        assert!(world.get::<game_tags::Tags>(entity_b).unwrap().has(fire),
            "Non-adjacent entities should not interact");
    }
}
