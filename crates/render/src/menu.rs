use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_menu(frame: &mut Frame, area: Rect) {
    let title = vec![Line::from(Span::styled(
        "CARAPACE",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    let instructions = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Press ENTER to start",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Press Q to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING);

    let content: Vec<Line> = title
        .into_iter()
        .chain(instructions)
        .chain(vec![Line::from(""), draw_controls_help()])
        .collect();

    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub fn draw_world_creation_screen(frame: &mut Frame, area: Rect, seed: u64, width: u32, height: u32) {
    let title = vec![
        Line::from(Span::styled(
            "WORLD CREATION",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(" Seed: {}", seed)),
        Line::from(format!(" Width: {}", width)),
        Line::from(format!(" Height: {}", height)),
        Line::from(""),
        Line::from(Span::styled("Press ENTER to create world", Style::default().fg(Color::White))),
        Line::from(Span::styled("Press ESC to go back", Style::default().fg(Color::DarkGray))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING);

    let paragraph = Paragraph::new(title)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub fn draw_character_creation_screen(frame: &mut Frame, area: Rect, name: &str) {
    let display_name = if name.is_empty() { "_" } else { name };
    let title = vec![
        Line::from(Span::styled(
            "CHARACTER CREATION",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(" Name: {}", display_name)),
        Line::from(""),
        Line::from(Span::styled("Press ENTER to begin your journey", Style::default().fg(Color::White))),
        Line::from(Span::styled("Press ESC to go back", Style::default().fg(Color::DarkGray))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING);

    let paragraph = Paragraph::new(title)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

pub fn draw_controls_help() -> Line<'static> {
    Line::from(Span::styled(
        "WASD/Arrows: Move | X: Examine | G: Get | I: Inventory | E: Equip | U: Unequip | C: Craft | T: Talk | J: Journal | ESC: Pause | Q: Quit",
        Style::default().fg(Color::DarkGray),
    ))
}
