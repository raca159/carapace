#[derive(Debug, Clone)]
pub struct Camera {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub target_x: u32,
    pub target_y: u32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub scroll_speed: u32,
    pub lerp_speed: f32,
}

impl Camera {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            target_x: 0,
            target_y: 0,
            viewport_width,
            viewport_height,
            scroll_speed: 2,
            lerp_speed: 8.0,
        }
    }

    pub fn move_up(&mut self, _map_height: u32) {
        self.target_y = self.target_y.saturating_sub(self.scroll_speed);
        self.target_y = self.clamp_y(self.target_y, _map_height);
    }

    pub fn move_down(&mut self, map_height: u32) {
        self.target_y = self.clamp_y(self.target_y + self.scroll_speed, map_height);
    }

    pub fn move_left(&mut self, _map_width: u32) {
        self.target_x = self.target_x.saturating_sub(self.scroll_speed);
    }

    pub fn move_right(&mut self, map_width: u32) {
        self.target_x = self.clamp_x(self.target_x + self.scroll_speed, map_width);
    }

    pub fn centered_on(&mut self, x: u32, y: u32, map_width: u32, map_height: u32) {
        let tx = x.saturating_sub(self.viewport_width / 2);
        let ty = y.saturating_sub(self.viewport_height / 2);
        let tx = self.clamp_x(tx, map_width);
        let ty = self.clamp_y(ty, map_height);
        self.x = tx;
        self.y = ty;
        self.target_x = tx;
        self.target_y = ty;
    }

    pub fn lerp_to_target(&mut self, dt: f32) {
        if self.x == self.target_x && self.y == self.target_y {
            return;
        }
        let t = (self.lerp_speed * dt).min(1.0);
        let fx = self.x as f32 + (self.target_x as f32 - self.x as f32) * t;
        let fy = self.y as f32 + (self.target_y as f32 - self.y as f32) * t;
        self.x = fx.round() as u32;
        self.y = fy.round() as u32;
        if (self.x as f32 - self.target_x as f32).abs() < 0.5 {
            self.x = self.target_x;
        }
        if (self.y as f32 - self.target_y as f32).abs() < 0.5 {
            self.y = self.target_y;
        }
    }

    pub fn pan_to(&mut self, target: (u32, u32), map_width: u32, map_height: u32) {
        self.target_x = self.clamp_x(target.0, map_width);
        self.target_y = self.clamp_y(target.1, map_height);
    }

    pub fn follow_z(&mut self, player_z: u32) {
        self.z = player_z;
    }

    fn clamp_x(&self, x: u32, map_width: u32) -> u32 {
        x.min(map_width.saturating_sub(self.viewport_width))
    }

    fn clamp_y(&self, y: u32, map_height: u32) -> u32 {
        y.min(map_height.saturating_sub(self.viewport_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_starts_at_origin() {
        let cam = Camera::new(80, 24);
        assert_eq!(cam.x, 0);
        assert_eq!(cam.y, 0);
        assert_eq!(cam.target_x, 0);
        assert_eq!(cam.target_y, 0);
    }

    #[test]
    fn camera_lerp_to_target_moves_toward_target() {
        let mut cam = Camera::new(80, 24);
        cam.target_x = 100;
        cam.target_y = 50;
        cam.lerp_to_target(0.016);
        assert!(cam.x > 0);
        assert!(cam.y > 0);
        assert!(cam.x <= cam.target_x);
        assert!(cam.y <= cam.target_y);
    }

    #[test]
    fn camera_lerp_snaps_when_close() {
        let mut cam = Camera::new(80, 24);
        cam.x = 99;
        cam.y = 49;
        cam.target_x = 100;
        cam.target_y = 50;
        cam.lerp_to_target(0.1);
        assert_eq!(cam.x, 100);
        assert_eq!(cam.y, 50);
    }

    #[test]
    fn camera_move_right_clamped() {
        let mut cam = Camera::new(80, 24);
        for _ in 0..200 {
            cam.move_right(200);
        }
        cam.lerp_to_target(1.0);
        assert_eq!(cam.target_x, 200 - 80);
        assert_eq!(cam.x, 200 - 80);
    }

    #[test]
    fn camera_move_left_clamped_at_zero() {
        let mut cam = Camera::new(80, 24);
        cam.move_left(200);
        assert_eq!(cam.x, 0);
    }

    #[test]
    fn camera_centered_on() {
        let mut cam = Camera::new(80, 24);
        cam.centered_on(100, 100, 200, 200);
        assert_eq!(cam.x, 60);
        assert_eq!(cam.y, 88);
    }
}
