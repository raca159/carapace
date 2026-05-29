use bevy_ecs::prelude::*;
use game_tags::{TagId, TagRegistry};

#[derive(Debug, Clone)]
pub enum GameEvent {
    Interaction {
        entity: Option<Entity>,
        nearby_entity: Option<Entity>,
        kind: InteractionKind,
        source_tags: Vec<TagId>,
        target_tags: Vec<TagId>,
        result_tags: Vec<TagId>,
        consumed_tags: Vec<TagId>,
    },
    Combat {
        attacker: Option<Entity>,
        target: Option<Entity>,
        attacker_name: String,
        target_name: String,
        damage_dealt: u32,
        target_hp_remaining: u32,
        target_died: bool,
        is_player_attacker: bool,
        is_player_target: bool,
    },
    PlayerDied,
    ItemPickedUp {
        item_name: String,
    },
    ItemConsumed {
        item_name: String,
        healed: u32,
        poisoned: bool,
        extinguished: bool,
    },
    ItemEquipped {
        item_name: String,
    },
    ItemUnequipped {
        item_name: String,
    },
    ItemThrown {
        item_name: String,
        hit_entity: bool,
        hit_name: Option<String>,
        damage: u32,
        target_died: bool,
    },
    ItemCrafted {
        item_name: String,
    },
    QuestAccepted {
        name: String,
    },
    QuestCompleted {
        name: String,
    },
    StatusApplied {
        entity: Option<Entity>,
        tag_id: TagId,
    },
    StatusRemoved {
        entity: Option<Entity>,
        tag_id: TagId,
    },
    Dialogue {
        speaker: Option<Entity>,
        speaker_name: String,
    },
    DungeonEntered {
        name: String,
    },
    DungeonDescended {
        depth: u32,
    },
    DungeonExited,
    LootDropped {
        items: Vec<String>,
    },
    Message(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionKind {
    Self_,
    Cross,
}

#[derive(Resource, Debug, Clone)]
pub struct GameEventLog {
    pub events: Vec<GameEvent>,
    pub max: usize,
}

impl GameEventLog {
    pub fn new(max: usize) -> Self {
        Self {
            events: Vec::new(),
            max,
        }
    }

    pub fn push(&mut self, event: GameEvent) {
        if self.events.len() >= self.max {
            self.events.remove(0);
        }
        self.events.push(event);
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct EventBus {
    pub events: Vec<GameEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }
}

fn tag_name(tag_id: TagId, registry: &TagRegistry) -> String {
    registry.tag_by_id(tag_id).name.clone()
}

pub fn format_event(event: &GameEvent, registry: &TagRegistry) -> String {
    match event {
        GameEvent::Interaction {
            kind,
            source_tags,
            target_tags,
            result_tags,
            consumed_tags,
            ..
        } => {
            let sources: Vec<String> = source_tags.iter().map(|&t| tag_name(t, registry)).collect();
            let targets: Vec<String> = target_tags.iter().map(|&t| tag_name(t, registry)).collect();
            let results: Vec<String> = result_tags.iter().map(|&t| tag_name(t, registry)).collect();
            let consumed: Vec<String> = consumed_tags.iter().map(|&t| tag_name(t, registry)).collect();

            let mut parts = Vec::new();
            if !consumed.is_empty() {
                parts.push(format!("{} fades", consumed.join(", ")));
            }
            if !results.is_empty() {
                let prefix = match kind {
                    InteractionKind::Self_ => "A reaction creates",
                    InteractionKind::Cross => "The interaction creates",
                };
                parts.push(format!("{} {}", prefix, results.join(", ")));
            }
            if parts.is_empty() {
                match kind {
                    InteractionKind::Self_ => format!("{} meets {}", sources.join("+"), targets.join("+")),
                    InteractionKind::Cross => format!("{} reacts with {}", sources.join("+"), targets.join("+")),
                }
            } else {
                parts.join(". ")
            }
        }
        GameEvent::Combat {
            attacker_name,
            target_name,
            damage_dealt,
            target_hp_remaining,
            target_died,
            is_player_attacker,
            ..
        } => {
            if *target_died {
                if *is_player_attacker {
                    format!("You hit {} for {} damage. It dies!", target_name, damage_dealt)
                } else {
                    format!("{} kills {}!", attacker_name, target_name)
                }
            } else if *is_player_attacker {
                format!("You hit {} for {} damage. ({} HP remaining)", target_name, damage_dealt, target_hp_remaining)
            } else {
                format!("{} hits {} for {} damage! ({} HP remaining)", attacker_name, target_name, damage_dealt, target_hp_remaining)
            }
        }
        GameEvent::PlayerDied => "You have died!".to_string(),
        GameEvent::ItemPickedUp { item_name } => format!("Picked up {}.", item_name),
        GameEvent::ItemConsumed { item_name, healed, poisoned, extinguished } => {
            if *healed > 0 {
                format!("You consume the {}. Healed {} HP.", item_name, healed)
            } else if *poisoned {
                format!("You consume the {}. You feel sick...", item_name)
            } else if *extinguished {
                format!("You consume the {}. The flames die down.", item_name)
            } else {
                format!("You consume the {}.", item_name)
            }
        }
        GameEvent::ItemEquipped { item_name } => format!("You equip the {}.", item_name),
        GameEvent::ItemUnequipped { item_name } => format!("You unequip the {}.", item_name),
        GameEvent::ItemThrown { item_name, hit_entity, hit_name, damage, target_died } => {
            if *hit_entity {
                if *target_died {
                    format!("You hit {} with the {} for {} damage! It dies!", hit_name.as_deref().unwrap_or("something"), item_name, damage)
                } else {
                    format!("You hit {} with the {} for {} damage!", hit_name.as_deref().unwrap_or("something"), item_name, damage)
                }
            } else {
                format!("You throw the {}. It lands on the ground.", item_name)
            }
        }
        GameEvent::ItemCrafted { item_name } => format!("You craft a {}.", item_name),
        GameEvent::QuestAccepted { name } => format!("Quest accepted: {}!", name),
        GameEvent::QuestCompleted { name } => format!("Quest completed: {}!", name),
        GameEvent::StatusApplied { tag_id, .. } => {
            let tn = tag_name(*tag_id, registry);
            format!("{} status gained.", tn)
        }
        GameEvent::StatusRemoved { tag_id, .. } => {
            let tn = tag_name(*tag_id, registry);
            format!("{} status fades.", tn)
        }
        GameEvent::Dialogue { speaker_name, .. } => format!("{} speaks.", speaker_name),
        GameEvent::DungeonEntered { name } => format!("You descend into the {}.", name),
        GameEvent::DungeonDescended { depth } => format!("You descend to depth {}...", depth),
        GameEvent::DungeonExited => "You ascend back to the surface.".to_string(),
        GameEvent::LootDropped { items } => format!("Dropped: {}.", items.join(", ")),
        GameEvent::Message(msg) => msg.clone(),
    }
}

pub fn push_event(world: &mut World, event: GameEvent) {
    let registry = world
        .get_resource::<TagRegistry>()
        .cloned();
    let message = registry
        .as_ref()
        .map(|r| format_event(&event, r))
        .unwrap_or_else(|| format!("{:?}", event));

    if let Some(mut log) = world.get_resource_mut::<crate::components::MessageLog>() {
        log.push(message);
    }
    if let Some(mut event_log) = world.get_resource_mut::<GameEventLog>() {
        event_log.push(event);
    }
}
