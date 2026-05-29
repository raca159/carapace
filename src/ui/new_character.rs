use bevy::prelude::*;
use crate::ui::{UiEntities, CharacterName};

pub fn spawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>, name: Res<CharacterName>) {
    let display_name = if name.0.is_empty() { "Adventurer".to_string() } else { name.0.clone() };
    let root = commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
    )).with_children(|parent| {
        parent.spawn((Text("CHARACTER CREATION".to_string()), TextFont { font_size: 24.0, ..default() }, TextColor(Color::srgb(0.0, 1.0, 1.0)), Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() }));
        parent.spawn((Text(format!("Name: {}", display_name)), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 1.0, 1.0)), Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() }));
        parent.spawn((Text("ENTER — Begin  |  ESC — Back".to_string()), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.5, 0.5, 0.5)), Node { margin: UiRect::top(Val::Px(30.0)), ..default() }));
        parent.spawn((Text("A-Z, Space, Backspace — Type your name".to_string()), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.3, 0.3, 0.3))));
    }).id();
    ui.root = Some(root);
}

pub fn despawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
    if let Some(root) = ui.root.take() { commands.entity(root).despawn(); }
}
