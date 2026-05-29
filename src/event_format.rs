use std::collections::HashMap;

use bevy_ecs::prelude::{Entity, Resource, World};

use game_core::{EventBus, GameEvent, InteractionKind, MessageLog, Name};
use game_tags::{TagId, TagRegistry, Tags};

#[derive(Debug, Clone, Resource)]
pub struct EventFormats {
    pub templates: HashMap<String, String>,
}

impl EventFormats {
    pub fn get(&self, key: &str) -> Option<&String> {
        self.templates.get(key)
    }
}

pub fn load_event_formats(toml_content: &str) -> Result<EventFormats, toml::de::Error> {
    let raw: HashMap<String, toml::Value> = toml::from_str(toml_content)?;
    let mut templates = HashMap::new();

    for (key, value) in raw {
        if let Some(table) = value.as_table()
            && let Some(toml::Value::String(template)) = table.get("template") {
                templates.insert(key, template.clone());
        }
    }

    Ok(EventFormats { templates })
}

fn entity_name(world: &World, entity: Entity) -> String {
    world
        .get::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "something".to_string())
}

fn tag_name(registry: &TagRegistry, tag_id: TagId) -> String {
    registry.tag_by_id(tag_id).name.clone()
}

fn apply_template(template: &str, placeholders: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in placeholders {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

fn format_with_template(
    event: &GameEvent,
    formats: &EventFormats,
    world: &World,
    registry: &TagRegistry,
) -> Option<String> {
    match event {
        GameEvent::Combat {
            attacker_name,
            target_name,
            damage_dealt,
            target_hp_remaining,
            target_died,
            is_player_attacker,
            is_player_target,
            ..
        } => {
            if *is_player_attacker && *target_died {
                formats.get("entity_killed").map(|t| {
                    apply_template(t, &[
                        ("victim_name", target_name),
                        ("damage", &damage_dealt.to_string()),
                    ])
                })
            } else if *is_player_attacker {
                formats.get("entity_damaged").map(|t| {
                    apply_template(t, &[
                        ("defender_name", target_name),
                        ("amount", &damage_dealt.to_string()),
                        ("hp_remaining", &target_hp_remaining.to_string()),
                    ])
                })
            } else if *is_player_target {
                formats.get("player_damaged").map(|t| {
                    apply_template(t, &[
                        ("attacker_name", attacker_name),
                        ("amount", &damage_dealt.to_string()),
                        ("hp_current", &target_hp_remaining.to_string()),
                        ("hp_max", &target_hp_remaining.to_string()),
                    ])
                })
            } else {
                None
            }
        }
        GameEvent::PlayerDied => formats.get("player_death").cloned(),
        GameEvent::ItemPickedUp { item_name } => formats
            .get("pickup_item")
            .map(|t| apply_template(t, &[("item_name", item_name)])),
        GameEvent::ItemConsumed {
            item_name,
            healed,
            poisoned,
            extinguished,
        } => {
            if *healed > 0 {
                formats.get("item_consumed.healed").map(|t| {
                    apply_template(t, &[
                        ("item_name", item_name),
                        ("healed", &healed.to_string()),
                    ])
                })
            } else if *poisoned {
                formats.get("item_consumed.poisoned").map(|t| {
                    apply_template(t, &[("item_name", item_name)])
                })
            } else if *extinguished {
                formats.get("item_consumed.extinguished").map(|t| {
                    apply_template(t, &[("item_name", item_name)])
                })
            } else {
                formats.get("item_consumed").map(|t| {
                    apply_template(t, &[("item_name", item_name)])
                })
            }
        }
        GameEvent::ItemEquipped { item_name } => formats
            .get("item_equipped")
            .map(|t| apply_template(t, &[("item_name", item_name)])),
        GameEvent::ItemUnequipped { item_name } => formats
            .get("item_unequipped")
            .map(|t| apply_template(t, &[("item_name", item_name)])),
        GameEvent::ItemThrown {
            item_name,
            hit_entity,
            hit_name,
            damage,
            target_died,
            ..
        } => {
            if *hit_entity {
                if *target_died {
                    formats.get("item_thrown.kill").map(|t| {
                        apply_template(t, &[
                            ("item_name", item_name),
                            ("target_name", hit_name.as_deref().unwrap_or("something")),
                            ("damage", &damage.to_string()),
                        ])
                    })
                } else {
                    formats.get("item_thrown").map(|t| {
                        apply_template(t, &[
                            ("item_name", item_name),
                            ("target_name", hit_name.as_deref().unwrap_or("something")),
                            ("damage", &damage.to_string()),
                            ("hp_remaining", "?"),
                        ])
                    })
                }
            } else {
                formats.get("item_thrown.ground").map(|t| {
                    apply_template(t, &[("item_name", item_name)])
                })
            }
        }
        GameEvent::ItemCrafted { item_name } => formats
            .get("item_crafted")
            .map(|t| apply_template(t, &[("item_name", item_name)])),
        GameEvent::QuestAccepted { name } => formats
            .get("quest_accepted")
            .map(|t| apply_template(t, &[("quest_name", name)])),
        GameEvent::QuestCompleted { name } => formats
            .get("quest_completed")
            .map(|t| apply_template(t, &[("quest_name", name)])),
        GameEvent::StatusApplied { entity, tag_id } => {
            let entity = (*entity)?;
            formats.get("status_applied").map(|t| {
                apply_template(t, &[
                    ("status", &tag_name(registry, *tag_id)),
                    ("entity_name", &entity_name(world, entity)),
                ])
            })
        }
        GameEvent::StatusRemoved { entity, tag_id } => {
            let entity = (*entity)?;
            formats.get("status_expired").map(|t| {
                apply_template(t, &[
                    ("status", &tag_name(registry, *tag_id)),
                    ("entity_name", &entity_name(world, entity)),
                ])
            })
        }
        GameEvent::Dialogue { speaker, speaker_name } => {
            let name = if let Some(entity) = speaker {
                entity_name(world, *entity)
            } else {
                speaker_name.clone()
            };
            Some(format!("{} speaks.", name))
        }
        GameEvent::DungeonEntered { name } => formats
            .get("dungeon_entered")
            .map(|t| apply_template(t, &[("dungeon_type", name)])),
        GameEvent::DungeonDescended { depth: _ } => {
            formats.get("dungeon_descended").cloned()
        }
        GameEvent::DungeonExited => formats.get("dungeon_exited").cloned(),
        GameEvent::LootDropped { items } => formats
            .get("creature_loot_dropped")
            .map(|t| apply_template(t, &[("dropped_items", &items.join(", "))])),
        GameEvent::Message(msg) => Some(msg.clone()),
        GameEvent::Interaction { .. } => None,
    }
}

pub fn format_events(world: &mut World) {
    let registry = world.get_resource::<TagRegistry>().cloned();
    let formats = world.get_resource::<EventFormats>().cloned();

    let events = {
        let mut bus = match world.get_resource_mut::<EventBus>() {
            Some(b) => b,
            None => return,
        };
        bus.drain()
    };

    if events.is_empty() {
        return;
    }

    for event in &events {
        let message = formats
            .as_ref()
            .zip(registry.as_ref())
            .and_then(|(f, r)| format_with_template(event, f, world, r))
            .unwrap_or_else(|| {
                registry
                    .as_ref()
                    .map(|r| game_core::format_event(event, r))
                    .unwrap_or_else(|| format!("{:?}", event))
            });

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.push(message);
        }
    }
}

fn push_to_bus(world: &mut World, event: GameEvent) {
    if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
        bus.push(event);
    }
}

