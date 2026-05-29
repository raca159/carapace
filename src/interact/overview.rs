use bevy::prelude::*;
use game_core::screen::AppScreen;
use game_core::world_overview::WorldOverviewState;
use game_world::cascade::LocationMap;
use crate::render::GameWorld;

#[derive(Resource, Default)]
pub struct OverviewOverlay(pub Option<Entity>);

pub fn update_world_overview(
    mut commands: Commands,
    mut game_world: ResMut<GameWorld>,
    mut overlay: ResMut<OverviewOverlay>,
) {
    if let Some(old) = overlay.0.take() { commands.entity(old).despawn(); }

    let active = match game_world.0.get_resource::<WorldOverviewState>() {
        Some(s) if s.active => true,
        _ => return,
    };

    let ov = match game_world.0.get_resource::<WorldOverviewState>() {
        Some(s) => s.clone(),
        None => return,
    };

    let mut lines = vec!["=== WORLD OVERVIEW ===".to_string()];
    lines.push(format!("Position: ({}, {})", ov.player_x, ov.player_y));
    lines.push(format!("Cursor: ({}, {})", ov.cursor_x, ov.cursor_y));
    lines.push("".to_string());

    // Show locations
    if let Some(lm) = game_world.0.get_resource::<LocationMap>() {
        if lm.locations.is_empty() {
            lines.push("No locations discovered.".to_string());
        } else {
            lines.push(format!("Locations ({}):", lm.locations.len()));
            for loc in &lm.locations {
                let dx = (ov.player_x as i32 - loc.x as i32).unsigned_abs();
                let dy = (ov.player_y as i32 - loc.y as i32).unsigned_abs();
                let dist = dx + dy;
                lines.push(format!("  {} ({}): @({},{}) — {} tiles away",
                    loc.name, loc.location_type, loc.x, loc.y, dist));
            }
        }
    }

    lines.push("".to_string());
    lines.push("[M] Close  |  Arrows: move cursor".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.8, 1.0, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
    )).id();
    overlay.0 = Some(root);
}
