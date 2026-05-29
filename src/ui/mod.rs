use bevy::prelude::*;
use game_core::screen::AppScreen;
use game_core::save::SaveGame;

mod create_world;
mod input;
mod main_menu;
mod new_character;
pub mod world_gen_progress;

#[derive(Resource)]
pub struct WorldGenParams {
    pub seed: u64,
    pub width: u32,
    pub height: u32,
}

impl Default for WorldGenParams {
    fn default() -> Self {
        Self {
            seed: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42),
            width: 200,
            height: 200,
        }
    }
}

#[derive(Resource, Default)]
pub struct CharacterName(pub String);

#[derive(Resource)]
pub struct MenuSelection {
    pub cursor: usize,
    pub items: Vec<&'static str>,
}

impl Default for MenuSelection {
    fn default() -> Self {
        Self {
            cursor: 0,
            items: vec!["New Game", "Load Game", "Quit"],
        }
    }
}

#[derive(Resource, Default)]
pub struct SaveFileList {
    pub files: Vec<String>,
    pub cursor: usize,
    pub error: Option<String>,
    pub selected_save: Option<Box<SaveGame>>,
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
            .init_resource::<CharacterName>()
            .init_resource::<MenuSelection>()
            .init_resource::<UiEntities>()
            .init_resource::<SaveFileList>()
            .add_systems(Update, input::handle_keyboard_input)
            .add_systems(OnEnter(AppScreen::MainMenu), main_menu::spawn_ui)
            .add_systems(OnExit(AppScreen::MainMenu), main_menu::despawn_ui)
            .add_systems(OnEnter(AppScreen::CreateWorld), create_world::spawn_ui)
            .add_systems(OnExit(AppScreen::CreateWorld), create_world::despawn_ui)
            .add_systems(OnEnter(AppScreen::NewCharacter), new_character::spawn_ui)
            .add_systems(OnExit(AppScreen::NewCharacter), new_character::despawn_ui)
            .add_systems(OnEnter(AppScreen::LoadGame), main_menu::spawn_load_game_ui)
            .add_systems(OnExit(AppScreen::LoadGame), main_menu::despawn_ui);
    }
}
