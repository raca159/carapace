use bevy::prelude::*;
use game_core::screen::AppScreen;
use crate::ui::{WorldGenParams, CharacterName, MenuSelection, SaveFileList};

fn key_to_digit(key: &KeyCode) -> Option<u32> {
    Some(match key {
        KeyCode::Digit0 => 0, KeyCode::Digit1 => 1, KeyCode::Digit2 => 2,
        KeyCode::Digit3 => 3, KeyCode::Digit4 => 4, KeyCode::Digit5 => 5,
        KeyCode::Digit6 => 6, KeyCode::Digit7 => 7, KeyCode::Digit8 => 8,
        KeyCode::Digit9 => 9,
        KeyCode::Numpad0 => 0, KeyCode::Numpad1 => 1, KeyCode::Numpad2 => 2,
        KeyCode::Numpad3 => 3, KeyCode::Numpad4 => 4, KeyCode::Numpad5 => 5,
        KeyCode::Numpad6 => 6, KeyCode::Numpad7 => 7, KeyCode::Numpad8 => 8,
        KeyCode::Numpad9 => 9,
        _ => return None,
    })
}

fn key_to_letter(key: &KeyCode) -> Option<char> {
    Some(match key {
        KeyCode::KeyA => 'A', KeyCode::KeyB => 'B', KeyCode::KeyC => 'C',
        KeyCode::KeyD => 'D', KeyCode::KeyE => 'E', KeyCode::KeyF => 'F',
        KeyCode::KeyG => 'G', KeyCode::KeyH => 'H', KeyCode::KeyI => 'I',
        KeyCode::KeyJ => 'J', KeyCode::KeyK => 'K', KeyCode::KeyL => 'L',
        KeyCode::KeyM => 'M', KeyCode::KeyN => 'N', KeyCode::KeyO => 'O',
        KeyCode::KeyP => 'P', KeyCode::KeyQ => 'Q', KeyCode::KeyR => 'R',
        KeyCode::KeyS => 'S', KeyCode::KeyT => 'T', KeyCode::KeyU => 'U',
        KeyCode::KeyV => 'V', KeyCode::KeyW => 'W', KeyCode::KeyX => 'X',
        KeyCode::KeyY => 'Y', KeyCode::KeyZ => 'Z',
        _ => return None,
    })
}

pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    screen: Res<State<AppScreen>>,
    mut next_screen: ResMut<NextState<AppScreen>>,
    mut params: ResMut<WorldGenParams>,
    mut name: ResMut<CharacterName>,
    mut menu: ResMut<MenuSelection>,
    mut saves: ResMut<SaveFileList>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
        return;
    }
    match screen.get() {
        AppScreen::MainMenu => handle_main_menu(&keyboard, &mut next_screen, &mut menu),
        AppScreen::LoadGame => handle_load_game(&keyboard, &mut next_screen, &mut saves),
        AppScreen::CreateWorld => handle_create_world(&keyboard, &mut next_screen, &mut params, &mut name),
        AppScreen::NewCharacter => handle_new_character(&keyboard, &mut next_screen, &mut name),
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
    menu: &mut ResMut<MenuSelection>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) && menu.cursor > 0 { menu.cursor -= 1; }
    if keyboard.just_pressed(KeyCode::ArrowDown) && menu.cursor + 1 < menu.items.len() { menu.cursor += 1; }
    if keyboard.just_pressed(KeyCode::Enter) {
        match menu.cursor {
            0 => { if AppScreen::transition_allowed(&AppScreen::MainMenu, &AppScreen::CreateWorld) { next_screen.set(AppScreen::CreateWorld); } }
            1 => { if AppScreen::transition_allowed(&AppScreen::MainMenu, &AppScreen::LoadGame) { next_screen.set(AppScreen::LoadGame); } }
            _ => {}
        }
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        if AppScreen::transition_allowed(&AppScreen::MainMenu, &AppScreen::LoadGame) { next_screen.set(AppScreen::LoadGame); }
    }
}

