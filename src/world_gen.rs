use bevy_ecs::prelude::World;

    use game_core::{Glyph, Health, Inventory, Player, Position, Equipment, WeatherSensitive};
use game_core::{WeatherState, WeatherContext};
use game_core::crafting::load_crafting_recipes;
use game_core::encounters::{Encounters, load_encounters};
use game_core::narrative::{LoreFragmentsResource, NarrativeEvents, load_lore_fragments, load_narrative_events};
use game_core::npc_action::load_npc_action_weights;
use game_core::weather_tags_for_context;
use game_world::{
    cascade::CascadeEngine, load_behavior_rules, load_biome_rules, load_factions,
    load_loot_tables, load_spawn_rules, load_world_config_str, populate_inventories, spawn_entities,
    LootTables, MapLayer, TilePos,
    WorldConfig, WorldMap, WorldPlugin, WorldSeed, BehaviorRules,
};

pub fn generate_world(ecs_world: &mut World, seed: WorldSeed, width: u32, height: u32) {
    let tags_toml = include_str!("../assets/config/tags.toml");
    let registry = game_tags::load_tag_registry(tags_toml).expect("Failed to load tags");
    ecs_world.insert_resource(registry.clone());

    let cascade = CascadeEngine::load(
        include_str!("../assets/config/items.toml"),
        include_str!("../assets/config/region_biomes.toml"),
        include_str!("../assets/config/faction_economy.toml"),
        include_str!("../assets/config/location_types.toml"),
    ).expect("Failed to load cascade engine configs");
    ecs_world.insert_resource(cascade);

    let interactions_toml = include_str!("../assets/config/interactions.toml");
    let interaction_rules = game_tags::load_interaction_rules(interactions_toml, &registry)
        .expect("Failed to load interaction rules");
    ecs_world.insert_resource(interaction_rules);

    let config = WorldConfig {
        seed,
        width,
        height,
    };
    ecs_world.insert_resource(config);

    let gen_config =
        load_world_config_str(include_str!("../assets/config/world.toml")).expect("Failed to load world config");
    let biome_classifier =
        load_biome_rules(include_str!("../assets/config/biome_rules.toml"))
            .expect("Failed to load biome rules");

    WorldPlugin::generate_and_spawn(ecs_world, gen_config, biome_classifier);

    // Stage 2a: Place locations on the map and compute economies
    let map = ecs_world.resource::<WorldMap>().clone();
    let width = map.width;
    let height = map.height;
    let config = ecs_world.resource::<WorldConfig>();
    let seed = config.seed;

    let mut tile_biomes: Vec<Vec<game_tags::TagId>> = Vec::with_capacity((width * height) as usize);
    let mut tile_query = ecs_world.query::<&game_tags::Tags>();
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let entity = map.tiles[idx];
            let tags = tile_query.get(ecs_world, entity).expect("tile should have Tags");
            tile_biomes.push(tags.iter_present().collect());
        }
    }

    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.0.wrapping_add(1000));
    let cascade = ecs_world.resource::<CascadeEngine>().clone();
    let registry = ecs_world.resource::<game_tags::TagRegistry>().clone();
    let locations = game_world::cascade::locations::place_locations(
        &cascade, &registry, width, height, &tile_biomes, &mut rng,
    );
    ecs_world.insert_resource(game_world::cascade::LocationMap { locations: locations.clone() });

    let mut economies = std::collections::HashMap::new();
    for loc in &locations {
        let idx = (loc.y * width + loc.x) as usize;
        let biome_tags = &tile_biomes[idx];
        let pricing = game_world::cascade::economy::compute_location_economy(
            biome_tags, loc.faction.as_deref(), &loc.tags, &cascade, &registry,
        );
        economies.insert(loc.id, pricing);
    }

    // Stage 2b: Trade routes between economies
    use game_world::cascade::trade::{generate_trade_routes, apply_trade_to_economy, TradeRoutes};
    let trade_routes = generate_trade_routes(&locations, 3);
    apply_trade_to_economy(&trade_routes, &mut economies);
    ecs_world.insert_resource(TradeRoutes(trade_routes));
    ecs_world.insert_resource(game_world::cascade::RegionEconomies { economies });

    let factions_toml = include_str!("../assets/config/factions.toml");
    let (_, faction_relationships) = load_factions(factions_toml).expect("Failed to load factions");
    ecs_world.insert_resource(faction_relationships);

    let behavior_toml = include_str!("../assets/config/behavior_rules.toml");
    let behavior_rules = load_behavior_rules(behavior_toml).expect("Failed to load behavior rules");
    ecs_world.insert_resource(BehaviorRules(behavior_rules));

    let quests_toml = include_str!("../assets/config/quests.toml");
    let quest_templates = game_core::quest::load_quest_templates(quests_toml).expect("Failed to load quest templates");
    ecs_world.insert_resource(game_core::QuestTemplates { templates: quest_templates });

    let npc_toml = include_str!("../assets/config/npc_personalities.toml");
    let npc_personalities = game_core::npc_personality::load_npc_personalities(npc_toml).expect("Failed to load NPC personalities");
    ecs_world.insert_resource(game_core::npc_personality::NpcPersonalitiesResource { personalities: npc_personalities });

    let actions_toml = include_str!("../assets/config/npc_actions.toml");
    let action_weights = load_npc_action_weights(actions_toml).expect("Failed to load NPC action weights");
    ecs_world.insert_resource(action_weights);

    let dialogue_toml = include_str!("../assets/config/dialogue.toml");
    let dialogue_lines = game_core::dialogue::load_dialogue(dialogue_toml)
        .expect("Failed to load dialogue");
    ecs_world.insert_resource(game_core::DialogueLinesResource { lines: dialogue_lines });

    let crafting_toml = include_str!("../assets/config/crafting.toml");
    let crafting_recipes = load_crafting_recipes(crafting_toml)
        .expect("Failed to load crafting recipes");
    ecs_world.insert_resource(crate::interact::craft::CraftingRecipesResource { recipes: crafting_recipes });

    let encounters_toml = include_str!("../assets/config/encounters.toml");
    let encounter_defs = load_encounters(encounters_toml).expect("Failed to load encounters");
    ecs_world.insert_resource(Encounters::new(encounter_defs));

    let narrative_toml = include_str!("../assets/config/narrative_events.toml");
    let narrative_events = load_narrative_events(narrative_toml).expect("Failed to load narrative events");
    ecs_world.insert_resource(NarrativeEvents { events: narrative_events });

    let loot_toml = include_str!("../assets/config/loot_tables.toml");
    let loot_table_defs = load_loot_tables(loot_toml).expect("Failed to load loot tables");
    ecs_world.insert_resource(LootTables { tables: loot_table_defs });

    let lore_toml = include_str!("../assets/config/lore_fragments.toml");
    let lore_fragments = load_lore_fragments(lore_toml).expect("Failed to load lore fragments");
    ecs_world.insert_resource(LoreFragmentsResource { fragments: lore_fragments });

    ecs_world.insert_resource(MapLayer::default());
    ecs_world.insert_resource(WeatherState::new());
    let ws = ecs_world.get_resource::<WeatherState>().unwrap();
    let initial_tags = weather_tags_for_context(ws, &ws.time);
    ecs_world.insert_resource(WeatherContext { tags: initial_tags, ..Default::default() });

    ecs_world.insert_resource(game_core::world_overview::WorldOverviewState::new(
        game_core::world_overview::WorldOverviewMode::ReadOnly,
    ));
}

