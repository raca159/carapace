use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use bevy_ecs::prelude::World;
use bevy_ecs::query::With;

use game_core::ExamineMode;
use game_core::{Health, Player, Position};
use game_world::WorldMap;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

// const EXAMINE_PANEL_WIDTH: u16 = 34;

pub fn draw_examine_panel(frame: &mut Frame, area: Rect, ecs_world: &mut World) {
    let (cx, cy) = {
        let em = ecs_world.get_resource::<ExamineMode>();
        match em {
            Some(mode) if mode.active => (mode.cursor.x, mode.cursor.y),
            _ => {
                let mut player_query =
                    ecs_world.query_filtered::<&Position, With<Player>>();
                match player_query.single(ecs_world) {
                    Ok(pos) => (pos.x, pos.y),
                    Err(_) => return,
                }
            }
        }
    };

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!(" Cursor: ({}, {})", cx, cy),
        Style::default().fg(Color::Cyan),
    )));

    if let Some(map) = ecs_world.get_resource::<WorldMap>() {
        let m = map.clone();
        let tile_pos = game_world::TilePos::new(cx, cy);
        if let Some(entity) = m.get(tile_pos) {
            let mut tile_query = ecs_world.query::<&game_world::Tile>();
            if let Ok(tile) = tile_query.get(ecs_world, entity) {
                lines.push(Line::from(Span::styled(
                    format!(" Biome: {}", tile.biome_name),
                    Style::default().fg(Color::Green),
                )));
                lines.push(Line::from(Span::styled(
                    format!(" Color: {:?}", tile.color),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    let mut entity_query = ecs_world.query_filtered::<(&Position, &game_core::Glyph), With<Player>>();
    if let Ok((pos, glyph)) = entity_query.single(ecs_world) {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(" Player: {}", glyph.char),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Pos: ({}, {})", pos.x, pos.y),
            Style::default().fg(Color::White),
        )));

        let mut hp_query = ecs_world.query_filtered::<&Health, With<Player>>();
        if let Ok(hp) = hp_query.single(ecs_world) {
            lines.push(Line::from(Span::styled(
                format!(" HP: {}/{}", hp.current, hp.max),
                Style::default().fg(Color::Green),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " [X] Close ",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::White))
        .padding(PANEL_PADDING)
        .title(" Examine ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
