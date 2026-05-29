use bevy_ecs::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerStats {
    pub enemies_defeated: u32,
    pub items_collected: u32,
}