pub fn spawn_player(ecs_world: &mut World, camera: &mut game_render::Camera) -> TilePos {
    let registry = ecs_world
        .resource::<game_tags::TagRegistry>()
        .clone();
    let map = ecs_world.resource::<WorldMap>().clone();

    let walkable_id = registry.tag_id("WALKABLE").expect("WALKABLE tag must exist");

    let center_x = map.width / 2;
    let center_y = map.height / 2;

    let mut spawn_pos = TilePos::new(center_x, center_y);

    'search: for radius in 1..map.width.max(map.height) {
        for &(sx, sy) in &[
            (center_x + radius, center_y),
            (center_x.saturating_sub(radius), center_y),
            (center_x, center_y + radius),
            (center_x, center_y.saturating_sub(radius)),
            (center_x + radius, center_y + radius),
            (center_x + radius, center_y.saturating_sub(radius)),
            (center_x.saturating_sub(radius), center_y + radius),
            (
                center_x.saturating_sub(radius),
                center_y.saturating_sub(radius),
            ),
        ] {
            if sx >= map.width || sy >= map.height {
                continue;
            }
            let pos = TilePos::new(sx, sy);
            if let Some(entity) = map.get(pos) {
                let mut query = ecs_world.query::<&game_tags::Tags>();
                if let Ok(tags) = query.get(ecs_world, entity)
                    && tags.has(walkable_id)
                {
                    spawn_pos = pos;
                    break 'search;
                }
            }
        }
    }

    let player_tags = game_tags::Tags::new(registry.tag_count());

    ecs_world.spawn((
        Player,
        WeatherSensitive,
        Position { x: spawn_pos.x, y: spawn_pos.y, z: 0 },
        Health {
            current: 100,
            max: 100,
        },
        Glyph {
            char: '@',
            color: (255, 255, 0),
        },
        player_tags,
        Inventory {
            items: vec![],
            capacity: 20,
        },
        Equipment::default(),
    ));

    camera.centered_on(spawn_pos, &map);

    spawn_pos
}

pub fn spawn_game_entities(ecs_world: &mut World, player_pos: TilePos) {
    let spawn_rules_toml = include_str!("../assets/config/spawn_rules.toml");
    let rules = match load_spawn_rules(spawn_rules_toml) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to load spawn rules: {}", e);
            return;
        }
    };
    spawn_entities(ecs_world, &rules, player_pos);
    populate_inventories(ecs_world, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_world_inserts_interaction_rules() {
        let mut world = World::new();
        let seed = WorldSeed::from_value(42);
        generate_world(&mut world, seed, 20, 20);

        assert!(world.get_resource::<game_tags::InteractionRules>().is_some(),
            "InteractionRules should be inserted as a resource");
        assert!(world.get_resource::<game_tags::TagRegistry>().is_some(),
            "TagRegistry should be inserted as a resource");
    }
}
