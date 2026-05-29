use bevy::prelude::*;
use game_core::screen::AppScreen;
use crate::ui::WorldGenParams;
use std::hash::{Hash, Hasher};

pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    screen: Res<State<AppScreen>>,
    mut next_screen: ResMut<NextState<AppScreen>>,
    mut params: ResMut<WorldGenParams>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
        return;
    }

    match screen.get() {
        AppScreen::MainMenu => handle_main_menu(&keyboard, &mut next_screen),
        AppScreen::CreateWorld => handle_create_world(&keyboard, &mut next_screen, &mut params),
        AppScreen::NewCharacter => handle_new_character(&keyboard, &mut next_screen),
        AppScreen::PauseMenu => {
            if keyboard.just_pressed(KeyCode::Escape) {
                if AppScreen::transition_allowed(&AppScreen::PauseMenu, &AppScreen::InWorld) {
                    next_screen.set(AppScreen::InWorld);
                }
            }
            if keyboard.just_pressed(KeyCode::KeyQ) {
                exit.write(AppExit::Success);
            }
        }
        AppScreen::Dead => {
            if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Escape) {
                if AppScreen::transition_allowed(&AppScreen::Dead, &AppScreen::MainMenu) {
                    next_screen.set(AppScreen::MainMenu);
                }
            }
        }
        _ => {}
    }
}

fn handle_main_menu(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        if AppScreen::transition_allowed(&AppScreen::MainMenu, &AppScreen::CreateWorld) {
            next_screen.set(AppScreen::CreateWorld);
        }
    }
}

fn handle_create_world(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
    params: &mut ResMut<WorldGenParams>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        if AppScreen::transition_allowed(&AppScreen::CreateWorld, &AppScreen::NewCharacter) {
            next_screen.set(AppScreen::NewCharacter);
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        if AppScreen::transition_allowed(&AppScreen::CreateWorld, &AppScreen::MainMenu) {
            next_screen.set(AppScreen::MainMenu);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyR) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.seed.hash(&mut hasher);
        params.seed = hasher.finish();
    }
}

fn handle_new_character(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        if AppScreen::transition_allowed(&AppScreen::NewCharacter, &AppScreen::InWorld) {
            next_screen.set(AppScreen::InWorld);
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        if AppScreen::transition_allowed(&AppScreen::NewCharacter, &AppScreen::CreateWorld) {
            next_screen.set(AppScreen::CreateWorld);
        }
    }
}
