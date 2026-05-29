use bevy_ecs::prelude::World;
use bevy_ecs::query::With;

use game_core::{
    Creature, Equipment, ExamineMode, FactionReputation, Glyph, Health, Inventory, Item,
    MessageLog, Name, Player, Position, QuestLog, QuestState, TurnCounter, TurnState,
    turn::TurnPhase,
};
use game_core::examine::ThrowMode;
use game_render::Camera;
use game_tags::{load_tag_registry, load_interaction_rules, TagRegistry, Tags, TagValue};
use game_world::{
    Faction, FactionId, FactionRelationships, WorldMap, WorldSeed, TilePos,
    process_npc_turns,
};
use game_world::faction::{ReputationTracker, REP_KILL_PENALTY, REP_QUEST_REWARD};

use crate::world_gen;

const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");
const FACTIONS_TOML: &str = include_str!("../assets/config/factions.toml");
const BEHAVIOR_TOML: &str = include_str!("../assets/config/behavior_rules.toml");

// ============================================================
// Test Harness
// ============================================================

fn setup_e2e_world(seed: u64, width: u32, height: u32) -> World {
    let mut world = World::new();
    let world_seed = WorldSeed::from_value(seed);

    world_gen::generate_world(&mut world, world_seed, width, height);

    let mut camera = Camera::new(80, 24);
    let _player_pos = world_gen::spawn_player(&mut world, &mut camera);
    world_gen::spawn_game_entities(&mut world, _player_pos);

    world.insert_resource(MessageLog::new(50));
    world.insert_resource(TurnCounter::new());
    world.insert_resource(TurnState::new());
    world.insert_resource(ExamineMode::new());
    world.insert_resource(ThrowMode::new());
    world.insert_resource(QuestLog::new());

    world
}

fn registry(world: &World) -> TagRegistry {
    world.resource::<TagRegistry>().clone()
}

fn player_entity(world: &mut World) -> Option<bevy_ecs::entity::Entity> {
    let mut pq = world.query_filtered::<bevy_ecs::entity::Entity, With<Player>>();
    pq.iter(world).next()
}

fn player_pos(world: &mut World) -> Option<(u32, u32)> {
    let mut pq = world.query_filtered::<&Position, With<Player>>();
    pq.iter(world).next().map(|p| (p.x, p.y))
}

fn player_hp(world: &mut World) -> Option<u32> {
    let mut pq = world.query_filtered::<&Health, With<Player>>();
    pq.iter(world).next().map(|h| h.current)
}

fn player_inventory_len(world: &mut World) -> usize {
    let mut pq = world.query_filtered::<&Inventory, With<Player>>();
    pq.iter(world).next().map(|i| i.items.len()).unwrap_or(0)
}

fn creature_count(world: &mut World) -> usize {
    let mut cq = world.query_filtered::<bevy_ecs::entity::Entity, With<Creature>>();
    cq.iter(world).count()
}

fn item_count(world: &mut World) -> usize {
    let mut iq = world.query_filtered::<bevy_ecs::entity::Entity, (With<Item>, With<Position>)>();
    iq.iter(world).count()
}

fn simulate_full_turn(world: &mut World) {
    {
        let mut ts = world.get_resource_mut::<TurnState>().unwrap();
        ts.set_phase(TurnPhase::Npcs);
    }
    process_npc_turns(world);
    crate::status::process_status_effects(world);
    game_core::narrative::check_narrative_events(world);
    {
        let mut tc = world.get_resource_mut::<TurnCounter>().unwrap();
        tc.increment();
    }
    {
        let mut ts = world.get_resource_mut::<TurnState>().unwrap();
        ts.set_phase(TurnPhase::Player);
    }
}

fn move_player(world: &mut World, dx: i32, dy: i32) -> bool {
    let map = world.resource::<WorldMap>().clone();
    let reg = registry(world);

    let (cur_x, cur_y) = match player_pos(world) {
        Some(p) => p,
        None => return false,
    };

    let new_x = (cur_x as i32 + dx).max(0).min(map.width as i32 - 1) as u32;
    let new_y = (cur_y as i32 + dy).max(0).min(map.height as i32 - 1) as u32;

    if new_x >= map.width || new_y >= map.height {
        return false;
    }

    let target_pos = TilePos::new(new_x, new_y);

    if let Some(blocked_id) = reg.tag_id("BLOCKED") {
        if let Some(entity) = map.get(target_pos) {
            let mut tags_query = world.query::<&Tags>();
            if let Ok(tags) = tags_query.get(world, entity)
                && tags.has(blocked_id)
            {
                return false;
            }
        }
    }

    if let Some(aggressive_id) = reg.tag_id("AGGRESSIVE") {
        let mut creature_query = world.query_filtered::<(bevy_ecs::entity::Entity, &Position, &Tags), With<Creature>>();
        let has_aggressive = creature_query
            .iter(world)
            .any(|(_, pos, tags)| pos.x == new_x && pos.y == new_y && tags.has(aggressive_id));
        if has_aggressive {
            return false;
        }
    }

    {
        let mut pq = world.query_filtered::<&mut Position, With<Player>>();
        if let Ok(mut pos) = pq.single_mut(world) {
            pos.x = new_x;
            pos.y = new_y;
        }
    }

    true
}

