use bevy::prelude::*;
use game_core::{DialogueLinesResource, MessageLog, QuestGiver, Name, Player, WeatherContext};
use game_core::dialogue::select_dialogue;
use game_core::npc_action::{NpcContext, EnvironmentContext, NpcActionWeights, score_action};
use game_core::emotion::NpcEmotionalState;
use game_tags::{TagRegistry, Tags};
use game_world::{Faction, FactionRelationships, FactionStanding, PLAYER_FACTION_ID};
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct TalkPanel(pub Option<Entity>);

pub fn build_npc_context(ecs_world: &World, npc_entity: Entity) -> NpcContext {
    let scores = ecs_world.get::<NpcEmotionalState>(npc_entity)
        .map(|_| ecs_world.get::<game_core::PersonalityScores>(npc_entity).cloned().unwrap_or_default())
        .unwrap_or_default();
    let tags = ecs_world.get::<game_tags::Tags>(npc_entity).cloned().unwrap_or(Tags::new(0));
    let _faction_id: Option<game_tags::TagId> = ecs_world.get::<Faction>(npc_entity).map(|f| f.faction_id)
        .and_then(|_fid| None);
    let health = ecs_world.get::<game_core::Health>(npc_entity).copied().unwrap_or(game_core::Health { current: 1, max: 1 });
    let can_barter = ecs_world.get_resource::<TagRegistry>()
        .and_then(|r| r.tag_id("CAN_BARTER"))
        .is_some_and(|id| tags.has(id));
    let is_quest_giver = ecs_world.get::<QuestGiver>(npc_entity).is_some();

    NpcContext {
        scores,
        tags,
        faction_id: None,
        is_quest_giver,
        can_barter,
        health_ratio: health.current as f32 / health.max as f32,
    }
}

pub fn build_environment_context(ecs_world: &World) -> EnvironmentContext {
    let wc = ecs_world.get_resource::<WeatherContext>();
    let registry = ecs_world.get_resource::<TagRegistry>();

    let is_night = wc.as_ref().is_some_and(|wc| {
        let dark_id = registry.and_then(|r| r.tag_id("DARK"));
        let dim_id = registry.and_then(|r| r.tag_id("DIM"));
        dark_id.is_some_and(|id| wc.applied_tags.contains(&id))
            || dim_id.is_some_and(|id| wc.applied_tags.contains(&id))
    });
    let weather_tag_ids = wc.map(|wc| wc.applied_tags.clone()).unwrap_or_default();
    let weather_tags = wc.map(|wc| wc.tags.clone()).unwrap_or_default();

    EnvironmentContext {
        weather_tags,
        weather_tag_ids,
        near_location: None,
        is_night,
    }
}

pub fn update_talk_panel(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut panel: ResMut<TalkPanel>,
    game_world: Res<crate::render::GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    let npc_entity = match &interact.active {
        Some(InteractMode::Talk { npc_entity }) => *npc_entity,
        _ => return,
    };

    let ecs_world = &game_world.0;
    let name = ecs_world.get::<Name>(npc_entity)
        .map(|n| n.0.clone()).unwrap_or_else(|| "Someone".to_string());
    let emotional_state = ecs_world.get::<NpcEmotionalState>(npc_entity)
        .cloned().unwrap_or_default();

    let mut lines = vec![format!("┌─ {} ─────────────────────┐", name)];

    // Show current emotional state
    let mood = match emotional_state.dominant_emotion {
        game_core::emotion::Emotion::Happy => " (friendly)",
        game_core::emotion::Emotion::Angry => " (angry)",
        game_core::emotion::Emotion::Fearful => " (fearful)",
        _ => "",
    };
    lines.push(format!("│{}", mood));

    // Show available actions
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r,
        None => return,
    };
    let weights = match ecs_world.get_resource::<NpcActionWeights>() {
        Some(w) => w,
        None => return,
    };
    let npc = build_npc_context(ecs_world, npc_entity);
    let env = build_environment_context(ecs_world);

    let action_ids = ["speak_greeting", "offer_trade", "give_quest", "end_conversation", "speak_hostile", "flee", "attack"];
    let mut scored: Vec<(&str, f32)> = action_ids.iter()
        .map(|id| (*id, score_action(id, &npc, None, &env, weights, registry)))
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (action_id, _score) in scored.iter().take(4) {
        let key = match *action_id {
            "speak_greeting" | "speak_hostile" => "[T] Talk",
            "offer_trade" => "[B] Barter",
            "give_quest" => "[Q] Quests",
            "end_conversation" => "[Esc] Leave",
            "flee" => "[Esc] Leave (uneasy)",
            _ => continue,
        };
        lines.push(format!("│  {}", key));
    }
    lines.push("└──────────────────────────────────┘".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(root);
}

