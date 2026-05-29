use std::sync::OnceLock;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::Frame;

use bevy_ecs::prelude::{Entity, World};
use bevy_ecs::query::With;

use game_core::color_theme::desaturate_color;
use game_core::{Glyph, Player, Position};
use game_world::{Tile, TilePos, WorldMap};

use crate::camera::Camera;
use crate::GlyphRegistry;

static SHARED_GLYPH_REGISTRY: OnceLock<GlyphRegistry> = OnceLock::new();

/// Load (or return the cached) shared glyph registry from `assets/config/glyphs.toml`.
pub fn load_glyph_registry() -> &'static GlyphRegistry {
    SHARED_GLYPH_REGISTRY.get_or_init(|| GlyphRegistry::load("assets/config/glyphs.toml"))
}

/// Resolve a tile's glyph character, falling back to the registry fallback
/// if the glyph is unknown.
fn resolve_tile_char(tile: &Tile, registry: &GlyphRegistry) -> char {
    registry.resolve_char(tile.glyph)
}

/// Resolve an entity's glyph character, falling back to the registry fallback
/// if the glyph is unknown.
fn resolve_entity_char(glyph: &Glyph, registry: &GlyphRegistry) -> char {
    registry.resolve_char(glyph.char)
}

pub const PANEL_PADDING: Padding = Padding::new(1, 1, 1, 1);

pub const UNICODE_BORDER_SET: border::Set = border::Set {
    top_left: "\u{250c}",
    top_right: "\u{2510}",
    bottom_left: "\u{2514}",
    bottom_right: "\u{2518}",
    vertical_left: "\u{2502}",
    vertical_right: "\u{2502}",
    horizontal_top: "\u{2500}",
    horizontal_bottom: "\u{2500}",
};

pub const EXAMINE_PANEL_WIDTH: u16 = 34;

pub type ScreenEntry = (u16, u16, (u8, u8, u8), char, u32);

fn depth_dim(color: (u8, u8, u8), z: u32) -> (u8, u8, u8) {
    if z == 0 {
        return color;
    }
    let factor = 1.0 - (z as f32 * 0.15).min(0.75);
    (
        (color.0 as f32 * factor) as u8,
        (color.1 as f32 * factor) as u8,
        (color.2 as f32 * factor) as u8,
    )
}

pub fn tile_color_for(tile: &Tile) -> Color {
    let dim = desaturate_color(tile.color, 0.35);
    Color::Rgb(dim.0, dim.1, dim.2)
}

pub fn draw_gameplay_world(
    frame: &mut Frame,
    area: Rect,
    ecs_world: &mut World,
    camera: &mut Camera,
) {
    let registry = load_glyph_registry();

    camera.viewport_width = area.width as u32;
    camera.viewport_height = area.height as u32;

    let mut tile_query = ecs_world.query::<&Tile>();
    let buf = frame.buffer_mut();

    if let Some(map) = ecs_world.get_resource::<WorldMap>() {
        let mut screen_entities: Vec<(u16, u16, Entity)> = Vec::new();
        for screen_y in 0..area.height {
            let world_y = camera.y + screen_y as u32;
            for screen_x in 0..area.width {
                let world_x = camera.x + screen_x as u32;
                if world_x >= map.width || world_y >= map.height {
                    continue;
                }
                let entity = map.get_unchecked(TilePos::new(world_x, world_y));
                screen_entities.push((area.x + screen_x, area.y + screen_y, entity));
            }
        }
        let _ = map;
        for (x, y, entity) in screen_entities {
            if let Ok(tile) = tile_query.get(ecs_world, entity)
                && let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(resolve_tile_char(tile, registry));
                    let dim = desaturate_color(tile.color, 0.35);
                    cell.set_fg(Color::Rgb(dim.0, dim.1, dim.2));
                }
        }
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(UNICODE_BORDER_SET)
            .border_style(Style::default().fg(Color::Yellow))
            .padding(PANEL_PADDING)
            .title("Loading...");
        let paragraph = Paragraph::new("Generating world...").block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let mut entity_query = ecs_world.query::<(Entity, &Position, &Glyph)>();
    let mut entity_screen: Vec<ScreenEntry> = Vec::new();
    let mut player_screen: Option<ScreenEntry> = None;

    let mut player_marker = ecs_world.query_filtered::<Entity, With<Player>>();
    let player_entity = player_marker.iter(ecs_world).next();

    for (entity, pos, glyph) in entity_query.iter(ecs_world) {
        let sx = pos.x as i64 - camera.x as i64;
        let sy = pos.y as i64 - camera.y as i64;

        if sx >= 0 && sx < area.width as i64 && sy >= 0 && sy < area.height as i64 {
            let color = depth_dim(glyph.color, pos.z);
            let ch = resolve_entity_char(glyph, registry);
            let entry = (area.x + sx as u16, area.y + sy as u16, color, ch, pos.z);

            if Some(entity) == player_entity {
                player_screen = Some(entry);
            } else {
                entity_screen.push(entry);
            }
        }
    }

    entity_screen.sort_unstable_by_key(|e| e.4);

    for &(x, y, color, ch, _) in &entity_screen {
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(ch);
            cell.set_fg(Color::Rgb(color.0, color.1, color.2));
        }
    }

    if let Some((x, y, color, ch, _)) = player_screen
        && let Some(cell) = buf.cell_mut((x, y))
    {
        cell.set_char(ch);
        cell.set_fg(Color::Rgb(color.0, color.1, color.2));
    }
}
