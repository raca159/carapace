use std::collections::HashMap;

use bevy_ecs::prelude::*;
use game_core::Player;

use crate::faction::{Faction, FactionId};
use game_core::Position;

const DEFAULT_CELL_SIZE: u32 = 8;

#[derive(Debug, Clone)]
pub struct SpatialEntry {
    pub entity: Entity,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub faction_id: Option<FactionId>,
    pub is_player: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct SpatialHashGrid {
    cell_size: u32,
    cells: HashMap<(i32, i32, i32), Vec<SpatialEntry>>,
}

impl SpatialHashGrid {
    pub fn new(cell_size: u32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn cell(&self, x: u32, y: u32, z: u32) -> (i32, i32, i32) {
        (
            (x / self.cell_size) as i32,
            (y / self.cell_size) as i32,
            (z / self.cell_size) as i32,
        )
    }

    pub fn rebuild(&mut self, world: &mut World) {
        self.cells.clear();
        let mut q = world.query::<(Entity, &Position, Option<&Faction>)>();
        for (entity, pos, faction) in q.iter(world) {
            let is_player = world.get::<Player>(entity).is_some();
            let cell = self.cell(pos.x, pos.y, pos.z);
            self.cells.entry(cell).or_default().push(SpatialEntry {
                entity,
                x: pos.x,
                y: pos.y,
                z: pos.z,
                faction_id: faction.map(|f| f.faction_id),
                is_player,
            });
        }
    }

    pub fn query_range(&self, x: u32, y: u32, z: u32, range: u32) -> Vec<SpatialEntry> {
        let cell_radius = (range / self.cell_size) as i32 + 1;
        let cell = self.cell(x, y, z);
        let mut result = Vec::new();

        for cdx in -cell_radius..=cell_radius {
            for cdy in -cell_radius..=cell_radius {
                for cdz in -cell_radius..=cell_radius {
                    let key = (cell.0 + cdx, cell.1 + cdy, cell.2 + cdz);
                    if let Some(entries) = self.cells.get(&key) {
                        for entry in entries {
                            let dx = x.abs_diff(entry.x);
                            let dy = y.abs_diff(entry.y);
                            let dz = z.abs_diff(entry.z);
                            if dx + dy + dz <= range {
                                result.push(entry.clone());
                            }
                        }
                    }
                }
            }
        }

        result
    }

    pub fn query_range_2d(&self, x: u32, y: u32, range: u32) -> Vec<SpatialEntry> {
        self.query_range(x, y, 0, range)
    }

    pub fn all_entries(&self) -> Vec<SpatialEntry> {
        self.cells.values().flat_map(|v| v.iter().cloned()).collect()
    }
}

impl Default for SpatialHashGrid {
    fn default() -> Self {
        Self::new(DEFAULT_CELL_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cell_size() {
        let grid = SpatialHashGrid::default();
        assert_eq!(grid.cell_size, 8);
    }

    #[test]
    fn test_new_custom_cell_size() {
        let grid = SpatialHashGrid::new(16);
        assert_eq!(grid.cell_size, 16);
    }

    #[test]
    fn test_rebuild_populates_grid() {
        let mut world = World::new();
        world.spawn((Position { x: 5, y: 10, z: 0 },));
        world.spawn((Position { x: 20, y: 30, z: 0 },));
        world.spawn((Position { x: 100, y: 200, z: 0 },)).insert(Player);

        let mut grid = SpatialHashGrid::default();
        grid.rebuild(&mut world);

        let all = grid.all_entries();
        assert_eq!(all.len(), 3);
        assert!(all.iter().any(|e| e.x == 5 && e.y == 10));
        assert!(all.iter().any(|e| e.x == 20 && e.y == 30));
        assert!(all.iter().any(|e| e.is_player));
    }

    #[test]
    fn test_query_range_exact_position() {
        let mut grid = SpatialHashGrid::default();
        let mut world = World::new();
        let e1 = world.spawn((Position { x: 5, y: 5, z: 0 },)).id();
        world.spawn((Position { x: 6, y: 5, z: 0 },));
        world.spawn((Position { x: 5, y: 6, z: 0 },));
        world.spawn((Position { x: 10, y: 10, z: 0 },));
        grid.rebuild(&mut world);

        let results = grid.query_range(5, 5, 0, 1);
        let entity_ids: Vec<_> = results.iter().map(|e| e.entity).collect();
        assert!(entity_ids.contains(&e1), "should include entity at query origin");
        assert_eq!(results.len(), 3, "radius 1 from (5,5) should include (5,5), (6,5), (5,6)");
    }

    #[test]
    fn test_query_range_excludes_distant() {
        let mut grid = SpatialHashGrid::default();
        let mut world = World::new();
        world.spawn((Position { x: 5, y: 5, z: 0 },));
        world.spawn((Position { x: 20, y: 20, z: 0 },));
        grid.rebuild(&mut world);

        let results = grid.query_range(5, 5, 0, 3);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].x, 5);
        assert_eq!(results[0].y, 5);
    }