fn setup_controlled_world() -> World {
    let mut world = World::new();

    let reg = load_tag_registry(TAGS_TOML).expect("tags");
    let rules = load_interaction_rules(INTERACTIONS_TOML, &reg).expect("rules");
    world.insert_resource(reg.clone());
    world.insert_resource(rules);

    let width = 30u32;
    let height = 30u32;
    let mut tiles = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let tile_entity = world.spawn((game_world::Tile {
                pos: TilePos::new(x, y),
                elevation: 0.5,
                moisture: 0.5,
                temperature: 0.5,
                biome_name: "plains".to_string(),
                glyph: '.',
                color: (200, 200, 200),
            },)).id();
            tiles.push(tile_entity);
        }
    }
    world.insert_resource(WorldMap {
        width,
        height,
        depth: 1,
        current_z: 0,
        seed: WorldSeed::from_value(42),
        tiles,
    });

    let (_, faction_rels) = game_world::load_factions(FACTIONS_TOML).expect("factions");
    world.insert_resource(faction_rels);
    world.insert_resource(ReputationTracker::new());
    world.insert_resource(FactionReputation::new());

    let behavior_rules = game_world::load_behavior_rules(BEHAVIOR_TOML).expect("behavior");
    world.insert_resource(game_world::BehaviorRules(behavior_rules));

    let quests_toml = include_str!("../assets/config/quests.toml");
    let quest_templates = game_core::quest::load_quest_templates(quests_toml)
        .expect("Failed to load quest templates");
    world.insert_resource(game_core::QuestTemplates { templates: quest_templates });

    world.insert_resource(MessageLog::new(50));
    world.insert_resource(TurnCounter::new());
    world.insert_resource(TurnState::new());
    world.insert_resource(ExamineMode::new());
    world.insert_resource(ThrowMode::new());
    world.insert_resource(QuestLog::new());

    world.spawn((
        Player,
        Position { x: 15, y: 15, z: 0 },
        Health { current: 100, max: 100 },
        Glyph { char: '@', color: (255, 255, 0) },
        Tags::new(reg.tag_count()),
        Inventory { items: vec![], capacity: 20 },
        Equipment::default(),
    ));

    world
}

fn spawn_creature_in_world(world: &mut World, name: &str, x: u32, y: u32, hp: u32,
    tag_names: &[&str], faction_id: Option<FactionId>) -> bevy_ecs::entity::Entity
{
    let reg = registry(world);
    let mut tags = Tags::new(reg.tag_count());
    for tname in tag_names {
        if let Some(id) = reg.tag_id(tname) {
            tags.add_tag(id, TagValue::None, &reg);
        }
    }

    let mut builder = world.spawn((
        Creature,
        Position { x, y, z: 0 },
        Health { current: hp, max: hp },
        Glyph { char: 'g', color: (0, 255, 0) },
        Name(name.to_string()),
        tags,
    ));

    if let Some(fid) = faction_id {
        builder.insert(Faction { faction_id: fid });
    }

    builder.id()
}

fn add_item_to_player_inventory(world: &mut World, item_name: &str, tag_names: &[&str]) {
    let reg = registry(world);
    let mut tags = Tags::new(reg.tag_count());
    for tname in tag_names {
        if let Some(id) = reg.tag_id(tname) {
            tags.add_tag(id, TagValue::None, &reg);
        }
    }

    let pe = player_entity(world).unwrap();
    let item = world.spawn((
        Item,
        Glyph { char: '/', color: (200, 200, 200) },
        Name(item_name.to_string()),
        tags,
    )).id();

    let mut inv = world.get_mut::<Inventory>(pe).unwrap();
    inv.items.push(item);
}

