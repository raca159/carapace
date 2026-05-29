use bevy_ecs::prelude::*;
use rand::Rng;

use game_tags::{TagRegistry, TagValue, Tags};

/// 10 personality traits scored 0-100
#[derive(Component, Debug, Clone, Default)]
pub struct PersonalityScores {
    pub aggression: u8,
    pub bravery: u8,
    pub sociability: u8,
    pub orderliness: u8,
    pub curiosity: u8,
    pub industriousness: u8,
    pub honesty: u8,
    pub spirituality: u8,
    pub gregariousness: u8,
    pub volatility: u8,
}

impl PersonalityScores {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        Self {
            aggression: rng.random_range(20..=80),
            bravery: rng.random_range(20..=80),
            sociability: rng.random_range(20..=80),
            orderliness: rng.random_range(20..=80),
            curiosity: rng.random_range(20..=80),
            industriousness: rng.random_range(20..=80),
            honesty: rng.random_range(20..=80),
            spirituality: rng.random_range(20..=80),
            gregariousness: rng.random_range(20..=80),
            volatility: rng.random_range(20..=80),
        }
    }
}

/// Derive behavioral tags from personality scores
pub fn tags_from_personality(
    scores: &PersonalityScores,
    tags: &mut Tags,
    registry: &TagRegistry,
) {
    if let Some(id) = registry.tag_id("AGGRESSIVE") {
        if scores.aggression > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("PEACEFUL") {
        if scores.aggression < 30 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("COWARDLY") {
        if scores.bravery < 30 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("FEARLESS") {
        if scores.bravery > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("CURIOUS") {
        if scores.curiosity > 60 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("TERRITORIAL") {
        if scores.orderliness > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    fn setup() -> TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).unwrap()
    }

    #[test]
    fn high_aggression_gets_aggressive_tag() {
        let registry = setup();
        let mut tags = Tags::new(registry.tag_count());
        let scores = PersonalityScores {
            aggression: 85, bravery: 50, sociability: 50,
            orderliness: 50, curiosity: 50, industriousness: 50,
            honesty: 50, spirituality: 50, gregariousness: 50, volatility: 50,
        };
        tags_from_personality(&scores, &mut tags, &registry);
        assert!(tags.has(registry.tag_id("AGGRESSIVE").unwrap()), "high aggression should get AGGRESSIVE tag");
    }

    #[test]
    fn low_aggression_gets_peaceful_tag() {
        let registry = setup();
        let mut tags = Tags::new(registry.tag_count());
        let scores = PersonalityScores {
            aggression: 20, bravery: 50, sociability: 50,
            orderliness: 50, curiosity: 50, industriousness: 50,
            honesty: 50, spirituality: 50, gregariousness: 50, volatility: 50,
        };
        tags_from_personality(&scores, &mut tags, &registry);
        assert!(tags.has(registry.tag_id("PEACEFUL").unwrap()), "low aggression should get PEACEFUL tag");
    }

    #[test]
    fn random_scores_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let scores = PersonalityScores::new_random(&mut rng);
        assert!(scores.aggression >= 20 && scores.aggression <= 80);
        assert!(scores.bravery >= 20 && scores.bravery <= 80);
    }
}