fn handle_load_game(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
    saves: &mut ResMut<SaveFileList>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        saves.error = None;
        if AppScreen::transition_allowed(&AppScreen::LoadGame, &AppScreen::MainMenu) { next_screen.set(AppScreen::MainMenu); }
        return;
    }
    if saves.files.is_empty() { return; }
    if keyboard.just_pressed(KeyCode::ArrowUp) && saves.cursor > 0 { saves.cursor -= 1; }
    if keyboard.just_pressed(KeyCode::ArrowDown) && saves.cursor + 1 < saves.files.len() { saves.cursor += 1; }
    if keyboard.just_pressed(KeyCode::Enter) {
        let path = std::path::PathBuf::from("saves").join(&saves.files[saves.cursor]);
        match game_core::save::load_game(&path) {
            Ok(save_data) => {
                saves.selected_save = Some(Box::new(save_data));
                if AppScreen::transition_allowed(&AppScreen::LoadGame, &AppScreen::InWorld) { next_screen.set(AppScreen::InWorld); }
            }
            Err(e) => { saves.error = Some(e); }
        }
    }
}

fn handle_create_world(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
    params: &mut ResMut<WorldGenParams>,
    name: &mut ResMut<CharacterName>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if AppScreen::transition_allowed(&AppScreen::CreateWorld, &AppScreen::MainMenu) { next_screen.set(AppScreen::MainMenu); }
        return;
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        if AppScreen::transition_allowed(&AppScreen::CreateWorld, &AppScreen::NewCharacter) {
            if name.0.is_empty() { name.0 = "Adventurer".to_string(); }
            next_screen.set(AppScreen::NewCharacter);
        }
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.seed.hash(&mut hasher);
        params.seed = hasher.finish();
        return;
    }
    if keyboard.just_pressed(KeyCode::Backspace) {
        let seed_str = params.seed.to_string();
        let new_str = seed_str.chars().take(seed_str.len().saturating_sub(1)).collect::<String>();
        params.seed = new_str.parse().unwrap_or(0);
        return;
    }
    for key in keyboard.get_just_pressed() {
        if let Some(d) = key_to_digit(key) {
            let seed_str = params.seed.to_string();
            if seed_str.len() < 20 {
                let new_str = format!("{}{}", seed_str, d);
                params.seed = new_str.parse().unwrap_or(0);
            }
            return;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) { params.width = (params.width.saturating_sub(10)).max(50); }
    if keyboard.just_pressed(KeyCode::ArrowRight) { params.width = (params.width + 10).min(500); }
    if keyboard.just_pressed(KeyCode::ArrowUp) { params.height = (params.height + 10).min(500); }
    if keyboard.just_pressed(KeyCode::ArrowDown) { params.height = (params.height.saturating_sub(10)).max(50); }
}

fn handle_new_character(
    keyboard: &Res<ButtonInput<KeyCode>>,
    next_screen: &mut ResMut<NextState<AppScreen>>,
    name: &mut ResMut<CharacterName>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if AppScreen::transition_allowed(&AppScreen::NewCharacter, &AppScreen::CreateWorld) { next_screen.set(AppScreen::CreateWorld); }
        return;
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        if AppScreen::transition_allowed(&AppScreen::NewCharacter, &AppScreen::InWorld) {
            if name.0.is_empty() { name.0 = "Adventurer".to_string(); }
            next_screen.set(AppScreen::InWorld);
        }
        return;
    }
    if keyboard.just_pressed(KeyCode::Backspace) { name.0.pop(); return; }
    if keyboard.just_pressed(KeyCode::Space) && name.0.len() < 24 { name.0.push(' '); return; }
    if name.0.len() < 24 {
        for key in keyboard.get_just_pressed() {
            if let Some(c) = key_to_letter(key) { name.0.push(c); return; }
        }
    }
}