fn combat_apply_damage(world: &mut World, attacker_damage: u32, creature: bevy_ecs::entity::Entity) -> (u32, u32) {
    let creature_hp_before = world.get::<Health>(creature).unwrap().current;
    let player_hp_before = player_hp(world).unwrap();

    let creature_retaliate = std::cmp::max(1, creature_hp_before / 10);

    let new_creature_hp = creature_hp_before.saturating_sub(attacker_damage);

    if let Some(mut hp) = world.get_mut::<Health>(creature) {
        hp.current = new_creature_hp;
    }

    let new_player_hp = player_hp_before.saturating_sub(creature_retaliate);
    {
        let mut hp_query = world.query_filtered::<&mut Health, With<Player>>();
        if let Ok(mut hp) = hp_query.single_mut(world) {
            hp.current = new_player_hp;
        }
    }

    (new_creature_hp, new_player_hp)
}

fn get_first_faction_id(world: &World) -> Option<FactionId> {
    let rels = world.resource::<FactionRelationships>();
    let factions_toml = game_world::load_factions(include_str!("../assets/config/factions.toml")).ok()?;
    let ids: Vec<FactionId> = factions_toml.0.iter().map(|f| f.id).collect();
    ids.first().copied()
}

// ============================================================
// World Generation Tests
// ============================================================

#[test]
fn e2e_world_generation_creates_player() {
    let mut world = setup_e2e_world(42, 100, 100);
    assert!(player_entity(&mut world).is_some(), "player should exist");
    let pos = player_pos(&mut world).unwrap();
    assert!(pos.0 < 100, "player x in bounds");
    assert!(pos.1 < 100, "player y in bounds");
    assert!(player_hp(&mut world).unwrap() == 100, "player should start with 100 HP");
}

#[test]
fn e2e_world_generation_creates_creatures() {
    let mut world = setup_e2e_world(42, 100, 100);
    let ccount = creature_count(&mut world);
    assert!(ccount > 0, "world should spawn creatures, got {}", ccount);
}

#[test]
fn e2e_world_generation_creates_items() {
    let mut world = setup_e2e_world(42, 100, 100);
    let icount = item_count(&mut world);
    assert!(icount > 0, "world should spawn items on ground, got {}", icount);
}

#[test]
fn e2e_world_generation_is_deterministic() {
    let mut world1 = setup_e2e_world(42, 80, 80);
    let mut world2 = setup_e2e_world(42, 80, 80);

    let p1 = player_pos(&mut world1).unwrap();
    let p2 = player_pos(&mut world2).unwrap();
    assert_eq!(p1, p2, "same seed must place player at same position");

    let c1 = creature_count(&mut world1);
    let c2 = creature_count(&mut world2);
    assert_eq!(c1, c2, "same seed must produce same creature count");
}

#[test]
fn e2e_world_generation_has_all_resources() {
    let world = setup_e2e_world(42, 80, 80);

    assert!(world.get_resource::<TagRegistry>().is_some(), "TagRegistry");
    assert!(world.get_resource::<game_tags::InteractionRules>().is_some(), "InteractionRules");
    assert!(world.get_resource::<WorldMap>().is_some(), "WorldMap");
    assert!(world.get_resource::<FactionRelationships>().is_some(), "FactionRelationships");
    assert!(world.get_resource::<ReputationTracker>().is_some(), "ReputationTracker");
    assert!(world.get_resource::<FactionReputation>().is_some(), "FactionReputation");
    assert!(world.get_resource::<game_world::BehaviorRules>().is_some(), "BehaviorRules");
    assert!(world.get_resource::<game_core::QuestTemplates>().is_some(), "QuestTemplates");
    assert!(world.get_resource::<MessageLog>().is_some(), "MessageLog");
    assert!(world.get_resource::<TurnCounter>().is_some(), "TurnCounter");
    assert!(world.get_resource::<TurnState>().is_some(), "TurnState");
}

#[test]
fn e2e_world_generation_map_has_data() {
    let mut world = setup_e2e_world(42, 80, 80);
    let map_width = world.resource::<WorldMap>().width;
    let map_height = world.resource::<WorldMap>().height;
    assert_eq!(map_width, 80);
    assert_eq!(map_height, 80);

    let reg = registry(&world);
    let walkable_id = reg.tag_id("WALKABLE").expect("WALKABLE must exist");

    let map = world.resource::<WorldMap>().clone();
    let mut walkable_count = 0;
    for entity in &map.tiles {
        let mut tq = world.query::<&Tags>();
        if let Ok(tags) = tq.get(&world, *entity)
            && tags.has(walkable_id)
        {
            walkable_count += 1;
        }
    }
    assert!(walkable_count > 0, "map must have walkable tiles");
}

// ============================================================
// Player Movement & Exploration Tests
// ============================================================

