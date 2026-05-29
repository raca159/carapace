use game_world::{TilePos, WorldMap};

pub struct Camera {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub scroll_speed: u32,
}

impl Camera {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            viewport_width,
            viewport_height,
            scroll_speed: 2,
        }
    }

    pub fn move_up(&mut self, _map: &WorldMap) {
        self.y = self.y.saturating_sub(self.scroll_speed);
    }

    pub fn move_down(&mut self, map: &WorldMap) {
        self.y = (self.y + self.scroll_speed).min(map.height.saturating_sub(self.viewport_height));
    }

    pub fn move_left(&mut self, _map: &WorldMap) {
        self.x = self.x.saturating_sub(self.scroll_speed);
    }

    pub fn move_right(&mut self, map: &WorldMap) {
        self.x = (self.x + self.scroll_speed).min(map.width.saturating_sub(self.viewport_width));
    }

    pub fn centered_on(&mut self, pos: TilePos, map: &WorldMap) {
        self.x = pos.x.saturating_sub(self.viewport_width / 2);
        self.y = pos.y.saturating_sub(self.viewport_height / 2);
        self.clamp(map);
    }

    fn clamp(&mut self, map: &WorldMap) {
        self.x = self.x.min(map.width.saturating_sub(self.viewport_width));
        self.y = self.y.min(map.height.saturating_sub(self.viewport_height));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    fn make_map(width: u32, height: u32) -> WorldMap {
        WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: game_world::WorldSeed(0),
            tiles: (0..width * height)
                .map(Entity::from_raw)
                .collect(),
        }
    }

    #[test]
    fn camera_starts_at_origin() {
        let cam = Camera::new(80, 24);
        assert_eq!(cam.x, 0);
        assert_eq!(cam.y, 0);
    }

    #[test]
    fn camera_move_right_clamped() {
        let map = make_map(200, 200);
        let mut cam = Camera::new(80, 24);
        for _ in 0..200 {
            cam.move_right(&map);
        }
        assert_eq!(cam.x, 200 - 80);
    }

    #[test]
    fn camera_move_left_clamped_at_zero() {
        let map = make_map(200, 200);
        let mut cam = Camera::new(80, 24);
        cam.move_left(&map);
        assert_eq!(cam.x, 0);
    }

    #[test]
    fn camera_centered_on() {
        let map = make_map(200, 200);
        let mut cam = Camera::new(80, 24);
        cam.centered_on(TilePos::new(100, 100), &map);
        assert_eq!(cam.x, 60);
        assert_eq!(cam.y, 88);
    }
}
