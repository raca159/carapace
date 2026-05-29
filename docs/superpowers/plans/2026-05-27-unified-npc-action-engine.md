# Unified NPC Action Engine Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace three disconnected NPC systems (behavior scoring, dialogue selection, dead barter) with a single utility-AI action engine where every possible NPC action is scored by the same function: personality × tags × standing × environment.

**Architecture:** `score_action()` function receives `NpcAction`, `NpcContext`, `EntityContext`, `EnvironmentContext` and returns a utility score. Highest-scoring action selected. Talk hub reads scored actions → player options derived from top-ranked NPC actions.

**Tech Stack:** game-core, game-tags, CascadeEngine, PersonalityScores, WeatherContext, LocationMap

---

### Task 1: NpcAction enum + context structs

**Files:**
- Create: `crates/core/src/npc_action.rs`

```rust
use bevy_ecs::prelude::*;
use crate::PersonalityScores;
use game_tags::Tags;

/// All possible NPC actions — movement and social
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
pub struct NpcContext {
    pub scores: PersonalityScores,
    pub tags: Tags,
    pub faction_id: Option<game_tags::TagId>,
    pub is_quest_giver: bool,
    pub can_barter: bool,
    pub health_ratio: f32,  // current/max
}

/// Target/player context
pub struct EntityContext {
    pub tags: Tags,
    pub faction_id: Option<game_tags::TagId>,
    pub health_ratio: f32,
    pub distance: f32,
}

/// Environment context from weather + biome + location
pub struct EnvironmentContext {
    pub weather_tags: Vec<String>,
    pub near_location: Option<String>,  // "city", "dungeon", etc.
    pub is_night: bool,
}
```

### Task 2: npc_actions.toml config + loading

**Files:**
- Create: `assets/config/npc_actions.toml`

```toml
[[action]]
id = "speak_greeting"
base_weight = 10.0
personality_modifiers = { sociability = 1.5, curiosity = 1.2 }
standing_modifiers = { ally = 1.5, neutral = 1.0, hostile = 0.3 }
description = "Greet the player"

[[action]]
id = "speak_hostile"
base_weight = 8.0
personality_modifiers = { aggression = 1.8, volatility = 1.3 }
standing_modifiers = { hostile = 2.0, neutral = 0.5 }
description = "Threaten the player"

[[action]]
id = "offer_trade"
base_weight = 8.0
personality_modifiers = { sociability = 1.3, industriousness = 1.2 }
standing_modifiers = { ally = 1.5, neutral = 1.0, hostile = 0.1 }
requires_tags = ["CAN_BARTER"]
description = "Offer to trade goods"

[[action]]
id = "attack"
base_weight = 12.0
personality_modifiers = { aggression = 1.8, bravery = 1.3 }
standing_modifiers = { hostile = 3.0, neutral = 0.5, ally = 0.1 }
env_modifiers = [{ tag = "DARK", mult = 1.2 }]
requires_tags = ["AGGRESSIVE"]
description = "Attack the target"

[[action]]
id = "flee"
base_weight = 15.0
personality_modifiers = { bravery = 0.3 }
standing_modifiers = { hostile = 1.5, neutral = 0.5 }
env_modifiers = [{ tag = "DARK", mult = 1.1 }]
description = "Flee from the target"

[[action]]
id = "give_quest"
base_weight = 5.0
personality_modifiers = { sociability = 1.2, industriousness = 1.3 }
standing_modifiers = { ally = 1.5, neutral = 1.0, hostile = 0.1 }
requires_tags = ["QUEST_GIVER"]
description = "Offer a quest"

[[action]]
id = "end_conversation"
base_weight = 6.0
personality_modifiers = { sociability = 0.5, gregariousness = 0.5 }
description = "End the conversation"
```

Add to `npc_action.rs`:
```rust
use serde::Deserialize;
use std::collections::HashMap;

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
```

### Task 3: score_action() function

**Files:**
- Modify: `crates/core/src/npc_action.rs`