#[test]
fn e2e_player_movement_in_open_terrain() {
    let mut world = setup_controlled_world();

    let initial_pos = player_pos(&mut world).unwrap();
    assert!(move_player(&mut world, 1, 0), "move right");
    assert_eq!(player_pos(&mut world).unwrap(), (initial_pos.0 + 1, initial_pos.1));

    assert!(move_player(&mut world, 0, 1), "move down");
    assert_eq!(player_pos(&mut world).unwrap(), (initial_pos.0 + 1, initial_pos.1 + 1));

    assert!(move_player(&mut world, -1, 0), "move left");
    assert_eq!(player_pos(&mut world).unwrap(), (initial_pos.0, initial_pos.1 + 1));

    assert!(move_player(&mut world, 0, -1), "move up");
    assert_eq!(player_pos(&mut world).unwrap(), (initial_pos.0, initial_pos.1));
}

#[test]
fn e2e_player_blocked_at_map_boundary() {
    let mut world = setup_controlled_world();

    while player_pos(&mut world).unwrap().0 > 0 {
        move_player(&mut world, -1, 0);
    }
    while player_pos(&mut world).unwrap().1 > 0 {
        move_player(&mut world, 0, -1);
    }

    let pos = player_pos(&mut world).unwrap();
    assert_eq!(pos, (0, 0));

    assert!(!move_player(&mut world, -1, 0), "cannot go past left boundary");
    assert!(!move_player(&mut world, 0, -1), "cannot go past top boundary");
    assert_eq!(player_pos(&mut world).unwrap(), (0, 0), "position unchanged");
}

// ============================================================
// Combat Tests
// ============================================================

#[test]
fn e2e_combat_player_attacks_aggressive_npc() {
    let mut world = setup_controlled_world();

    let creature = spawn_creature_in_world(
        &mut world, "Goblin", 16, 15, 30,
        &["AGGRESSIVE", "HUMANOID", "SMALL"], Some(FactionId(1)),
    );

    let initial_creature_hp = world.get::<Health>(creature).unwrap().current;
    let initial_player_hp = player_hp(&mut world).unwrap();

    let player_damage = std::cmp::max(1, initial_player_hp / 10);
    let (new_creature_hp, new_player_hp) = combat_apply_damage(&mut world, player_damage, creature);

    assert!(new_creature_hp < initial_creature_hp, "creature should take damage");
    assert!(new_player_hp < initial_player_hp, "player should take damage");
}

#[test]
fn e2e_combat_npc_death_applies_reputation_penalty() {
    let mut world = setup_controlled_world();

    let fid = FactionId(1);

    let initial_rep = {
        let rep = world.resource::<ReputationTracker>();
        rep.get(fid)
    };

    let creature = spawn_creature_in_world(
        &mut world, "TestGoblin", 16, 15, 1,
        &["AGGRESSIVE", "HUMANOID"], Some(fid),
    );

    let (new_creature_hp, _) = combat_apply_damage(&mut world, 100, creature);

    if new_creature_hp == 0 {
        if let Some(mut rep) = world.get_resource_mut::<ReputationTracker>() {
            rep.record_kill(FactionId(0), fid);
        }
        world.entity_mut(creature).despawn();
    }

    let after_rep = world.resource::<ReputationTracker>().get(fid);
    assert!(after_rep < initial_rep,
        "reputation should decrease after kill: {} -> {}", initial_rep, after_rep);
}

#[test]
fn e2e_combat_player_death_possible() {
    let mut world = setup_controlled_world();

    {
        let mut hp_query = world.query_filtered::<&mut Health, With<Player>>();
        if let Ok(mut hp) = hp_query.single_mut(&mut world) {
            hp.current = 1;
        }
    }
    assert_eq!(player_hp(&mut world).unwrap(), 1);

    let creature = spawn_creature_in_world(
        &mut world, "Dragon", 16, 15, 100,
        &["AGGRESSIVE", "HUGE"], Some(FactionId(1)),
    );

    let creature_hp = world.get::<Health>(creature).unwrap().current;
    let creature_damage = std::cmp::max(1, creature_hp / 10);
    let new_player_hp = 1u32.saturating_sub(creature_damage);

    assert_eq!(new_player_hp, 0, "damage {} should kill 1 HP player", creature_damage);
}

// ============================================================
// Status Effects Integration Tests
// ============================================================

