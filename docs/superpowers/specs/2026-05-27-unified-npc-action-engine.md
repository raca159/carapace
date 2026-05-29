# Unified NPC Action Engine Design

> **Status:** Spec — pending review
> **Goal:** Replace three disconnected NPC systems (behavior scoring, dialogue selection, dead barter) with a single utility-AI action engine where every possible NPC action is scored by the same function: personality × tags × standing × environment.

**Architecture:** A single `score_action()` function that takes any NPC action and returns a utility score. The highest-scoring action is selected. The talk hub reads the NPC's current action set to derive player-facing interaction options. Personality scores (DF-style) + entity tags + faction standing + environment/weather all feed into the scoring. Modeled after The Sims utility AI with Dwarf Fortress personality facets.

**Tech Stack:** bevy_ecs, game-tags, game-core (PersonalityScores, FactionRelationships, Tags), CascadeEngine, WeatherContext, LocationMap

---

## 1. Core Concept

### Current (broken):
```
Behavior Scoring → movement only (chase/flee/guard/wander)
Dialogue Selection → tag match → single utterance (separate system)
Barter → dead code (separate system)
Combat → hardcoded in game/mod.rs (separate system)
```

### Unified (target):
```
NpcActionEngine → all actions scored by shared function:
  score = base_weight × personality_mod × tag_mod × standing_mod × env_mod

Actions scored: Move, Speak, Trade, Attack, GiveQuest, Flee, EndConversation
               (same engine, same inputs, same scoring)

Talk hub = state machine that reads NPC's current action set
           → derives player options from top-ranked NPC actions
```

---

## 2. Personality Score Model (DF-style)

The existing `PersonalityScores` component (10 traits, 0-100) is extended with **value priorities** and **emotional state**.

```rust
// Current (exists)
pub struct PersonalityScores {
    pub aggression: u8,     // 0-100
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

// New additions
pub struct NpcEmotionalState {
    pub stress: f32,              // 0.0-1.0, accumulates from events
    pub dominant_emotion: Emotion, // Happy, Angry, Fearful, Neutral, etc.
    pub last_interaction: Option<Entity>,  // who they last interacted with
    pub conversation_state: ConversationState, // idle, talking, trading, etc.
}

pub enum Emotion { Neutral, Happy, Angry, Fearful, Sad, Surprised, Disgusted }

pub enum ConversationState {
    Idle,
    Greeting,
    Dialogue,
    Trading,
    Questing,
    Fighting,
    Fleeing,
    Goodbye,
}
```

**Score derivation from personality (existing, to be wired):**
```
aggression > 70  → AGGRESSIVE tag
aggression < 30  → PEACEFUL tag
bravery < 30    → COWARDLY tag
bravery > 70    → FEARLESS tag
curiosity > 60  → CURIOUS tag
orderliness > 70 → TERRITORIAL tag
```

---

## 3. Action System

### Action enum

```rust
pub enum NpcAction {
    // Movement (existing behavior.rs actions)
    Move { dx: i32, dy: i32 },
    Flee { dx: i32, dy: i32 },
    Wait,
    
    // Social (new)
    Speak { line: String },
    OfferTrade,
    AttackTarget { target: Entity },
    GiveQuest { quest_id: String },
    RequestBarter,
    EndConversation,
    
    // Reactive (triggered by player actions)
    RespondToGreeting,
    RespondToTrade,
    RespondToThreat,
    RespondToQuest,
}
```

### Action ranked by `score_action()`

```rust
pub fn score_action(
    action: &NpcAction,
    npc: &NpcContext,           // personality, tags, scores, faction, equipment
    target: Option<&EntityContext>,  // who they're interacting with (player or other NPC)
    environment: &EnvironmentContext, // biome tags, weather tags, location proximity
    registry: &TagRegistry,
) -> f32 {
    let base = action_base_weight(action);          // config-driven
    let personality = personality_modifier(action, &npc.scores);  // 0.5-2.0
    let standing = standing_modifier(action, &npc.faction, target); // 0.1-2.0
    let tags = tag_modifier(action, &npc.tags, registry);          // 0.5-2.0
    let env = environment_modifier(action, environment);            // 0.5-1.5
    let context = conversation_modifier(action, &npc.emotional_state); // 0.1-2.0
    
    base * personality * standing * tags * env * context
}
```

