use bevy::prelude::*;
use game_core::{Inventory, Name, Glyph};
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct LootPanel(pub Option<Entity>);

pub fn update_loot_panel(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut panel: ResMut<LootPanel>,
    game_world: ResMut<crate::render::GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    let (container_entity, cursor) = match &interact.active {
        Some(InteractMode::Looting { container_entity, cursor }) => (*container_entity, *cursor),
        _ => return,
    };

    let ecs_world = &game_world.0;

    let container_name = ecs_world.get::<Name>(container_entity)
        .map(|n| n.0.clone())
        .unwrap_or_else(|| "Container".to_string());

    let inv = ecs_world.get::<Inventory>(container_entity).cloned();

    let mut lines = vec![format!("╭─ {} ─╮", container_name)];

    match inv {
        Some(ref i) if !i.items.is_empty() => {
            for (idx, &item_entity) in i.items.iter().enumerate() {
                let name = ecs_world.get::<Name>(item_entity)
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| "?".to_string());
                let glyph_char = ecs_world.get::<Glyph>(item_entity)
                    .map(|g| g.char)
                    .unwrap_or('?');
                let marker = if idx == cursor { ">" } else { " " };
                lines.push(format!("│ {} {} {}", marker, glyph_char, name));
            }
        }
        _ => {
            lines.push("│  (empty)".to_string());
        }
    }

    lines.push("├──────────────────────────────────┤".to_string());
    lines.push("│  [Enter] take  [T] take all".to_string());
    lines.push("│  [Esc] close".to_string());
    lines.push("╰──────────────────────────────────╯".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(28.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(root);
}
