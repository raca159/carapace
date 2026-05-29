use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::fs;

// ---------------------------------------------------------------------------
// TOML deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct WfcTileDef {
    pub id: String,
    pub glyph: char,
    pub color: [u8; 3],
    #[serde(default = "default_weight")]
    pub weight: f32,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_weight() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct WfcAdjacencyDef {
    pub up: Vec<String>,
    pub down: Vec<String>,
    pub left: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocationTypeDef {
    City,
    Dungeon,
    SpecialSite,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TilesetConditionsDef {
    #[serde(default)]
    pub biomes: Vec<String>,
    #[serde(default)]
    pub factions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WfcTilesetDef {
    pub name: String,
    pub description: Option<String>,
    pub location_type: LocationTypeDef,
    #[serde(rename = "tile")]
    pub tiles: Vec<WfcTileDef>,
    pub adjacency: HashMap<String, WfcAdjacencyDef>,
    pub conditions: TilesetConditionsDef,
}

// ---------------------------------------------------------------------------
// Runtime types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    City,
    Dungeon,
    SpecialSite,
}

impl From<LocationTypeDef> for LocationType {
    fn from(d: LocationTypeDef) -> Self {
        match d {
            LocationTypeDef::City => LocationType::City,
            LocationTypeDef::Dungeon => LocationType::Dungeon,
            LocationTypeDef::SpecialSite => LocationType::SpecialSite,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WfcTile {
    pub id: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub weight: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TilesetConditions {
    pub biomes: Vec<String>,
    pub factions: Vec<String>,
}

/// Pre-computed adjacency compatibility.
/// compat[direction][tile_a][tile_b] == true  means tile_a can have tile_b
/// adjacent in that direction.
#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    pub up: Vec<Vec<bool>>,
    pub down: Vec<Vec<bool>>,
    pub left: Vec<Vec<bool>>,
    pub right: Vec<Vec<bool>>,
}

#[derive(Debug, Clone)]
pub struct WfcTileset {
    pub name: String,
    pub description: Option<String>,
    pub location_type: LocationType,
    pub tiles: Vec<WfcTile>,
    pub compatibility: CompatibilityMatrix,
    pub conditions: TilesetConditions,
}

#[derive(Debug, Clone)]
pub struct WfcOutputTile {
    pub x: u32,
    pub y: u32,
    pub tile_id: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WfcLocation {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<WfcOutputTile>,
    pub name: String,
    pub location_type: LocationType,
    pub seed: u64,
    pub tileset_name: String,
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

pub fn load_wfc_tileset(toml_str: &str) -> Result<WfcTileset, Box<dyn std::error::Error>> {
    let def: WfcTilesetDef = toml::from_str(toml_str)?;
    tileset_from_def(def)
}

pub fn load_wfc_tileset_file(path: &str) -> Result<WfcTileset, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    load_wfc_tileset(&content)
}

fn tileset_from_def(def: WfcTilesetDef) -> Result<WfcTileset, Box<dyn std::error::Error>> {
    let location_type = LocationType::from(def.location_type);

    let mut tiles: Vec<WfcTile> = Vec::new();
    let mut id_to_idx: HashMap<&str, usize> = HashMap::new();

    for (i, t) in def.tiles.iter().enumerate() {
        tiles.push(WfcTile {
            id: t.id.clone(),
            glyph: t.glyph,
            color: (t.color[0], t.color[1], t.color[2]),
            weight: t.weight,
            tags: t.tags.clone(),
        });
        id_to_idx.insert(&t.id, i);
    }

    let n = tiles.len();
    let mut up = vec![vec![false; n]; n];
    let mut down = vec![vec![false; n]; n];
    let mut left = vec![vec![false; n]; n];
    let mut right = vec![vec![false; n]; n];

    for (tile_id, adj) in &def.adjacency {
        let idx = *id_to_idx
            .get(tile_id.as_str())
            .ok_or_else(|| format!("tile '{}' in adjacency not found in tileset", tile_id))?;

        for other in &adj.up {
            if let Some(&oi) = id_to_idx.get(other.as_str()) {
                up[idx][oi] = true;
            }
        }
        for other in &adj.down {
            if let Some(&oi) = id_to_idx.get(other.as_str()) {
                down[idx][oi] = true;
            }
        }
        for other in &adj.left {
            if let Some(&oi) = id_to_idx.get(other.as_str()) {
                left[idx][oi] = true;
            }
        }
        for other in &adj.right {
            if let Some(&oi) = id_to_idx.get(other.as_str()) {
                right[idx][oi] = true;
            }
        }
    }

    Ok(WfcTileset {
        name: def.name,
        description: def.description,
        location_type,
        tiles,
        compatibility: CompatibilityMatrix { up, down, left, right },
        conditions: TilesetConditions {
            biomes: def.conditions.biomes,
            factions: def.conditions.factions,
        },
    })
}

// ---------------------------------------------------------------------------
// Tileset selection
// ---------------------------------------------------------------------------

pub fn select_tileset<'a>(
    tilesets: &'a [WfcTileset],
    biome: Option<&str>,
    faction: Option<&str>,
) -> Option<&'a WfcTileset> {
    // Score each tileset: prefer matching biome, then faction
    let mut scored: Vec<(i32, &WfcTileset)> = tilesets
        .iter()
        .map(|ts| {
            let mut score = 0i32;
            if let Some(b) = biome
                && ts.conditions.biomes.iter().any(|cb| cb == b) {
                    score += 10;
            }
            if let Some(f) = faction
                && ts.conditions.factions.iter().any(|cf| cf == f) {
                    score += 5;
            }
            (score, ts)
        })
        .collect();

    scored.sort_by_key(|a| std::cmp::Reverse(a.0));

    // Return highest-scoring tileset (>= 1 means at least biome matched)
    scored.first().filter(|(s, _)| *s > 0).map(|(_, ts)| *ts)
}

// ---------------------------------------------------------------------------
// Solver state
// ---------------------------------------------------------------------------

struct SolverState {
    width: usize,
    height: usize,
    tile_count: usize,
    /// per cell: which tile indices are still possible
    possibilities: Vec<Vec<bool>>,
    collapsed: Vec<bool>,
    weights: Vec<f32>,
    compat: CompatibilityMatrix,
}

impl SolverState {
    fn new(width: usize, height: usize, tileset: &WfcTileset) -> Self {
        let n = width * height;
        let tile_count = tileset.tiles.len();
        let mut possibilities = vec![vec![true; tile_count]; n];

        // Enforce border-type tiles on the perimeter
        let border_tags: Vec<usize> = tileset
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.tags.iter().any(|tag| tag == "border"))
            .map(|(i, _)| i)
            .collect();

        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    let idx = y * width + x;
                    for (ti, possible) in possibilities[idx].iter_mut().enumerate().take(tile_count) {
                        if !border_tags.contains(&ti) {
                            *possible = false;
                        }
                    }
                }
            }
        }

        let weights = tileset.tiles.iter().map(|t| t.weight).collect();

        Self {
            width,
            height,
            tile_count,
            possibilities,
            collapsed: vec![false; n],
            weights,
            compat: tileset.compatibility.clone(),
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn entropy(&self, cell: usize) -> f32 {
        let mut sum_w = 0.0f64;
        let mut sum_log = 0.0f64;
        for ti in 0..self.tile_count {
            if self.possibilities[cell][ti] {
                let w = self.weights[ti] as f64;
                sum_w += w;
                sum_log += w * w.ln();
            }
        }
        if sum_w == 0.0 {
            return f32::INFINITY;
        }
        (sum_w.ln() - sum_log / sum_w) as f32
    }

    fn lowest_entropy_cell(&self) -> Option<usize> {
        let mut best = None;
        let mut best_e = f32::INFINITY;
        for i in 0..self.possibilities.len() {
            if !self.collapsed[i] {
                let e = self.entropy(i);
                if e < best_e {
                    best_e = e;
                    best = Some(i);
                }
            }
        }
        best
    }

    /// Collapse a specific cell by weighted-random tile selection.
    fn collapse(&mut self, cell: usize, rng: &mut StdRng) {
        let candidates: Vec<usize> = (0..self.tile_count)
            .filter(|&ti| self.possibilities[cell][ti])
            .collect();

        if candidates.is_empty() {
            // Contradiction – fallback to first tile
            for ti in 0..self.tile_count {
                self.possibilities[cell][ti] = ti == 0;
            }
            self.collapsed[cell] = true;
            return;
        }

        let total: f32 = candidates.iter().map(|&ti| self.weights[ti]).sum();
        let mut roll = rng.random::<f32>() * total;
        let chosen = candidates.iter().find(|&&ti| {
            roll -= self.weights[ti];
            roll <= 0.0
        }).copied().unwrap_or(candidates[0]);

        for ti in 0..self.tile_count {
            self.possibilities[cell][ti] = ti == chosen;
        }
        self.collapsed[cell] = true;
    }

    /// Propagate constraints from `cell` outward using a BFS queue.
    fn propagate(&mut self, cell: usize) {
        let mut queue = VecDeque::new();
        queue.push_back(cell);

        while let Some(cur) = queue.pop_front() {
            let cx = cur % self.width;
            let cy = cur / self.width;

            let possible: Vec<usize> = (0..self.tile_count)
                .filter(|&ti| self.possibilities[cur][ti])
                .collect();

            // Take compat references first, before any mutable borrow
            let compat_up = self.compat.up.clone();
            let compat_down = self.compat.down.clone();
            let compat_left = self.compat.left.clone();
            let compat_right = self.compat.right.clone();

            // up
            if cy > 0 {
                let n = self.idx(cx, cy - 1);
                if !self.collapsed[n] && self.constrain_fn(n, &compat_up, &possible) {
                    queue.push_back(n);
                }
            }
            // down
            if cy < self.height - 1 {
                let n = self.idx(cx, cy + 1);
                if !self.collapsed[n] && self.constrain_fn(n, &compat_down, &possible) {
                    queue.push_back(n);
                }
            }
            // left
            if cx > 0 {
                let n = self.idx(cx - 1, cy);
                if !self.collapsed[n] && self.constrain_fn(n, &compat_left, &possible) {
                    queue.push_back(n);
                }
            }
            // right
            if cx < self.width - 1 {
                let n = self.idx(cx + 1, cy);
                if !self.collapsed[n] && self.constrain_fn(n, &compat_right, &possible) {
                    queue.push_back(n);
                }
            }
        }
    }

    fn constrain_fn(
        &mut self,
        nidx: usize,
        compat_fn: &[Vec<bool>],
        current_possible: &[usize],
    ) -> bool {
        let mut changed = false;
        for (ti, possible) in self.possibilities[nidx].iter_mut().enumerate().take(self.tile_count) {
            if !*possible {
                continue;
            }
            let ok = current_possible.iter().any(|&ct| compat_fn[ct][ti]);
            if !ok {
                *possible = false;
                changed = true;
            }
        }
        changed
    }

    /// Run the full WFC solve.
    fn solve(&mut self, rng: &mut StdRng) {
        while let Some(cell) = self.lowest_entropy_cell() {
            self.collapse(cell, rng);
            self.propagate(cell);
        }
    }

    fn into_result(self, tileset: &WfcTileset) -> WfcLocation {
        let mut tiles = Vec::with_capacity(self.possibilities.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let chosen = (0..self.tile_count)
                    .find(|&ti| self.possibilities[idx][ti])
                    .unwrap_or(0);
                let tile = &tileset.tiles[chosen];
                tiles.push(WfcOutputTile {
                    x: x as u32,
                    y: y as u32,
                    tile_id: tile.id.clone(),
                    glyph: tile.glyph,
                    color: tile.color,
                    tags: tile.tags.clone(),
                });
            }
        }
        WfcLocation {
            width: self.width as u32,
            height: self.height as u32,
            tiles,
            name: tileset.name.clone(),
            location_type: tileset.location_type,
            seed: 0,
            tileset_name: tileset.name.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn generate_wfc_location(
    tileset: &WfcTileset,
    width: u32,
    height: u32,
    seed: u64,
) -> WfcLocation {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = SolverState::new(width as usize, height as usize, tileset);
    state.solve(&mut rng);
    let mut loc = state.into_result(tileset);
    loc.seed = seed;
    loc
}

// ---------------------------------------------------------------------------
// Integration with world generation
// ---------------------------------------------------------------------------

/// Place WFC-generated locations into the game world.
/// Currently a placeholder that returns generated locations without ECS
/// spawning (the caller handles entity placement).
pub fn generate_conditioned_location(
    tilesets: &[WfcTileset],
    width: u32,
    height: u32,
    seed: u64,
    biome: Option<&str>,
    faction: Option<&str>,
) -> Option<WfcLocation> {
    let tileset = select_tileset(tilesets, biome, faction)?;
    Some(generate_wfc_location(tileset, width, height, seed))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tileset_toml() -> &'static str {
        r##"name = "TestTileset"
description = "Minimal tileset for unit tests"
location_type = "city"

[[tile]]
id = "EMPTY"
glyph = " "
color = [0, 0, 0]
weight = 1.0
tags = ["border"]

[[tile]]
id = "FLOOR"
glyph = "."
color = [128, 128, 128]
weight = 2.0
tags = ["floor"]

[[tile]]
id = "WALL"
glyph = "#"
color = [64, 64, 64]
weight = 2.0
tags = ["wall"]

[conditions]
biomes = ["BIOME_TEST"]
factions = []

[adjacency.EMPTY]
up = ["EMPTY", "WALL"]
down = ["EMPTY", "WALL"]
left = ["EMPTY", "WALL"]
right = ["EMPTY", "WALL"]

[adjacency.FLOOR]
up = ["WALL", "FLOOR"]
down = ["WALL", "FLOOR"]
left = ["WALL", "FLOOR"]
right = ["WALL", "FLOOR"]

[adjacency.WALL]
up = ["EMPTY", "WALL", "FLOOR"]
down = ["EMPTY", "WALL", "FLOOR"]
left = ["EMPTY", "WALL", "FLOOR"]
right = ["EMPTY", "WALL", "FLOOR"]
"##
    }

    #[test]
    fn test_load_tileset() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        assert_eq!(ts.name, "TestTileset");
        assert_eq!(ts.tiles.len(), 3);
        assert_eq!(ts.location_type, LocationType::City);
    }

    #[test]
    fn test_load_tileset_adjacency() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        // EMPTY can have EMPTY and WALL above it
        assert!(ts.compatibility.up[0][0]); // EMPTY above EMPTY
        assert!(ts.compatibility.up[0][2]); // WALL above EMPTY
        assert!(!ts.compatibility.up[0][1]); // FLOOR above EMPTY
    }

    #[test]
    fn test_generate_deterministic_same_seed() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        let loc1 = generate_wfc_location(&ts, 10, 10, 42);
        let loc2 = generate_wfc_location(&ts, 10, 10, 42);

        assert_eq!(loc1.tiles.len(), loc2.tiles.len());
        for (t1, t2) in loc1.tiles.iter().zip(loc2.tiles.iter()) {
            assert_eq!(t1.tile_id, t2.tile_id, "determinism mismatch at ({},{})", t1.x, t1.y);
            assert_eq!(t1.x, t2.x);
            assert_eq!(t1.y, t2.y);
        }
    }

    #[test]
    fn test_generate_different_seeds_differ() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        let loc1 = generate_wfc_location(&ts, 10, 10, 1);
        let loc2 = generate_wfc_location(&ts, 10, 10, 2);

        let differ = loc1.tiles.iter().zip(loc2.tiles.iter()).any(|(t1, t2)| t1.tile_id != t2.tile_id);
        assert!(differ, "different seeds should produce different layouts");
    }

    #[test]
    fn test_border_constraint() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        let loc = generate_wfc_location(&ts, 8, 8, 42);

        // Border cells must be EMPTY (index 0)
        for tile in &loc.tiles {
            let on_border = tile.x == 0 || tile.x == 7 || tile.y == 0 || tile.y == 7;
            if on_border {
                assert_eq!(
                    tile.tile_id, "EMPTY",
                    "border cell ({},{}) must be EMPTY, got {}",
                    tile.x, tile.y, tile.tile_id
                );
            }
        }
    }

    #[test]
    fn test_internal_adjacency_valid() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        let loc = generate_wfc_location(&ts, 8, 8, 42);

        // Build tile_id -> index lookup
        let id_to_idx: HashMap<&str, usize> = ts
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.as_str(), i))
            .collect();

        for y in 0..loc.height {
            for x in 0..loc.width {
                let idx = (y * loc.width + x) as usize;
                let tile = &loc.tiles[idx];
                let ti = id_to_idx[tile.tile_id.as_str()];

                // Check neighbor above
                if y > 0 {
                    let above = &loc.tiles[((y - 1) * loc.width + x) as usize];
                    let ai = id_to_idx[above.tile_id.as_str()];
                    assert!(
                        ts.compatibility.up[ti][ai],
                        "tile {} at ({},{}) invalid above neighbor {} at ({},{})",
                        tile.tile_id, x, y, above.tile_id, x, y - 1
                    );
                }
                // Check neighbor below
                if y + 1 < loc.height {
                    let below = &loc.tiles[((y + 1) * loc.width + x) as usize];
                    let bi = id_to_idx[below.tile_id.as_str()];
                    assert!(
                        ts.compatibility.down[ti][bi],
                        "tile {} at ({},{}) invalid below neighbor {}",
                        tile.tile_id, x, y, below.tile_id
                    );
                }
                // Check left
                if x > 0 {
                    let left = &loc.tiles[(y * loc.width + x - 1) as usize];
                    let li = id_to_idx[left.tile_id.as_str()];
                    assert!(
                        ts.compatibility.left[ti][li],
                        "tile {} at ({},{}) invalid left neighbor {}",
                        tile.tile_id, x, y, left.tile_id
                    );
                }
                // Check right
                if x + 1 < loc.width {
                    let right = &loc.tiles[(y * loc.width + x + 1) as usize];
                    let ri = id_to_idx[right.tile_id.as_str()];
                    assert!(
                        ts.compatibility.right[ti][ri],
                        "tile {} at ({},{}) invalid right neighbor {}",
                        tile.tile_id, x, y, right.tile_id
                    );
                }
            }
        }
    }

    #[test]
    fn test_load_vampire_city_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "VampireCity");
        assert_eq!(ts.tiles.len(), 8);
        assert_eq!(ts.location_type, LocationType::City);
    }

    #[test]
    fn test_load_trench_nest_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "TrenchNest");
        assert_eq!(ts.tiles.len(), 9);
    }

    #[test]
    fn test_load_sanguine_manse_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/sanguine_manse.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "SanguineManse");
        assert_eq!(ts.tiles.len(), 8);
    }

    #[test]
    fn test_load_human_settlement_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/human_settlement.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "HumanSettlement");
        assert_eq!(ts.tiles.len(), 8);
    }

    #[test]
    fn test_load_familiar_den_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/familiar_den.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "FamiliarDen");
        assert_eq!(ts.tiles.len(), 9);
    }

    #[test]
    fn test_load_cryo_vault_tileset() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/cryo_vault.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        assert_eq!(ts.name, "CryoVault");
        assert_eq!(ts.tiles.len(), 9);
    }

    #[test]
    fn test_select_tileset_by_biome() {
        let toml1 = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let toml2 = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts1 = load_wfc_tileset(toml1).unwrap();
        let ts2 = load_wfc_tileset(toml2).unwrap();

        let tilesets = [ts1, ts2];

        let selected = select_tileset(&tilesets, Some("BIOME_SWAMP"), None);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "VampireCity");

        let selected = select_tileset(&tilesets, Some("BIOME_DUNGEON"), None);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "TrenchNest");
    }

    #[test]
    fn test_select_tileset_no_match() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        let tilesets = [ts];
        let selected = select_tileset(&tilesets, Some("BIOME_OCEAN"), None);
        assert!(selected.is_none());
    }

    #[test]
    fn test_generate_vampire_city() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        let loc = generate_wfc_location(&ts, 20, 20, 12345);
        assert_eq!(loc.width, 20);
        assert_eq!(loc.height, 20);
        assert_eq!(loc.tiles.len(), 400);
        assert_eq!(loc.tileset_name, "VampireCity");
    }

    #[test]
    fn test_generate_conditioned_location() {
        let toml1 = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let toml2 = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts1 = load_wfc_tileset(toml1).unwrap();
        let ts2 = load_wfc_tileset(toml2).unwrap();
        let tilesets = [ts1, ts2];

        let loc = generate_conditioned_location(
            &tilesets,
            15, 15, 99,
            Some("BIOME_SWAMP"),
            None,
        );
        assert!(loc.is_some());
        assert_eq!(loc.as_ref().unwrap().tileset_name, "VampireCity");

        let loc = generate_conditioned_location(
            &tilesets,
            15, 15, 99,
            Some("BIOME_DUNGEON"),
            None,
        );
        assert!(loc.is_some());
        assert_eq!(loc.unwrap().tileset_name, "TrenchNest");
    }

    #[test]
    fn test_wfc_deterministic_tileset_file() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts = load_wfc_tileset(toml).unwrap();

        let loc1 = generate_wfc_location(&ts, 12, 12, 777);
        let loc2 = generate_wfc_location(&ts, 12, 12, 777);

        assert_eq!(loc1.tiles.len(), loc2.tiles.len());
        for (t1, t2) in loc1.tiles.iter().zip(loc2.tiles.iter()) {
            assert_eq!(t1.tile_id, t2.tile_id);
            assert_eq!(t1.glyph, t2.glyph);
            assert_eq!(t1.color, t2.color);
        }
    }

    #[test]
    fn test_wfc_output_tile_metadata() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/sanguine_manse.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        let loc = generate_wfc_location(&ts, 5, 5, 42);

        for tile in &loc.tiles {
            assert!(tile.x < loc.width);
            assert!(tile.y < loc.height);
            assert!(!tile.tile_id.is_empty());
            // All tiles should have at least their tag
            assert!(!tile.tags.is_empty(), "tile {} has no tags", tile.tile_id);
        }
    }

    #[test]
    fn test_solver_converges() {
        let toml = sample_tileset_toml();
        let ts = load_wfc_tileset(toml).unwrap();

        // Small grid, many seeds – WFC should always converge
        for seed in 0..20 {
            let loc = generate_wfc_location(&ts, 4, 4, seed);
            assert_eq!(
                loc.tiles.len(),
                16,
                "failed to converge for seed {}",
                seed
            );
            // All cells must be collapsed to a single tile
            let all_collapsed = loc.tiles.iter().all(|t| !t.tile_id.is_empty());
            assert!(all_collapsed, "seed {} left uncollapsed cells", seed);
        }
    }

    // -----------------------------------------------------------------------
    // Comprehensive verification: all 6 production tilesets
    // -----------------------------------------------------------------------

    fn load_tileset(name: &str) -> WfcTileset {
        let toml = match name {
            "VampireCity" => include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml"),
            "HumanSettlement" => include_str!("../../../assets/config/wfc_tilesets/human_settlement.toml"),
            "SanguineManse" => include_str!("../../../assets/config/wfc_tilesets/sanguine_manse.toml"),
            "TrenchNest" => include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml"),
            "FamiliarDen" => include_str!("../../../assets/config/wfc_tilesets/familiar_den.toml"),
            "CryoVault" => include_str!("../../../assets/config/wfc_tilesets/cryo_vault.toml"),
            _ => panic!("unknown tileset {}", name),
        };
        load_wfc_tileset(toml).unwrap()
    }

    fn all_tilesets() -> Vec<WfcTileset> {
        vec![
            load_tileset("VampireCity"),
            load_tileset("HumanSettlement"),
            load_tileset("SanguineManse"),
            load_tileset("TrenchNest"),
            load_tileset("FamiliarDen"),
            load_tileset("CryoVault"),
        ]
    }

    #[test]
    fn test_all_tilesets_load_and_generate() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc = generate_wfc_location(ts, 10, 10, 42);
            assert_eq!(
                loc.tiles.len(),
                100,
                "{}: generation produced wrong tile count",
                ts.name
            );
            assert_eq!(loc.tileset_name, ts.name);
        }
    }

    #[test]
    fn test_border_constraint_all_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc = generate_wfc_location(ts, 12, 12, 42);
            let border_tile_ids: Vec<&str> = ts
                .tiles
                .iter()
                .filter(|t| t.tags.iter().any(|tag| tag == "border"))
                .map(|t| t.id.as_str())
                .collect();
            assert!(
                !border_tile_ids.is_empty(),
                "{}: no border-tagged tiles found",
                ts.name
            );
            for tile in &loc.tiles {
                let on_border =
                    tile.x == 0 || tile.x == loc.width - 1 || tile.y == 0 || tile.y == loc.height - 1;
                if on_border {
                    assert!(
                        border_tile_ids.contains(&tile.tile_id.as_str()),
                        "{}: border cell ({},{}) = {} expected border tile {:?}",
                        ts.name,
                        tile.x,
                        tile.y,
                        tile.tile_id,
                        border_tile_ids
                    );
                }
            }
        }
    }

    #[test]
    fn test_internal_adjacency_all_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc = generate_wfc_location(ts, 15, 15, 42);
            let id_to_idx: Vec<usize> = loc
                .tiles
                .iter()
                .map(|t| {
                    ts.tiles
                        .iter()
                        .position(|tt| tt.id == t.tile_id)
                        .expect("tile id from output not in tileset")
                })
                .collect();

            let w = loc.width as usize;
            let h = loc.height as usize;
            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    let ti = id_to_idx[idx];
                    if y > 0 {
                        let ai = id_to_idx[(y - 1) * w + x];
                        assert!(
                            ts.compatibility.up[ti][ai],
                            "{}: tile {} at ({},{}) invalid above neighbor {}",
                            ts.name,
                            loc.tiles[idx].tile_id,
                            x,
                            y,
                            loc.tiles[(y - 1) * w + x].tile_id
                        );
                    }
                    if y + 1 < h {
                        let bi = id_to_idx[(y + 1) * w + x];
                        assert!(
                            ts.compatibility.down[ti][bi],
                            "{}: tile {} at ({},{}) invalid below neighbor {}",
                            ts.name,
                            loc.tiles[idx].tile_id,
                            x,
                            y,
                            loc.tiles[(y + 1) * w + x].tile_id
                        );
                    }
                    if x > 0 {
                        let li = id_to_idx[y * w + x - 1];
                        assert!(
                            ts.compatibility.left[ti][li],
                            "{}: tile {} at ({},{}) invalid left neighbor {}",
                            ts.name,
                            loc.tiles[idx].tile_id,
                            x,
                            y,
                            loc.tiles[y * w + x - 1].tile_id
                        );
                    }
                    if x + 1 < w {
                        let ri = id_to_idx[y * w + x + 1];
                        assert!(
                            ts.compatibility.right[ti][ri],
                            "{}: tile {} at ({},{}) invalid right neighbor {}",
                            ts.name,
                            loc.tiles[idx].tile_id,
                            x,
                            y,
                            loc.tiles[y * w + x + 1].tile_id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_determinism_all_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc1 = generate_wfc_location(ts, 10, 10, 42);
            let loc2 = generate_wfc_location(ts, 10, 10, 42);
            for (t1, t2) in loc1.tiles.iter().zip(loc2.tiles.iter()) {
                assert_eq!(
                    t1.tile_id, t2.tile_id,
                    "{}: determinism failed at ({},{})",
                    ts.name, t1.x, t1.y
                );
            }
        }
    }

    #[test]
    fn test_different_seeds_differ_all_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc1 = generate_wfc_location(ts, 10, 10, 1);
            let loc2 = generate_wfc_location(ts, 10, 10, 2);
            let differ = loc1
                .tiles
                .iter()
                .zip(loc2.tiles.iter())
                .any(|(t1, t2)| t1.tile_id != t2.tile_id);
            assert!(
                differ,
                "{}: different seeds should produce different layouts",
                ts.name
            );
        }
    }

    #[test]
    fn test_convergence_all_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            for seed in 0..10 {
                let loc = generate_wfc_location(ts, 6, 6, seed);
                assert_eq!(
                    loc.tiles.len(),
                    36,
                    "{}: failed to converge for seed {}",
                    ts.name,
                    seed
                );
                let all_collapsed = loc.tiles.iter().all(|t| !t.tile_id.is_empty());
                assert!(
                    all_collapsed,
                    "{}: seed {} left uncollapsed cells",
                    ts.name,
                    seed
                );
            }
        }
    }

    #[test]
    fn test_convergence_various_grid_sizes() {
        let ts = load_wfc_tileset(sample_tileset_toml()).unwrap();
        let sizes = [(1, 1), (2, 2), (3, 5), (8, 8), (15, 10), (30, 30)];
        for &(w, h) in &sizes {
            for seed in [0, 1, 999, 123456789] {
                let loc = generate_wfc_location(&ts, w, h, seed);
                assert_eq!(
                    loc.tiles.len(),
                    (w * h) as usize,
                    "size {}x{} seed {}: wrong tile count",
                    w,
                    h,
                    seed
                );
            }
        }
    }

    #[test]
    fn test_extreme_seeds() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/cryo_vault.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        let extreme_seeds = [0u64, 1, u64::MAX, u64::MAX - 1, 9876543210];
        for &seed in &extreme_seeds {
            let loc = generate_wfc_location(&ts, 8, 8, seed);
            assert_eq!(loc.tiles.len(), 64, "seed {}: wrong tile count", seed);
            assert_eq!(loc.seed, seed, "seed not preserved in output");
        }
    }

    #[test]
    fn test_minimal_grid_tilesets() {
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let loc = generate_wfc_location(ts, 1, 1, 42);
            assert_eq!(loc.tiles.len(), 1, "{}: 1x1 failed", ts.name);
            // Single cell is on all borders, must be border tile
            assert!(
                loc.tiles[0].tags.iter().any(|t| t == "border"),
                "{}: 1x1 tile should be border tile",
                ts.name
            );
        }
    }

    #[test]
    fn test_select_tileset_by_faction() {
        let toml_vc = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let toml_tn = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts_vc = load_wfc_tileset(toml_vc).unwrap();
        let ts_tn = load_wfc_tileset(toml_tn).unwrap();
        let tilesets = [ts_vc, ts_tn];

        let selected = select_tileset(&tilesets, None, Some("sanguine_elite"));
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "VampireCity");

        let selected = select_tileset(&tilesets, None, Some("great_carapace"));
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "TrenchNest");
    }

    #[test]
    fn test_select_tileset_biome_and_faction() {
        let toml_vc = include_str!("../../../assets/config/wfc_tilesets/vampire_city.toml");
        let toml_sm = include_str!("../../../assets/config/wfc_tilesets/sanguine_manse.toml");
        let ts_vc = load_wfc_tileset(toml_vc).unwrap();
        let ts_sm = load_wfc_tileset(toml_sm).unwrap();
        let tilesets = [ts_vc, ts_sm];

        // Both match biome, VC matches faction → VC wins
        let selected = select_tileset(
            &tilesets,
            Some("BIOME_SWAMP"),
            Some("sanguine_elite"),
        );
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "VampireCity");

        // Neither matches biome nor faction → none
        let selected = select_tileset(
            &tilesets,
            Some("BIOME_OCEAN"),
            Some("free_settlements"),
        );
        assert!(selected.is_none());
    }

    #[test]
    fn test_weight_produces_variety() {
        let toml = include_str!("../../../assets/config/wfc_tilesets/trench_nest.toml");
        let ts = load_wfc_tileset(toml).unwrap();
        // Generate a large grid to get enough tile variety
        let loc = generate_wfc_location(&ts, 30, 30, 42);
        let mut seen = std::collections::HashSet::new();
        for tile in &loc.tiles {
            seen.insert(tile.tile_id.as_str());
        }
        // Should produce at least 5 of the 9 tile types
        assert!(
            seen.len() >= 5,
            "TrenchNest: only produced {} unique tile types out of 9 (weights may be too extreme)",
            seen.len()
        );
    }

    #[test]
    fn test_tileset_adjacency_symmetric() {
        // Verify adjacency compatibility is symmetric:
        // if tile A allows B above it, then B should allow A below it
        let tilesets = all_tilesets();
        for ts in &tilesets {
            let n = ts.tiles.len();
            for a in 0..n {
                for b in 0..n {
                    // up(a) = allowed tiles above A → corresponds to B's down compat
                    assert_eq!(
                        ts.compatibility.up[a][b],
                        ts.compatibility.down[b][a],
                        "{}: asymmetry up/down at tiles {}<->{}",
                        ts.name,
                        ts.tiles[a].id,
                        ts.tiles[b].id
                    );
                    // left(a) = allowed tiles left of A → corresponds to B's right compat
                    assert_eq!(
                        ts.compatibility.left[a][b],
                        ts.compatibility.right[b][a],
                        "{}: asymmetry left/right at tiles {}<->{}",
                        ts.name,
                        ts.tiles[a].id,
                        ts.tiles[b].id
                    );
                }
            }
        }
    }
}
