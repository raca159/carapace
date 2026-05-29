use bevy_ecs::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use crate::PersonalityScores;
use game_tags::Tags;

/// All possible NPC actions — movement and social
#[derive(Debug, Clone)]
pub enum NpcAction {
    Move { dx: i32, dy: i32 },
    Flee { dx: i32, dy: i32 },
    Wait,
    Speak { line: String },
    OfferTrade,
    AttackTarget { target: bevy_ecs::entity::Entity },
    GiveQuest,
    EndConversation,
}

/// NPC's internal context for action scoring
#[derive(Debug, Clone)]
pub struct NpcContext {
    pub scores: PersonalityScores,
    pub tags: Tags,
    pub faction_id: Option<game_tags::TagId>,
    pub is_quest_giver: bool,
    pub can_barter: bool,
    pub health_ratio: f32,
}

impl Default for NpcContext {
    fn default() -> Self {
        Self {
            scores: PersonalityScores::default(),
            tags: Tags::new(0),
            faction_id: None,
            is_quest_giver: false,
            can_barter: false,
            health_ratio: 1.0,
        }
    }
}

/// Target/player context
#[derive(Debug, Clone)]
pub struct EntityContext {
    pub tags: Tags,
    pub faction_id: Option<game_tags::TagId>,
    pub health_ratio: f32,
    pub distance: f32,
}

impl Default for EntityContext {
    fn default() -> Self {
        Self {
            tags: Tags::new(0),
            faction_id: None,
            health_ratio: 1.0,
            distance: 1.0,
        }
    }
}

