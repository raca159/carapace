use std::collections::HashMap;

use bevy::image::{Image, TextureAtlasLayout};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteDef {
    pub glyph: char,
    #[serde(default)]
    pub color: Option<SpriteColor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitmapConfig {
    #[serde(default = "default_bitmap_width")]
    pub width: u32,
    #[serde(default = "default_bitmap_height")]
    pub height: u32,
    #[serde(default = "default_bitmap_scale")]
    pub scale: u32,
}

fn default_bitmap_width() -> u32 {
    5
}
fn default_bitmap_height() -> u32 {
    7
}
fn default_bitmap_scale() -> u32 {
    3
}

impl Default for BitmapConfig {
    fn default() -> Self {
        Self {
            width: default_bitmap_width(),
            height: default_bitmap_height(),
            scale: default_bitmap_scale(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum GlyphBitmapDef {
    Legacy(Vec<u8>),
    Structured {
        width: Option<u32>,
        height: Option<u32>,
        data: Vec<u8>,
    },
}

impl GlyphBitmapDef {
    pub fn resolve(&self, default_width: u32, default_height: u32) -> GlyphBitmap {
        match self {
            GlyphBitmapDef::Legacy(data) => GlyphBitmap {
                width: default_width,
                height: default_height,
                data: data.clone(),
            },
            GlyphBitmapDef::Structured {
                width,
                height,
                data,
            } => GlyphBitmap {
                width: width.unwrap_or(default_width),
                height: height.unwrap_or(default_height),
                data: data.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteAtlasConfig {
    pub tile_size: Option<u32>,
    pub fallback: SpriteDef,
    pub sprites: HashMap<String, SpriteDef>,
    pub tiles: Option<HashMap<String, String>>,
    pub entities: Option<HashMap<String, String>>,
    #[serde(default)]
    pub bitmap_config: Option<BitmapConfig>,
    #[serde(default)]
    pub glyph_bitmaps: HashMap<char, GlyphBitmapDef>,
}

#[derive(Resource)]
pub struct SpriteAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub tile_size: f32,
    pub tile_map: HashMap<String, SpriteLookup>,
    pub entity_map: HashMap<String, SpriteLookup>,
    pub fallback: SpriteLookup,
    glyph_index_map: HashMap<char, usize>,
}

#[derive(Debug, Clone)]
pub struct SpriteLookup {
    pub atlas_index: usize,
    pub color: (f32, f32, f32),
    pub glyph: char,
}

pub fn load_atlas_config(path: &str) -> SpriteAtlasConfig {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("Warning: {path} not found, using defaults");
        String::new()
    });
    if raw.is_empty() {
        return SpriteAtlasConfig {
            tile_size: None,
            fallback: SpriteDef {
                glyph: '?',
                color: Some(SpriteColor {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                }),
            },
            sprites: HashMap::new(),
            tiles: None,
            entities: None,
            bitmap_config: None,
            glyph_bitmaps: HashMap::new(),
        };
    }
    toml::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("Error parsing {path}: {e}, using defaults");
        SpriteAtlasConfig {
            tile_size: None,
            fallback: SpriteDef {
                glyph: '?',
                color: Some(SpriteColor {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                }),
            },
            sprites: HashMap::new(),
            tiles: None,
            entities: None,
            bitmap_config: None,
            glyph_bitmaps: HashMap::new(),
        }
    })
}

const BUILTIN_GLYPHS: &[(char, &[u8])] = &[
    (' ', &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
    ('.', &[0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b00000, 0b00000]),
    (',', &[0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b00100, 0b01000]),
    ('"', &[0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000]),
    ('~', &[0b00000, 0b00000, 0b10001, 0b01110, 0b00000, 0b00000, 0b00000]),
    ('#', &[0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010]),
    ('@', &[0b01110, 0b10001, 0b10101, 0b11111, 0b10101, 0b10001, 0b01110]),
];

fn builtin_glyph(glyph: char) -> Option<GlyphBitmap> {
    for &(ch, data) in BUILTIN_GLYPHS {
        if ch == glyph {
            return Some(GlyphBitmap {
                width: 5,
                height: 7,
                data: data.to_vec(),
            });
        }
    }
    None
}

fn fallback_bitmap() -> GlyphBitmap {
    GlyphBitmap {
        width: 5,
        height: 7,
        data: vec![
            0b01110, 0b10001, 0b10001, 0b01110, 0b00100, 0b00000, 0b00100,
        ],
    }
}

fn resolve_bitmap(
    glyph: char,
    custom: &HashMap<char, GlyphBitmapDef>,
    bitmap_config: &BitmapConfig,
) -> GlyphBitmap {
    if let Some(def) = custom.get(&glyph) {
        let resolved = def.resolve(bitmap_config.width, bitmap_config.height);
        if (resolved.data.len() as u32) >= resolved.height {
            return resolved;
        }
    }
    if let Some(bmp) = builtin_glyph(glyph) {
        return bmp;
    }
    fallback_bitmap()
}

pub fn build_sprite_atlas(
    config: &SpriteAtlasConfig,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) -> SpriteAtlas {
    let tile_size = config.tile_size.unwrap_or(32);
    let ts = tile_size;
    let bitmap_config = config
        .bitmap_config
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let scale = bitmap_config.scale as usize;

    let mut all_glyphs: Vec<char> = Vec::new();
    all_glyphs.push(config.fallback.glyph);
    for def in config.sprites.values() {
        if !all_glyphs.contains(&def.glyph) {
            all_glyphs.push(def.glyph);
        }
    }

    let num_glyphs = all_glyphs.len() as u32;
    let atlas_width = ts * num_glyphs;
    let atlas_height = ts;
    let mut data = vec![0u8; (atlas_width * atlas_height * 4) as usize];

    for (gi, &glyph) in all_glyphs.iter().enumerate() {
        let bitmap = resolve_bitmap(glyph, &config.glyph_bitmaps, &bitmap_config);
        let tile_px = bitmap.width as usize;
        let tile_py = bitmap.height as usize;
        let rw = tile_px * scale;
        let rh = tile_py * scale;
        let ox = ((ts as usize) - rw) / 2;
        let oy = ((ts as usize) - rh) / 2;
        let x_off = gi as u32 * ts;

        for (row, &row_bits) in bitmap.data.iter().enumerate().take(tile_py) {
            for col in 0..tile_px {
                if (row_bits >> (tile_px - 1 - col)) & 1 == 0 {
                    continue;
                }
                let px0 = x_off as usize + ox + col * scale;
                let py0 = oy + row * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = px0 + dx;
                        let py = py0 + dy;
                        if px < atlas_width as usize && py < atlas_height as usize {
                            let idx = (py * atlas_width as usize + px) * 4;
                            data[idx] = 255;
                            data[idx + 1] = 255;
                            data[idx + 2] = 255;
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    let image = Image::new(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let texture = images.add(image);
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(ts),
        num_glyphs,
        1,
        None,
        None,
    ));

    fn glyph_index(glyph: char, all_glyphs: &[char]) -> usize {
        all_glyphs.iter().position(|&c| c == glyph).unwrap_or(0)
    }

    let fallback_idx = glyph_index(config.fallback.glyph, &all_glyphs);
    let fallback_color = config
        .fallback
        .color
        .as_ref()
        .map(|c| (c.r, c.g, c.b))
        .unwrap_or((1.0, 0.0, 1.0));

    let tile_map: HashMap<String, SpriteLookup> = match &config.tiles {
        Some(tiles) => tiles
            .iter()
            .map(|(key, sprite_name)| {
                let def = config.sprites.get(sprite_name);
                let (glyph, color) = match def {
                    Some(d) => {
                        let c = d
                            .color
                            .as_ref()
                            .map(|c| (c.r, c.g, c.b))
                            .unwrap_or(fallback_color);
                        (d.glyph, c)
                    }
                    None => (config.fallback.glyph, fallback_color),
                };
                (
                    key.clone(),
                    SpriteLookup {
                        atlas_index: glyph_index(glyph, &all_glyphs),
                        color,
                        glyph,
                    },
                )
            })
            .collect(),
        None => HashMap::new(),
    };

    let entity_map: HashMap<String, SpriteLookup> = match &config.entities {
        Some(entities) => entities
            .iter()
            .map(|(key, sprite_name)| {
                let def = config.sprites.get(sprite_name);
                let (glyph, color) = match def {
                    Some(d) => {
                        let c = d
                            .color
                            .as_ref()
                            .map(|c| (c.r, c.g, c.b))
                            .unwrap_or(fallback_color);
                        (d.glyph, c)
                    }
                    None => (config.fallback.glyph, fallback_color),
                };
                (
                    key.clone(),
                    SpriteLookup {
                        atlas_index: glyph_index(glyph, &all_glyphs),
                        color,
                        glyph,
                    },
                )
            })
            .collect(),
        None => HashMap::new(),
    };

    let fallback = SpriteLookup {
        atlas_index: fallback_idx,
        color: fallback_color,
        glyph: config.fallback.glyph,
    };

    let glyph_index_map: HashMap<char, usize> = all_glyphs
        .iter()
        .enumerate()
        .map(|(i, &c)| (c, i))
        .collect();

    SpriteAtlas {
        texture,
        layout,
        tile_size: ts as f32,
        tile_map,
        entity_map,
        fallback,
        glyph_index_map,
    }
}

pub fn setup_atlas(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let config = load_atlas_config("assets/sprites/atlas.toml");
    let atlas = build_sprite_atlas(&config, images, layouts);
    commands.insert_resource(atlas);
}

impl SpriteAtlas {
    pub fn tile_sprite(&self, tile_type: &str) -> &SpriteLookup {
        self.tile_map
            .get(tile_type)
            .unwrap_or(&self.fallback)
    }

    pub fn entity_sprite(&self, entity_type: &str) -> &SpriteLookup {
        self.entity_map
            .get(entity_type)
            .unwrap_or(&self.fallback)
    }

    pub fn glyph_sprite(&self, glyph: char) -> &SpriteLookup {
        for (_key, lookup) in self.tile_map.iter().chain(self.entity_map.iter()) {
            if lookup.glyph == glyph {
                return lookup;
            }
        }
        &self.fallback
    }

    pub fn glyph_index(&self, glyph: char) -> usize {
        self.glyph_index_map
            .get(&glyph)
            .copied()
            .unwrap_or(self.fallback.atlas_index)
    }
}