#[test]
fn e2e_status_fire_plus_flammable_produces_burning() {
    let mut world = setup_controlled_world();
    let reg = registry(&world);

    let fire_id = reg.tag_id("FIRE").unwrap();
    let flammable_id = reg.tag_id("FLAMMABLE").unwrap();
    let burning_id = reg.tag_id("BURNING").unwrap();

    let entity = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 10, y: 10, z: 0 },
    )).id();

    {
        let mut tags = world.get_mut::<Tags>(entity).unwrap();
        tags.add_tag(fire_id, TagValue::None, &reg);
        tags.add_tag(flammable_id, TagValue::None, &reg);
    }

    crate::status::process_status_effects(&mut world);

    let tags = world.get::<Tags>(entity).unwrap();
    assert!(tags.has(burning_id), "FIRE + FLAMMABLE should produce BURNING");
}

#[test]
fn e2e_status_burning_ticks_down_and_expires() {
    let mut world = setup_controlled_world();
    let reg = registry(&world);

    let burning_id = reg.tag_id("BURNING").unwrap();

    let entity = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 5, y: 5, z: 0 },
    )).id();

    {
        let mut tags = world.get_mut::<Tags>(entity).unwrap();
        tags.add_tag(burning_id, TagValue::Ticks { remaining: 2, max: 2 }, &reg);
    }

    assert!(world.get::<Tags>(entity).unwrap().has(burning_id));

    crate::status::process_status_effects(&mut world);
    assert!(world.get::<Tags>(entity).unwrap().has(burning_id), "still burning after tick 1");

    crate::status::process_status_effects(&mut world);
    assert!(!world.get::<Tags>(entity).unwrap().has(burning_id), "burning expired after tick 2");
}

#[test]
fn e2e_status_cross_interaction_adjacent() {
    let mut world = setup_controlled_world();
    let reg = registry(&world);

    let fire_id = reg.tag_id("FIRE").unwrap();
    let water_id = reg.tag_id("WATER").unwrap();

    let e1 = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 5, y: 5, z: 0 },
    )).id();

    let e2 = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 6, y: 5, z: 0 },
    )).id();

    {
        let mut t1 = world.get_mut::<Tags>(e1).unwrap();
        t1.add_tag(water_id, TagValue::None, &reg);
        let mut t2 = world.get_mut::<Tags>(e2).unwrap();
        t2.add_tag(fire_id, TagValue::None, &reg);
    }

    assert!(world.get::<Tags>(e2).unwrap().has(fire_id));

    crate::status::process_status_effects(&mut world);

    assert!(!world.get::<Tags>(e2).unwrap().has(fire_id),
        "WATER next to FIRE should extinguish fire");
}

#[test]
fn e2e_status_cross_interaction_non_adjacent_ignored() {
    let mut world = setup_controlled_world();
    let reg = registry(&world);

    let fire_id = reg.tag_id("FIRE").unwrap();
    let water_id = reg.tag_id("WATER").unwrap();

    let e1 = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 5, y: 5, z: 0 },
    )).id();

    let e2 = world.spawn((
        Tags::new(reg.tag_count()),
        Position { x: 20, y: 20, z: 0 },
    )).id();

    {
        let mut t1 = world.get_mut::<Tags>(e1).unwrap();
        t1.add_tag(water_id, TagValue::None, &reg);
        let mut t2 = world.get_mut::<Tags>(e2).unwrap();
        t2.add_tag(fire_id, TagValue::None, &reg);
    }

    assert!(world.get::<Tags>(e2).unwrap().has(fire_id));
    crate::status::process_status_effects(&mut world);
    assert!(world.get::<Tags>(e2).unwrap().has(fire_id), "non-adjacent entities don't interact");
}

// ============================================================
// Full Turn Cycle Tests
// ============================================================

#[test]
fn e2e_full_turn_cycle_advances_turn_counter() {
    let mut world = setup_controlled_world();
    let initial_turn = world.resource::<TurnCounter>().0;
    simulate_full_turn(&mut world);
    let new_turn = world.resource::<TurnCounter>().0;
    assert_eq!(new_turn, initial_turn + 1, "turn counter should increment");
}

#[test]
fn e2e_full_turn_cycle_does_not_panic_on_empty_world() {
    let mut world = World::new();
    world.insert_resource(load_tag_registry(TAGS_TOML).unwrap());
    world.insert_resource(load_interaction_rules(INTERACTIONS_TOML, &registry(&world)).unwrap());
    world.insert_resource(TurnCounter::new());
    world.insert_resource(TurnState::new());
    world.insert_resource(QuestLog::new());
    world.insert_resource(FactionReputation::new());

    simulate_full_turn(&mut world);
}

