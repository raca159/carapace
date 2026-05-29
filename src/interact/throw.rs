use bevy::prelude::*;
use game_core::Name;
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct ThrowOverlay(pub Option<Entity>);

pub fn update_throw_overlay(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut overlay: ResMut<ThrowOverlay>,
    game_world: Res<crate::render::GameWorld>,
) {
    if let Some(old) = overlay.0.take() { commands.entity(old).despawn(); }

    match &interact.active {
        Some(InteractMode::ItemSelection { mode: crate::interact::SelectionMode::Throw, items, cursor }) => {
            let mut lines = vec!["┌─ Throw ───────────────────────────┐".to_string()];
            for (i, &item) in items.iter().enumerate() {
                let name = game_world.0.get::<Name>(item)
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| "?".to_string());
                let marker = if i == *cursor { ">" } else { " " };
                lines.push(format!("│ {} {}. {}", marker, i + 1, name));
            }
            lines.push("│".to_string());
            lines.push("│  [Enter] Select  |  Esc Cancel".to_string());
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
            overlay.0 = Some(root);
        }
        Some(InteractMode::ThrowTargeting { cursor_x, cursor_y, .. }) => {
            let text = format!(
                "Throw at: ({}, {})\nArrow keys to aim  |  Enter to throw  |  Esc cancel",
                cursor_x, cursor_y
            );
            let root = commands.spawn((
                Text(text),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(8.0),
                    top: Val::Px(80.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
            )).id();
            overlay.0 = Some(root);
        }
        _ => {}
    }
}
