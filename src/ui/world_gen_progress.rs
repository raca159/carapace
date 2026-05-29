use bevy::prelude::*;

pub const TOTAL_STAGES: usize = 7;
pub const STAGE_NAMES: [&str; 7] = [
    "Configs", "Terrain", "Locations", "Economy",
    "Game Data", "Finalize", "Spawning",
];

#[derive(Resource, Default)]
pub struct WorldGenProgress {
    pub current_stage: usize,
    pub stage_timer: f32,
    pub done: bool,
}
