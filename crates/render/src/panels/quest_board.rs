use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use bevy_ecs::prelude::World;

use game_core::QuestBoardState;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_quest_board_overlay(frame: &mut Frame, ecs_world: &mut World, cursor: usize) {
    let size = frame.area();
    let panel_width = 52u16.min(size.width);
    let panel_height = 22u16.min(size.height.saturating_sub(4));

    let x = size.width.saturating_sub(panel_width) / 2;
    let y = 2;

    let area = Rect::new(x, y, panel_width, panel_height);

    let mut lines: Vec<Line> = Vec::new();

    let board_state = ecs_world.get_resource::<QuestBoardState>()
        .cloned()
        .unwrap_or_default();

    lines.push(Line::from(Span::styled(
        " Available Quests",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if board_state.available_quests.is_empty() {
        lines.push(Line::from(Span::styled(
            " No quests available right now.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, entry) in board_state.available_quests.iter().enumerate() {
            let selector = if i == cursor { ">" } else { " " };
            let name_style = if i == cursor {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", selector), Style::default().fg(Color::Yellow)),
                Span::styled(&entry.name, name_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(&entry.description, Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(&entry.target_info, Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    Reward: ", Style::default().fg(Color::Yellow)),
                Span::styled(&entry.reward_info, Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(Span::styled(
        " Up/Down: Browse | Enter: Accept | B/Esc: Close",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Rgb(160, 120, 60)))
        .padding(PANEL_PADDING)
        .title(" Quest Board ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