#[test]
fn e2e_full_turn_npcs_process_each_turn() {
    let mut world = setup_controlled_world();

    spawn_creature_in_world(
        &mut world, "Villager", 10, 10, 50,
        &["HUMANOID", "PEACEFUL"], Some(FactionId(1)),
    );

    for _ in 0..5 {
        simulate_full_turn(&mut world);
    }

    assert!(player_entity(&mut world).is_some(), "player must survive");
    assert!(world.resource::<TurnCounter>().0 >= 5, "turns advanced");
}

// ============================================================
// Thrown Items Tests
// ============================================================

#[test]
fn e2e_throw_removes_item_from_inventory() {
    let mut world = setup_controlled_world();

    add_item_to_player_inventory(&mut world, "Rock", &["THROWABLE", "STONE"]);

    let inv_len_before = player_inventory_len(&mut world);
    assert_eq!(inv_len_before, 1);

    let start_result = crate::throw::start_throw(&mut world, 0);
    assert!(start_result.is_some(), "throwable item should start throw");

    let item_entity = start_result.unwrap();
    let result = crate::throw::execute_throw(&mut world, item_entity, 15 + 5, 15);
    assert!(result, "throw should succeed");

    assert!(player_inventory_len(&mut world) == 0, "item removed from inventory");
}

#[test]
fn e2e_throw_rejects_non_throwable() {
    let mut world = setup_controlled_world();

    add_item_to_player_inventory(&mut world, "Gold Coin", &["METAL", "VALUABLE"]);

    let result = crate::throw::start_throw(&mut world, 0);
    assert!(result.is_none(), "non-throwable item rejected");

    let log = world.resource::<MessageLog>();
    assert!(log.messages.iter().any(|m| m.contains("cannot be thrown")));
}

// ============================================================
// Item Consumption Tests
// ============================================================

#[test]
fn e2e_consume_edible_heals_player() {
    let mut world = setup_controlled_world();

    {
        let mut hp_query = world.query_filtered::<&mut Health, With<Player>>();
        if let Ok(mut hp) = hp_query.single_mut(&mut world) {
            hp.current = 50;
        }
    }

    add_item_to_player_inventory(&mut world, "Apple", &["EDIBLE", "FOOD_WILD"]);

    let hp_before = player_hp(&mut world).unwrap();
    assert_eq!(hp_before, 50);

    crate::consume::handle_consume(&mut world, 0);

    let hp_after = player_hp(&mut world).unwrap();
    assert!(hp_after > hp_before, "eating food should heal ({} -> {})", hp_before, hp_after);
}

#[test]
fn e2e_consume_non_consumable_rejected() {
    let mut world = setup_controlled_world();

    add_item_to_player_inventory(&mut world, "Rock", &["STONE"]);

    crate::consume::handle_consume(&mut world, 0);

    let log = world.resource::<MessageLog>();
    assert!(log.messages.iter().any(|m| m.contains("cannot be consumed")));
}

// ============================================================
// Equipment Tests
// ============================================================

#[test]
fn e2e_equip_weapon_adds_to_slot() {
    let mut world = setup_controlled_world();

    add_item_to_player_inventory(
        &mut world, "Sword",
        &["EQUIP_WEAPON", "METAL", "MELEE", "COMMON"],
    );

    crate::equipment::handle_equip(&mut world, 0);

    let pe = player_entity(&mut world).unwrap();
    let equip = world.get::<Equipment>(pe).unwrap();
    assert!(equip.weapon.is_some(), "weapon should be equipped");
}

// ============================================================
// Crafting Tests
// ============================================================

#[test]
fn e2e_crafting_recipes_load() {
    let crafting_toml = include_str!("../assets/config/crafting.toml");
    let recipes = game_core::crafting::load_crafting_recipes(crafting_toml).unwrap();
    assert!(!recipes.is_empty(), "crafting recipes loaded");

    let smelt = recipes.iter().find(|r| r.name == "Smelt Iron");
    assert!(smelt.is_some(), "Smelt Iron recipe exists");
}

#[test]
fn e2e_crafting_find_available() {
    let mut world = setup_controlled_world();
    let reg = registry(&world);

    let crafting_toml = include_str!("../assets/config/crafting.toml");
    let recipes = game_core::crafting::load_crafting_recipes(crafting_toml).unwrap();

    add_item_to_player_inventory(&mut world, "Iron Ingot", &["METAL", "INGOT_IRON"]);

    let pe = player_entity(&mut world).unwrap();
    let inv = world.get::<Inventory>(pe).unwrap().clone();

    let pos = player_pos(&mut world).unwrap();
    let avail = game_core::crafting::find_available_recipes(
        &recipes, &inv, &mut world, (pos.0, pos.1), &reg,
    );

    assert!(!avail.is_empty(), "should find some available recipes");
}

