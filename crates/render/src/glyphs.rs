use std::collections::HashMap;

use serde::Deserialize;

/// Deserialized glyph definition from glyphs.toml
#[derive(Debug, Clone, Deserialize)]
pub struct GlyphDef {
    pub glyph: char,
    pub color: [u8; 3],
}

/// Named glyph entry as stored in glyphs.toml (`[glyph.<name>]` sections).
#[derive(Debug, Clone, Deserialize)]
struct GlyphEntry {
    #[serde(rename = "char")]
    char_val: CharOrString,
    default_color: GlyphColor,
    #[serde(default)]
    bitmap: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CharOrString {
    Char(char),
    String(String),
}

impl CharOrString {
    fn to_char(&self) -> char {
        match self {
            CharOrString::Char(c) => *c,
            CharOrString::String(s) => {
                let trimmed = s.trim().trim_matches('"');
                trimmed.chars().next().unwrap_or('?')
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct GlyphColor {
    r: f32,
    g: f32,
    b: f32,
}

impl GlyphColor {
    fn to_u8(&self) -> [u8; 3] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        ]
    }
}

/// Wrapper to deserialize the `[glyph.*]` table sections.
#[derive(Debug, Deserialize)]
struct GlyphConfigFile {
    #[serde(rename = "glyph")]
    glyph_entries: HashMap<String, GlyphEntry>,
}

/// Shared glyph registry consumed by all render pipelines.
///
/// Loaded from `assets/config/glyphs.toml` at startup. Provides named
/// lookups for terrain tiles, entity archetypes, and a complete index
/// of every glyph character used in the game with its default color.
#[derive(Debug, Clone)]
pub struct GlyphRegistry {
    pub fallback: GlyphDef,
    pub tiles: HashMap<String, GlyphDef>,
    pub entities: HashMap<String, GlyphDef>,
    glyph_index: HashMap<char, [u8; 3]>,
}

fn name_category(name: &str) -> &str {
    // Heuristic: if name matches a known biome/terrain key, it's a tile glyph.
    match name {
        "grass" | "dirt" | "stone" | "sand" | "water" | "forest" | "swamp"
        | "snow" | "lava" | "tech_terminal" => "tile",
        _ => "entity",
    }
}

impl GlyphRegistry {
    pub fn load(path: &str) -> Self {
        let raw = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Warning: could not read {path}: {e}, using empty registry");
            String::new()
        });
        if raw.is_empty() {
            return Self::empty();
        }
        match toml::from_str::<GlyphConfigFile>(&raw) {
            Ok(cfg) => Self::from_config(cfg),
            Err(e) => {
                eprintln!("Warning: error parsing {path}: {e}, using empty registry");
                Self::empty()
            }
        }
    }

    fn from_config(cfg: GlyphConfigFile) -> Self {
        let fallback = GlyphDef {
            glyph: '?',
            color: [255, 0, 255],
        };

        let mut tiles: HashMap<String, GlyphDef> = HashMap::new();
        let mut entities: HashMap<String, GlyphDef> = HashMap::new();
        let mut glyph_index: HashMap<char, [u8; 3]> = HashMap::new();

        for (name, entry) in &cfg.glyph_entries {
            let ch = entry.char_val.to_char();
            let color = entry.default_color.to_u8();
            let def = GlyphDef { glyph: ch, color };

            glyph_index.insert(ch, color);

            match name_category(name) {
                "tile" => {
                    tiles.insert(name.clone(), def);
                }
                _ => {
                    entities.insert(name.clone(), def);
                }
            }
        }

        Self {
            fallback,
            tiles,
            entities,
            glyph_index,
        }
    }

    pub fn empty() -> Self {
        Self {
            fallback: GlyphDef {
                glyph: '?',
                color: [255, 0, 255],
            },
            tiles: HashMap::new(),
            entities: HashMap::new(),
            glyph_index: HashMap::new(),
        }
    }

    pub fn tile_glyph(&self, name: &str) -> Option<&GlyphDef> {
        self.tiles.get(name)
    }

    pub fn entity_glyph(&self, name: &str) -> Option<&GlyphDef> {
        self.entities.get(name)
    }

    /// Look up the default color for a glyph character.
    pub fn color_for_char(&self, ch: char) -> Option<[u8; 3]> {
        self.glyph_index.get(&ch).copied()
    }

    /// Return true if the character is registered in the glyph index.
    pub fn is_known(&self, ch: char) -> bool {
        self.glyph_index.contains_key(&ch)
    }

    /// Return the fallback glyph if `ch` is unknown, otherwise `ch`.
    pub fn resolve_char(&self, ch: char) -> char {
        if self.is_known(ch) { ch } else { self.fallback.glyph }
    }

    /// Number of registered glyph characters.
    pub fn glyph_count(&self) -> usize {
        self.glyph_index.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> GlyphRegistry {
        // Load from the project root using a path relative to the workspace.
        // When cargo test runs, the CWD is the project root for workspace tests.
        let raw = include_str!("../../../assets/config/glyphs.toml");
        match toml::from_str::<GlyphConfigFile>(raw) {
            Ok(cfg) => GlyphRegistry::from_config(cfg),
            Err(e) => {
                panic!("Failed to parse embedded glyphs.toml: {e}");
            }
        }
    }

    #[test]
    fn load_glyphs_toml() {
        let reg = test_registry();
        assert!(reg.glyph_count() >= 30, "should have many glyphs, got {}", reg.glyph_count());
    }

    #[test]
    fn fallback_is_question_mark() {
        let reg = test_registry();
        assert_eq!(reg.fallback.glyph, '?');
        assert_eq!(reg.fallback.color, [255, 0, 255]);
    }

    #[test]
    fn known_and_unknown_chars() {
        let reg = test_registry();
        assert!(reg.is_known('@'), "@ should be known");
        assert!(reg.is_known('"'), "quote should be known");
        assert!(!reg.is_known('\0'), "null should be unknown");
        assert!(!reg.is_known('¿'), "inverted ? should be unknown");
    }

    #[test]
    fn resolve_known_returns_same() {
        let reg = test_registry();
        assert_eq!(reg.resolve_char('@'), '@');
        assert_eq!(reg.resolve_char('.'), '.');
    }

    #[test]
    fn resolve_unknown_returns_fallback() {
        let reg = test_registry();
        assert_eq!(reg.resolve_char('\0'), '?');
    }

    #[test]
    fn tile_lookup() {
        let reg = test_registry();
        let grass = reg.tile_glyph("grass").expect("grass tile should exist");
        assert_eq!(grass.glyph, '"');
    }

    #[test]
    fn entity_lookup() {
        let reg = test_registry();
        let player = reg.entity_glyph("player").expect("player entity should exist");
        assert_eq!(player.glyph, '@');
    }

    #[test]
    fn color_for_char() {
        let reg = test_registry();
        let color = reg.color_for_char('@').expect("@ should have a color");
        assert_eq!(color, [0, 255, 255]);
    }

    #[test]
    fn empty_registry_fallback() {
        let reg = GlyphRegistry::empty();
        assert_eq!(reg.fallback.glyph, '?');
        assert_eq!(reg.fallback.color, [255, 0, 255]);
        assert_eq!(reg.glyph_count(), 0);
    }

    #[test]
    fn tiles_and_entities_populated() {
        let reg = test_registry();
        assert!(reg.tiles.len() >= 5, "should have tile definitions, got {}", reg.tiles.len());
        assert!(reg.entities.len() >= 20, "should have entity definitions, got {}", reg.entities.len());
    }
}