/// Environment context from weather + biome + location
#[derive(Debug, Clone, Default)]
pub struct EnvironmentContext {
    pub weather_tags: Vec<String>,
    pub weather_tag_ids: Vec<game_tags::TagId>,
    pub near_location: Option<String>,
    pub is_night: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionWeightDef {
    pub id: String,
    pub base_weight: f32,
    #[serde(default)]
    pub personality_modifiers: HashMap<String, f32>,
    #[serde(default)]
    pub standing_modifiers: HashMap<String, f32>,
    #[serde(default)]
    pub tag_modifiers: Vec<TagModifierDef>,
    #[serde(default)]
    pub env_modifiers: Vec<EnvModifierDef>,
    #[serde(default)]
    pub requires_tags: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagModifierDef {
    pub tag: String,
    pub mult: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvModifierDef {
    pub tag: String,
    pub mult: f32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NpcActionWeights {
    pub actions: Vec<ActionWeightDef>,
    pub by_id: HashMap<String, ActionWeightDef>,
}

pub fn load_npc_action_weights(toml_str: &str) -> Result<NpcActionWeights, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct TomlFile { #[serde(rename = "action")] actions: Vec<ActionWeightDef>, }
    let file: TomlFile = toml::from_str(toml_str)?;
    let by_id: HashMap<_, _> = file.actions.iter().map(|a| (a.id.clone(), a.clone())).collect();
    Ok(NpcActionWeights { actions: file.actions, by_id })
}

/// Score a single NPC action given all context
pub fn score_action(
    action_id: &str,
    npc: &NpcContext,
    target: Option<&EntityContext>,
    environment: &EnvironmentContext,
    weights: &NpcActionWeights,
    registry: &game_tags::TagRegistry,
) -> f32 {
    let def = match weights.by_id.get(action_id) {
        Some(d) => d,
        None => return 0.0,
    };

    for required in &def.requires_tags {
        if let Some(tid) = registry.tag_id(required) {
            if !npc.tags.has(tid) { return 0.0; }
        }
    }

    let mut score = def.base_weight;

    for (trait_name, weight) in &def.personality_modifiers {
        let trait_value = get_personality_trait(&npc.scores, trait_name);
        score *= 1.0 + weight * (trait_value as f32 - 50.0) / 100.0;
    }

    if let Some(target_context) = target {
        let standing = determine_standing(&npc.tags, target_context, registry);
        if let Some(mult) = def.standing_modifiers.get(standing.as_str()) {
            score *= mult;
        }
    }

    for tm in &def.tag_modifiers {
        if let Some(tid) = registry.tag_id(&tm.tag) {
            if npc.tags.has(tid) { score *= tm.mult; }
        }
    }

    for em in &def.env_modifiers {
        if !environment.weather_tag_ids.is_empty() {
            if let Some(tid) = registry.tag_id(&em.tag) {
                if environment.weather_tag_ids.contains(&tid) { score *= em.mult; }
            }
        } else if environment.weather_tags.contains(&em.tag) {
            score *= em.mult;
        }
    }

    score.max(0.0)
}

fn get_personality_trait(scores: &crate::PersonalityScores, name: &str) -> u8 {
    match name {
        "aggression" => scores.aggression,
        "bravery" => scores.bravery,
        "sociability" => scores.sociability,
        "orderliness" => scores.orderliness,
        "curiosity" => scores.curiosity,
        "industriousness" => scores.industriousness,
        "honesty" => scores.honesty,
        "spirituality" => scores.spirituality,
        "gregariousness" => scores.gregariousness,
        "volatility" => scores.volatility,
        _ => 50,
    }
}

fn determine_standing(_npc_tags: &game_tags::Tags, _target: &EntityContext, _registry: &game_tags::TagRegistry) -> String {
    "neutral".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::PersonalityScores;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");
    const ACTIONS_TOML: &str = include_str!("../../../assets/config/npc_actions.toml");

    fn setup() -> (NpcActionWeights, game_tags::TagRegistry) {
        let registry = game_tags::load_tag_registry(TAGS_TOML).unwrap();
        let weights = load_npc_action_weights(ACTIONS_TOML).unwrap();
        (weights, registry)
    }

    #[test]
    fn load_actions_parses_all_entries() {
        let (weights, _) = setup();
        assert_eq!(weights.actions.len(), 7, "should load 7 action definitions");
        assert!(weights.by_id.contains_key("speak_greeting"), "should contain greet action");
        assert!(weights.by_id.contains_key("attack"), "should contain attack action");
    }

    #[test]
    fn aggressive_npc_scores_attack_higher_than_peaceful() {
        let (weights, registry) = setup();
        let agg_scores = PersonalityScores { aggression: 80, ..PersonalityScores::default() };
        let peaceful_scores = PersonalityScores { aggression: 20, ..PersonalityScores::default() };

        let mut tags_agg = game_tags::Tags::new(registry.tag_count());
        let mut tags_peace = game_tags::Tags::new(registry.tag_count());
        crate::personality::tags_from_personality(&agg_scores, &mut tags_agg, &registry);
        crate::personality::tags_from_personality(&peaceful_scores, &mut tags_peace, &registry);

        let npc_agg = NpcContext { scores: agg_scores, tags: tags_agg, ..Default::default() };
        let npc_peace = NpcContext { scores: peaceful_scores, tags: tags_peace, ..Default::default() };
        let env = EnvironmentContext::default();
        let target = EntityContext::default();

        let score_agg = score_action("attack", &npc_agg, Some(&target), &env, &weights, &registry);
        let score_peace = score_action("attack", &npc_peace, Some(&target), &env, &weights, &registry);
        assert!(score_agg > score_peace, "aggressive NPC should score attack higher than peaceful");
    }

    #[test]
    fn requires_tags_blocks_action() {
        let (weights, registry) = setup();
        let npc = NpcContext::default();
        let env = EnvironmentContext::default();
        let score = score_action("offer_trade", &npc, None, &env, &weights, &registry);
        assert_eq!(score, 0.0, "without CAN_BARTER tag, trade should be unavailable");
    }

    #[test]
    fn unknown_action_returns_zero() {
        let (weights, registry) = setup();
        let npc = NpcContext::default();
        let env = EnvironmentContext::default();
        let score = score_action("nonexistent_action", &npc, None, &env, &weights, &registry);
        assert_eq!(score, 0.0, "unknown action should score 0");
    }

    #[test]
    fn personality_modifier_scales_score() {
        let (weights, registry) = setup();
        let high_bravery = PersonalityScores { bravery: 90, ..PersonalityScores::default() };
        let low_bravery = PersonalityScores { bravery: 10, ..PersonalityScores::default() };

        let npc_high = NpcContext { scores: high_bravery, ..Default::default() };
        let npc_low = NpcContext { scores: low_bravery, ..Default::default() };
        let env = EnvironmentContext::default();
        let target = EntityContext::default();

        let high = score_action("flee", &npc_high, Some(&target), &env, &weights, &registry);
        let low = score_action("flee", &npc_low, Some(&target), &env, &weights, &registry);
        assert!(high > low, "high bravery should score flee higher than low bravery");
    }
}