```rust
use game_tags::{TagId, TagRegistry};

/// Score a single NPC action given all context
pub fn score_action(
    action_id: &str,
    npc: &NpcContext,
    target: Option<&EntityContext>,
    environment: &EnvironmentContext,
    weights: &NpcActionWeights,
    registry: &TagRegistry,
) -> f32 {
    let def = match weights.by_id.get(action_id) {
        Some(d) => d,
        None => return 0.0,
    };

    // Check required tags
    for required in &def.requires_tags {
        if let Some(tid) = registry.tag_id(required) {
            if !npc.tags.has(tid) { return 0.0; }
        }
    }

    let mut score = def.base_weight;

    // Personality modifier
    for (trait_name, weight) in &def.personality_modifiers {
        let trait_value = get_personality_trait(&npc.scores, trait_name);
        score *= 1.0 + weight * (trait_value as f32 - 50.0) / 100.0;
    }

    // Standing modifier
    if let Some(target_context) = target {
        let standing = determine_standing(&npc.tags, target_context, registry);
        if let Some(mult) = def.standing_modifiers.get(standing.as_str()) {
            score *= mult;
        }
    }

    // Tag modifiers
    for tm in &def.tag_modifiers {
        if let Some(tid) = registry.tag_id(&tm.tag) {
            if npc.tags.has(tid) { score *= tm.mult; }
        }
    }

    // Environment modifiers
    for em in &def.env_modifiers {
        if environment.weather_tags.contains(&em.tag) { score *= em.mult; }
    }

    score.max(0.0)
}

fn get_personality_trait(scores: &PersonalityScores, name: &str) -> u8 {
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

fn determine_standing(npc_tags: &Tags, target: &EntityContext, registry: &TagRegistry) -> String {
    // Simplified: if target has AGGRESSIVE and we're PEACEFUL, hostile
    let aggressive = registry.tag_id("AGGRESSIVE");
    let npc_peaceful = registry.tag_id("PEACEFUL");
    if aggressive.is_some_and(|a| target.tags.has(a))
        && npc_peaceful.is_some_and(|p| npc_tags.has(p)) {
        return "hostile".to_string();
    }
    "neutral".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personality::PersonalityScores;

    fn setup() -> (NpcActionWeights, TagRegistry) {
        let toml = include_str!("../../../../assets/config/npc_actions.toml");
        let tags_toml = include_str!("../../../../assets/config/tags.toml");
        let registry = game_tags::load_tag_registry(tags_toml).unwrap();
        let weights = load_npc_action_weights(toml).unwrap();
        (weights, registry)
    }

    #[test]
    fn aggressive_npc_scores_attack_higher() {
        let (weights, registry) = setup();
        let aggressive_scores = PersonalityScores { aggression: 80, ..PersonalityScores::default() };
        let npc = NpcContext { scores: aggressive_scores, tags: game_tags::Tags::new(registry.tag_count()), faction_id: None, is_quest_giver: false, can_barter: false, health_ratio: 1.0 };
        let env = EnvironmentContext { weather_tags: vec![], near_location: None, is_night: false };
        let target = EntityContext { tags: game_tags::Tags::new(registry.tag_count()), faction_id: None, health_ratio: 1.0, distance: 3.0 };

        let peaceful_scores = PersonalityScores { aggression: 20, ..PersonalityScores::default() };
        let npc_peaceful = NpcContext { scores: peaceful_scores, ..npc };

        let score_agg = score_action("attack", &npc, Some(&target), &env, &weights, &registry);
        let score_peace = score_action("attack", &npc_peaceful, Some(&target), &env, &weights, &registry);
        assert!(score_agg > score_peace, "aggressive NPC should score attack higher than peaceful");
    }

    #[test]
    fn requires_tags_blocks_action() {
        let (weights, registry) = setup();
        let npc = NpcContext { can_barter: false, ..Default::default() };
        // Manually construct with no CAN_BARTER tag
        let env = EnvironmentContext { weather_tags: vec![], near_location: None, is_night: false };
        let score = score_action("offer_trade", &npc, None, &env, &weights, &registry);
        assert!(score == 0.0, "without CAN_BARTER tag, trade should be unavailable");
    }
}
```

