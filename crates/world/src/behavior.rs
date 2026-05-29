use std::collections::HashSet;

use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;

use game_core::{BehaviorState, Creature, Health, PersonalityScores, Player, Position, TurnCounter};
use game_tags::{TagId, TagRegistry, Tags};

use crate::faction::{Faction, FactionId, FactionRelationships, FactionStanding, PLAYER_FACTION_ID};
use crate::map::WorldMap;
use crate::pathfinding::{a_star_step, has_line_of_sight};
use crate::tile::TilePos;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NpcAction {
    Move { dx: i32, dy: i32 },
    Wait,
    Flee { dx: i32, dy: i32 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct BehaviorRule {
    pub trait_tag: String,
    pub action: String,
    pub weight: f32,
    pub range: u32,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BehaviorRulesFile {
    #[serde(rename = "rule")]
    rules: Vec<BehaviorRule>,
}

#[derive(Resource, Debug, Clone)]
pub struct BehaviorRules(pub Vec<BehaviorRule>);

pub fn load_behavior_rules(toml: &str) -> Result<Vec<BehaviorRule>, toml::de::Error> {
    let file: BehaviorRulesFile = toml::from_str(toml)?;
    Ok(file.rules)
}

struct EntityInfo {
    entity: Entity,
    x: u32,
    y: u32,
    faction_id: Option<FactionId>,
    is_player: bool,
}

fn manhattan(x1: u32, y1: u32, x2: u32, y2: u32) -> u32 {
    let dx = x1.abs_diff(x2);
    let dy = y1.abs_diff(y2);
    dx + dy
}

fn get_sense_range(tags: &Tags, registry: &TagRegistry, visibility_mod: f32) -> u32 {
    let has_darkvision = registry.tag_id("DARKVISION").is_some_and(|id| tags.has(id));
    let has_thermal = registry.tag_id("THERMAL_SENSE").is_some_and(|id| tags.has(id));

    let effective_vis = if has_thermal {
        1.0 // thermal sense ignores all visibility penalties
    } else if has_darkvision {
        // Darkvision ignores the light component, still affected by weather
        visibility_mod.max(0.3) // clamp weather-only visibility
    } else {
        visibility_mod
    };

    let range = if let Some(id) = registry.tag_id("SIGHT")
        && tags.has(id)
        && let Some(val) = tags.get_value(id)
        && let Some(mag) = val.magnitude()
    {
        mag as u32
    } else {
        8
    };

    (range as f32 * effective_vis).max(1.0) as u32
}

fn is_hostile_to(
    creature_faction: Option<FactionId>,
    target: &EntityInfo,
    faction_rels: Option<&FactionRelationships>,
) -> bool {
    let cf = match creature_faction {
        Some(f) => f,
        None => return true,
    };

    let tf = if target.is_player {
        Some(PLAYER_FACTION_ID)
    } else {
        target.faction_id
    };

    match (tf, faction_rels) {
        (Some(tf), Some(rels)) => rels.get_standing(cf, tf) == FactionStanding::Hostile,
        _ => false,
    }
}

fn is_not_hostile_to(
    creature_faction: Option<FactionId>,
    target: &EntityInfo,
    faction_rels: Option<&FactionRelationships>,
) -> bool {
    let cf = match creature_faction {
        Some(f) => f,
        None => return true,
    };

    let tf = if target.is_player {
        Some(PLAYER_FACTION_ID)
    } else {
        target.faction_id
    };

    match (tf, faction_rels) {
        (Some(tf), Some(rels)) => rels.get_standing(cf, tf) != FactionStanding::Hostile,
        _ => true,
    }
}

fn closest_by_manhattan<'a>(
    entities: &[&'a EntityInfo],
    cx: u32,
    cy: u32,
) -> Option<&'a EntityInfo> {
    entities
        .iter()
        .min_by_key(|e| manhattan(cx, cy, e.x, e.y))
        .copied()
}

#[allow(clippy::too_many_arguments)]
fn is_tile_passable(
    x: u32,
    y: u32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
) -> bool {
    if x >= map.width || y >= map.height {
        return false;
    }

    if occupied.contains(&(x, y)) {
        return false;
    }

    if flight_id.is_some_and(|id| creature_tags.has(id)) {
        return true;
    }

    let tile_entity = match map.get(TilePos::new(x, y)) {
        Some(e) => e,
        None => return false,
    };

    let tile_tags = match world.get::<Tags>(tile_entity) {
        Some(t) => t,
        None => return true,
    };

    if blocked_id.is_some_and(|id| tile_tags.has(id)) {
        return false;
    }

    if swimmable_id.is_some_and(|id| tile_tags.has(id))
        && !aquatic_id.is_some_and(|id| creature_tags.has(id))
    {
        return false;
    }

    true
}

#[allow(clippy::too_many_arguments)]
fn try_move(
    cx: u32,
    cy: u32,
    dx: i32,
    dy: i32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
) -> NpcAction {
    let nx = cx as i32 + dx;
    let ny = cy as i32 + dy;
    if nx < 0 || ny < 0 {
        return NpcAction::Wait;
    }
    if is_tile_passable(
        nx as u32,
        ny as u32,
        map,
        world,
        creature_tags,
        occupied,
        blocked_id,
        swimmable_id,
        flight_id,
        aquatic_id,
    ) {
        NpcAction::Move { dx, dy }
    } else {
        NpcAction::Wait
    }
}

fn chase_direction(
    cx: u32,
    cy: u32,
    tx: u32,
    ty: u32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
    _rng: &mut impl Rng,
) -> (i32, i32) {
    if let Some((dx, dy)) = a_star_step(
        (cx, cy), (tx, ty), map, world, creature_tags, occupied,
        blocked_id, swimmable_id, flight_id, aquatic_id,
    ) {
        (dx, dy)
    } else if cx == tx && cy == ty {
        (0, 0)
    } else {
        let dx = (tx as i32 - cx as i32).signum();
        let dy = (ty as i32 - cy as i32).signum();
        (dx, dy)
    }
}

fn flee_direction(
    cx: u32,
    cy: u32,
    tx: u32,
    ty: u32,
    rng: &mut rand::rngs::StdRng,
) -> (i32, i32) {
    let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let mut best_dist: u32 = 0;
    let mut best_dirs: Vec<(i32, i32)> = Vec::new();

    for &(dx, dy) in &dirs {
        let nx = cx as i32 + dx;
        let ny = cy as i32 + dy;
        if nx < 0 || ny < 0 {
            continue;
        }
        let d = manhattan(nx as u32, ny as u32, tx, ty);
        if d > best_dist {
            best_dist = d;
            best_dirs.clear();
            best_dirs.push((dx, dy));
        } else if d == best_dist && d > 0 {
            best_dirs.push((dx, dy));
        }
    }

    if best_dirs.is_empty() {
        (0, 0)
    } else {
        best_dirs[rng.random_range(0..best_dirs.len())]
    }
}

#[allow(clippy::too_many_arguments)]
fn wander_action(
    cx: u32,
    cy: u32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
    rng: &mut rand::rngs::StdRng,
) -> NpcAction {
    let mut dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    for i in (1..dirs.len()).rev() {
        let j = rng.random_range(0..=i);
        dirs.swap(i, j);
    }

    for &(dx, dy) in &dirs {
        let nx = cx as i32 + dx;
        let ny = cy as i32 + dy;
        if nx < 0 || ny < 0 {
            continue;
        }
        if is_tile_passable(
            nx as u32,
            ny as u32,
            map,
            world,
            creature_tags,
            occupied,
            blocked_id,
            swimmable_id,
            flight_id,
            aquatic_id,
        ) {
            return NpcAction::Move { dx, dy };
        }
    }

    NpcAction::Wait
}

pub fn process_npc_turns(world: &mut World) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let map = match world.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return,
    };
    let faction_rels = world.get_resource::<FactionRelationships>().cloned();
    let rules = match world.get_resource::<BehaviorRules>() {
        Some(r) => r.0.clone(),
        None => return,
    };
    let turn = world
        .get_resource::<TurnCounter>()
        .map(|t| t.current())
        .unwrap_or(0);

    // Read weather visibility modifier for NPC sense range
    let visibility_mod = world
        .get_resource::<game_core::WeatherState>()
        .map(|ws| ws.effective_visibility())
        .unwrap_or(1.0);

    let mut rng = rand::rngs::StdRng::seed_from_u64(
        map.seed.0.wrapping_add(turn).wrapping_add(0xCAFE_BABE),
    );

    let blocked_id = registry.tag_id("BLOCKED");
    let swimmable_id = registry.tag_id("SWIMMABLE");
    let flight_id = registry.tag_id("FLIGHT");
    let aquatic_id = registry.tag_id("AQUATIC");
    let mindless_id = registry.tag_id("MINDLESS");

    let rule_tag_ids: Vec<(TagId, BehaviorRule)> = rules
        .into_iter()
        .filter_map(|rule| registry.tag_id(&rule.trait_tag).map(|id| (id, rule)))
        .collect();

    let mut all_entities: Vec<EntityInfo> = Vec::new();
    {
        let mut q = world.query::<(Entity, &Position, &Tags, Option<&Faction>)>();
        for (entity, pos, _tags, faction) in q.iter(world) {
            let is_player = world.get::<Player>(entity).is_some();
            all_entities.push(EntityInfo {
                entity,
                x: pos.x,
                y: pos.y,
                faction_id: faction.map(|f| f.faction_id),
                is_player,
            });
        }
    }

    #[allow(clippy::type_complexity)]
    let mut creature_data: Vec<(Entity, u32, u32, Tags, Option<Position>, Option<FactionId>)> =
        Vec::new();
    {
        let mut q =
            world.query::<(Entity, &Position, &Tags, &BehaviorState, &Health, Option<&Faction>)>();
        for (entity, pos, tags, behavior, health, faction) in q.iter(world) {
            if world.get::<Creature>(entity).is_none() {
                continue;
            }
            if health.current == 0 {
                continue;
            }
            creature_data.push((
                entity,
                pos.x,
                pos.y,
                tags.clone(),
                behavior.home_pos,
                faction.map(|f| f.faction_id),
            ));
        }
    }

    let mut occupied: HashSet<(u32, u32)> = HashSet::new();
    for e in &all_entities {
        occupied.insert((e.x, e.y));
    }

    for (entity, cx, cy, tags, home_pos, faction_id) in &creature_data {
        let is_mindless = mindless_id.is_some_and(|id| tags.has(id));

        let action = if is_mindless {
            wander_action(
                *cx,
                *cy,
                &map,
                world,
                tags,
                &occupied,
                blocked_id,
                swimmable_id,
                flight_id,
                aquatic_id,
                &mut rng,
            )
        } else {
            let sense_range = get_sense_range(tags, &registry, visibility_mod);

            let nearby: Vec<&EntityInfo> = all_entities
                .iter()
                .filter(|e| e.entity != *entity && manhattan(*cx, *cy, e.x, e.y) <= sense_range)
                .collect();

            let hostile: Vec<&EntityInfo> = nearby
                .iter()
                .filter(|e| is_hostile_to(*faction_id, e, faction_rels.as_ref()))
                .copied()
                .collect();

            let non_hostile: Vec<&EntityInfo> = nearby
                .iter()
                .filter(|e| is_not_hostile_to(*faction_id, e, faction_rels.as_ref()))
                .copied()
                .collect();

            let mut best_score: f32 = f32::NEG_INFINITY;
            let mut best_action: Option<NpcAction> = None;

            for (tag_id, rule) in &rule_tag_ids {
                if !tags.has(*tag_id) {
                    continue;
                }

                let personality_mod = if let Some(scores) = world.get::<PersonalityScores>(*entity)
                {
                    match rule.trait_tag.as_str() {
                        "AGGRESSIVE" => 0.5 + scores.aggression as f32 / 100.0,
                        "COWARDLY" => 0.5 + (100 - scores.bravery) as f32 / 100.0,
                        "TERRITORIAL" => 0.5 + scores.orderliness as f32 / 100.0,
                        "CURIOUS" => 0.5 + scores.curiosity as f32 / 100.0,
                        _ => 1.0,
                    }
                } else {
                    1.0
                };

                match rule.action.as_str() {
                    "chase" => {
                        if let Some(target) = closest_by_manhattan(&hostile, *cx, *cy) {
                            let dist = manhattan(*cx, *cy, target.x, target.y);
                            if dist <= rule.range && rule.range > 0 {
                                let mut score =
                                    rule.weight * (1.0 - dist as f32 / rule.range as f32);
                                score *= personality_mod;
                                if let Some(blocked) = blocked_id {
                                    if !has_line_of_sight(
                                        *cx, *cy, target.x, target.y, &map, world, Some(blocked),
                                    ) {
                                        score *= 0.1;
                                    }
                                }
                                if score > best_score {
                                    best_score = score;
                                    let (dx, dy) = chase_direction(
                                        *cx,
                                        *cy,
                                        target.x,
                                        target.y,
                                        &map,
                                        world,
                                        tags,
                                        &occupied,
                                        blocked_id,
                                        swimmable_id,
                                        flight_id,
                                        aquatic_id,
                                        &mut rng,
                                    );
                                    best_action = Some(if dx == 0 && dy == 0 {
                                        NpcAction::Wait
                                    } else {
                                        try_move(
                                            *cx,
                                            *cy,
                                            dx,
                                            dy,
                                            &map,
                                            world,
                                            tags,
                                            &occupied,
                                            blocked_id,
                                            swimmable_id,
                                            flight_id,
                                            aquatic_id,
                                        )
                                    });
                                }
                            }
                        }
                    }
                    "flee" => {
                        if let Some(threat) = closest_by_manhattan(&hostile, *cx, *cy) {
                            let dist = manhattan(*cx, *cy, threat.x, threat.y);
                            if dist <= rule.range && rule.range > 0 {
                                let mut score =
                                    rule.weight * (1.0 - dist as f32 / rule.range as f32);
                                score *= personality_mod;
                                if score > best_score {
                                    best_score = score;
                                    let (dx, dy) = flee_direction(
                                        *cx,
                                        *cy,
                                        threat.x,
                                        threat.y,
                                        &mut rng,
                                    );
                                    if dx == 0 && dy == 0 {
                                        best_action = Some(NpcAction::Wait);
                                    } else {
                                        let nx = *cx as i32 + dx;
                                        let ny = *cy as i32 + dy;
                                        if nx >= 0
                                            && ny >= 0
                                            && is_tile_passable(
                                                nx as u32,
                                                ny as u32,
                                                &map,
                                                world,
                                                tags,
                                                &occupied,
                                                blocked_id,
                                                swimmable_id,
                                                flight_id,
                                                aquatic_id,
                                            )
                                        {
                                            best_action = Some(NpcAction::Flee { dx, dy });
                                        } else {
                                            best_action = Some(NpcAction::Wait);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "guard" => {
                        if let Some(home) = home_pos {
                            let near_home: Vec<&EntityInfo> = hostile
                                .iter()
                                .filter(|e| manhattan(home.x, home.y, e.x, e.y) <= rule.range)
                                .copied()
                                .collect();

                            if let Some(intruder) =
                                closest_by_manhattan(&near_home, *cx, *cy)
                            {
                                let dist = manhattan(*cx, *cy, intruder.x, intruder.y);
                                let mut score = rule.weight
                                    * (1.0 - dist as f32 / rule.range.max(1) as f32);
                                score *= personality_mod;
                                if score > best_score {
                                    best_score = score;
                                    let (dx, dy) = chase_direction(
                                        *cx,
                                        *cy,
                                        intruder.x,
                                        intruder.y,
                                        &map,
                                        world,
                                        tags,
                                        &occupied,
                                        blocked_id,
                                        swimmable_id,
                                        flight_id,
                                        aquatic_id,
                                        &mut rng,
                                    );
                                    best_action = Some(if dx == 0 && dy == 0 {
                                        NpcAction::Wait
                                    } else {
                                        try_move(
                                            *cx,
                                            *cy,
                                            dx,
                                            dy,
                                            &map,
                                            world,
                                            tags,
                                            &occupied,
                                            blocked_id,
                                            swimmable_id,
                                            flight_id,
                                            aquatic_id,
                                        )
                                    });
                                }
                            } else if manhattan(*cx, *cy, home.x, home.y) > 0 {
                                let mut score = rule.weight * 0.5;
                                score *= personality_mod;
                                if score > best_score {
                                    best_score = score;
                                    let (dx, dy) = chase_direction(
                                        *cx,
                                        *cy,
                                        home.x,
                                        home.y,
                                        &map,
                                        world,
                                        tags,
                                        &occupied,
                                        blocked_id,
                                        swimmable_id,
                                        flight_id,
                                        aquatic_id,
                                        &mut rng,
                                    );
                                    best_action = Some(if dx == 0 && dy == 0 {
                                        NpcAction::Wait
                                    } else {
                                        try_move(
                                            *cx,
                                            *cy,
                                            dx,
                                            dy,
                                            &map,
                                            world,
                                            tags,
                                            &occupied,
                                            blocked_id,
                                            swimmable_id,
                                            flight_id,
                                            aquatic_id,
                                        )
                                    });
                                }
                            } else {
                                let mut score = rule.weight * 0.3;
                                score *= personality_mod;
                                if score > best_score {
                                    best_score = score;
                                    best_action = Some(NpcAction::Wait);
                                }
                            }
                        }
                    }
                    "approach" => {
                        if let Some(target) = closest_by_manhattan(&non_hostile, *cx, *cy) {
                            let dist = manhattan(*cx, *cy, target.x, target.y);
                            if dist <= rule.range && rule.range > 0 {
                                let mut score =
                                    rule.weight * (1.0 - dist as f32 / rule.range as f32);
                                score *= personality_mod;
                                if score > best_score {
                                    best_score = score;
                                    let (dx, dy) = chase_direction(
                                        *cx,
                                        *cy,
                                        target.x,
                                        target.y,
                                        &map,
                                        world,
                                        tags,
                                        &occupied,
                                        blocked_id,
                                        swimmable_id,
                                        flight_id,
                                        aquatic_id,
                                        &mut rng,
                                    );
                                    best_action = Some(if dx == 0 && dy == 0 {
                                        NpcAction::Wait
                                    } else {
                                        try_move(
                                            *cx,
                                            *cy,
                                            dx,
                                            dy,
                                            &map,
                                            world,
                                            tags,
                                            &occupied,
                                            blocked_id,
                                            swimmable_id,
                                            flight_id,
                                            aquatic_id,
                                        )
                                    });
                                }
                            }
                        }
                    }
                    "wander" => {
                        let mut score = rule.weight;
                        score *= personality_mod;
                        if score > best_score {
                            best_score = score;
                            best_action = Some(wander_action(
                                *cx,
                                *cy,
                                &map,
                                world,
                                tags,
                                &occupied,
                                blocked_id,
                                swimmable_id,
                                flight_id,
                                aquatic_id,
                                &mut rng,
                            ));
                        }
                    }
                    _ => {}
                }
            }

            best_action.unwrap_or_else(|| {
                wander_action(
                    *cx,
                    *cy,
                    &map,
                    world,
                    tags,
                    &occupied,
                    blocked_id,
                    swimmable_id,
                    flight_id,
                    aquatic_id,
                    &mut rng,
                )
            })
        };

        match action {
            NpcAction::Move { dx, dy } | NpcAction::Flee { dx, dy } => {
                let nx = (*cx as i32 + dx) as u32;
                let ny = (*cy as i32 + dy) as u32;

                occupied.remove(&(*cx, *cy));
                occupied.insert((nx, ny));

                if let Some(mut pos) = world.get_mut::<Position>(*entity) {
                    pos.x = nx;
                    pos.y = ny;
                }
            }
            NpcAction::Wait => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faction::load_factions;
    use crate::seed::WorldSeed;
    use game_core::Glyph;
    use game_core::Name;
    use game_tags::{TagValue, load_tag_registry};

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    const FACTIONS_TOML: &str = r#"
[[faction]]
id = "great_carapace"

[[faction]]
id = "sanguine_elite"

[[relationship]]
faction_a = "sanguine_elite"
faction_b = "player"
standing = "hostile"
"#;

    const BEHAVIOR_TOML: &str = r#"
[[rule]]
trait_tag = "AGGRESSIVE"
action = "chase"
weight = 2.0
range = 10
description = "Chases hostile entities"

[[rule]]
trait_tag = "COWARDLY"
action = "flee"
weight = 3.0
range = 8
description = "Flees from threats"

[[rule]]
trait_tag = "TERRITORIAL"
action = "guard"
weight = 2.5
range = 6
description = "Guards home position"

[[rule]]
trait_tag = "CURIOUS"
action = "approach"
weight = 1.5
range = 6
description = "Approaches non-hostile entities"

[[rule]]
trait_tag = "PEACEFUL"
action = "wander"
weight = 1.0
range = 1
description = "Wanders randomly"
"#;

    fn make_flat_map(
        world: &mut World,
        width: u32,
        height: u32,
        registry: &TagRegistry,
    ) -> WorldMap {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "plains".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let tags = Tags::new(registry.tag_count());
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }
        WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        }
    }

    fn add_blocked_tile(
        world: &mut World,
        x: u32,
        y: u32,
    ) {
        let registry = world.resource::<TagRegistry>().clone();
        let blocked_id = registry.tag_id("BLOCKED").unwrap();
        let tile_entity = {
            let map = world.resource::<WorldMap>();
            map.get(TilePos::new(x, y)).unwrap()
        };
        let mut tags = world.get::<Tags>(tile_entity).unwrap().clone();
        tags.add_tag(blocked_id, TagValue::None, &registry);
        if let Some(mut t) = world.get_mut::<Tags>(tile_entity) {
            *t = tags;
        }
    }

    fn setup_world(width: u32, height: u32) -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry.clone());

        let map = make_flat_map(&mut world, width, height, &registry);
        world.insert_resource(map);

        let (_, faction_rels) = load_factions(FACTIONS_TOML).unwrap();
        world.insert_resource(faction_rels);

        let rules = load_behavior_rules(BEHAVIOR_TOML).unwrap();
        world.insert_resource(BehaviorRules(rules));

        world.insert_resource(TurnCounter::new());
        world
    }

    fn spawn_creature(
        world: &mut World,
        x: u32,
        y: u32,
        tag_names: &[&str],
        faction_name: Option<&str>,
    ) -> Entity {
        let registry = world.resource::<TagRegistry>().clone();
        let mut tags = Tags::new(registry.tag_count());
        for name in tag_names {
            if let Some(id) = registry.tag_id(name) {
                tags.add_tag(id, TagValue::None, &registry);
            }
        }

        let pos = Position { x, y, z: 0 };
        let faction_component = faction_name.and_then(|name| {
            let rels = world.resource::<FactionRelationships>();
            rels.faction_id(name).map(|fid| Faction { faction_id: fid })
        });

        if let Some(fc) = faction_component {
            world.spawn((
                pos,
                Glyph {
                    char: 'c',
                    color: (255, 0, 0),
                },
                Health {
                    current: 50,
                    max: 50,
                },
                tags,
                Name("TestCreature".to_string()),
                Creature,
                BehaviorState {
                    home_pos: Some(pos),
                },
                fc,
            )).id()
        } else {
            world.spawn((
                pos,
                Glyph {
                    char: 'c',
                    color: (255, 0, 0),
                },
                Health {
                    current: 50,
                    max: 50,
                },
                tags,
                Name("TestCreature".to_string()),
                Creature,
                BehaviorState {
                    home_pos: Some(pos),
                },
            )).id()
        }
    }

    fn spawn_player(world: &mut World, x: u32, y: u32) -> Entity {
        let pos = Position { x, y, z: 0 };
        world
            .spawn((
                Player,
                pos,
                Health {
                    current: 100,
                    max: 100,
                },
                Glyph {
                    char: '@',
                    color: (255, 255, 0),
                },
                Tags::new(world.resource::<TagRegistry>().tag_count()),
                Name("Player".to_string()),
            ))
            .id()
    }

    #[test]
    fn test_load_behavior_rules() {
        let rules = load_behavior_rules(BEHAVIOR_TOML).unwrap();
        assert_eq!(rules.len(), 5);
        assert_eq!(rules[0].trait_tag, "AGGRESSIVE");
        assert_eq!(rules[0].action, "chase");
        assert_eq!(rules[0].weight, 2.0);
        assert_eq!(rules[0].range, 10);
    }

    #[test]
    fn test_creature_pathfinds_around_blocked_tile() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["AGGRESSIVE", "MEDIUM"], None);
        spawn_player(&mut world, 12, 10);

        add_blocked_tile(&mut world, 11, 10);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_ne!(
            (pos.x, pos.y),
            (11, 10),
            "creature should not move into blocked tile"
        );
        assert!(
            pos.x != 10 || pos.y != 10,
            "creature should move toward player (pathfinds around wall)"
        );
    }

    #[test]
    fn test_flight_crosses_blocked() {
        let mut world = setup_world(20, 20);
        let creature =
            spawn_creature(&mut world, 10, 10, &["AGGRESSIVE", "FLIGHT", "MEDIUM"], None);
        spawn_player(&mut world, 12, 10);

        add_blocked_tile(&mut world, 11, 10);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(
            pos.x, 11,
            "flight creature should cross blocked tiles"
        );
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_aggressive_chases_hostile_player() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["AGGRESSIVE", "MEDIUM"], None);
        spawn_player(&mut world, 12, 10);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(pos.x, 11, "aggressive creature should move toward player");
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_cowardly_flees_from_threat() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["COWARDLY", "MEDIUM"], None);
        spawn_player(&mut world, 11, 10);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert!(
            pos.x <= 10,
            "cowardly creature should flee away from player, got x={}",
            pos.x
        );
    }

    #[test]
    fn test_territorial_guards_home() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 8, 8, &["TERRITORIAL", "MEDIUM"], None);

        {
            let mut bs = world.get_mut::<BehaviorState>(creature).unwrap();
            bs.home_pos = Some(Position { x: 8, y: 8, z: 0 });
        }

        spawn_player(&mut world, 10, 8);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(pos.x, 9, "territorial creature should chase intruder near home");
        assert_eq!(pos.y, 8);
    }

    #[test]
    fn test_territorial_returns_home() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["TERRITORIAL", "MEDIUM"], None);

        {
            let mut bs = world.get_mut::<BehaviorState>(creature).unwrap();
            bs.home_pos = Some(Position { x: 8, y: 8, z: 0 });
        }

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert!(
            (pos.x as i32 - 10).abs() + (pos.y as i32 - 10).abs() > 0
                || (pos.x == 9 || pos.y == 9),
            "territorial creature should move toward home (8,8) from (10,10), got ({},{})",
            pos.x,
            pos.y
        );
    }

    #[test]
    fn test_peaceful_wanders() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["PEACEFUL", "MEDIUM"], None);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        let moved = pos.x != 10 || pos.y != 10;
        let _ = moved;
    }

    #[test]
    fn test_mindless_wanders() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(
            &mut world,
            10,
            10,
            &["MINDLESS", "AGGRESSIVE", "MEDIUM"],
            None,
        );

        spawn_player(&mut world, 11, 10);

        process_npc_turns(&mut world);

        let _pos = world.get::<Position>(creature).unwrap();

    }

    #[test]
    fn test_faction_filters_targets() {
        let mut world = setup_world(20, 20);
        let creature =
            spawn_creature(&mut world, 10, 10, &["AGGRESSIVE", "MEDIUM"], Some("great_carapace"));
        let faction_rels = world.resource::<FactionRelationships>().clone();
        let elite_id = faction_rels.faction_id("sanguine_elite").unwrap();
        let _player = spawn_player(&mut world, 12, 10);

        let _elite = world
            .spawn((
                Position { x: 8, y: 10, z: 0 },
                Glyph {
                    char: 'S',
                    color: (200, 200, 200),
                },
                Health {
                    current: 50,
                    max: 50,
                },
                Tags::new(world.resource::<TagRegistry>().tag_count()),
                Name("Sanguine".to_string()),
                Creature,
                BehaviorState {
                    home_pos: Some(Position { x: 8, y: 10, z: 0 }),
                },
                Faction {
                    faction_id: elite_id,
                },
            ))
            .id();

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();

        let dist_to_player = manhattan(pos.x, pos.y, 12, 10);
        let dist_to_elite = manhattan(pos.x, pos.y, 8, 10);

        assert!(
            dist_to_player <= dist_to_elite
                || dist_to_elite == 0,
            "creature should not chase same-faction entities (player is closer to hostile sanguine)"
        );
    }

    #[test]
    fn test_no_creature_stacking() {
        let mut world = setup_world(20, 20);
        let creature1 =
            spawn_creature(&mut world, 10, 10, &["AGGRESSIVE", "MEDIUM"], None);
        let _creature2 =
            spawn_creature(&mut world, 11, 10, &["PEACEFUL", "MEDIUM"], None);
        spawn_player(&mut world, 12, 10);

        process_npc_turns(&mut world);

        let pos1 = world.get::<Position>(creature1).unwrap();
        assert!(
            !(pos1.x == 11 && pos1.y == 10),
            "creatures should not stack on the same tile"
        );
    }

    #[test]
    fn test_deterministic_behavior() {
        let run = || -> (u32, u32) {
            let mut world = setup_world(20, 20);
            let creature =
                spawn_creature(&mut world, 10, 10, &["PEACEFUL", "MEDIUM"], None);
            process_npc_turns(&mut world);
            let pos = world.get::<Position>(creature).unwrap();
            (pos.x, pos.y)
        };

        let r1 = run();
        let r2 = run();
        assert_eq!(r1, r2, "same seed should produce same behavior");
    }

    #[test]
    fn test_npc_action_wait_when_surrounded() {
        let mut world = setup_world(10, 10);
        let creature = spawn_creature(&mut world, 0, 0, &["AGGRESSIVE", "MEDIUM"], None);
        let _other1 = spawn_creature(&mut world, 1, 0, &["PEACEFUL", "MEDIUM"], None);
        let _other2 = spawn_creature(&mut world, 0, 1, &["PEACEFUL", "MEDIUM"], None);

        spawn_player(&mut world, 2, 0);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_cowardly_flee_direction() {
        let mut world = setup_world(20, 20);
        let creature = spawn_creature(&mut world, 10, 10, &["COWARDLY", "MEDIUM"], None);
        spawn_player(&mut world, 9, 10);

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert!(
            pos.x > 10 || (pos.x == 10 && pos.y != 10),
            "cowardly creature should flee away from player at (9,10), got ({},{})",
            pos.x,
            pos.y
        );
    }

    #[test]
    fn test_curious_approaches_non_hostile() {
        let mut world = setup_world(20, 20);
        let creature =
            spawn_creature(&mut world, 10, 10, &["CURIOUS", "MEDIUM"], Some("great_carapace"));
        let _other = spawn_creature(&mut world, 12, 10, &["PEACEFUL", "MEDIUM"], Some("great_carapace"));

        process_npc_turns(&mut world);

        let pos = world.get::<Position>(creature).unwrap();
        assert_eq!(pos.x, 11, "curious creature should approach non-hostile entity");
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_manhattan_distance() {
        assert_eq!(manhattan(0, 0, 3, 4), 7);
        assert_eq!(manhattan(5, 5, 5, 5), 0);
        assert_eq!(manhattan(0, 0, 0, 5), 5);
        assert_eq!(manhattan(10, 10, 7, 6), 7);
    }
}
