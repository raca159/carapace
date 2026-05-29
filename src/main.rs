mod consume;
mod equipment;
mod event_format;
mod game;
mod interact;
mod location_entry;
mod render;
mod plugin;
mod reputation_sync;
mod status;
mod throw;
mod ui;
mod weather_pipeline;
mod world_gen;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Carapace".to_string(),
                resolution: (1280.0, 960.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(plugin::CarapacePlugin)
        .run();
}