### Action weights defined in config (TOML)

```toml
# assets/config/npc_actions.toml
[[action]]
id = "speak_greeting"
base_weight = 10.0
personality_modifiers = { sociability = 1.5, shyness = 0.3 }
tag_modifiers = [{ tag = "PEACEFUL", mult = 1.5 }, { tag = "AGGRESSIVE", mult = 0.5 }]
standing_modifiers = [{ standing = "ally", mult = 1.5 }, { standing = "hostile", mult = 0.3 }]
description = "Greet the player"

[[action]]
id = "offer_trade"
base_weight = 8.0
personality_modifiers = { sociability = 1.3, greed = 1.5 }
tag_modifiers = [{ tag = "CAN_BARTER", mult = 2.0 }]
standing_modifiers = [{ standing = "ally", mult = 1.5 }, { standing = "neutral", mult = 1.0 }]
requires = [{ tag = "CAN_BARTER" }]
description = "Offer to trade goods"

[[action]]
id = "attack"
base_weight = 12.0
personality_modifiers = { aggression = 1.8, bravery = 1.3 }
tag_modifiers = [{ tag = "AGGRESSIVE", mult = 2.0 }]
standing_modifiers = [{ standing = "hostile", mult = 3.0 }]
env_modifiers = [{ tag = "DARK", mult = 1.2 }]
description = "Attack the player"

[[action]]
id = "flee"
base_weight = 15.0
personality_modifiers = { bravery = 0.3, shyness = 1.5 }
tag_modifiers = [{ tag = "COWARDLY", mult = 2.0 }]
standing_modifiers = [{ standing = "hostile", mult = 1.0 }]
description = "Flee from the player"

[[action]]
id = "give_quest"
base_weight = 5.0
personality_modifiers = { sociability = 1.2, industriousness = 1.3 }
tag_modifiers = [{ tag = "QUEST_GIVER", mult = 2.0 }]
requires = [{ tag = "QUEST_GIVER" }]
description = "Offer a quest to the player"
```

---

## 4. Scoring Modifiers

### Personality modifier (per action)

Each action can define how personality scores affect its utility:
```
personality_mod = 1.0 + sum(weight * (score - 50) / 100 for each relevant trait)
```

Example for `attack` action with `aggression = 80`:
```
personality_mod = 1.0 + 1.8 * (80 - 50) / 100 = 1.0 + 0.54 = 1.54
```

For a peaceful NPC with `aggression = 20`:
```
personality_mod = 1.0 + 1.8 * (20 - 50) / 100 = 1.0 - 0.54 = 0.46
```

### Standing modifier

Based on faction relationship between NPC and target:
```
standing = FactionRelationships.get_standing(npc_faction, target_faction)
standing_mod = match standing {
    Ally     → action.standing_ally_mult    (default 1.5)
    Neutral  → action.standing_neutral_mult  (default 1.0)
    Hostile  → action.standing_hostile_mult  (default 0.5 for social, 3.0 for combat)
}
```

### Environment modifier

From WeatherContext tags + biome tags + location proximity:
```
env_mod = 1.0
if weather.tags contains "DARK"     → env_mod *= 1.2 (more dangerous at night)
if weather.tags contains "STORMY"   → env_mod *= 0.8 (less trading in storm)
if near_location == "city"          → env_mod *= 1.3 (more social in cities)
```

### Conversation context modifier

Emotional state + previous action history:
```
context_mod = match emotional_state.dominant_emotion {
    Angry   → attack +2.0, speak -0.5
    Happy   → speak +1.5, trade +1.3
    Fearful → flee +2.0, speak -0.7
    _       → 1.0
}

// Escalation: if player just attacked, NPC can't go back to idle
if last_action == "attack" → all social actions × 0.1
```

---

## 5. Talk Hub as State Machine

The talk hub becomes a **conversation state machine** that reads the NPC's current action scoring and derives player options from it:

```
Player presses E on NPC
  → NpcActionEngine scores all actions for this NPC
  → Top 3-4 actions become player options in talk hub:
      If "Speak" is top → [T] Talk
      If "OfferTrade" is top → [B] Barter
      If "GiveQuest" is top → [Q] Quests
      If "Attack" is top → [F] Fight (defensive)
      If "Flee" is top → [G] Let them go
  
Player selects an option
  → Action is resolved (dialogue shown, trade UI opens, etc.)
  → NPC's emotional state updates based on interaction outcome
  → Re-score actions for next round
  → Show new options
```

