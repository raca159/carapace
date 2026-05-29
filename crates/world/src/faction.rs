use std::collections::HashMap;

use bevy_ecs::prelude::{Component, Resource};
use serde::Deserialize;

pub const PLAYER_FACTION_ID: FactionId = FactionId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FactionId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct Faction {
    pub faction_id: FactionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FactionStanding {
    Ally,
    #[default]
    Neutral,
    Hostile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactionDef {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipDef {
    pub faction_a: String,
    pub faction_b: String,
    pub standing: FactionStanding,
}

#[derive(Debug, Clone, Deserialize)]
struct FactionsFile {
    #[serde(rename = "faction")]
    factions: Vec<FactionDef>,
    #[serde(rename = "relationship")]
    relationships: Vec<RelationshipDef>,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct FactionRelationships {
    map: HashMap<(FactionId, FactionId), FactionStanding>,
    name_to_id: HashMap<String, FactionId>,
}

impl FactionRelationships {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_standing(&self, a: FactionId, b: FactionId) -> FactionStanding {
        if a == b {
            return FactionStanding::Ally;
        }
        let key = if a.0 < b.0 { (a, b) } else { (b, a) };
        self.map.get(&key).copied().unwrap_or(FactionStanding::Neutral)
    }

    pub fn faction_id(&self, name: &str) -> Option<FactionId> {
        self.name_to_id.get(name).copied()
    }

    fn insert(&mut self, a: FactionId, b: FactionId, standing: FactionStanding) {
        let key = if a.0 < b.0 { (a, b) } else { (b, a) };
        self.map.insert(key, standing);
    }

    pub fn name_id_pairs(&self) -> impl Iterator<Item = (&String, &FactionId)> {
        self.name_to_id.iter()
    }

    pub fn faction_name(&self, id: FactionId) -> Option<String> {
        self.name_to_id
            .iter()
            .find(|(_, fid)| **fid == id)
            .map(|(name, _)| name.clone())
    }
}

pub fn load_factions(
    toml: &str,
) -> Result<(Vec<FactionDef>, FactionRelationships), toml::de::Error> {
    let file: FactionsFile = toml::from_str(toml)?;

    let mut name_to_id = HashMap::new();
    name_to_id.insert("player".to_string(), PLAYER_FACTION_ID);
    for (i, faction) in file.factions.iter().enumerate() {
        name_to_id.insert(faction.id.clone(), FactionId((i + 1) as u8));
    }

    let mut relationships = FactionRelationships {
        name_to_id,
        ..Default::default()
    };

    for rel in &file.relationships {
        let a = relationships
            .name_to_id
            .get(&rel.faction_a)
            .copied()
            .unwrap_or_else(|| panic!("unknown faction: {}", rel.faction_a));
        let b = relationships
            .name_to_id
            .get(&rel.faction_b)
            .copied()
            .unwrap_or_else(|| panic!("unknown faction: {}", rel.faction_b));
        relationships.insert(a, b, rel.standing);
    }

    Ok((file.factions, relationships))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FACTIONS_TOML: &str = r#"
[[faction]]
id = "great_carapace"

[[faction]]
id = "sanguine_elite"

[[faction]]
id = "familiars"

[[faction]]
id = "free_humanity"

[[faction]]
id = "the_remnant"

[[relationship]]
faction_a = "great_carapace"
faction_b = "free_humanity"
standing = "hostile"

[[relationship]]
faction_a = "sanguine_elite"
faction_b = "familiars"
standing = "ally"

[[relationship]]
faction_a = "sanguine_elite"
faction_b = "free_humanity"
standing = "hostile"

[[relationship]]
faction_a = "free_humanity"
faction_b = "the_remnant"
standing = "ally"
"#;

    #[test]
    fn test_load_factions_count() {
        let (factions, _) = load_factions(FACTIONS_TOML).unwrap();
        assert_eq!(factions.len(), 5);
    }

    #[test]
    fn test_faction_ids_assigned_sequentially() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        assert_eq!(rels.faction_id("great_carapace"), Some(FactionId(1)));
        assert_eq!(rels.faction_id("sanguine_elite"), Some(FactionId(2)));
        assert_eq!(rels.faction_id("familiars"), Some(FactionId(3)));
        assert_eq!(rels.faction_id("free_humanity"), Some(FactionId(4)));
        assert_eq!(rels.faction_id("the_remnant"), Some(FactionId(5)));
    }

    #[test]
    fn test_unknown_faction_returns_none() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        assert_eq!(rels.faction_id("nonexistent"), None);
    }

    #[test]
    fn test_get_standing_hostile() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        let carapace = rels.faction_id("great_carapace").unwrap();
        let humanity = rels.faction_id("free_humanity").unwrap();
        assert_eq!(rels.get_standing(carapace, humanity), FactionStanding::Hostile);
        assert_eq!(rels.get_standing(humanity, carapace), FactionStanding::Hostile);
    }

    #[test]
    fn test_get_standing_ally() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        let elite = rels.faction_id("sanguine_elite").unwrap();
        let fam = rels.faction_id("familiars").unwrap();
        assert_eq!(rels.get_standing(elite, fam), FactionStanding::Ally);
        assert_eq!(rels.get_standing(fam, elite), FactionStanding::Ally);
    }

    #[test]
    fn test_get_standing_default_neutral() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        let carapace = rels.faction_id("great_carapace").unwrap();
        let fam = rels.faction_id("familiars").unwrap();
        assert_eq!(rels.get_standing(carapace, fam), FactionStanding::Neutral);
    }

    #[test]
    fn test_get_standing_same_faction_ally() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        let carapace = rels.faction_id("great_carapace").unwrap();
        assert_eq!(rels.get_standing(carapace, carapace), FactionStanding::Ally);
    }

    #[test]
    fn test_player_faction_constant() {
        assert_eq!(PLAYER_FACTION_ID, FactionId(0));
    }

    #[test]
    fn test_player_standing_neutral_by_default() {
        let (_, rels) = load_factions(FACTIONS_TOML).unwrap();
        let carapace = rels.faction_id("great_carapace").unwrap();
        assert_eq!(
            rels.get_standing(PLAYER_FACTION_ID, carapace),
            FactionStanding::Neutral
        );
    }

    #[test]
    fn test_explicit_player_hostile_relationship() {
        let toml = r#"
[[faction]]
id = "sanguine_elite"

[[relationship]]
faction_a = "sanguine_elite"
faction_b = "player"
standing = "hostile"
"#;
        let (_, rels) = load_factions(toml).unwrap();
        let elite = rels.faction_id("sanguine_elite").unwrap();
        assert_eq!(
            rels.get_standing(PLAYER_FACTION_ID, elite),
            FactionStanding::Hostile
        );
    }

    #[test]
    fn test_symmetric_lookup_arbitrary_ids() {
        let mut rels = FactionRelationships::new();
        let a = FactionId(10);
        let b = FactionId(20);
        rels.insert(a, b, FactionStanding::Hostile);
        assert_eq!(rels.get_standing(a, b), FactionStanding::Hostile);
        assert_eq!(rels.get_standing(b, a), FactionStanding::Hostile);
    }
}

