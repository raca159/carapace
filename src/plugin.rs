use bevy::prelude::*;
use crate::game::GamePlugin;
use crate::interact::InteractPlugin;
use crate::render::RenderPlugin;
use crate::ui::UiPlugin;

pub struct CarapacePlugin;

impl Plugin for CarapacePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(UiPlugin)
            .add_plugins(RenderPlugin)
            .add_plugins(GamePlugin)
            .add_plugins(InteractPlugin);
    }
}
