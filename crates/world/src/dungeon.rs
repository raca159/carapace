use bevy_ecs::prelude::*;
use rand::SeedableRng;
use rand::Rng;
use rand::rngs::StdRng;

use crate::tile::{Tile, TilePos};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DungeonType {
    Crypt,
    CaveSystem,
    Sewer,
    FloodedVault,
}

impl DungeonType {
    pub fn from_biome(biome: &str) -> Option<Self> {
        match biome {
            "AncientVault" => Some(DungeonType::Crypt),
            "SubterraneanRift" => Some(DungeonType::CaveSystem),
            "RuinedCity" => Some(DungeonType::Sewer),
            "DeepTrench" => Some(DungeonType::FloodedVault),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            DungeonType::Crypt => "Ancient Vault",
            DungeonType::CaveSystem => "Cave System",
            DungeonType::Sewer => "Sewer",
            DungeonType::FloodedVault => "Flooded Vault",
        }
    }

    pub fn wall_glyph(&self) -> char {
        match self {
            DungeonType::Crypt => '#',
            DungeonType::CaveSystem => '#',
            DungeonType::Sewer => '#',
            DungeonType::FloodedVault => '#',
        }
    }

    pub fn wall_color(&self) -> (u8, u8, u8) {
        match self {
            DungeonType::Crypt => (80, 70, 90),
            DungeonType::CaveSystem => (90, 80, 70),
            DungeonType::Sewer => (60, 80, 60),
            DungeonType::FloodedVault => (40, 60, 100),
        }
    }

    pub fn floor_glyph(&self) -> char {
        match self {
            DungeonType::Crypt => '.',
            DungeonType::CaveSystem => '.',
            DungeonType::Sewer => '.',
            DungeonType::FloodedVault => '.',
        }
    }

    pub fn floor_color(&self) -> (u8, u8, u8) {
        match self {
            DungeonType::Crypt => (140, 130, 120),
            DungeonType::CaveSystem => (130, 120, 110),
            DungeonType::Sewer => (120, 130, 110),
            DungeonType::FloodedVault => (100, 120, 150),
        }
    }

    pub fn corridor_glyph(&self) -> char {
        ','
    }

    pub fn corridor_color(&self) -> (u8, u8, u8) {
        (110, 100, 90)
    }

