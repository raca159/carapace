use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorldOverviewMode {
    #[default]
    ReadOnly,
    SpawnSelection,
    InWorld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverviewTab {
    #[default]
    Minimap,
    History,
    CivSummary,
}

#[derive(Debug, Clone, Resource)]
pub struct WorldOverviewState {
    pub active: bool,
    pub mode: WorldOverviewMode,
    pub zoom: u32,
    pub pan_x: u32,
    pub pan_y: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
    pub player_x: u32,
    pub player_y: u32,
    pub tab: OverviewTab,
    pub history_scroll: usize,
    pub civ_scroll: usize,
}

impl Default for WorldOverviewState {
    fn default() -> Self {
        Self {
            active: false,
            mode: WorldOverviewMode::ReadOnly,
            zoom: 0,
            pan_x: 0,
            pan_y: 0,
            cursor_x: 0,
            cursor_y: 0,
            player_x: 0,
            player_y: 0,
            tab: OverviewTab::Minimap,
            history_scroll: 0,
            civ_scroll: 0,
        }
    }
}

impl WorldOverviewState {
    pub fn new(mode: WorldOverviewMode) -> Self {
        Self {
            active: true,
            mode,
            ..Default::default()
        }
    }

    pub fn zoom_in(&mut self) {
        if self.zoom < 2 {
            self.zoom += 1;
        }
    }

    pub fn zoom_out(&mut self) {
        if self.zoom > 0 {
            self.zoom -= 1;
        }
    }

    pub fn tiles_per_cell(&self) -> u32 {
        1u32 << (2 - self.zoom)
    }

    pub fn move_cursor_up(&mut self, _map_height: u32) {
        let step = self.tiles_per_cell();
        self.cursor_y = self.cursor_y.saturating_sub(step);
    }

    pub fn move_cursor_down(&mut self, map_height: u32) {
        let step = self.tiles_per_cell();
        self.cursor_y = (self.cursor_y + step).min(map_height.saturating_sub(1));
    }

    pub fn move_cursor_left(&mut self, _map_width: u32) {
        let step = self.tiles_per_cell();
        self.cursor_x = self.cursor_x.saturating_sub(step);
    }

    pub fn move_cursor_right(&mut self, map_width: u32) {
        let step = self.tiles_per_cell();
        self.cursor_x = (self.cursor_x + step).min(map_width.saturating_sub(1));
    }
}