pub fn emit_status_applied(world: &mut World, entity: Entity, tag_id: TagId) {
    push_to_bus(world, GameEvent::StatusApplied {
        entity: Some(entity),
        tag_id,
    });
}

pub fn emit_status_removed(world: &mut World, entity: Entity, tag_id: TagId) {
    push_to_bus(world, GameEvent::StatusRemoved {
        entity: Some(entity),
        tag_id,
    });
}

pub fn emit_tag_diff_events(world: &mut World, entity: Entity, old_tags: &Tags) {
    let new_tags = match world.get::<Tags>(entity) {
        Some(t) => t.clone(),
        None => return,
    };

    for tag_id in new_tags.iter_present() {
        if !old_tags.has(tag_id) {
            emit_status_applied(world, entity, tag_id);
        }
    }

    for tag_id in old_tags.iter_present() {
        if !new_tags.has(tag_id) {
            emit_status_removed(world, entity, tag_id);
        }
    }
}

pub fn emit_self_interaction_events(
    world: &mut World,
    entity: Entity,
    rules: &[&game_tags::InteractionRule],
    _old_tags: &Tags,
) {
    for rule in rules {
        push_to_bus(world, GameEvent::Interaction {
            entity: Some(entity),
            nearby_entity: None,
            kind: InteractionKind::Self_,
            source_tags: vec![rule.tag_a],
            target_tags: vec![rule.tag_b],
            result_tags: rule.produces.clone(),
            consumed_tags: rule.consumes.clone(),
        });
    }
}

pub fn emit_cross_interaction_events(
    world: &mut World,
    entity_a: Entity,
    entity_b: Entity,
    matched_rules: &[(&game_tags::InteractionRule, bool)],
) {
    for (rule, reversed) in matched_rules {
        let source_tags = if *reversed {
            vec![rule.tag_b]
        } else {
            vec![rule.tag_a]
        };
        let target_tags = if *reversed {
            vec![rule.tag_a]
        } else {
            vec![rule.tag_b]
        };

        push_to_bus(world, GameEvent::Interaction {
            entity: Some(entity_a),
            nearby_entity: Some(entity_b),
            kind: InteractionKind::Cross,
            source_tags,
            target_tags,
            result_tags: rule.produces.clone(),
            consumed_tags: rule.consumes.clone(),
        });
    }
}
