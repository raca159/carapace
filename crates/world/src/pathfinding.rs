use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy_ecs::prelude::*;
use game_tags::{TagId, Tags};

use crate::map::WorldMap;
use crate::tile::TilePos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    pos: (u32, u32),
    g: u32,
    h: u32,
    f: u32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.h.cmp(&self.h))
            .then_with(|| self.pos.cmp(&other.pos))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan(x1: u32, y1: u32, x2: u32, y2: u32) -> u32 {
    x1.abs_diff(x2) + y1.abs_diff(y2)
}

#[allow(clippy::too_many_arguments)]
fn is_tile_passable_for_path(
    x: u32,
    y: u32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
    start: (u32, u32),
    goal: (u32, u32),
) -> bool {
    if x >= map.width || y >= map.height {
        return false;
    }

    if (x, y) != start && occupied.contains(&(x, y)) && (x, y) != goal {
        return false;
    }

    if flight_id.is_some_and(|id| creature_tags.has(id)) {
        return true;
    }

    let tile_entity = match map.get(TilePos::new(x, y)) {
        Some(e) => e,
        None => return false,
    };

    let tile_tags = match world.get::<Tags>(tile_entity) {
        Some(t) => t,
        None => return true,
    };

    if blocked_id.is_some_and(|id| tile_tags.has(id)) {
        return false;
    }

    if swimmable_id.is_some_and(|id| tile_tags.has(id))
        && !aquatic_id.is_some_and(|id| creature_tags.has(id))
    {
        return false;
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn a_star_step(
    start: (u32, u32),
    goal: (u32, u32),
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
) -> Option<(i32, i32)> {
    if start == goal {
        return Some((0, 0));
    }

    if goal.0 >= map.width || goal.1 >= map.height {
        return None;
    }

    let mut open = BinaryHeap::new();
    let mut g_scores: HashMap<(u32, u32), u32> = HashMap::new();
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();

    let h = manhattan(start.0, start.1, goal.0, goal.1);
    open.push(Node {
        pos: start,
        g: 0,
        h,
        f: h,
    });
    g_scores.insert(start, 0);

    let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    while let Some(current) = open.pop() {
        if current.pos == goal {
            let mut pos = goal;
            while let Some(&prev) = came_from.get(&pos) {
                if prev == start {
                    let dx = pos.0 as i32 - start.0 as i32;
                    let dy = pos.1 as i32 - start.1 as i32;
                    return Some((dx, dy));
                }
                pos = prev;
            }
            return None;
        }

        let current_g = match g_scores.get(&current.pos) {
            Some(&g) => g,
            None => continue,
        };

        for &(dx, dy) in &dirs {
            let nx = current.pos.0 as i32 + dx;
            let ny = current.pos.1 as i32 + dy;
            if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
                continue;
            }
            let nx = nx as u32;
            let ny = ny as u32;

            if !is_tile_passable_for_path(
                nx, ny, map, world, creature_tags, occupied, blocked_id, swimmable_id, flight_id,
                aquatic_id, start, goal,
            ) {
                continue;
            }

            let new_g = current_g + 1;
            let should_insert = match g_scores.get(&(nx, ny)) {
                Some(&existing) => new_g < existing,
                None => true,
            };

            if should_insert {
                g_scores.insert((nx, ny), new_g);
                let h = manhattan(nx, ny, goal.0, goal.1);
                open.push(Node {
                    pos: (nx, ny),
                    g: new_g,
                    h,
                    f: new_g + h,
                });
                came_from.insert((nx, ny), current.pos);
            }
        }
    }

    None
}

pub fn has_line_of_sight(
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    map: &WorldMap,
    world: &World,
    blocked_id: Option<TagId>,
) -> bool {
    if x1 == x2 && y1 == y2 {
        return true;
    }

    let blocked_id = match blocked_id {
        Some(id) => id,
        None => return true,
    };

    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut x = x1 as i32;
    let mut y = y1 as i32;

    loop {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }

        if x == x2 as i32 && y == y2 as i32 {
            break;
        }

        let nx = x as u32;
        let ny = y as u32;

        if nx >= map.width || ny >= map.height {
            return false;
        }

        if let Some(tile_entity) = map.get(TilePos::new(nx, ny))
            && let Some(tile_tags) = world.get::<Tags>(tile_entity)
            && tile_tags.has(blocked_id)
        {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::WorldSeed;
    use game_tags::{TagRegistry, load_tag_registry};

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    fn make_flat_map(
        world: &mut World,
        width: u32,
        height: u32,
        registry: &TagRegistry,
    ) -> WorldMap {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let tile = crate::tile::Tile {
                    pos: TilePos::new(x, y),
                    elevation: 0.5,
                    moisture: 0.5,
                    temperature: 0.5,
                    biome_name: "plains".to_string(),
                    glyph: '.',
                    color: (200, 200, 200),
                };
                let tags = Tags::new(registry.tag_count());
                let entity = world.spawn((tile, tags)).id();
                tiles.push(entity);
            }
        }
        WorldMap {
            width,
            height,
            depth: 1,
            current_z: 0,
            seed: WorldSeed::from_value(42),
            tiles,
        }
    }

    fn add_blocked_tile(world: &mut World, x: u32, y: u32) {
        let registry = world.resource::<TagRegistry>().clone();
        let blocked_id = registry.tag_id("BLOCKED").unwrap();
        let tile_entity = {
            let map = world.resource::<WorldMap>();
            map.get(TilePos::new(x, y)).unwrap()
        };
        let mut tags = world.get::<Tags>(tile_entity).unwrap().clone();
        tags.add_tag(blocked_id, game_tags::TagValue::None, &registry);
        if let Some(mut t) = world.get_mut::<Tags>(tile_entity) {
            *t = tags;
        }
    }

    fn setup_world(width: u32, height: u32) -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry.clone());

        let map = make_flat_map(&mut world, width, height, &registry);
        world.insert_resource(map);

        world
    }

    #[test]
    fn test_a_star_straight_line() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (1, 5),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert_eq!(step, Some((1, 0)));
    }

    #[test]
    fn test_a_star_vertical_line() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (5, 1),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert_eq!(step, Some((0, 1)));
    }

    #[test]
    fn test_a_star_around_wall() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 3, 5);
        add_blocked_tile(&mut world, 4, 5);
        add_blocked_tile(&mut world, 5, 5);

        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (1, 5),
            (7, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert!(step.is_some(), "A* should find a path around the wall");
        let (dx, _) = step.unwrap();
        let _ = dx;
    }

    #[test]
    fn test_a_star_blocked_goal() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 5, 5);

        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (1, 5),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert_eq!(step, None, "should return None when goal is blocked");
    }

    #[test]
    fn test_a_star_out_of_bounds_goal() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (5, 5),
            (20, 20),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert_eq!(step, None);
    }

    #[test]
    fn test_a_star_same_position() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (5, 5),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert_eq!(step, Some((0, 0)));
    }

    #[test]
    fn test_a_stav_avoids_occupied() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let mut occupied = HashSet::new();
        occupied.insert((2, 5));

        let step = a_star_step(
            (1, 5),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert!(step.is_some());
        let (dx, dy) = step.unwrap();
        assert!(!(dx == 1 && dy == 0), "should avoid occupied tile (2,5)");
    }

    #[test]
    fn test_a_star_start_not_blocked_by_own_position() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let mut occupied = HashSet::new();
        occupied.insert((1, 5));
        occupied.insert((2, 5));

        let step = a_star_step(
            (1, 5),
            (5, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert!(
            step.is_some(),
            "A* should not consider own position as blocked"
        );
    }

    #[test]
    fn test_a_star_partial_wall_forces_detour() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 3, 4);
        add_blocked_tile(&mut world, 3, 5);
        add_blocked_tile(&mut world, 3, 6);

        let map = world.resource::<WorldMap>().clone();
        let tags = Tags::new(registry.tag_count());
        let occupied = HashSet::new();

        let step = a_star_step(
            (1, 5),
            (7, 5),
            &map,
            &world,
            &tags,
            &occupied,
            registry.tag_id("BLOCKED"),
            registry.tag_id("SWIMMABLE"),
            registry.tag_id("FLIGHT"),
            registry.tag_id("AQUATIC"),
        );

        assert!(step.is_some(), "A* should find detour around wall");
    }

    #[test]
    fn test_has_line_of_sight_clear() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(
            1,
            5,
            5,
            5,
            &map,
            &world,
            registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_blocked() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 3, 5);

        let map = world.resource::<WorldMap>().clone();

        assert!(!has_line_of_sight(
            1,
            5,
            5,
            5,
            &map,
            &world,
            registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_diagonal() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(
            0, 0, 3, 3, &map, &world, registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_diagonal_blocked() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 2, 2);

        let map = world.resource::<WorldMap>().clone();

        assert!(!has_line_of_sight(
            0, 0, 3, 3, &map, &world, registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_adjacent() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(
            5, 5, 6, 5, &map, &world, registry.tag_id("BLOCKED"),
        ));
        assert!(has_line_of_sight(
            5, 5, 5, 6, &map, &world, registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_same_tile() {
        let world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();
        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(
            5, 5, 5, 5, &map, &world, registry.tag_id("BLOCKED"),
        ));
    }

    #[test]
    fn test_has_line_of_sight_no_blocked_tag() {
        let world = setup_world(10, 10);
        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(1, 5, 5, 5, &map, &world, None));
    }

    #[test]
    fn test_has_line_of_sight_blocked_diagonal_not_on_line() {
        let mut world = setup_world(10, 10);
        let registry = world.resource::<TagRegistry>().clone();

        add_blocked_tile(&mut world, 5, 5);

        let map = world.resource::<WorldMap>().clone();

        assert!(has_line_of_sight(
            0, 0, 0, 7, &map, &world, registry.tag_id("BLOCKED"),
        ));
    }
}
