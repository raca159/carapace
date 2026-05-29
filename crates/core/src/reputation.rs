use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone, Default)]
pub struct FactionReputation {
    pub standings: HashMap<String, i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReputationRank {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Honored,
    Exalted,
}

impl FactionReputation {
    pub fn new() -> Self {
        Self { standings: HashMap::new() }
    }

    pub fn get(&self, faction: &str) -> i32 {
        self.standings.get(faction).copied().unwrap_or(0)
    }

    pub fn set(&mut self, faction: &str, value: i32) {
        self.standings.insert(faction.to_string(), value);
    }

    pub fn modify(&mut self, faction: &str, delta: i32) -> i32 {
        let current = self.get(faction);
        let new = (current + delta).clamp(-10000, 10000);
        self.standings.insert(faction.to_string(), new);
        new
    }

    pub fn rank(&self, faction: &str) -> ReputationRank {
        match self.get(faction) {
            v if v <= -1500 => ReputationRank::Hostile,
            v if v <= -500 => ReputationRank::Unfriendly,
            v if v < 500 => ReputationRank::Neutral,
            v if v < 1500 => ReputationRank::Friendly,
            v if v < 3000 => ReputationRank::Honored,
            _ => ReputationRank::Exalted,
        }
    }

    pub fn rank_name(rank: ReputationRank) -> &'static str {
        match rank {
            ReputationRank::Hostile => "Hostile",
            ReputationRank::Unfriendly => "Unfriendly",
            ReputationRank::Neutral => "Neutral",
            ReputationRank::Friendly => "Friendly",
            ReputationRank::Honored => "Honored",
            ReputationRank::Exalted => "Exalted",
        }
    }

    pub fn standing_for_dialogue(&self, faction: &str) -> String {
        match self.rank(faction) {
            ReputationRank::Hostile => "hostile".to_string(),
            ReputationRank::Unfriendly => "unfriendly".to_string(),
            ReputationRank::Neutral => "neutral".to_string(),
            ReputationRank::Friendly => "friendly".to_string(),
            ReputationRank::Honored => "ally".to_string(),
            ReputationRank::Exalted => "ally".to_string(),
        }
    }

    pub fn is_hostile(&self, faction: &str) -> bool {
        matches!(self.rank(faction), ReputationRank::Hostile)
    }

    pub fn faction_names(&self) -> Vec<&String> {
        self.standings.keys().collect()
    }

    /// Sync from a ReputationTracker value (scale -100..100) with 100x scaling.
    pub fn sync_from_tracker(&mut self, faction_name: &str, tracker_value: i32) {
        let scaled = (tracker_value as i64 * 100).clamp(-10000, 10000) as i32;
        self.set(faction_name, scaled);
    }
}

impl std::fmt::Display for ReputationRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", FactionReputation::rank_name(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reputation_is_zero() {
        let rep = FactionReputation::new();
        assert_eq!(rep.get("merchants"), 0);
    }

    #[test]
    fn modify_changes_standing() {
        let mut rep = FactionReputation::new();
        rep.set("merchants", 100);
        let new = rep.modify("merchants", 50);
        assert_eq!(new, 150);
        assert_eq!(rep.get("merchants"), 150);
    }

    #[test]
    fn negative_standing_works() {
        let mut rep = FactionReputation::new();
        rep.set("sanguine_elite", -100);
        rep.modify("sanguine_elite", -50);
        assert_eq!(rep.get("sanguine_elite"), -150);
    }

    #[test]
    fn unknown_faction_returns_zero() {
        let rep = FactionReputation::new();
        assert_eq!(rep.get("nonexistent"), 0);
        assert_eq!(rep.rank("nonexistent"), ReputationRank::Neutral);
    }

    #[test]
    fn rank_thresholds() {
        let mut rep = FactionReputation::new();

        rep.set("f", -2000);
        assert_eq!(rep.rank("f"), ReputationRank::Hostile);

        rep.set("f", -1000);
        assert_eq!(rep.rank("f"), ReputationRank::Unfriendly);

        rep.set("f", -600);
        assert_eq!(rep.rank("f"), ReputationRank::Unfriendly);

        rep.set("f", -400);
        assert_eq!(rep.rank("f"), ReputationRank::Neutral);

        rep.set("f", 0);
        assert_eq!(rep.rank("f"), ReputationRank::Neutral);

        rep.set("f", 400);
        assert_eq!(rep.rank("f"), ReputationRank::Neutral);

        rep.set("f", 600);
        assert_eq!(rep.rank("f"), ReputationRank::Friendly);

        rep.set("f", 2000);
        assert_eq!(rep.rank("f"), ReputationRank::Honored);

        rep.set("f", 5000);
        assert_eq!(rep.rank("f"), ReputationRank::Exalted);
    }

    #[test]
    fn standing_for_dialogue_maps_correctly() {
        let mut rep = FactionReputation::new();
        rep.set("guild", -2000);
        assert_eq!(rep.standing_for_dialogue("guild"), "hostile");

        rep.set("guild", 2000);
        assert_eq!(rep.standing_for_dialogue("guild"), "ally");

        rep.set("guild", 0);
        assert_eq!(rep.standing_for_dialogue("guild"), "neutral");
    }

    #[test]
    fn is_hostile_check() {
        let mut rep = FactionReputation::new();
        rep.set("enemy", -2000);
        assert!(rep.is_hostile("enemy"));

        rep.set("enemy", -400);
        assert!(!rep.is_hostile("enemy"));
    }

    #[test]
    fn modifies_clamped() {
        let mut rep = FactionReputation::new();
        rep.set("x", 9999);
        rep.modify("x", 100);
        assert_eq!(rep.get("x"), 10000);

        rep.set("x", -9999);
        rep.modify("x", -100);
        assert_eq!(rep.get("x"), -10000);
    }

    #[test]
    fn reputation_rank_display() {
        assert_eq!(format!("{}", ReputationRank::Hostile), "Hostile");
        assert_eq!(format!("{}", ReputationRank::Neutral), "Neutral");
        assert_eq!(format!("{}", ReputationRank::Exalted), "Exalted");
    }

    #[test]
    fn faction_names_returns_known_factions() {
        let mut rep = FactionReputation::new();
        rep.set("guild_a", 100);
        rep.set("guild_b", -50);
        let names = rep.faction_names();
        assert_eq!(names.len(), 2);
        assert!(names.iter().any(|n| n.as_str() == "guild_a"));
        assert!(names.iter().any(|n| n.as_str() == "guild_b"));
    }

    #[test]
    fn reputation_rank_ordering() {
        assert!(ReputationRank::Hostile < ReputationRank::Neutral);
        assert!(ReputationRank::Friendly > ReputationRank::Unfriendly);
        assert_eq!(ReputationRank::Neutral, ReputationRank::Neutral);
    }
}
