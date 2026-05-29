use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use bevy_ecs::world::World;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_pause_overlay(frame: &mut Frame, area: Rect) {
    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "PAUSED",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press ESC to resume",
            Style::default().fg(Color::White),
        )),
    ];

    let centered = Layout::vertical([
        Constraint::Length(area.height / 3),
        Constraint::Min(1),
    ])
    .split(area);

    let paragraph = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(paragraph, centered[1]);
}

pub fn draw_death_overlay(frame: &mut Frame, area: Rect, ecs_world: &World) {
    let turns = ecs_world
        .get_resource::<game_core::TurnCounter>()
        .map(|t| t.current())
        .unwrap_or(0);
    let stats = ecs_world
        .get_resource::<game_core::PlayerStats>()
        .cloned()
        .unwrap_or_default();

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "YOU DIED",
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "--- Summary ---",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!(" Turns survived:  {}", turns),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(Span::styled(
        format!(" Enemies slain:   {}", stats.enemies_defeated),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(Span::styled(
        format!(" Items collected: {}", stats.items_collected),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        " [Enter] Return to menu   [Q] Quit",
        Style::default().fg(Color::White),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Red))
        .padding(PANEL_PADDING);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
