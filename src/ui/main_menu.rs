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
                Text("CARAPACE".to_string()),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(0.0, 1.0, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
            ));
            parent.spawn((
                Text("Press ENTER to create a new world".to_string()),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));
            parent.spawn((
                Text("Q — Quit".to_string()),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
                Node { margin: UiRect::top(Val::Px(40.0)), ..default() },
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
