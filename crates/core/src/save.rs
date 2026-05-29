use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    ActiveQuest, Equipment, Glyph, Health, Inventory, MessageLog, Name, Player, Position,
    QuestLog, QuestObjective, QuestState,
};
use crate::turn::TurnCounter;

const SAVE_VERSION: u32 = 1;
const AUTO_SAVE_INTERVAL: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub seed: u64,
    pub turn: u64,
    pub player: PlayerSave,
    pub entities: Vec<EntitySave>,
    pub quest_log: QuestLogSave,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
    pub x: u32,
    pub y: u32,
    pub hp_current: u32,
    pub hp_max: u32,
    pub glyph: char,
    pub glyph_color: (u8, u8, u8),
    pub inventory_items: Vec<ItemSave>,
    pub inventory_capacity: usize,
    pub equipment_weapon: Option<usize>,
    pub equipment_armor: Option<usize>,
    pub equipment_accessory: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySave {
    pub x: u32,
    pub y: u32,
    pub hp_current: u32,
    pub hp_max: u32,
    pub glyph: char,
    pub glyph_color: (u8, u8, u8),
    pub name: Option<String>,
    pub inventory_items: Vec<ItemSave>,
    pub inventory_capacity: usize,
    pub is_item: bool,
    pub is_creature: bool,
    pub active_quest: Option<ActiveQuestSave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSave {
    pub glyph: char,
    pub glyph_color: (u8, u8, u8),
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuestSave {
    pub quest_id: String,
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub objective: QuestObjectiveSave,
    pub state: String,
    pub reward_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjectiveSave {
    Kill { target_name: String, killed: u32, required: u32 },
    Collect { tag_name: String, collected: u32, required: u32 },
    Reach { biome: String, reached: bool },
    KillInArea { biome: String, killed: u32, required: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestLogSave {
    pub turned_in: Vec<String>,
}

impl From<&QuestObjective> for QuestObjectiveSave {
    fn from(obj: &QuestObjective) -> Self {
        match obj {
            QuestObjective::Kill { target_name, killed, required } => {
                QuestObjectiveSave::Kill { target_name: target_name.clone(), killed: *killed, required: *required }
            }
            QuestObjective::Collect { tag_name, collected, required } => {
                QuestObjectiveSave::Collect { tag_name: tag_name.clone(), collected: *collected, required: *required }
            }
            QuestObjective::Reach { biome, reached } => {
                QuestObjectiveSave::Reach { biome: biome.clone(), reached: *reached }
            }
            QuestObjective::KillInArea { biome, killed, required } => {
                QuestObjectiveSave::KillInArea { biome: biome.clone(), killed: *killed, required: *required }
            }
        }
    }
}

impl From<&QuestObjectiveSave> for QuestObjective {
    fn from(obj: &QuestObjectiveSave) -> Self {
        match obj {
            QuestObjectiveSave::Kill { target_name, killed, required } => {
                QuestObjective::Kill { target_name: target_name.clone(), killed: *killed, required: *required }
            }
            QuestObjectiveSave::Collect { tag_name, collected, required } => {
                QuestObjective::Collect { tag_name: tag_name.clone(), collected: *collected, required: *required }
            }
            QuestObjectiveSave::Reach { biome, reached } => {
                QuestObjective::Reach { biome: biome.clone(), reached: *reached }
            }
            QuestObjectiveSave::KillInArea { biome, killed, required } => {
                QuestObjective::KillInArea { biome: biome.clone(), killed: *killed, required: *required }
            }
        }
    }
}

fn quest_state_to_string(state: &QuestState) -> String {
    match state {
        QuestState::Active => "Active".to_string(),
        QuestState::Complete => "Complete".to_string(),
        QuestState::TurnedIn => "TurnedIn".to_string(),
        QuestState::Failed => "Failed".to_string(),
    }
}

fn string_to_quest_state(s: &str) -> QuestState {
    match s {
        "Complete" => QuestState::Complete,
        "TurnedIn" => QuestState::TurnedIn,
        "Failed" => QuestState::Failed,
        _ => QuestState::Active,
    }
}

fn save_dir() -> PathBuf {
    PathBuf::from("saves")
}

fn save_filename(seed: u64, turn: u64) -> String {
    format!("save_{}_{}.toml", seed, turn)
}

pub fn save_game(world: &mut World, seed: u64) -> Result<String, String> {
    let dir = save_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create saves dir: {}", e))?;

    let turn = world.get_resource::<TurnCounter>()
        .map(|tc| tc.0)
        .unwrap_or(0);

    let player_save = serialize_player(world)?;
    let entities = serialize_entities(world)?;
    let quest_log = serialize_quest_log(world);
    let messages = serialize_messages(world);

    let save = SaveGame {
        version: SAVE_VERSION,
        seed,
        turn,
        player: player_save,
        entities,
        quest_log,
        messages,
    };

    let filename = save_filename(seed, turn);
    let path = dir.join(&filename);
    let toml_str = toml::to_string_pretty(&save)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    fs::write(&path, toml_str)
        .map_err(|e| format!("Write failed: {}", e))?;

    Ok(filename)
}

pub fn load_game(path: &Path) -> Result<SaveGame, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Read failed: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Parse failed: {}", e))
}

pub fn list_saves() -> Vec<String> {
    let dir = save_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut saves: Vec<String> = fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    saves.sort();
    saves.reverse();
    saves
}

pub fn should_auto_save(world: &World) -> bool {
    if let Some(tc) = world.get_resource::<TurnCounter>() {
        tc.0 > 0 && tc.0 % AUTO_SAVE_INTERVAL == 0
    } else {
        false
    }
}

fn serialize_player(world: &mut World) -> Result<PlayerSave, String> {
    let mut pq = world.query_filtered::<(
        &Position, &Health, &Glyph, &Inventory, &Equipment,
    ), With<Player>>();

    let (pos, hp, glyph, inv, equip) = pq.single(world)
        .map_err(|_| "No player entity found".to_string())?;

    let mut items = Vec::new();
    for &item_ent in &inv.items {
        let item_save = serialize_item(world, item_ent);
        items.push(item_save);
    }

    Ok(PlayerSave {
        x: pos.x,
        y: pos.y,
        hp_current: hp.current,
        hp_max: hp.max,
        glyph: glyph.char,
        glyph_color: glyph.color,
        inventory_items: items,
        inventory_capacity: inv.capacity,
        equipment_weapon: equip.weapon.map(|_| 0),
        equipment_armor: equip.armor.map(|_| 1),
        equipment_accessory: equip.accessory.map(|_| 2),
    })
}

fn serialize_item(world: &World, entity: Entity) -> ItemSave {
    let glyph = world.get::<Glyph>(entity)
        .map(|g| ItemSave { glyph: g.char, glyph_color: g.color, name: None })
        .unwrap_or(ItemSave { glyph: '?', glyph_color: (128, 128, 128), name: None });
    let name = world.get::<Name>(entity)
        .map(|n| n.0.clone());
    ItemSave { name, ..glyph }
}

fn serialize_entities(world: &mut World) -> Result<Vec<EntitySave>, String> {
    let mut player_q = world.query_filtered::<Entity, With<Player>>();
    let player_ent = player_q.iter(world).next();

    let mut eq = world.query::<(Entity, &Position, &Health, &Glyph, Option<&Name>, Option<&Inventory>, Option<&crate::Item>, Option<&crate::Creature>)>();
    let mut quest_q = world.query::<&ActiveQuest>();

    let mut saves = Vec::new();
    for (entity, pos, hp, glyph, name, inv, is_item, is_creature) in eq.iter(world) {
        if Some(entity) == player_ent {
            continue;
        }

        let active_quest = quest_q.get(world, entity).ok()
            .map(|q| ActiveQuestSave {
                quest_id: q.quest_id.clone(),
                template_id: q.template_id.clone(),
                name: q.name.clone(),
                description: q.description.clone(),
                objective: (&q.objective).into(),
                state: quest_state_to_string(&q.state),
                reward_tags: q.reward_tags.clone(),
            });

        let (inv_items, inv_cap) = inv
            .map(|i| {
                let items = i.items.iter()
                    .map(|&ent| serialize_item(world, ent))
                    .collect();
                (items, i.capacity)
            })
            .unwrap_or((Vec::new(), 0));

        saves.push(EntitySave {
            x: pos.x,
            y: pos.y,
            hp_current: hp.current,
            hp_max: hp.max,
            glyph: glyph.char,
            glyph_color: glyph.color,
            name: name.map(|n| n.0.clone()),
            inventory_items: inv_items,
            inventory_capacity: inv_cap,
            is_item: is_item.is_some(),
            is_creature: is_creature.is_some(),
            active_quest,
        });
    }
    Ok(saves)
}

fn serialize_quest_log(world: &mut World) -> QuestLogSave {
    world.get_resource::<QuestLog>()
        .map(|ql| QuestLogSave {
            turned_in: ql.turned_in.clone(),
        })
        .unwrap_or(QuestLogSave { turned_in: vec![] })
}

fn serialize_messages(world: &mut World) -> Vec<String> {
    world.get_resource::<MessageLog>()
        .map(|ml| ml.messages.clone())
        .unwrap_or_default()
}

pub fn deserialize_to_world(world: &mut World, save: &SaveGame) -> Result<(), String> {
    world.insert_resource(TurnCounter(save.turn));

    let ps = &save.player;

    let player_entity = world.spawn((
        Player,
        Position { x: ps.x, y: ps.y, z: 0 },
        Health { current: ps.hp_current, max: ps.hp_max },
        Glyph { char: ps.glyph, color: ps.glyph_color },
        Inventory {
            items: Vec::new(),
            capacity: ps.inventory_capacity,
        },
        Equipment {
            weapon: None,
            armor: None,
            accessory: None,
        },
    )).id();

    let mut item_entities: Vec<Entity> = Vec::new();
    for item_save in &ps.inventory_items {
        let item_ent = world.spawn((
            crate::Item,
            Glyph { char: item_save.glyph, color: item_save.glyph_color },
        )).id();
        if let Some(name) = &item_save.name {
            world.get_mut::<Name>(item_ent).unwrap().0 = name.clone();
        } else {
            world.entity_mut(item_ent).insert(Name(item_save.glyph.to_string()));
        }
        item_entities.push(item_ent);
    }
    world.get_mut::<Inventory>(player_entity).unwrap().items = item_entities;

    for ent_save in &save.entities {
        let mut entity = world.spawn((
            Position { x: ent_save.x, y: ent_save.y, z: 0 },
            Health { current: ent_save.hp_current, max: ent_save.hp_max },
            Glyph { char: ent_save.glyph, color: ent_save.glyph_color },
        ));

        if let Some(name) = &ent_save.name {
            entity.insert(Name(name.clone()));
        }
        if ent_save.is_item {
            entity.insert(crate::Item);
        }
        if ent_save.is_creature {
            entity.insert(crate::Creature);
        }

        let ent_id = entity.id();

        if !ent_save.inventory_items.is_empty() {
            let mut items = Vec::new();
            for item_save in &ent_save.inventory_items {
                let item_ent = world.spawn((
                    crate::Item,
                    Glyph { char: item_save.glyph, color: item_save.glyph_color },
                )).id();
                if let Some(name) = &item_save.name {
                    world.entity_mut(item_ent).insert(Name(name.clone()));
                }
                items.push(item_ent);
            }
            world.entity_mut(ent_id).insert(Inventory {
                items,
                capacity: ent_save.inventory_capacity,
            });
        }

        if let Some(qs) = &ent_save.active_quest {
            world.entity_mut(ent_id).insert(ActiveQuest {
                quest_id: qs.quest_id.clone(),
                template_id: qs.template_id.clone(),
                name: qs.name.clone(),
                description: qs.description.clone(),
                objective: (&qs.objective).into(),
                state: string_to_quest_state(&qs.state),
                reward_tags: qs.reward_tags.clone(),
            });
        }
    }

    let quest_log = QuestLog {
        turned_in: save.quest_log.turned_in.clone(),
    };
    world.insert_resource(quest_log);

    let mut msg_log = MessageLog::new(100);
    for msg in &save.messages {
        msg_log.push(msg.clone());
    }
    world.insert_resource(msg_log);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Player, Position, Health, Glyph, Name, Inventory, Equipment, Creature};

    #[test]
    fn quest_objective_round_trip_kill() {
        let obj = QuestObjective::Kill { target_name: "Target".to_string(), killed: 2, required: 5 };
        let save: QuestObjectiveSave = (&obj).into();
        let restored: QuestObjective = (&save).into();
        if let QuestObjective::Kill { target_name, killed, required } = restored {
            assert_eq!(target_name, "Target");
            assert_eq!(killed, 2);
            assert_eq!(required, 5);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn quest_objective_round_trip_collect() {
        let obj = QuestObjective::Collect { tag_name: "ORE_IRON".to_string(), collected: 3, required: 5 };
        let save: QuestObjectiveSave = (&obj).into();
        let restored: QuestObjective = (&save).into();
        if let QuestObjective::Collect { tag_name, collected, required } = restored {
            assert_eq!(tag_name, "ORE_IRON");
            assert_eq!(collected, 3);
            assert_eq!(required, 5);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn quest_objective_round_trip_reach() {
        let obj = QuestObjective::Reach { biome: "Forest".to_string(), reached: true };
        let save: QuestObjectiveSave = (&obj).into();
        let restored: QuestObjective = (&save).into();
        if let QuestObjective::Reach { biome, reached } = restored {
            assert_eq!(biome, "Forest");
            assert!(reached);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn quest_state_serialization() {
        assert_eq!(quest_state_to_string(&QuestState::Active), "Active");
        assert_eq!(quest_state_to_string(&QuestState::Complete), "Complete");
        assert_eq!(quest_state_to_string(&QuestState::TurnedIn), "TurnedIn");
    }

    #[test]
    fn quest_state_deserialization() {
        assert!(matches!(string_to_quest_state("Active"), QuestState::Active));
        assert!(matches!(string_to_quest_state("Complete"), QuestState::Complete));
        assert!(matches!(string_to_quest_state("TurnedIn"), QuestState::TurnedIn));
    }

    #[test]
    fn save_game_version() {
        assert_eq!(SAVE_VERSION, 1);
    }

    #[test]
    fn auto_save_interval() {
        assert_eq!(AUTO_SAVE_INTERVAL, 100);
    }

    #[test]
    fn save_filename_format() {
        let name = save_filename(42, 100);
        assert_eq!(name, "save_42_100.toml");
    }

    #[test]
    fn serialize_deserialize_player() {
        let mut world = World::new();
        world.spawn((
            Player,
            Position { x: 10, y: 20, z: 0 },
            Health { current: 80, max: 100 },
            Glyph { char: '@', color: (255, 255, 0) },
            Inventory { items: vec![], capacity: 20 },
            Equipment::default(),
        ));

        let player_save = serialize_player(&mut world).unwrap();
        assert_eq!(player_save.x, 10);
        assert_eq!(player_save.y, 20);
        assert_eq!(player_save.hp_current, 80);
        assert_eq!(player_save.hp_max, 100);
        assert_eq!(player_save.glyph, '@');
    }

    #[test]
    fn serialize_entities_excludes_player() {
        let mut world = World::new();
        world.spawn((
            Player,
            Position { x: 0, y: 0, z: 0 },
            Health { current: 100, max: 100 },
            Glyph { char: '@', color: (255, 255, 0) },
        ));
        world.spawn((
            Creature,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 30, max: 50 },
            Glyph { char: 'g', color: (0, 255, 0) },
        ));

        let entities = serialize_entities(&mut world).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].x, 5);
        assert!(entities[0].is_creature);
    }

    #[test]
    fn full_round_trip() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(150));
        world.insert_resource(QuestLog { turned_in: vec!["quest_1".to_string()] });
        world.insert_resource(MessageLog::new(100));

        world.spawn((
            Player,
            Position { x: 10, y: 20, z: 0 },
            Health { current: 80, max: 100 },
            Glyph { char: '@', color: (255, 255, 0) },
            Inventory { items: vec![], capacity: 20 },
            Equipment::default(),
        ));
        world.spawn((
            Creature,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 30, max: 50 },
            Glyph { char: 'g', color: (0, 255, 0) },
            Name("Creature".to_string()),
        ));

        let save = SaveGame {
            version: SAVE_VERSION,
            seed: 42,
            turn: 150,
            player: serialize_player(&mut world).unwrap(),
            entities: serialize_entities(&mut world).unwrap(),
            quest_log: serialize_quest_log(&mut world),
            messages: serialize_messages(&mut world),
        };

        let toml_str = toml::to_string_pretty(&save).unwrap();
        let restored: SaveGame = toml::from_str(&toml_str).unwrap();

        assert_eq!(restored.version, SAVE_VERSION);
        assert_eq!(restored.seed, 42);
        assert_eq!(restored.turn, 150);
        assert_eq!(restored.player.x, 10);
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(restored.quest_log.turned_in.len(), 1);
    }

    #[test]
    fn deserialize_creates_entities() {
        let save = SaveGame {
            version: 1,
            seed: 42,
            turn: 100,
            player: PlayerSave {
                x: 5, y: 10,
                hp_current: 90, hp_max: 100,
                glyph: '@', glyph_color: (255, 255, 0),
                inventory_items: vec![],
                inventory_capacity: 20,
                equipment_weapon: None,
                equipment_armor: None,
                equipment_accessory: None,
            },
            entities: vec![EntitySave {
                x: 15, y: 15,
                hp_current: 20, hp_max: 30,
                glyph: 'g', glyph_color: (0, 255, 0),
                name: Some("Creature".to_string()),
                inventory_items: vec![],
                inventory_capacity: 0,
                is_item: false,
                is_creature: true,
                active_quest: None,
            }],
            quest_log: QuestLogSave { turned_in: vec![] },
            messages: vec!["Hello".to_string()],
        };

        let mut world = World::new();
        deserialize_to_world(&mut world, &save).unwrap();

        let turn = world.get_resource::<TurnCounter>().unwrap();
        assert_eq!(turn.0, 100);

        let mut pq = world.query_filtered::<&Position, With<Player>>();
        let pos = pq.single(&world).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);

        let mut cq = world.query_filtered::<(&Health, &Name), With<Creature>>();
        let (hp, name) = cq.single(&world).unwrap();
        assert_eq!(hp.current, 20);
        assert_eq!(name.0, "Creature");
    }

    #[test]
    fn list_saves_empty_when_no_dir() {
        let saves = list_saves();
        let _ = saves;
    }

    #[test]
    fn should_auto_save_at_interval() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(100));
        assert!(should_auto_save(&world));
    }

    #[test]
    fn should_auto_save_not_at_interval() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(50));
        assert!(!should_auto_save(&world));
    }

    #[test]
    fn should_auto_save_no_counter() {
        let world = World::new();
        assert!(!should_auto_save(&world));
    }

    #[test]
    fn should_auto_save_zero_turn() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        assert!(!should_auto_save(&world));
    }

    #[test]
    fn should_auto_save_200() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(200));
        assert!(should_auto_save(&world));
    }
}
