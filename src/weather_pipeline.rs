use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

use game_core::{EnvironmentalScores, WeatherState, WeatherContext, weather_tags_for_context};
use game_tags::{TagId, TagRegistry, Tags, TagValue};
use game_world::biome::BiomeEnvironment;

pub fn apply_environmental_tags(world: &mut World) {
    let (weather_state, tag_registry, world_map) = {
        let ws = match world.get_resource::<WeatherState>() {
            Some(ws) => ws.clone(),
            None => return,
        };
        let reg = match world.get_resource::<TagRegistry>() {
            Some(r) => r.clone(),
            None => return,
        };
        let wm = world.get_resource::<game_world::WorldMap>().cloned();
        (ws, reg, wm)
    };

    let def = weather_state.active_weather();
    let old_applied = world.get_resource::<WeatherContext>()
        .map(|wc| wc.applied_tags.clone())
        .unwrap_or_default();

    // Collect descriptive weather tags (RAINY, STORMY, etc.) — environmental
    // condition tags (DARK, WET, COLD) come from score resolution per tile
    let descriptive_tags: Vec<TagId> = weather_tags_for_context(&weather_state, &weather_state.time)
        .iter()
        .filter_map(|name| tag_registry.tag_id(name))
        .collect();

    // Remove stale tags from all entities
    let stale_entities: Vec<Entity> = {
        let mut q = world.query::<Entity>();
        q.iter(world).collect()
    };
    for entity in stale_entities {
        if let Some(mut tags) = world.get_mut::<Tags>(entity) {
            for tag_id in &old_applied {
                if tags.has(*tag_id) {
                    tags.remove_tag(*tag_id, &tag_registry);
                }
            }
        }
    }

    // Apply per-tile environmental scores from biome data + weather
    if let Some(ref map) = world_map {
        let blocked_id = tag_registry.tag_id("BLOCKED");
        for &tile_entity in &map.tiles {
            // Skip blocked tiles (no gameplay tags needed there)
            if let Some(tags) = world.get::<Tags>(tile_entity) {
                if blocked_id.is_some_and(|id| tags.has(id)) {
                    continue;
                }
            }
            let base = world.get::<BiomeEnvironment>(tile_entity)
                .copied()
                .unwrap_or(BiomeEnvironment { light: 50, temperature: 50, moisture: 50 });
            let scores = EnvironmentalScores::compute(
                EnvironmentalScores { light: base.light, temperature: base.temperature, moisture: base.moisture },
                &def.modifiers,
                &weather_state.time,
            );
            let score_tags = scores.resolve_tags(&tag_registry);

            if let Some(mut tags) = world.get_mut::<Tags>(tile_entity) {
                for &tag_id in &score_tags {
                    if !tags.has(tag_id) {
                        tags.add_tag(tag_id, TagValue::None, &tag_registry);
                    }
                }
                // Apply descriptive weather tags to non-BLOCKS_WEATHER tiles
                let blocks_weather = tag_registry.tag_id("BLOCKS_WEATHER")
                    .is_some_and(|id| tags.has(id));
                if !blocks_weather {
                    for &tag_id in &descriptive_tags {
                        if !tags.has(tag_id) {
                            tags.add_tag(tag_id, TagValue::None, &tag_registry);
                        }
                    }
                }
            }
        }
    }

    // Apply score tags to WeatherSensitive entities (use global average as base)
    let base_entity_scores = EnvironmentalScores {
        light: 50,
        temperature: 50,
        moisture: 50,
    };
    let entity_score_tags = EnvironmentalScores::compute(base_entity_scores, &def.modifiers, &weather_state.time)
        .resolve_tags(&tag_registry);

    let sensitive_entities: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, bevy_ecs::query::With<game_core::WeatherSensitive>>();
        q.iter(world).collect()
    };
    for entity in &sensitive_entities {
        if let Some(mut tags) = world.get_mut::<Tags>(*entity) {
            for &tag_id in &entity_score_tags {
                if !tags.has(tag_id) {
                    tags.add_tag(tag_id, TagValue::None, &tag_registry);
                }
            }
            // Apply descriptive weather tags to entities too
            for &tag_id in &descriptive_tags {
                if !tags.has(tag_id) {
                    tags.add_tag(tag_id, TagValue::None, &tag_registry);
                }
            }
        }
    }

    // Update WeatherContext
    let all_tags: Vec<TagId> = entity_score_tags.iter()
        .chain(descriptive_tags.iter())
        .copied()
        .collect();
    if let Some(mut wc) = world.get_resource_mut::<WeatherContext>() {
        wc.applied_tags = all_tags;
        wc.tags = weather_tags_for_context(&weather_state, &weather_state.time);
    }
}
