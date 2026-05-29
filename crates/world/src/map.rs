use bevy_ecs::prelude::*;

use crate::seed::WorldSeed;
use crate::tile::TilePos;

#[derive(Resource, Debug, Clone)]
pub struct WorldMap {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub current_z: u32,
    pub seed: WorldSeed,
    pub tiles: Vec<Entity>,
}

impl WorldMap {
    pub fn get(&self, pos: TilePos) -> Option<Entity> {
        if pos.x < self.width && pos.y < self.height && pos.z == self.current_z {
            Some(self.tiles[pos.to_index(self.width)])
        } else {
            None
        }
    }

    pub fn get_unchecked(&self, pos: TilePos) -> Entity {
        self.tiles[pos.to_index(self.width)]
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = TilePos> {
        let (w, h) = (self.width, self.height);
        (0..h).flat_map(move |y| (0..w).map(move |x| TilePos::new(x, y)))
    }
}
