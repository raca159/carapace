use bevy::prelude::*;
use game_core::quest::QuestBoardState;
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct QuestBoardPanel(pub Option<Entity>);

pub fn update_quest_board_panel(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut panel: ResMut<QuestBoardPanel>,
    mut game_world: ResMut<crate::render::GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    if !matches!(&interact.active, Some(InteractMode::QuestBoard)) {
        return;
    }

    let ecs_world = &mut game_world.0;

    // Check if refresh is needed
    game_core::quest::check_quest_board_refresh(ecs_world);

    let board_state = match ecs_world.get_resource::<QuestBoardState>() {
        Some(s) => s.clone(),
        None => return,
    };

    let mut lines = vec!["┌─ Quest Board ──────────────────────┐".to_string()];
    for (i, entry) in board_state.available_quests.iter().enumerate() {
        let num = i + 1;
        lines.push(format!("│ {}. {}", num, entry.name));
        lines.push(format!("│    {}", entry.description));
    }
    if board_state.available_quests.is_empty() {
        lines.push("│  No quests available.".to_string());
    }
    lines.push("│".to_string());
    lines.push("│  [1-9] Accept  |  Esc".to_string());
    lines.push("└────────────────────────────────────┘".to_string());

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

#[allow(dead_code)]
pub fn start_quest_giver(
    _ecs_world: &mut World,
    _npc_entity: bevy_ecs::entity::Entity,
    interact_state: &mut InteractState,
) {
    interact_state.active = Some(InteractMode::QuestBoard);
}
