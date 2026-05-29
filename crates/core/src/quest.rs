use std::collections::HashSet;

use bevy_ecs::prelude::*;
use bevy_ecs::entity::Entity;
use rand::Rng;
use rand::SeedableRng;
use serde::Deserialize;

use crate::{EventBus, GameEvent, Glyph, Inventory, Item, Name, Position};
use game_tags::{TagId, TagRegistry, TagValue, Tags};

#[derive(Component, Debug, Clone)]
pub struct QuestGiver;

#[derive(Component, Debug, Clone)]
pub struct QuestBoard;

#[derive(Debug, Clone)]
pub struct QuestBoardEntry {
    pub slot: usize,
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub objective_type: String,
    pub target_info: String,
    pub reward_info: String,
}

#[derive(Resource, Debug, Clone)]
pub struct QuestBoardState {
    pub available_quests: Vec<QuestBoardEntry>,
    pub turn_count_since_refresh: u32,
    pub refresh_interval: u32,
}

impl Default for QuestBoardState {
    fn default() -> Self {
        Self {
            available_quests: Vec::new(),
            turn_count_since_refresh: 0,
            refresh_interval: 20,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct ActiveQuest {
    pub quest_id: String,
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub objective: QuestObjective,
    pub state: QuestState,
    pub reward_tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum QuestObjective {
    Kill { target_name: String, killed: u32, required: u32 },
    Collect { tag_name: String, collected: u32, required: u32 },
    Reach { biome: String, reached: bool },
    KillInArea { biome: String, killed: u32, required: u32 },
}

impl QuestObjective {
    pub fn progress_text(&self) -> String {
        match self {
            QuestObjective::Kill { target_name, killed, required } => {
                format!("{}/{} {}", killed, required, target_name)
            }
            QuestObjective::Collect { tag_name, collected, required } => {
                format!("{}/{} {}", collected, required, tag_name)
            }
            QuestObjective::Reach { biome, reached } => {
                if *reached {
                    format!("Reached {}", biome)
                } else {
                    format!("Find {}", biome)
                }
            }
            QuestObjective::KillInArea { biome, killed, required } => {
                format!("{}/{} in {}", killed, required, biome)
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            QuestObjective::Kill { killed, required, .. } => killed >= required,
            QuestObjective::Collect { collected, required, .. } => collected >= required,
            QuestObjective::Reach { reached, .. } => *reached,
            QuestObjective::KillInArea { killed, required, .. } => killed >= required,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestState {
    Active,
    Complete,
    Failed,
    TurnedIn,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct QuestTemplates {
    pub templates: Vec<QuestTemplate>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct QuestLog {
    pub turned_in: Vec<String>,
}

impl QuestLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_turn_in(&mut self, quest_name: String) {
        self.turned_in.push(quest_name);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestTemplate {
    pub id: String,
    pub name_pattern: String,
    pub description_pattern: String,
    pub objective_type: String,
    #[serde(default)]
    pub target_tags: Vec<String>,
    #[serde(default)]
    pub reward_tags: Vec<String>,
    #[serde(default)]
    pub biome_tags: Vec<String>,
    #[serde(default = "default_one")]
    pub required_count: u32,
    #[serde(default = "default_three")]
    pub kill_count: u32,
}

fn default_one() -> u32 { 1 }
fn default_three() -> u32 { 3 }

#[derive(Debug, Clone, Deserialize)]
struct QuestsToml {
    #[serde(rename = "quest_template")]
    templates: Vec<QuestTemplate>,
}

pub fn load_quest_templates(toml_str: &str) -> Result<Vec<QuestTemplate>, toml::de::Error> {
    let file: QuestsToml = toml::from_str(toml_str)?;
    Ok(file.templates)
}

pub fn generate_quests(
    templates: &[QuestTemplate],
    world: &mut World,
    registry: &TagRegistry,
) -> Vec<ActiveQuest> {
    let mut quests = Vec::new();
    let mut quest_counter: u64 = 0;

    let entities: Vec<(Entity, Tags, Option<Position>, Option<String>)> = {
        let mut query = world.query::<(Entity, &Tags, Option<&Position>, Option<&Name>)>();
        query.iter(world).map(|(e, t, p, n)| {
            (e, t.clone(), p.copied(), n.map(|nm| nm.0.clone()))
        }).collect()
    };

    let active_template_ids: HashSet<String> = {
        let mut aq = world.query::<&ActiveQuest>();
        aq.iter(world).map(|q| q.template_id.clone()).collect()
    };

    for template in templates {
        if active_template_ids.contains(&template.id) {
            continue;
        }

        match template.objective_type.as_str() {
            "kill" => {
                let target_tag_ids: Vec<TagId> = template.target_tags.iter()
                    .filter_map(|n| registry.tag_id(n))
                    .collect();
                if target_tag_ids.is_empty() {
                    continue;
                }
                let mut found_name: Option<String> = None;
                for (_, tags, _, name) in &entities {
                    if target_tag_ids.iter().all(|&id| tags.has(id)) {
                        found_name = name.clone().or_else(|| {
                            template.target_tags.first().map(|t| t.replace("_", " ").to_lowercase())
                        });
                        break;
                    }
                }
                if let Some(target_name) = found_name {
                    quest_counter += 1;
                    let name = template.name_pattern.replace("{target}", &target_name);
                    let desc = template.description_pattern.replace("{target}", &target_name);
                    quests.push(ActiveQuest {
                        quest_id: format!("quest_{}", quest_counter),
                        template_id: template.id.clone(),
                        name,
                        description: desc,
                        objective: QuestObjective::Kill {
                            target_name,
                            killed: 0,
                            required: template.required_count,
                        },
                        state: QuestState::Active,
                        reward_tags: template.reward_tags.clone(),
                    });
                }
            }
            "collect" => {
                let resource_name = template.target_tags.first()
                    .map(|t| t.replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "resources".to_string());
                quest_counter += 1;
                let name = template.name_pattern.replace("{resource}", &resource_name);
                let desc = template.description_pattern.replace("{resource}", &resource_name);
                let tag_name = template.target_tags.first().cloned().unwrap_or_default();
                quests.push(ActiveQuest {
                    quest_id: format!("quest_{}", quest_counter),
                    template_id: template.id.clone(),
                    name,
                    description: desc,
                    objective: QuestObjective::Collect {
                        tag_name: tag_name.replace("_", " "),
                        collected: 0,
                        required: template.required_count,
                    },
                    state: QuestState::Active,
                    reward_tags: template.reward_tags.clone(),
                });
            }
            "reach" => {
                let biome_name = template.biome_tags.first()
                    .map(|t| t.trim_start_matches("BIOME_").replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                quest_counter += 1;
                let name = template.name_pattern.replace("{biome}", &biome_name);
                let desc = template.description_pattern.replace("{biome}", &biome_name);
                quests.push(ActiveQuest {
                    quest_id: format!("quest_{}", quest_counter),
                    template_id: template.id.clone(),
                    name,
                    description: desc,
                    objective: QuestObjective::Reach {
                        biome: biome_name,
                        reached: false,
                    },
                    state: QuestState::Active,
                    reward_tags: template.reward_tags.clone(),
                });
            }
            "kill_area" => {
                let biome_name = template.biome_tags.first()
                    .map(|t| t.trim_start_matches("BIOME_").replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                quest_counter += 1;
                let name = template.name_pattern.replace("{biome}", &biome_name);
                let desc = template.description_pattern.replace("{biome}", &biome_name);
                quests.push(ActiveQuest {
                    quest_id: format!("quest_{}", quest_counter),
                    template_id: template.id.clone(),
                    name,
                    description: desc,
                    objective: QuestObjective::KillInArea {
                        biome: biome_name,
                        killed: 0,
                        required: template.kill_count,
                    },
                    state: QuestState::Active,
                    reward_tags: template.reward_tags.clone(),
                });
            }
            _ => {}
        }
    }

    quests
}

pub fn check_quest_completion(quest: &mut ActiveQuest) -> bool {
    if quest.state != QuestState::Active {
        return false;
    }
    if quest.objective.is_complete() {
        quest.state = QuestState::Complete;
        return true;
    }
    false
}

pub fn track_kill(world: &mut World, killed_name: &str) {
    let mut quest_query = world.query::<&mut ActiveQuest>();
    let mut updates: Vec<(String, u32)> = Vec::new();
    for quest in quest_query.iter_mut(world) {
        if quest.state != QuestState::Active {
            continue;
        }
        if let QuestObjective::Kill { ref target_name, killed, required, .. } = quest.objective
            && target_name.to_lowercase() == killed_name.to_lowercase() && killed < required {
                updates.push((quest.quest_id.clone(), killed + 1));
            }
    }
    drop(quest_query);
    for (quest_id, new_killed) in updates {
        let mut quest_query = world.query::<&mut ActiveQuest>();
        for mut quest in quest_query.iter_mut(world) {
            if quest.quest_id == quest_id {
                if let QuestObjective::Kill { ref mut killed, .. } = quest.objective {
                    *killed = new_killed;
                }
                check_quest_completion(&mut quest);
            }
        }
    }
}

pub fn track_collect(world: &mut World, tag_name: &str) {
    let tag_lower = tag_name.to_lowercase().replace(" ", "_");
    let mut quest_query = world.query::<&mut ActiveQuest>();
    let mut updates: Vec<(String, u32)> = Vec::new();
    for quest in quest_query.iter_mut(world) {
        if quest.state != QuestState::Active {
            continue;
        }
        if let QuestObjective::Collect { ref tag_name, collected, required, .. } = quest.objective {
            let quest_tag_lower = tag_name.to_lowercase().replace(" ", "_");
            if quest_tag_lower == tag_lower && collected < required {
                updates.push((quest.quest_id.clone(), collected + 1));
            }
        }
    }
    drop(quest_query);
    for (quest_id, new_collected) in updates {
        let mut quest_query = world.query::<&mut ActiveQuest>();
        for mut quest in quest_query.iter_mut(world) {
            if quest.quest_id == quest_id {
                if let QuestObjective::Collect { ref mut collected, .. } = quest.objective {
                    *collected = new_collected;
                }
                check_quest_completion(&mut quest);
            }
        }
    }
}

pub fn track_reach(world: &mut World, biome_name: &str) {
    let biome_lower = biome_name.to_lowercase();
    let mut quest_query = world.query::<&mut ActiveQuest>();
    for mut quest in quest_query.iter_mut(world) {
        if quest.state != QuestState::Active {
            continue;
        }
        if let QuestObjective::Reach { ref biome, reached, .. } = quest.objective
            && !reached && biome.to_lowercase() == biome_lower {
                quest.objective = QuestObjective::Reach {
                    biome: biome.clone(),
                    reached: true,
                };
                check_quest_completion(&mut quest);
            }
    }
}

pub fn track_kill_area(world: &mut World, biome_name: &str) {
    let biome_lower = biome_name.to_lowercase();
    let mut quest_query = world.query::<&mut ActiveQuest>();
    let mut updates: Vec<(String, u32)> = Vec::new();
    for quest in quest_query.iter_mut(world) {
        if quest.state != QuestState::Active {
            continue;
        }
        if let QuestObjective::KillInArea { ref biome, killed, required, .. } = quest.objective
            && biome.to_lowercase() == biome_lower && killed < required {
                updates.push((quest.quest_id.clone(), killed + 1));
            }
    }
    drop(quest_query);
    for (quest_id, new_killed) in updates {
        let mut quest_query = world.query::<&mut ActiveQuest>();
        for mut quest in quest_query.iter_mut(world) {
            if quest.quest_id == quest_id {
                if let QuestObjective::KillInArea { ref mut killed, .. } = quest.objective {
                    *killed = new_killed;
                }
                check_quest_completion(&mut quest);
            }
        }
    }
}

pub fn turn_in_quest(
    reward_tags: &[String],
    world: &mut World,
    registry: &TagRegistry,
) -> Vec<Entity> {
    let mut reward_items = Vec::new();

    let reward_tag_ids: Vec<TagId> = reward_tags.iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();

    if !reward_tag_ids.is_empty() {
        let mut entity_tags = Tags::new(registry.tag_count());
        for &tag_id in &reward_tag_ids {
            entity_tags.add_tag(tag_id, TagValue::None, registry);
        }

        let first_tag_name = reward_tags.first()
            .map(|t| t.replace("_", " ").to_lowercase())
            .unwrap_or_else(|| "reward item".to_string());

        let entity = world.spawn((
            Item,
            Name(first_tag_name.clone()),
            Glyph { char: '*', color: (255, 215, 0) },
            entity_tags,
        )).id();

        reward_items.push(entity);

        if let Some(player_entity) = {
            let mut pq = world.query_filtered::<Entity, bevy_ecs::query::With<crate::Player>>();
            pq.single(world).ok()
        }
            && let Some(mut inv) = world.get_mut::<Inventory>(player_entity) {
                for &item in &reward_items {
                    inv.items.push(item);
                }
            }
    }

    reward_items
}

pub fn handle_quest_turn_in(world: &mut World, cursor: usize) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    let quests: Vec<(String, String, Vec<String>)> = {
        let mut qq = world.query::<&ActiveQuest>();
        qq.iter(world)
            .enumerate()
            .filter(|(_, q)| q.state == QuestState::Complete || q.state == QuestState::Active)
            .filter(|(i, _)| *i == cursor)
            .map(|(_, q)| (q.quest_id.clone(), q.name.clone(), q.reward_tags.clone()))
            .collect()
    };

    if let Some((quest_id, quest_name, reward_tags)) = quests.first().cloned() {
        let mut qq = world.query::<&mut ActiveQuest>();
        for quest in qq.iter_mut(world) {
            if quest.quest_id == quest_id && quest.state == QuestState::Complete {
                drop(qq);

                let _rewards = turn_in_quest(&reward_tags, world, &registry);

                let mut qq2 = world.query::<&mut ActiveQuest>();
                if let Some(mut quest) = qq2.iter_mut(world).find(|q| q.quest_id == quest_id) {
                    quest.state = QuestState::TurnedIn;
                }
                drop(qq2);

                if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
                    bus.push(GameEvent::QuestCompleted { name: quest_name.clone() });
                }

                if let Some(mut quest_log) = world.get_resource_mut::<QuestLog>() {
                    quest_log.record_turn_in(quest_name);
                }
                return;
            }
        }
    }
}

/// Check all active quests for failure conditions.
/// Currently a compatibility stub — the simple quest model lacks per-quest fail_conditions/turn_started.
pub fn check_quest_failures(
    _world: &mut World,
    _faction_standings: Option<&std::collections::HashMap<String, f32>>,
) {
    // No-op: the current ActiveQuest model uses a single objective without
    // fail conditions or turn tracking. Expand when multi-step quests land.
}

pub fn generate_board_quests(
    templates: &[QuestTemplate],
    world: &mut World,
    _registry: &TagRegistry,
    count: usize,
) -> Vec<QuestBoardEntry> {
    let mut entries = Vec::new();
    let seed: u64 = rand::random();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let active_template_ids: std::collections::HashSet<String> = {
        let mut aq = world.query::<&ActiveQuest>();
        aq.iter(world).map(|q| q.template_id.clone()).collect()
    };

    let candidates: Vec<&QuestTemplate> = templates.iter()
        .filter(|t| !active_template_ids.contains(&t.id))
        .collect();

    if candidates.is_empty() {
        return entries;
    }

    for slot in 0..count {
        let template = if let Some(t) = candidates.get(rng.random_range(0..candidates.len())) {
            t
        } else {
            continue;
        };

        let (name, desc, target_info) = match template.objective_type.as_str() {
            "kill" => {
                let target_name = template.target_tags.first()
                    .map(|t| t.replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "creature".to_string());
                let name = template.name_pattern.replace("{target}", &target_name);
                let desc = template.description_pattern.replace("{target}", &target_name);
                (name, desc, format!("Kill {}", target_name))
            }
            "collect" => {
                let resource_name = template.target_tags.first()
                    .map(|t| t.replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "resources".to_string());
                let name = template.name_pattern.replace("{resource}", &resource_name);
                let desc = template.description_pattern.replace("{resource}", &resource_name);
                (name, desc, format!("Gather {} (x{})", resource_name, template.required_count))
            }
            "reach" => {
                let biome_name = template.biome_tags.first()
                    .map(|t| t.trim_start_matches("BIOME_").replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                let name = template.name_pattern.replace("{biome}", &biome_name);
                let desc = template.description_pattern.replace("{biome}", &biome_name);
                (name, desc, format!("Reach {}", biome_name))
            }
            "kill_area" => {
                let biome_name = template.biome_tags.first()
                    .map(|t| t.trim_start_matches("BIOME_").replace("_", " ").to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                let name = template.name_pattern.replace("{biome}", &biome_name);
                let desc = template.description_pattern.replace("{biome}", &biome_name);
                (name, desc, format!("Slay {} in {}", template.kill_count, biome_name))
            }
            _ => continue,
        };

        let reward_info = template.reward_tags.first()
            .map(|t| t.replace("_", " ").to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        entries.push(QuestBoardEntry {
            slot,
            template_id: template.id.clone(),
            name,
            description: desc,
            objective_type: template.objective_type.clone(),
            target_info,
            reward_info,
        });
    }

    entries
}

pub fn accept_board_quest(
    world: &mut World,
    entry: &QuestBoardEntry,
) -> Option<ActiveQuest> {
    let (registry, template) = {
        let registry = world.get_resource::<TagRegistry>()?.clone();
        let templates = world.get_resource::<QuestTemplates>()?;
        let template = templates.templates.iter().find(|t| t.id == entry.template_id)?.clone();
        (registry, template)
    };

    let generated = generate_quests(std::slice::from_ref(&template), world, &registry);
    generated.into_iter().next()
}

pub fn check_quest_board_refresh(world: &mut World) {
    let needs_refresh = {
        let state = match world.get_resource::<QuestBoardState>() {
            Some(s) => s,
            None => return,
        };
        state.turn_count_since_refresh >= state.refresh_interval
    };

    if !needs_refresh {
        if let Some(mut state) = world.get_resource_mut::<QuestBoardState>() {
            state.turn_count_since_refresh += 1;
        }
        return;
    }

    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let templates = match world.get_resource::<QuestTemplates>() {
        Some(t) => t.templates.clone(),
        None => return,
    };

    let new_quests = generate_board_quests(&templates, world, &registry, 4);

    if let Some(mut state) = world.get_resource_mut::<QuestBoardState>() {
        state.available_quests = new_quests;
        state.turn_count_since_refresh = 0;
    }
}

pub fn check_player_near_quest_board(world: &mut World) -> bool {
    let player_pos = {
        let mut pq = world.query_filtered::<&crate::Position, bevy_ecs::query::With<crate::Player>>();
        match pq.single(world) {
            Ok(p) => (p.x, p.y),
            Err(_) => return false,
        }
    };

    let offsets: [(i32, i32); 9] = [
        (0, 0), (0, -1), (0, 1), (-1, 0), (1, 0),
        (-1, -1), (-1, 1), (1, -1), (1, 1),
    ];

    let mut board_query = world.query::<(&QuestBoard, &crate::Position)>();
    for (_, pos) in board_query.iter(world) {
        let dx = pos.x as i32 - player_pos.0 as i32;
        let dy = pos.y as i32 - player_pos.1 as i32;
        if offsets.contains(&(dx, dy)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Health, Player};

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    fn setup_registry() -> TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).expect("tags")
    }

    #[test]
    fn kill_objective_progress() {
        let obj = QuestObjective::Kill {
            target_name: "Target".to_string(),
            killed: 1,
            required: 3,
        };
        assert_eq!(obj.progress_text(), "1/3 Target");
        assert!(!obj.is_complete());
    }

    #[test]
    fn kill_objective_complete() {
        let obj = QuestObjective::Kill {
            target_name: "Target".to_string(),
            killed: 3,
            required: 3,
        };
        assert!(obj.is_complete());
    }

    #[test]
    fn collect_objective_progress() {
        let obj = QuestObjective::Collect {
            tag_name: "ORE IRON".to_string(),
            collected: 0,
            required: 3,
        };
        assert_eq!(obj.progress_text(), "0/3 ORE IRON");
        assert!(!obj.is_complete());
    }

    #[test]
    fn reach_objective_not_reached() {
        let obj = QuestObjective::Reach {
            biome: "forest".to_string(),
            reached: false,
        };
        assert_eq!(obj.progress_text(), "Find forest");
        assert!(!obj.is_complete());
    }

    #[test]
    fn reach_objective_reached() {
        let obj = QuestObjective::Reach {
            biome: "forest".to_string(),
            reached: true,
        };
        assert_eq!(obj.progress_text(), "Reached forest");
        assert!(obj.is_complete());
    }

    #[test]
    fn kill_in_area_objective() {
        let obj = QuestObjective::KillInArea {
            biome: "swamp".to_string(),
            killed: 2,
            required: 3,
        };
        assert_eq!(obj.progress_text(), "2/3 in swamp");
        assert!(!obj.is_complete());
    }

    #[test]
    fn quest_log_records_turn_in() {
        let mut log = QuestLog::new();
        log.record_turn_in("Hunt Complete".to_string());
        log.record_turn_in("Gather ORE IRON".to_string());
        assert_eq!(log.turned_in.len(), 2);
        assert_eq!(log.turned_in[0], "Hunt Complete");
    }

    #[test]
    fn quest_state_variants() {
        assert_eq!(QuestState::Active, QuestState::Active);
        assert_ne!(QuestState::Active, QuestState::Complete);
    }

    #[test]
    fn load_quest_templates_from_toml() {
        let toml = r#"
[[quest_template]]
id = "hunt_creature"
name_pattern = "Hunt the {target}"
description_pattern = "Slay the {target}."
objective_type = "kill"
target_tags = ["BEAST", "AGGRESSIVE"]
reward_tags = ["HERB_MEDICINAL", "COMMON"]
required_count = 1

[[quest_template]]
id = "gather_resource"
name_pattern = "Gather {resource}"
description_pattern = "Collect {resource}."
objective_type = "collect"
target_tags = ["ORE_IRON"]
required_count = 3
reward_tags = ["METAL", "UNCOMMON"]
"#;
        let templates = load_quest_templates(toml).unwrap();
        assert_eq!(templates.len(), 2);
        assert_eq!(templates[0].id, "hunt_creature");
        assert_eq!(templates[0].objective_type, "kill");
        assert_eq!(templates[0].target_tags, vec!["BEAST", "AGGRESSIVE"]);
        assert_eq!(templates[0].required_count, 1);
        assert_eq!(templates[1].id, "gather_resource");
        assert_eq!(templates[1].required_count, 3);
    }

    #[test]
    fn load_actual_quests_toml() {
        let quests_toml = include_str!("../../../assets/config/quests.toml");
        let templates = load_quest_templates(quests_toml).unwrap();
        assert!(templates.len() >= 4, "should have at least 4 templates");
        assert!(templates.iter().any(|t| t.id == "hunt_creature"));
        assert!(templates.iter().any(|t| t.id == "gather_resource"));
        assert!(templates.iter().any(|t| t.id == "clear_area"));
        assert!(templates.iter().any(|t| t.id == "explore_biome"));
    }

    #[test]
    fn generate_kill_quest_from_world() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let beast_id = registry.tag_id("BEAST").unwrap();
        let aggr_id = registry.tag_id("AGGRESSIVE").unwrap();

        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(beast_id, TagValue::None, &registry);
        tags.add_tag(aggr_id, TagValue::None, &registry);

        world.spawn((
            crate::Creature,
            Name("Target".to_string()),
            tags,
            Position { x: 10, y: 10, z: 0 },
            Health { current: 30, max: 30 },
        ));

        let templates = vec![QuestTemplate {
            id: "hunt_creature".to_string(),
            name_pattern: "Hunt the {target}".to_string(),
            description_pattern: "Slay the {target}.".to_string(),
            objective_type: "kill".to_string(),
            target_tags: vec!["BEAST".to_string(), "AGGRESSIVE".to_string()],
            reward_tags: vec!["HERB_MEDICINAL".to_string(), "COMMON".to_string()],
            biome_tags: vec![],
            required_count: 1,
            kill_count: 3,
        }];

        let quests = generate_quests(&templates, &mut world, &registry);
        assert_eq!(quests.len(), 1);
        assert_eq!(quests[0].name, "Hunt the Target");
        assert!(matches!(quests[0].objective, QuestObjective::Kill { ref target_name, .. } if target_name == "Target"));
    }

    #[test]
    fn generate_collect_quest() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let templates = vec![QuestTemplate {
            id: "gather_resource".to_string(),
            name_pattern: "Gather {resource}".to_string(),
            description_pattern: "Collect {resource}.".to_string(),
            objective_type: "collect".to_string(),
            target_tags: vec!["ORE_IRON".to_string()],
            reward_tags: vec!["METAL".to_string()],
            biome_tags: vec![],
            required_count: 3,
            kill_count: 3,
        }];

        let quests = generate_quests(&templates, &mut world, &registry);
        assert_eq!(quests.len(), 1);
        assert!(quests[0].name.to_lowercase().contains("ore iron"));
        assert!(matches!(quests[0].objective, QuestObjective::Collect { required: 3, .. }));
    }

    #[test]
    fn generate_reach_quest() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let templates = vec![QuestTemplate {
            id: "explore_biome".to_string(),
            name_pattern: "Explore the {biome}".to_string(),
            description_pattern: "Journey to the {biome}.".to_string(),
            objective_type: "reach".to_string(),
            target_tags: vec![],
            reward_tags: vec!["EQUIP_WEAPON".to_string()],
            biome_tags: vec!["BIOME_TEMPERATE_FOREST".to_string()],
            required_count: 1,
            kill_count: 3,
        }];

        let quests = generate_quests(&templates, &mut world, &registry);
        assert_eq!(quests.len(), 1);
        assert!(quests[0].name.contains("temperate forest"));
        assert!(matches!(quests[0].objective, QuestObjective::Reach { .. }));
    }

    #[test]
    fn generate_does_not_duplicate_active_quests() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "quest_1".to_string(),
            template_id: "hunt_creature".to_string(),
            name: "Hunt the Target".to_string(),
            description: "Slay it.".to_string(),
            objective: QuestObjective::Kill {
                target_name: "Target".to_string(),
                killed: 0,
                required: 1,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        let beast_id = registry.tag_id("BEAST").unwrap();
        let aggr_id = registry.tag_id("AGGRESSIVE").unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(beast_id, TagValue::None, &registry);
        tags.add_tag(aggr_id, TagValue::None, &registry);
        world.spawn((crate::Creature, Name("Target".to_string()), tags, Position { x: 10, y: 10, z: 0 }, Health { current: 30, max: 30 }));

        let templates = vec![QuestTemplate {
            id: "hunt_creature".to_string(),
            name_pattern: "Hunt the {target}".to_string(),
            description_pattern: "Slay it.".to_string(),
            objective_type: "kill".to_string(),
            target_tags: vec!["BEAST".to_string(), "AGGRESSIVE".to_string()],
            reward_tags: vec![],
            biome_tags: vec![],
            required_count: 1,
            kill_count: 3,
        }];

        let quests = generate_quests(&templates, &mut world, &registry);
        assert!(quests.is_empty(), "should not generate duplicate quest for active template");
    }

    #[test]
    fn check_quest_completion_transitions_state() {
        let mut quest = ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "test".to_string(),
            name: "Test".to_string(),
            description: "Desc".to_string(),
            objective: QuestObjective::Kill {
                target_name: "Target".to_string(),
                killed: 1,
                required: 1,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        };

        assert!(check_quest_completion(&mut quest));
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn check_quest_completion_not_done() {
        let mut quest = ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "test".to_string(),
            name: "Test".to_string(),
            description: "Desc".to_string(),
            objective: QuestObjective::Kill {
                target_name: "Target".to_string(),
                killed: 0,
                required: 1,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        };

        assert!(!check_quest_completion(&mut quest));
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn track_kill_updates_quest() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "hunt".to_string(),
            name: "Hunt the Target".to_string(),
            description: "Slay it.".to_string(),
            objective: QuestObjective::Kill {
                target_name: "Target".to_string(),
                killed: 0,
                required: 1,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_kill(&mut world, "Target");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::Kill { killed: 1, .. }));
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn track_kill_ignores_wrong_target() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "hunt".to_string(),
            name: "Hunt the Target".to_string(),
            description: "Slay it.".to_string(),
            objective: QuestObjective::Kill {
                target_name: "Target".to_string(),
                killed: 0,
                required: 1,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_kill(&mut world, "Large Predator");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::Kill { killed: 0, .. }));
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn track_collect_updates_quest() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "gather".to_string(),
            name: "Gather ore iron".to_string(),
            description: "Get some.".to_string(),
            objective: QuestObjective::Collect {
                tag_name: "ORE IRON".to_string(),
                collected: 0,
                required: 3,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_collect(&mut world, "ORE_IRON");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::Collect { collected: 1, .. }));
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn track_reach_updates_quest() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "explore".to_string(),
            name: "Explore the forest".to_string(),
            description: "Go there.".to_string(),
            objective: QuestObjective::Reach {
                biome: "temperate forest".to_string(),
                reached: false,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_reach(&mut world, "temperate forest");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::Reach { reached: true, .. }));
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn turn_in_quest_generates_rewards() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            Inventory { items: vec![], capacity: 20 },
            crate::Equipment::default(),
        )).id();

        let reward_tags = vec!["METAL".to_string(), "UNCOMMON".to_string()];
        let rewards = turn_in_quest(&reward_tags, &mut world, &registry);

        assert!(!rewards.is_empty());

        let inv = world.get::<Inventory>(player).unwrap();
        assert_eq!(inv.items.len(), 1, "reward should be in player inventory");
    }

    #[test]
    fn turn_in_quest_empty_tags_no_reward() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let rewards = turn_in_quest(&[], &mut world, &registry);
        assert!(rewards.is_empty());
    }

    #[test]
    fn track_kill_area_updates_matching_quest() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "clear_area".to_string(),
            name: "Clear the swamp".to_string(),
            description: "Clear them out.".to_string(),
            objective: QuestObjective::KillInArea {
                biome: "swamp".to_string(),
                killed: 0,
                required: 3,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_kill_area(&mut world, "SWAMP");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::KillInArea { killed: 1, .. }));
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn track_kill_area_completes_quest() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "clear_area".to_string(),
            name: "Clear the swamp".to_string(),
            description: "Clear them out.".to_string(),
            objective: QuestObjective::KillInArea {
                biome: "swamp".to_string(),
                killed: 2,
                required: 3,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_kill_area(&mut world, "swamp");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::KillInArea { killed: 3, .. }));
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn track_kill_area_ignores_wrong_biome() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "clear_area".to_string(),
            name: "Clear the swamp".to_string(),
            description: "Clear them out.".to_string(),
            objective: QuestObjective::KillInArea {
                biome: "swamp".to_string(),
                killed: 0,
                required: 3,
            },
            state: QuestState::Active,
            reward_tags: vec![],
        });