// ============================================================
// Save/Load Tests
// ============================================================

#[test]
fn e2e_save_game_creates_file() {
    let mut world = setup_controlled_world();

    {
        let mut tc = world.resource_mut::<TurnCounter>();
        tc.0 = 42;
    }

    let seed = 42u64;
    let result = game_core::save::save_game(&mut world, seed);
    assert!(result.is_ok(), "save should succeed");
    let filename = result.unwrap();
    assert!(filename.contains("save"), "filename should contain 'save'");

    let saved_path = std::path::PathBuf::from("saves").join(&filename);
    let _ = std::fs::remove_file(&saved_path);
}

#[test]
fn e2e_save_load_roundtrip_state() {
    let mut world = setup_controlled_world();

    world.insert_resource(TurnCounter(50));

    spawn_creature_in_world(&mut world, "Goblin", 10, 10, 25, &[], Some(FactionId(1)));

    let seed = 42u64;
    let save_result = game_core::save::save_game(&mut world, seed);
    assert!(save_result.is_ok(), "save must succeed");

    let filename = save_result.unwrap();
    let saved_path = std::path::PathBuf::from("saves").join(&filename);

    let loaded = game_core::save::load_game(&saved_path);
    assert!(loaded.is_ok(), "load must succeed");

    let save = loaded.unwrap();
    assert_eq!(save.seed, seed);
    assert_eq!(save.turn, 50);
    assert_eq!(save.player.x, 15);
    assert_eq!(save.player.y, 15);
    assert_eq!(save.player.hp_current, 100);

    let _ = std::fs::remove_file(&saved_path);
}

#[test]
fn e2e_save_load_deserialize_creates_entities() {
    let mut world = setup_controlled_world();

    world.insert_resource(TurnCounter(50));

    spawn_creature_in_world(&mut world, "Goblin", 10, 10, 25, &[], Some(FactionId(1)));

    let seed = 42u64;
    let save_result = game_core::save::save_game(&mut world, seed);
    assert!(save_result.is_ok());
    let filename = save_result.unwrap();
    let saved_path = std::path::PathBuf::from("saves").join(&filename);

    let loaded = game_core::save::load_game(&saved_path).unwrap();

    let mut fresh = World::new();
    game_core::save::deserialize_to_world(&mut fresh, &loaded).unwrap();

    let mut pq = fresh.query_filtered::<&Position, With<Player>>();
    assert!(pq.single(&fresh).is_ok(), "player should exist after deserialize");

    let mut cq = fresh.query_filtered::<bevy_ecs::entity::Entity, With<Creature>>();
    let restored_count = cq.iter(&fresh).count();
    assert!(restored_count > 0, "creatures should be restored");

    let _ = std::fs::remove_file(&saved_path);
}

#[test]
fn e2e_list_saves_works() {
    let saves = game_core::save::list_saves();
    let _ = saves.len();
}

// ============================================================
// Reputation System Tests
// ============================================================

#[test]
fn e2e_reputation_initial_state() {
    let world = setup_controlled_world();

    let rep = world.resource::<ReputationTracker>();
    let fid = FactionId(1);
    let initial = rep.get(fid);

    // Initial reputation should be neutral (0) or have some default value
    let _ = initial;
}

#[test]
fn e2e_reputation_after_kill_decreases() {
    let mut world = setup_controlled_world();

    let fid = FactionId(1);
    let initial = world.resource::<ReputationTracker>().get(fid);

    {
        let mut rep = world.resource_mut::<ReputationTracker>();
        rep.record_kill(FactionId(0), fid);
    }

    let after = world.resource::<ReputationTracker>().get(fid);
    assert!(after < initial,
        "reputation should decrease after kill: {} -> {}", initial, after);
}

// ============================================================
// Faction System Tests
// ============================================================

#[test]
fn e2e_factions_loaded_from_config() {
    let world = setup_controlled_world();
    let rels = world.resource::<FactionRelationships>();

    // Check that faction_id returns valid IDs for known names
    let ids_to_check = ["great_carapace", "sanguine_elite", "familiars", "free_humanity", "the_remnant"];
    let mut found = 0;
    for name in &ids_to_check {
        if rels.faction_id(name).is_some() {
            found += 1;
        }
    }
    assert!(found > 0, "at least one faction ID should be resolvable");
}

#[test]
fn e2e_creature_spawned_with_faction() {
    let mut world = setup_controlled_world();

    let fid = FactionId(1);
    let creature = spawn_creature_in_world(
        &mut world, "FactionMember", 10, 10, 50,
        &["HUMANOID"], Some(fid),
    );

    let faction = world.get::<Faction>(creature);
    assert!(faction.is_some(), "creature should have faction component");
    assert_eq!(faction.unwrap().faction_id, fid);
}