pub const REP_KILL_PENALTY: i32 = -50;
pub const REP_ATTACK_PENALTY: i32 = -10;
pub const REP_QUEST_REWARD: i32 = 25;
pub const REP_THRESHOLD_HOSTILE: i32 = -100;
pub const REP_THRESHOLD_ALLY: i32 = 100;

#[derive(Debug, Clone, Default, Resource)]
pub struct ReputationTracker {
    standings: HashMap<(FactionId, FactionId), i32>,
    entity_faction: HashMap<bevy_ecs::entity::Entity, FactionId>,
}

impl ReputationTracker {
    pub fn new() -> Self { Self::default() }

    pub fn set_entity_faction(&mut self, entity: bevy_ecs::entity::Entity, faction: FactionId) {
        self.entity_faction.insert(entity, faction);
    }

    pub fn kill_entity(&mut self, killer_faction: FactionId, victim: bevy_ecs::entity::Entity) {
        if let Some(&victim_faction) = self.entity_faction.get(&victim) {
            let current = self.standings.get(&(killer_faction, victim_faction)).copied().unwrap_or(0);
            self.standings.insert((killer_faction, victim_faction), current + REP_KILL_PENALTY);
        }
    }

    pub fn modify_standing(&mut self, faction_a: FactionId, faction_b: FactionId, delta: i32) {
        let key = if faction_a.0 < faction_b.0 { (faction_a, faction_b) } else { (faction_b, faction_a) };
        let current = self.standings.get(&key).copied().unwrap_or(0);
        self.standings.insert(key, current + delta);
    }

    pub fn get_standing(&self, faction_a: FactionId, faction_b: FactionId) -> FactionStanding {
        let key = if faction_a.0 < faction_b.0 { (faction_a, faction_b) } else { (faction_b, faction_a) };
        match self.standings.get(&key).copied().unwrap_or(0) {
            v if v < REP_THRESHOLD_HOSTILE => FactionStanding::Hostile,
            v if v > REP_THRESHOLD_ALLY => FactionStanding::Ally,
            _ => FactionStanding::Neutral,
        }
    }

    pub fn get_current_value(&self, faction_a: FactionId, faction_b: FactionId) -> i32 {
        let key = if faction_a.0 < faction_b.0 { (faction_a, faction_b) } else { (faction_b, faction_a) };
        self.standings.get(&key).copied().unwrap_or(0)
    }

    pub fn all(&self) -> impl Iterator<Item = (&(FactionId, FactionId), &i32)> {
        self.standings.iter()
    }

    pub fn modify(&mut self, faction_a: FactionId, faction_b: FactionId, delta: i32) {
        self.modify_standing(faction_a, faction_b, delta)
    }
}