### Conversation flow example:

```
Round 1: Player approaches Sanguine Noble
  NPC scores: Speak(greeting)=45, OfferTrade=32, Attack=12
  Player sees: [T] Talk  [B] Barter
  
Round 2: Player presses T (talk)
  NPC responds with selected dialogue
  Emotional state: Happy (from successful interaction)
  Re-score: OfferTrade=48, Speak(continue)=40, GiveQuest=20
  Player sees: [T] Continue  [B] Barter  [Q] Quests
  
Round 3: Player presses B (barter)
  Trade UI opens with NPC inventory + prices from RegionEconomies
  Emotional state: Neutral (during trade)
  
Player exits trade
  Emotional state: Neutral
  Re-score: Speak(goodbye)=60, EndConversation=30
  Player sees: [T] Say goodbye  [Esc] Leave
```

---

## 6. Integration with Existing Systems

### Behavior movement scoring (behavior.rs)

The existing movement action scoring stays but its inputs are enriched:
- `chase_direction` already uses A* (wired in Task 5)
- Weight scores now modulated by personality scores (aggressive NPC chases harder)
- `has_line_of_sight` already checked (wired in Task 5)

### Dialogue selection (dialogue.rs)

`select_dialogue` is replaced by scored `Speak` actions:
```rust
// Instead of:
let line = select_dialogue(&npc_tags, standing_str, &dialogue_lines, &registry);

// Use:
let speak_action = NpcAction::Speak { line: select_dialogue(...) };
let score = score_action(&speak_action, &npc_context, &player_context, &env, &registry);
```

The existing `dialogue.toml` entries become the line pool from which `Speak` actions draw.

### Barter (barter.rs)

`resolve_barter_with_haggle` is called when the `OfferTrade` action is selected:
```rust
if selected_action == "offer_trade" {
    let result = resolve_barter_with_haggle(
        &offer, player_tags, npc_tags, &registry, &mut rng,
    );
    // Apply result, update emotional state
}
```

### Combat (game/mod.rs)

`resolve_combat` is called when `Attack` action is selected:
```rust
if selected_action == "attack" {
    resolve_combat(&mut game_world.0, npc_entity, ...);
}
```

---

## 7. New/Modified Files

### New files:
- `crates/core/src/npc_action.rs` — `NpcAction` enum, `NpcContext`, `EntityContext`, `EnvironmentContext`, `NpcActionWeights` resource (loaded from TOML)
- `crates/core/src/emotion.rs` — `NpcEmotionalState`, `Emotion`, `ConversationState`
- `assets/config/npc_actions.toml` — action weight definitions with personality/standing/tag/environment modifiers

### Modified files:
- `crates/core/src/personality.rs` — add `NpcEmotionalState` component, `emotion_from_stress()` function
- `crates/core/src/lib.rs` — add `pub mod npc_action;`, `pub mod emotion;`
- `src/game/mod.rs` — replace hardcoded combat call with action-engine routing
- `src/interact/talk.rs` — replace stub with state-machine talk hub
- `src/interact/mod.rs` — wire action engine into interaction routing
- `crates/world/src/behavior.rs` — enrich movement scoring with personality (optional enhancement)

---

## 8. Implementation Order

| Step | What | Dependencies |
|------|------|-------------|
| 1 | `NpcAction` enum, `NpcContext`, `EnvironmentContext` structs | None |
| 2 | `npc_actions.toml` config + load function + `NpcActionWeights` resource | Step 1 |
| 3 | `score_action()` function with personality/standing/tag/env modifiers | Steps 1-2 |
| 4 | `NpcEmotionalState` + `Emotion` + stress accumulation | None |
| 5 | Replace talk hub with state machine (reads scored actions → player options) | Steps 3-4, existing dialogue/barter |
| 6 | Wire `OfferTrade` → `resolve_barter_with_haggle` | Step 5 |
| 7 | Wire `Attack` → `resolve_combat` | Step 5 |
| 8 | Enrich movement behavior scoring with personality scores | Step 3 |