pub fn handle_talk_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interact: ResMut<InteractState>,
    mut game_world: ResMut<crate::render::GameWorld>,
) {
    let npc_entity = match &interact.active {
        Some(InteractMode::Talk { npc_entity }) => *npc_entity,
        _ => return,
    };

    let ecs_world = &mut game_world.0;

    // Score available actions — source of truth for dispatch gating
    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let weights = match ecs_world.get_resource::<NpcActionWeights>() {
        Some(w) => w.clone(),
        None => return,
    };
    let npc = build_npc_context(ecs_world, npc_entity);
    let env = build_environment_context(ecs_world);
    let action_ids = ["speak_greeting", "speak_hostile", "offer_trade", "give_quest", "attack"];
    let scored: std::collections::HashSet<&str> = action_ids.iter()
        .filter(|id| score_action(id, &npc, None, &env, &weights, &registry) > 0.0)
        .copied()
        .collect();

    // Escape always exits conversation
    if keyboard.just_pressed(KeyCode::Escape) {
        interact.active = None;
        return;
    }

    // [T] Talk — gated on speak_greeting or speak_hostile having a positive score
    if keyboard.just_pressed(KeyCode::KeyT) {
        if !scored.contains("speak_greeting") && !scored.contains("speak_hostile") {
            return;
        }
        let npc_tags = match ecs_world.get::<game_tags::Tags>(npc_entity) {
            Some(t) => t.clone(),
            None => return,
        };
        let dialogue_lines = match ecs_world.get_resource::<DialogueLinesResource>() {
            Some(r) => r.lines.clone(),
            None => return,
        };
        let standing_str = match ecs_world.get_resource::<FactionRelationships>() {
            Some(rels) => {
                let npc_faction = ecs_world.get::<Faction>(npc_entity).map(|f| f.faction_id);
                match npc_faction.and_then(|fid| Some(rels.get_standing(PLAYER_FACTION_ID, fid))) {
                    Some(FactionStanding::Ally) => "ally",
                    Some(FactionStanding::Neutral) => "neutral",
                    Some(FactionStanding::Hostile) => "hostile",
                    None => "neutral",
                }
            }
            None => "neutral",
        };
        let dialogue = select_dialogue(&npc_tags, standing_str, &dialogue_lines, &registry)
            .unwrap_or_else(|| "...".to_string());
        let name = ecs_world.get::<Name>(npc_entity).map(|n| n.0.clone()).unwrap_or_default();

        if let Some(mut msg) = ecs_world.get_resource_mut::<MessageLog>() {
            msg.messages.push(format!("{}: \"{}\"", name, dialogue));
        }

        if let Some(mut state) = ecs_world.get_mut::<NpcEmotionalState>(npc_entity) {
            state.apply_event(-0.1);
            state.conversation = game_core::emotion::ConversationState::Dialogue;
        }
        return;
    }

    // [B] Barter — gated on offer_trade score (replaces direct CAN_BARTER tag check)
    if keyboard.just_pressed(KeyCode::KeyB) {
        if !scored.contains("offer_trade") {
            if let Some(mut msg) = ecs_world.get_resource_mut::<MessageLog>() {
                msg.messages.push("They're not interested in trading.".to_string());
            }
            return;
        }
        crate::interact::trade::start_trade(ecs_world, npc_entity, &mut interact);
        return;
    }

    // [Q] Quests — gated on give_quest score (replaces direct QuestGiver component check)
    if keyboard.just_pressed(KeyCode::KeyQ) {
        if !scored.contains("give_quest") {
            return;
        }
        interact.active = Some(InteractMode::QuestBoard);
        return;
    }

    // [F] Fight — gated on attack score (replaces re-scoring with threshold check)
    if keyboard.just_pressed(KeyCode::KeyF) {
        if !scored.contains("attack") {
            return;
        }
        if ecs_world
            .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
            .single(ecs_world)
            .is_err()
        {
            return;
        }
        crate::game::resolve_combat(ecs_world, npc_entity, String::new(), game_core::Health { current: 1, max: 1 });
        if let Some(mut state) = ecs_world.get_mut::<NpcEmotionalState>(npc_entity) {
            state.apply_event(0.5);
        }
        interact.active = None;
        return;
    }
}
