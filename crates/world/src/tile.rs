use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl TilePos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y, z: 0 }
    }

    pub fn new_3d(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn to_index(&self, width: u32) -> usize {
        (self.y * width + self.x) as usize
    }

    pub fn to_index_3d(&self, width: u32, depth: u32) -> usize {
        (self.z * width * depth + self.y * width + self.x) as usize
    }
}

#[derive(Component, Debug, Clone)]
pub struct Tile {
    pub pos: TilePos,
    pub elevation: f32,
    pub moisture: f32,
    pub temperature: f32,
    pub biome_name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_pos_index() {
        let pos = TilePos::new(3, 2);
        assert_eq!(pos.to_index(10), 23);
    }
}
