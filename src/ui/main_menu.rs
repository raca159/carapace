use bevy::prelude::*;
use crate::ui::{UiEntities, MenuSelection, SaveFileList};

pub fn spawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>, menu: Res<MenuSelection>) {
    let root = commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
    )).with_children(|parent| {
        parent.spawn((Text("CARAPACE".to_string()), TextFont { font_size: 36.0, ..default() }, TextColor(Color::srgb(0.0, 1.0, 1.0)), Node { margin: UiRect::bottom(Val::Px(50.0)), ..default() }));
        for (i, item) in menu.items.iter().enumerate() {
            let selected = i == menu.cursor;
            let color = if selected { Color::srgb(1.0, 1.0, 1.0) } else { Color::srgb(0.6, 0.6, 0.6) };
            let prefix = if selected { "> " } else { "  " };
            parent.spawn((Text(format!("{}{}", prefix, item)), TextFont { font_size: 20.0, ..default() }, TextColor(color), Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() }));
        }
        parent.spawn((Text("↑↓ Navigate  |  Enter Select  |  Q Quit".to_string()), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.3, 0.3, 0.3)), Node { margin: UiRect::top(Val::Px(50.0)), ..default() }));
    }).id();
    ui.root = Some(root);
}

pub fn spawn_load_game_ui(mut commands: Commands, mut ui: ResMut<UiEntities>, mut saves: ResMut<SaveFileList>) {
    saves.files = game_core::save::list_saves();
    saves.cursor = 0;
    saves.error = None;
    let root = commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
    )).with_children(|parent| {
        parent.spawn((Text("LOAD GAME".to_string()), TextFont { font_size: 28.0, ..default() }, TextColor(Color::srgb(0.0, 1.0, 1.0)), Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() }));
        if saves.files.is_empty() {
            parent.spawn((Text("No save files found.".to_string()), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.8))));
        } else {
            for (i, file) in saves.files.iter().enumerate() {
                let selected = i == saves.cursor;
                let color = if selected { Color::srgb(1.0, 1.0, 1.0) } else { Color::srgb(0.6, 0.6, 0.6) };
                let prefix = if selected { "> " } else { "  " };
                parent.spawn((Text(format!("{}{}", prefix, file)), TextFont { font_size: 16.0, ..default() }, TextColor(color), Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() }));
            }
        }
        if let Some(ref err) = saves.error { parent.spawn((Text(format!("Error: {}", err)), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(1.0, 0.3, 0.3)), Node { margin: UiRect::top(Val::Px(16.0)), ..default() })); }
        parent.spawn((Text("↑↓ Navigate  |  Enter Load  |  Esc Back".to_string()), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.3, 0.3, 0.3)), Node { margin: UiRect::top(Val::Px(30.0)), ..default() }));
    }).id();
    ui.root = Some(root);
}

pub fn despawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
    if let Some(root) = ui.root.take() { commands.entity(root).despawn(); }
}
