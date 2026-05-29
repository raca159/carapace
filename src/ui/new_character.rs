use bevy::prelude::*;
use crate::ui::UiEntities;

pub fn spawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text("CHARACTER CREATION".to_string()),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::srgb(0.0, 1.0, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
            ));
            parent.spawn((
                Text("Name: Adventurer".to_string()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
            ));
            parent.spawn((
                Text("Press ENTER to begin your journey  |  ESC to go back".to_string()),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Node { margin: UiRect::top(Val::Px(30.0)), ..default() },
            ));
        })
        .id();

    ui.root = Some(root);
}

pub fn despawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
    if let Some(root) = ui.root.take() {
        commands.entity(root).despawn();
    }
}