### Task 4: NpcEmotionalState + Emotion

**Files:**
- Create: `crates/core/src/emotion.rs`

```rust
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion { Neutral, Happy, Angry, Fearful, Sad }

#[derive(Debug, Clone, Default)]
pub enum ConversationState { #[default] Idle, Greeting, Dialogue, Trading, Fighting, Goodbye }

#[derive(Component, Debug, Clone)]
pub struct NpcEmotionalState {
    pub stress: f32,
    pub dominant_emotion: Emotion,
    pub conversation: ConversationState,
}

impl Default for NpcEmotionalState {
    fn default() -> Self {
        Self { stress: 0.0, dominant_emotion: Emotion::Neutral, conversation: ConversationState::Idle }
    }
}

impl NpcEmotionalState {
    pub fn apply_event(&mut self, emotional_impact: f32) {
        self.stress = (self.stress + emotional_impact).clamp(0.0, 1.0);
        self.dominant_emotion = if self.stress > 0.7 { Emotion::Fearful }
            else if emotional_impact > 0.3 { Emotion::Angry }
            else if emotional_impact < -0.2 { Emotion::Happy }
            else { Emotion::Neutral };
    }
}
```

### Task 5: Replace talk hub with action-engine state machine

**Files:**
- Modify: `src/interact/talk.rs`

The talk hub now:
1. On E key on an NPC: score all possible actions for this NPC
2. Show top 3-4 actions as player options
3. On player selection, resolve the action
4. Update NPC emotional state
5. Re-score and show new options

```rust
// Key data structure
struct TalkState {
    npc_entity: Entity,
    npc_context: NpcContext,
    environment: EnvironmentContext,
    scored_actions: Vec<(String, f32)>,  // action_id, score
    emotional_state: NpcEmotionalState,
}
```

The existing `handle_talk_input` is replaced. Instead of just selecting dialogue, it:
- Builds `NpcContext` and `EnvironmentContext` from game state
- Calls `score_action()` for each possible action
- Renders the top 3-4 as numbered options
- Routes `T` to speak action, `B` to trade, `Q` to quest, `Esc` to end

The `update_talk_panel` renders the action list:
```
┌─ Speak with Merchant ───────┐
│                             │
│ [T] "Fine goods today."     │
│ [B] Barter                  │
│ [Q] Available quests        │
│ [Esc] Goodbye               │
└─────────────────────────────┘
```

### Task 6: Wire trade → barter resolution

**Files:**
- Modify: `src/interact/talk.rs`

When player selects Barter and `OfferTrade` is one of the top actions:
1. Call `resolve_barter_with_haggle` from `crates/core/src/barter.rs`
2. Get `RegionEconomies` for price multipliers
3. Get NPC's inventory from cascade engine
4. Show trade UI overlay
5. On trade complete, update NPC's emotional state

### Task 7: Wire attack → combat

**Files:**
- Modify: `src/game/mod.rs`

When the action engine selects `Attack` (or player initiates combat):
- Call existing `resolve_combat` from game/mod.rs
- Update NPC's emotional state based on combat outcome
- If NPC loses → emotional state becomes Fearful, next actions weighted toward Flee

### Task 8: Enrich movement behavior with personality

**Files:**
- Modify: `crates/world/src/behavior.rs`

In `process_npc_turns`, multiply each `BehaviorRule`'s weight by a personality modifier:
```rust
// Before scoring actions, compute personality modifier
let personality_mod = if let Some(scores) = world.get::<PersonalityScores>(creature) {
    match rule.trait_tag.as_str() {
        "AGGRESSIVE" => 0.5 + scores.aggression as f32 / 100.0,
        "COWARDLY" => 0.5 + (100 - scores.bravery) as f32 / 100.0,
        "TERRITORIAL" => 0.5 + scores.orderliness as f32 / 100.0,
        "CURIOUS" => 0.5 + scores.curiosity as f32 / 100.0,
        _ => 1.0,
    }
} else { 1.0 };

action_score *= personality_mod;
```

This makes an NPC with `aggression: 80` chase with ~1.3× the weight of one with `aggression: 50`.
