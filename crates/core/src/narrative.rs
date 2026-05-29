use std::collections::HashMap;

use bevy_ecs::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::{EventBus, GameEvent, Position, TurnCounter};
#[cfg(test)]
use crate::MessageLog;
use game_tags::{TagId, TagRegistry, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct LoreFragment {
    pub id: String,
    pub category: String,
    pub rarity: String,
    pub faction_source: String,
    pub title_template: String,
    pub content_template: String,
    #[serde(default)]
    pub discovery_tags: Vec<String>,
    #[serde(default)]
    pub discovery_context: String,
    #[serde(default)]
    pub persistence: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LoreFragmentsToml {
    #[serde(rename = "lore_fragment")]
    fragments: Vec<LoreFragment>,
}

pub fn load_lore_fragments(toml_str: &str) -> Result<Vec<LoreFragment>, toml::de::Error> {
    let file: LoreFragmentsToml = toml::from_str(toml_str)?;
    Ok(file.fragments)
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LoreFragmentsResource {
    pub fragments: Vec<LoreFragment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NarrativeTrigger {
    pub entity_tags: Vec<String>,
    #[serde(default)]
    pub nearby_tags: Vec<String>,
    #[serde(default)]
    pub player_tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NarrativeEvent {
    pub id: String,
    pub trigger: NarrativeTrigger,
    pub message: String,
    pub chance: f64,
    #[serde(default)]
    pub cooldown_ticks: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct NarrativeEventsToml {
    #[serde(rename = "narrative_event")]
    events: Vec<NarrativeEvent>,
}

pub fn load_narrative_events(toml_str: &str) -> Result<Vec<NarrativeEvent>, toml::de::Error> {
    let file: NarrativeEventsToml = toml::from_str(toml_str)?;
    Ok(file.events)
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NarrativeCooldowns {
    pub last_fired: HashMap<String, u64>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NarrativeEvents {
    pub events: Vec<NarrativeEvent>,
}

struct ResolvedNarrativeEvent {
    id: String,
    entity_tag_ids: Vec<TagId>,
    nearby_tag_ids: Vec<TagId>,
    player_tag_ids: Vec<TagId>,
    message: String,
    chance: f64,
    cooldown_ticks: u64,
}

fn resolve_tag_ids(tag_names: &[String], registry: &TagRegistry) -> Vec<TagId> {
    tag_names
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect()
}

fn entity_has_all_tags(tags: &Tags, tag_ids: &[TagId]) -> bool {
    tag_ids.iter().all(|&id| tags.has(id))
}

pub fn check_narrative_events(world: &mut World) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    let events = match world.get_resource::<NarrativeEvents>() {
        Some(e) => e.events.clone(),
        None => return,
    };

    let current_turn = match world.get_resource::<TurnCounter>() {
        Some(tc) => tc.current(),
        None => return,
    };

    let resolved: Vec<ResolvedNarrativeEvent> = events
        .iter()
        .map(|ev| ResolvedNarrativeEvent {
            id: ev.id.clone(),
            entity_tag_ids: resolve_tag_ids(&ev.trigger.entity_tags, &registry),
            nearby_tag_ids: resolve_tag_ids(&ev.trigger.nearby_tags, &registry),
            player_tag_ids: resolve_tag_ids(&ev.trigger.player_tags, &registry),
            message: ev.message.clone(),
            chance: ev.chance,
            cooldown_ticks: ev.cooldown_ticks,
        })
        .collect();

    let entities: Vec<(Entity, Tags, Option<Position>)> = {
        let mut query = world.query::<(Entity, &Tags, Option<&Position>)>();
        query
            .iter(world)
            .map(|(e, t, p)| (e, t.clone(), p.copied()))
            .collect()
    };

    let player_tags: Option<Tags> = {
        let mut pq = world.query_filtered::<&Tags, With<crate::Player>>();
        pq.single(world).ok().cloned()
    };

    let mut rng = rand::rng();
    let mut triggered_messages: Vec<String> = Vec::new();
    let mut cooldown_updates: Vec<(String, u64)> = Vec::new();

    for ev in &resolved {
        if ev.entity_tag_ids.is_empty() {
            continue;
        }

        if let Some(last) = world
            .get_resource::<NarrativeCooldowns>()
            .and_then(|c| c.last_fired.get(&ev.id).copied())
            && current_turn < last + ev.cooldown_ticks {
                continue;
            }

        if !ev.player_tag_ids.is_empty() {
            match &player_tags {
                Some(pt) if entity_has_all_tags(pt, &ev.player_tag_ids) => {}
                _ => continue,
            }
        }

        for (_entity, tags, pos) in &entities {
            if !entity_has_all_tags(tags, &ev.entity_tag_ids) {
                continue;
            }

            if !ev.nearby_tag_ids.is_empty() {
                let has_nearby = match pos {
                    Some(pos) => entities.iter().any(|(_, other_tags, other_pos)| {
                        let Some(other_pos) = other_pos else {
                            return false;
                        };
                        let dx = (pos.x as i32 - other_pos.x as i32).unsigned_abs();
                        let dy = (pos.y as i32 - other_pos.y as i32).unsigned_abs();
                        if dx > 1 || dy > 1 {
                            return false;
                        }
                        entity_has_all_tags(other_tags, &ev.nearby_tag_ids)
                    }),
                    None => false,
                };
                if !has_nearby {
                    continue;
                }
            }

            let roll: f64 = rng.random();
            if roll < ev.chance {
                triggered_messages.push(ev.message.clone());
                cooldown_updates.push((ev.id.clone(), current_turn));
                break;
            }
        }
    }

    if !triggered_messages.is_empty()
        && let Some(mut bus) = world.get_resource_mut::<EventBus>() {
            for msg in triggered_messages {
                bus.push(GameEvent::Message(msg));
            }
        }

    if !cooldown_updates.is_empty()
        && let Some(mut cooldowns) = world.get_resource_mut::<NarrativeCooldowns>() {
            for (id, turn) in cooldown_updates {
                cooldowns.last_fired.insert(id, turn);
            }
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_narrative_events_from_toml() {
        let toml = r#"
[[narrative_event]]
id = "fire_spread"
trigger = { entity_tags = ["BURNING"], nearby_tags = ["FLAMMABLE"] }
message = "The flames spread!"
chance = 0.3
cooldown_ticks = 50

[[narrative_event]]
id = "undead_repelled"
trigger = { entity_tags = ["UNDEAD"], nearby_tags = ["BLESSED"] }
message = "Holy energy repels the undead!"
chance = 1.0
"#;
        let events = load_narrative_events(toml).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, "fire_spread");
        assert_eq!(events[0].trigger.entity_tags, vec!["BURNING"]);
        assert_eq!(events[0].trigger.nearby_tags, vec!["FLAMMABLE"]);
        assert_eq!(events[0].chance, 0.3);
        assert_eq!(events[0].cooldown_ticks, 50);
        assert_eq!(events[1].id, "undead_repelled");
        assert_eq!(events[1].cooldown_ticks, 0);
    }

    #[test]
    fn load_narrative_events_with_player_tags() {
        let toml = r#"
[[narrative_event]]
id = "herb_reminder"
trigger = { entity_tags = ["POISONED"], player_tags = ["HERB_MEDICINAL"] }
message = "You remember the medicinal herbs..."
chance = 0.5
cooldown_ticks = 100
"#;
        let events = load_narrative_events(toml).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].trigger.player_tags, vec!["HERB_MEDICINAL"]);
    }

    #[test]
    fn narrative_cooldowns_default() {
        let cooldowns = NarrativeCooldowns::default();
        assert!(cooldowns.last_fired.is_empty());
    }

    #[test]
    fn check_events_triggers_on_matching_entity() {
        let mut world = World::new();
        let tags_toml = include_str!("../../tags/assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let reg = registry.clone();
        world.insert_resource(registry);
        world.insert_resource(TurnCounter::new());
        world.insert_resource(EventBus::new());
        world.insert_resource(NarrativeCooldowns::default());

        let fire_id = reg.tag_id("FIRE").unwrap();
        let flammable_id = reg.tag_id("FLAMMABLE").unwrap();

        let mut burning_tags = Tags::new(reg.tag_count());
        burning_tags.add_tag(fire_id, game_tags::TagValue::None, &reg);

        let mut flammable_tags = Tags::new(reg.tag_count());
        flammable_tags.add_tag(flammable_id, game_tags::TagValue::None, &reg);

        world.spawn((
            burning_tags,
            Position { x: 5, y: 5, z: 0 },
            crate::Name("Burning thing".into()),
        ));
        world.spawn((
            flammable_tags,
            Position { x: 6, y: 5, z: 0 },
            crate::Name("Wood".into()),
        ));

        let events = vec![NarrativeEvent {
            id: "fire_spread".into(),
            trigger: NarrativeTrigger {
                entity_tags: vec!["FIRE".into()],
                nearby_tags: vec!["FLAMMABLE".into()],
                player_tags: vec![],
            },
            message: "The flames spread!".into(),
            chance: 1.0,
            cooldown_ticks: 10,
        }];
        world.insert_resource(NarrativeEvents { events });

        check_narrative_events(&mut world);

        let bus = world.get_resource::<EventBus>().unwrap();
        assert!(bus.events.iter().any(|e| matches!(e, GameEvent::Message(m) if m == "The flames spread!")),
            "narrative event should push to EventBus");
    }

    #[test]
    fn check_events_respects_cooldown() {
        let mut world = World::new();
        let tags_toml = include_str!("../../tags/assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let reg = registry.clone();
        world.insert_resource(registry);
        world.insert_resource(TurnCounter::new());
        world.insert_resource(EventBus::new());
        world.insert_resource(NarrativeCooldowns::default());

        let fire_id = reg.tag_id("FIRE").unwrap();
        let mut tags = Tags::new(reg.tag_count());
        tags.add_tag(fire_id, game_tags::TagValue::None, &reg);
        world.spawn((tags, Position { x: 5, y: 5, z: 0 }, crate::Name("Thing".into())));

        let events = vec![NarrativeEvent {
            id: "fire".into(),
            trigger: NarrativeTrigger {
                entity_tags: vec!["FIRE".into()],
                nearby_tags: vec![],
                player_tags: vec![],
            },
            message: "Fire!".into(),
            chance: 1.0,
            cooldown_ticks: 10,
        }];
        world.insert_resource(NarrativeEvents { events });

        check_narrative_events(&mut world);
        assert_eq!(world.get_resource::<EventBus>().unwrap().events.len(), 1, "should fire on first check");
        world.get_resource_mut::<EventBus>().unwrap().drain();

        for _ in 0..5 {
            world.get_resource_mut::<TurnCounter>().unwrap().increment();
        }
        check_narrative_events(&mut world);
        assert_eq!(world.get_resource::<EventBus>().unwrap().events.len(), 0, "should not fire during cooldown");

        for _ in 0..5 {
            world.get_resource_mut::<TurnCounter>().unwrap().increment();
        }
        check_narrative_events(&mut world);
        assert_eq!(world.get_resource::<EventBus>().unwrap().events.len(), 1, "should fire after cooldown");
    }

    #[test]
    fn check_events_skips_when_tags_dont_match() {
        let mut world = World::new();
        let tags_toml = include_str!("../../tags/assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let reg = registry.clone();
        world.insert_resource(registry);
        world.insert_resource(TurnCounter::new());
        world.insert_resource(MessageLog::new(50));
        world.insert_resource(NarrativeCooldowns::default());

        let water_id = reg.tag_id("WATER").unwrap();
        let mut tags = Tags::new(reg.tag_count());
        tags.add_tag(water_id, game_tags::TagValue::None, &reg);
        world.spawn((tags, Position { x: 5, y: 5, z: 0 }));

        let events = vec![NarrativeEvent {
            id: "fire".into(),
            trigger: NarrativeTrigger {
                entity_tags: vec!["FIRE".into()],
                nearby_tags: vec![],
                player_tags: vec![],
            },
            message: "Fire!".into(),
            chance: 1.0,
            cooldown_ticks: 0,
        }];
        world.insert_resource(NarrativeEvents { events });

        check_narrative_events(&mut world);
        assert!(world.get_resource::<MessageLog>().unwrap().messages.is_empty());
    }

    #[test]
    fn check_events_requires_player_tags() {
        let mut world = World::new();
        let tags_toml = include_str!("../../tags/assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let reg = registry.clone();
        world.insert_resource(registry);
        world.insert_resource(TurnCounter::new());
        world.insert_resource(MessageLog::new(50));
        world.insert_resource(NarrativeCooldowns::default());

        let fire_id = reg.tag_id("FIRE").unwrap();
        let mut tags = Tags::new(reg.tag_count());
        tags.add_tag(fire_id, game_tags::TagValue::None, &reg);
        world.spawn((tags, Position { x: 5, y: 5, z: 0 }));

        let mut player_tags = Tags::new(reg.tag_count());
        let water_id = reg.tag_id("WATER").unwrap();
        player_tags.add_tag(water_id, game_tags::TagValue::None, &reg);
        world.spawn((crate::Player, player_tags));

        let events = vec![NarrativeEvent {
            id: "herb".into(),
            trigger: NarrativeTrigger {
                entity_tags: vec!["FIRE".into()],
                nearby_tags: vec![],
                player_tags: vec!["WOOD".into()],
            },
            message: "Herb reminder".into(),
            chance: 1.0,
            cooldown_ticks: 0,
        }];
        world.insert_resource(NarrativeEvents { events });

        check_narrative_events(&mut world);
        assert!(world.get_resource::<MessageLog>().unwrap().messages.is_empty(), "player doesn't have WOOD tag");
    }

    #[test]
    fn load_lore_fragments_from_toml() {
        let toml = r#"
[[lore_fragment]]
id = "test_fragment"
category = "world_history"
rarity = "common"
faction_source = "free_humanity"
title_template = "Test Title"
content_template = "Test content."
discovery_tags = ["HUMAN"]
discovery_context = "dialogue"
persistence = true
"#;
        let fragments = load_lore_fragments(toml).unwrap();
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].id, "test_fragment");
        assert_eq!(fragments[0].category, "world_history");
        assert_eq!(fragments[0].rarity, "common");
        assert_eq!(fragments[0].discovery_tags, vec!["HUMAN"]);
        assert!(fragments[0].persistence);
    }

    #[test]
    fn load_actual_lore_fragments_toml() {
        let lore_toml = include_str!("../../../assets/config/lore_fragments.toml");
        let fragments = load_lore_fragments(lore_toml).unwrap();
        assert!(fragments.len() >= 32, "should have at least 32 lore fragments");
        assert!(fragments.iter().any(|f| f.id == "the_great_collapse"));
        assert!(fragments.iter().any(|f| f.id == "carapace_origin"));
    }

    #[test]
    fn lore_fragments_resource_default() {
        let resource = LoreFragmentsResource::default();
        assert!(resource.fragments.is_empty());
    }
}