    pub fn enemy_tags(&self) -> &[&str] {
        match self {
            DungeonType::Crypt => &["UNDEAD", "CONSTRUCT", "AUTOMATON"],
            DungeonType::CaveSystem => &["CRUSTACEAN", "BEAST", "INSECT"],
            DungeonType::Sewer => &["VAMPIRE", "CULTIST", "RAT"],
            DungeonType::FloodedVault => &["AQUATIC", "ABERRATION"],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DungeonConfig {
    pub width: u32,
    pub height: u32,
    pub min_room_size: u32,
    pub max_room_size: u32,
    pub target_room_count: u32,
    pub corridor_width: u32,
    pub enemy_density: f32,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            width: 50,
            height: 40,
            min_room_size: 4,
            max_room_size: 10,
            target_room_count: 6,
            corridor_width: 1,
            enemy_density: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RoomRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl RoomRect {
    pub fn center(&self) -> (u32, u32) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn intersects(&self, other: &RoomRect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn area(&self) -> u32 {
        self.w * self.h
    }
}

#[derive(Debug, Clone)]
pub struct DungeonMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<DungeonTile>,
    pub rooms: Vec<RoomRect>,
    pub entrance: (u32, u32),
    pub dungeon_type: DungeonType,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonTileType {
    Wall,
    Floor,
    Corridor,
    EntranceStair,
    DeeperStair,
}

impl DungeonTileType {
    pub fn glyph(&self) -> char {
        match self {
            DungeonTileType::Wall => '#',
            DungeonTileType::Floor => '.',
            DungeonTileType::Corridor => '.',
            DungeonTileType::EntranceStair => '<',
            DungeonTileType::DeeperStair => '>',
        }
    }
}

#[derive(Debug, Clone)]
pub struct DungeonTile {
    pub pos: TilePos,
    pub tile_type: DungeonTileType,
    pub dungeon_type: DungeonType,
}

impl DungeonTile {
    pub fn glyph(&self) -> char {
        match self.tile_type {
            DungeonTileType::Wall => self.dungeon_type.wall_glyph(),
            DungeonTileType::Floor => self.dungeon_type.floor_glyph(),
            DungeonTileType::Corridor => self.dungeon_type.corridor_glyph(),
            DungeonTileType::EntranceStair => '<',
            DungeonTileType::DeeperStair => '>',
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self.tile_type {
            DungeonTileType::Wall => self.dungeon_type.wall_color(),
            DungeonTileType::Floor => self.dungeon_type.floor_color(),
            DungeonTileType::Corridor => self.dungeon_type.corridor_color(),
            DungeonTileType::EntranceStair => (0, 220, 220),
            DungeonTileType::DeeperStair => (220, 50, 50),
        }
    }

    pub fn to_tile(&self) -> Tile {
        Tile {
            pos: self.pos,
            elevation: match self.tile_type {
                DungeonTileType::Wall => 0.0,
                _ => 0.5,
            },
            moisture: 0.0,
            temperature: 0.5,
            biome_name: self.dungeon_type.name().to_string(),
            glyph: self.glyph(),
            color: self.color(),
        }
    }
}

#[derive(Debug, Clone)]
struct BspNode {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    left: Option<Box<BspNode>>,
    right: Option<Box<BspNode>>,
    room: Option<RoomRect>,
}

impl BspNode {
    fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            left: None,
            right: None,
            room: None,
        }
    }

    fn split(&mut self, rng: &mut StdRng, min_size: u32, depth: u32, max_depth: u32) {
        if depth >= max_depth {
            return;
        }

        let can_split_h = self.h >= min_size * 2;
        let can_split_v = self.w >= min_size * 2;

        if !can_split_h && !can_split_v {
            return;
        }

        let split_horizontal = if can_split_h && can_split_v {
            rng.random_bool(0.5)
        } else {
            can_split_h
        };

        if split_horizontal {
            let split_at = min_size + rng.random_range(0..(self.h - min_size * 2).max(1));
            let mut left = BspNode::new(self.x, self.y, self.w, split_at);
            let mut right = BspNode::new(self.x, self.y + split_at, self.w, self.h - split_at);
            left.split(rng, min_size, depth + 1, max_depth);
            right.split(rng, min_size, depth + 1, max_depth);
            self.left = Some(Box::new(left));
            self.right = Some(Box::new(right));
        } else {
            let split_at = min_size + rng.random_range(0..(self.w - min_size * 2).max(1));
            let mut left = BspNode::new(self.x, self.y, split_at, self.h);
            let mut right = BspNode::new(self.x + split_at, self.y, self.w - split_at, self.h);
            left.split(rng, min_size, depth + 1, max_depth);
            right.split(rng, min_size, depth + 1, max_depth);
            self.left = Some(Box::new(left));
            self.right = Some(Box::new(right));
        }
    }

    fn place_rooms(&mut self, rng: &mut StdRng, config: &DungeonConfig) {
        if let (Some(left), Some(right)) = (&mut self.left, &mut self.right) {
            left.place_rooms(rng, config);
            right.place_rooms(rng, config);
        } else {
            let margin = 1;
            let max_w = (self.w.saturating_sub(margin * 2)).min(config.max_room_size);
            let max_h = (self.h.saturating_sub(margin * 2)).min(config.max_room_size);

            if max_w < config.min_room_size || max_h < config.min_room_size {
                return;
            }

            let room_w = rng.random_range(config.min_room_size..=max_w);
            let room_h = rng.random_range(config.min_room_size..=max_h);
            let room_x = self.x + margin + rng.random_range(0..=(self.w.saturating_sub(room_w + margin * 2)));
            let room_y = self.y + margin + rng.random_range(0..=(self.h.saturating_sub(room_h + margin * 2)));

            self.room = Some(RoomRect {
                x: room_x,
                y: room_y,
                w: room_w,
                h: room_h,
            });
        }
    }

    fn collect_rooms(&self) -> Vec<RoomRect> {
        let mut rooms = Vec::new();
        if let Some(room) = &self.room {
            rooms.push(*room);
        }
        if let Some(left) = &self.left {
            rooms.extend(left.collect_rooms());
        }
        if let Some(right) = &self.right {
            rooms.extend(right.collect_rooms());
        }
        rooms
    }

    fn get_connections(&self) -> Vec<((u32, u32), (u32, u32))> {
        let mut connections = Vec::new();
        if let (Some(left), Some(right)) = (&self.left, &self.right) {
            let left_rooms = left.collect_rooms();
            let right_rooms = right.collect_rooms();
            if let (Some(lr), Some(rr)) = (left_rooms.first(), right_rooms.first()) {
                connections.push((lr.center(), rr.center()));
            }
            connections.extend(left.get_connections());
            connections.extend(right.get_connections());
        }
        connections
    }
}

#[derive(Debug, Clone)]
pub struct ActiveInterior {
    pub location_id: usize,
    pub location_type: String,
    pub interior_tags: Vec<game_tags::TagId>,
    pub environment: std::collections::HashMap<String, u32>,
    pub depth_range: Option<[u32; 2]>,
    pub saved_world_map: crate::map::WorldMap,
    pub saved_player_pos: (u32, u32),
}

#[derive(Resource, Default, Clone)]
pub struct MapLayer {
    pub active_interior: Option<ActiveInterior>,
    pub depth: u32,
}


pub fn generate_dungeon(
    config: &DungeonConfig,
    dungeon_type: DungeonType,
    seed: u64,
) -> DungeonMap {
    let mut rng = StdRng::seed_from_u64(seed);

    let mut root = BspNode::new(0, 0, config.width, config.height);
    let max_depth = (config.target_room_count as f32).log2().ceil() as u32 + 1;
    root.split(&mut rng, config.min_room_size, 0, max_depth);
    root.place_rooms(&mut rng, config);

    let rooms = root.collect_rooms();
    let connections = root.get_connections();

    let total = (config.width * config.height) as usize;
    let mut tiles = vec![DungeonTileType::Wall; total];

    for room in &rooms {
        for dy in 0..room.h {
            for dx in 0..room.w {
                let x = room.x + dx;
                let y = room.y + dy;
                if x < config.width && y < config.height {
                    tiles[(y * config.width + x) as usize] = DungeonTileType::Floor;
                }
            }
        }
    }

    for (start, end) in &connections {
        carve_corridor(&mut tiles, *start, *end, config.width, config.height);
    }

    let entrance = rooms.first()
        .map(|r| r.center())
        .unwrap_or((config.width / 2, config.height / 2));

    tiles[(entrance.1 * config.width + entrance.0) as usize] = DungeonTileType::EntranceStair;

    if rooms.len() > 1 {
        let last_room = rooms.last().unwrap();
        let deeper = last_room.center();
        tiles[(deeper.1 * config.width + deeper.0) as usize] = DungeonTileType::DeeperStair;
    }

    let dungeon_tiles: Vec<DungeonTile> = tiles
        .into_iter()
        .enumerate()
        .map(|(i, tt)| {
            let x = i as u32 % config.width;
            let y = i as u32 / config.width;
            DungeonTile {
                pos: TilePos::new(x, y),
                tile_type: tt,
                dungeon_type,
            }
        })
        .collect();

    DungeonMap {
        width: config.width,
        height: config.height,
        tiles: dungeon_tiles,
        rooms,
        entrance,
        dungeon_type,
        seed,
    }
}

fn carve_corridor(
    tiles: &mut [DungeonTileType],
    start: (u32, u32),
    end: (u32, u32),
    width: u32,
    height: u32,
) {
    let (mut x, mut y) = (start.0, start.1);
    let (tx, ty) = (end.0, end.1);

    while x != tx {
        if x < width && y < height {
            let idx = (y * width + x) as usize;
            if tiles[idx] == DungeonTileType::Wall {
                tiles[idx] = DungeonTileType::Corridor;
            }
        }
        if x < tx { x += 1; } else { x = x.saturating_sub(1); }
    }
    while y != ty {
        if x < width && y < height {
            let idx = (y * width + x) as usize;
            if tiles[idx] == DungeonTileType::Wall {
                tiles[idx] = DungeonTileType::Corridor;
            }
        }
        if y < ty { y += 1; } else { y = y.saturating_sub(1); }
    }
}

pub fn dungeon_spawn_positions(dungeon: &DungeonMap, rng: &mut impl Rng) -> Vec<(u32, u32)> {
    let mut positions = Vec::new();
    for room in &dungeon.rooms {
        let area = room.area() as f32;
        let count = (area * 0.3 * rng.random::<f32>()) as u32;
        for _ in 0..count {
            let x = room.x + rng.random_range(0..room.w);
            let y = room.y + rng.random_range(0..room.h);
            let idx = (y * dungeon.width + x) as usize;
            if let Some(tile) = dungeon.tiles.get(idx)
                && tile.tile_type == DungeonTileType::Floor {
                    positions.push((x, y));
                }
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_type_from_biome() {
        assert_eq!(DungeonType::from_biome("AncientVault"), Some(DungeonType::Crypt));
        assert_eq!(DungeonType::from_biome("SubterraneanRift"), Some(DungeonType::CaveSystem));
        assert_eq!(DungeonType::from_biome("RuinedCity"), Some(DungeonType::Sewer));
        assert_eq!(DungeonType::from_biome("DeepTrench"), Some(DungeonType::FloodedVault));
        assert_eq!(DungeonType::from_biome("Plains"), None);
    }

    #[test]
    fn dungeon_type_names() {
        assert_eq!(DungeonType::Crypt.name(), "Ancient Vault");
        assert_eq!(DungeonType::CaveSystem.name(), "Cave System");
        assert_eq!(DungeonType::Sewer.name(), "Sewer");
        assert_eq!(DungeonType::FloodedVault.name(), "Flooded Vault");
    }

    #[test]
    fn dungeon_type_enemy_tags() {
        let tags = DungeonType::Crypt.enemy_tags();
        assert!(tags.contains(&"UNDEAD"));
        let tags = DungeonType::CaveSystem.enemy_tags();
        assert!(tags.contains(&"CRUSTACEAN"));
    }

    #[test]
    fn room_rect_center() {
        let room = RoomRect { x: 10, y: 20, w: 8, h: 6 };
        assert_eq!(room.center(), (14, 23));
    }

    #[test]
    fn room_rect_intersects() {
        let a = RoomRect { x: 0, y: 0, w: 5, h: 5 };
        let b = RoomRect { x: 3, y: 3, w: 5, h: 5 };
        let c = RoomRect { x: 10, y: 10, w: 5, h: 5 };
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn generate_dungeon_creates_rooms() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        assert!(!dungeon.rooms.is_empty());
        assert!(dungeon.rooms.len() >= 2);
    }

    #[test]
    fn generate_dungeon_has_entrance_stair() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let entrance_tile = &dungeon.tiles[(dungeon.entrance.1 * dungeon.width + dungeon.entrance.0) as usize];
        assert_eq!(entrance_tile.tile_type, DungeonTileType::EntranceStair);
        assert_eq!(entrance_tile.glyph(), '<');
    }

    #[test]
    fn generate_dungeon_has_deeper_stair() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let has_deeper = dungeon.tiles.iter().any(|t| t.tile_type == DungeonTileType::DeeperStair);
        assert!(has_deeper);
    }

    #[test]
    fn generate_dungeon_deterministic() {
        let config = DungeonConfig::default();
        let d1 = generate_dungeon(&config, DungeonType::Crypt, 12345);
        let d2 = generate_dungeon(&config, DungeonType::Crypt, 12345);
        assert_eq!(d1.rooms.len(), d2.rooms.len());
        assert_eq!(d1.entrance, d2.entrance);
        for (t1, t2) in d1.tiles.iter().zip(d2.tiles.iter()) {
            assert_eq!(t1.tile_type, t2.tile_type);
        }
    }

    #[test]
    fn generate_dungeon_different_seeds_differ() {
        let config = DungeonConfig::default();
        let d1 = generate_dungeon(&config, DungeonType::Crypt, 1);
        let d2 = generate_dungeon(&config, DungeonType::Crypt, 2);
        let differ = d1.tiles.iter().zip(d2.tiles.iter())
            .any(|(t1, t2)| t1.tile_type != t2.tile_type);
        assert!(differ);
    }

    #[test]
    fn generate_dungeon_all_types() {
        let config = DungeonConfig::default();
        for dt in [DungeonType::Crypt, DungeonType::CaveSystem, DungeonType::Sewer, DungeonType::FloodedVault] {
            let dungeon = generate_dungeon(&config, dt, 42);
            assert!(!dungeon.rooms.is_empty(), "No rooms for {:?}", dt);
            assert_eq!(dungeon.dungeon_type, dt);
        }
    }

    #[test]
    fn generate_dungeon_rooms_within_bounds() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        for room in &dungeon.rooms {
            assert!(room.x + room.w <= config.width);
            assert!(room.y + room.h <= config.height);
        }
    }

    #[test]
    fn generate_dungeon_corridors_connect_rooms() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let floor_or_corridor: usize = dungeon.tiles.iter()
            .filter(|t| t.tile_type == DungeonTileType::Corridor || t.tile_type == DungeonTileType::Floor)
            .count();
        assert!(floor_or_corridor > dungeon.rooms.iter().map(|r| r.area() as usize).sum::<usize>());
    }

    #[test]
    fn dungeon_tile_glyphs() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        for tile in &dungeon.tiles {
            let _ = tile.glyph();
            let _ = tile.color();
        }
    }

    #[test]
    fn dungeon_tile_to_tile_conversion() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        for tile in &dungeon.tiles {
            let world_tile = tile.to_tile();
            assert_eq!(world_tile.glyph, tile.glyph());
            assert_eq!(world_tile.color, tile.color());
            assert_eq!(world_tile.pos, tile.pos);
        }
    }

    #[test]
    fn dungeon_spawn_positions_in_rooms() {
        let config = DungeonConfig::default();
        let dungeon = generate_dungeon(&config, DungeonType::Crypt, 42);
        let mut rng = StdRng::seed_from_u64(99);
        let positions = dungeon_spawn_positions(&dungeon, &mut rng);
        for (x, y) in &positions {
            assert!(*x < dungeon.width);
            assert!(*y < dungeon.height);
        }
    }

    #[test]
    fn map_layer_default_no_active_interior() {
        let layer = MapLayer::default();
        assert!(layer.active_interior.is_none());
    }

    #[test]
    fn dungeon_config_default_values() {
        let config = DungeonConfig::default();
        assert_eq!(config.width, 50);
        assert_eq!(config.height, 40);
        assert_eq!(config.min_room_size, 4);
        assert_eq!(config.max_room_size, 10);
        assert_eq!(config.target_room_count, 6);
    }
}