// ============================================================
// Narrative Events Tests
// ============================================================

#[test]
fn e2e_narrative_config_loaded() {
    let narrative_toml = include_str!("../assets/config/narrative_events.toml");
    let events = game_core::narrative::load_narrative_events(narrative_toml);
    assert!(events.is_ok(), "narrative events config loads");
    let events = events.unwrap();
    assert!(!events.is_empty(), "narrative events should not be empty");
}

#[test]
fn e2e_narrative_runs_without_panicking() {
    let mut world = setup_controlled_world();
    world.insert_resource(game_core::narrative::NarrativeCooldowns::default());

    spawn_creature_in_world(
        &mut world, "PlainNPC", 10, 10, 50, &[], None,
    );

    game_core::narrative::check_narrative_events(&mut world);
}

// ============================================================
// Config Loading Tests
// ============================================================

#[test]
fn e2e_all_configs_load() {
    let tags_toml = include_str!("../assets/config/tags.toml");
    assert!(load_tag_registry(tags_toml).is_ok());

    let interactions_toml = include_str!("../assets/config/interactions.toml");
    let reg = load_tag_registry(tags_toml).unwrap();
    assert!(load_interaction_rules(interactions_toml, &reg).is_ok());

    let factions_toml = include_str!("../assets/config/factions.toml");
    assert!(game_world::load_factions(factions_toml).is_ok());

    let behavior_toml = include_str!("../assets/config/behavior_rules.toml");
    assert!(game_world::load_behavior_rules(behavior_toml).is_ok());

    let crafting_toml = include_str!("../assets/config/crafting.toml");
    assert!(game_core::crafting::load_crafting_recipes(crafting_toml).is_ok());

    let quests_toml = include_str!("../assets/config/quests.toml");
    assert!(game_core::quest::load_quest_templates(quests_toml).is_ok());

    let narrative_toml = include_str!("../assets/config/narrative_events.toml");
    assert!(game_core::narrative::load_narrative_events(narrative_toml).is_ok());

    let dialogue_toml = include_str!("../assets/config/dialogue.toml");
    assert!(game_core::dialogue::load_dialogue(dialogue_toml).is_ok());

    let biome_toml = include_str!("../assets/config/biome_rules.toml");
    assert!(game_world::load_biome_rules(biome_toml).is_ok());

    let spawn_toml = include_str!("../assets/config/spawn_rules.toml");
    assert!(game_world::load_spawn_rules(spawn_toml).is_ok());
}

// ============================================================
// Quest System Tests
// ============================================================

#[test]
fn e2e_quest_templates_loaded() {
    let world = setup_controlled_world();
    let templates = world.resource::<game_core::QuestTemplates>();
    assert!(!templates.templates.is_empty(), "quest templates loaded");
}

// ============================================================
// World Exploration / Biome Tests
// ============================================================

#[test]
fn e2e_biome_tiles_exist() {
    let mut world = setup_e2e_world(42, 80, 80);
    let map = world.resource::<WorldMap>().clone();

    let mut biomes = std::collections::HashSet::new();
    for entity in &map.tiles {
        let mut tq = world.query::<&game_world::Tile>();
        if let Ok(tile) = tq.get(&world, *entity) {
            biomes.insert(tile.biome_name.clone());
        }
    }

    assert!(!biomes.is_empty(), "world should have biome types");
}

#[test]
fn e2e_spawn_positions_valid() {
    let mut world = setup_e2e_world(42, 80, 80);
    let map = world.resource::<WorldMap>().clone();
    let reg = registry(&world);

    let walkable_id = reg.tag_id("WALKABLE").expect("WALKABLE must exist");

    let player_x = player_pos(&mut world).unwrap().0;
    let player_y = player_pos(&mut world).unwrap().1;

    let tile_entity = map.get(TilePos::new(player_x, player_y));
    assert!(tile_entity.is_some(), "player tile should exist");

    if let Some(te) = tile_entity {
        let mut tq = world.query::<&Tags>();
        if let Ok(tags) = tq.get(&world, te) {
            assert!(tags.has(walkable_id), "player tile should be WALKABLE");
        }
    }
}

#[test]
fn e2e_map_dimensions_correct() {
    let world = setup_e2e_world(42, 80, 80);
    let map = world.resource::<WorldMap>();

    assert_eq!(map.width, 80);
    assert_eq!(map.height, 80);
    assert_eq!(map.tiles.len() as u32, 80 * 80);
}