    #[test]
    fn test_query_empty_grid() {
        let grid = SpatialHashGrid::default();
        assert!(grid.query_range(0, 0, 0, 100).is_empty());
        assert!(grid.all_entries().is_empty());
    }

    #[test]
    fn test_query_range_scales_with_radius() {
        let mut grid = SpatialHashGrid::default();
        let mut world = World::new();
        for x in 0..10u32 {
            for y in 0..10u32 {
                world.spawn((Position { x, y, z: 0 },));
            }
        }
        grid.rebuild(&mut world);

        let r1 = grid.query_range(5, 5, 0, 1).len();
        let r3 = grid.query_range(5, 5, 0, 3).len();
        let r5 = grid.query_range(5, 5, 0, 5).len();

        assert!(r1 < r3);
        assert!(r3 < r5);
        assert!(r1 >= 4); // (5,5) + 4 neighbors at radius 1
        assert!(r5 > r3); // larger radius includes more
    }

    #[test]
    fn test_cell_boundaries() {
        let mut grid = SpatialHashGrid::new(4);
        let mut world = World::new();
        world.spawn((Position { x: 3, y: 3, z: 0 },));
        world.spawn((Position { x: 4, y: 3, z: 0 },)); // adjacent, same row
        world.spawn((Position { x: 50, y: 50, z: 0 },)); // far away in different cell
        grid.rebuild(&mut world);

        // (3,3) and (4,3) are manhattan distance 1: both should be in range
        // (50,50) is manhattan distance 94 from (3,3) -> excluded
        assert_eq!(grid.query_range(3, 3, 0, 1).len(), 2);
    }

    #[test]
    fn test_all_entries_returns_all() {
        let mut grid = SpatialHashGrid::default();
        let mut world = World::new();
        world.spawn((Position { x: 1, y: 2, z: 0 },));
        world.spawn((Position { x: 100, y: 200, z: 0 },));
        grid.rebuild(&mut world);

        assert_eq!(grid.all_entries().len(), 2);
    }

    #[test]
    fn test_rebuild_clears_previous() {
        let mut grid = SpatialHashGrid::default();
        let mut world1 = World::new();
        world1.spawn((Position { x: 1, y: 1, z: 0 },));
        grid.rebuild(&mut world1);
        assert_eq!(grid.all_entries().len(), 1);

        let mut world2 = World::new();
        world2.spawn((Position { x: 10, y: 10, z: 0 },));
        world2.spawn((Position { x: 20, y: 20, z: 0 },));
        grid.rebuild(&mut world2);
        assert_eq!(grid.all_entries().len(), 2);
    }

    #[test]
    fn test_rebuild_with_faction() {
        use crate::faction::load_factions;

        let mut world = World::new();
        let factions_toml = r#"
[[faction]]
id = "test_faction"

[[faction]]
id = "player"

[[relationship]]
faction_a = "test_faction"
faction_b = "player"
standing = "neutral"
"#;
        let (_defs, rels) = load_factions(factions_toml).unwrap();
        let fid = rels.faction_id("test_faction").unwrap();

        world.spawn((Position { x: 0, y: 0, z: 0 }, Faction { faction_id: fid }));

        let mut grid = SpatialHashGrid::default();
        grid.rebuild(&mut world);

        let all = grid.all_entries();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].faction_id, Some(fid));
        assert!(!all[0].is_player);
    }
}