        track_kill_area(&mut world, "desert");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::KillInArea { killed: 0, .. }));
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn track_kill_area_does_not_exceed_required() {
        let mut world = World::new();
        let registry = setup_registry();
        world.insert_resource(registry.clone());

        world.spawn(ActiveQuest {
            quest_id: "q1".to_string(),
            template_id: "clear_area".to_string(),
            name: "Clear the swamp".to_string(),
            description: "Clear them out.".to_string(),
            objective: QuestObjective::KillInArea {
                biome: "swamp".to_string(),
                killed: 3,
                required: 3,
            },
            state: QuestState::Complete,
            reward_tags: vec![],
        });

        track_kill_area(&mut world, "swamp");

        let mut qq = world.query::<&ActiveQuest>();
        let quest = qq.single(&world).unwrap();
        assert!(matches!(&quest.objective, QuestObjective::KillInArea { killed: 3, .. }));
    }

    #[test]
    fn generate_kill_area_quest() {
        let registry = setup_registry();
        let mut world = World::new();
        world.insert_resource(registry.clone());

        let templates = vec![QuestTemplate {
            id: "clear_area".to_string(),
            name_pattern: "Clear the {biome}".to_string(),
            description_pattern: "Clear them out of the {biome}.".to_string(),
            objective_type: "kill_area".to_string(),
            target_tags: vec!["UNDEAD".to_string()],
            reward_tags: vec!["EQUIP_ARMOR".to_string()],
            biome_tags: vec!["BIOME_SWAMP".to_string()],
            required_count: 1,
            kill_count: 3,
        }];

        let quests = generate_quests(&templates, &mut world, &registry);
        assert_eq!(quests.len(), 1);
        assert!(quests[0].name.contains("swamp"));
        assert!(matches!(quests[0].objective, QuestObjective::KillInArea { killed: 0, required: 3, .. }));
    }
}
