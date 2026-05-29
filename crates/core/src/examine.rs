use bevy_ecs::prelude::*;

use crate::components::Position;

#[derive(Resource, Debug, Clone)]
pub struct ExamineMode {
    pub active: bool,
    pub cursor: Position,
}

impl ExamineMode {
    pub fn new() -> Self {
        Self {
            active: false,
            cursor: Position { x: 0, y: 0, z: 0 },
        }
    }

    pub fn set_cursor(&mut self, x: u32, y: u32) {
        self.cursor = Position { x, y, z: 0 };
    }
}

impl Default for ExamineMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examine_mode_new() {
        let mode = ExamineMode::new();
        assert!(!mode.active);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
    }

    #[test]
    fn examine_mode_default() {
        let mode = ExamineMode::default();
        assert!(!mode.active);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
    }

    #[test]
    fn examine_mode_set_cursor() {
        let mut mode = ExamineMode::new();
        mode.set_cursor(5, 10);
        assert_eq!(mode.cursor.x, 5);
        assert_eq!(mode.cursor.y, 10);
        mode.set_cursor(42, 99);
        assert_eq!(mode.cursor.x, 42);
        assert_eq!(mode.cursor.y, 99);
        mode.set_cursor(0, 0);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
    }

    #[test]
    fn examine_mode_active_toggle() {
        let mut mode = ExamineMode::new();
        assert!(!mode.active);
        mode.active = true;
        assert!(mode.active);
        mode.active = false;
        assert!(!mode.active);
    }
}
