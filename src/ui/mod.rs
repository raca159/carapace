use bevy::prelude::*;
use game_core::screen::AppScreen;

mod create_world;
mod input;
mod main_menu;
mod new_character;

#[derive(Resource, Default)]
pub struct WorldGenParams {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Resource, Default)]
pub struct UiEntities {
    pub root: Option<Entity>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WorldGenParams>()
            .init_resource::<UiEntities>()
            .add_systems(Update, input::handle_keyboard_input)
            .add_systems(OnEnter(AppScreen::MainMenu), main_menu::spawn_ui)
            .add_systems(OnExit(AppScreen::MainMenu), main_menu::despawn_ui)
            .add_systems(OnEnter(AppScreen::CreateWorld), create_world::spawn_ui)
            .add_systems(OnExit(AppScreen::CreateWorld), create_world::despawn_ui)
            .add_systems(OnEnter(AppScreen::NewCharacter), new_character::spawn_ui)
            .add_systems(OnExit(AppScreen::NewCharacter), new_character::despawn_ui);
    }
}
