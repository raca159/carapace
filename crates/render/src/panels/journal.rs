use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use bevy_ecs::prelude::World;

use game_core::{ActiveQuest, QuestLog, QuestState};

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_journal_overlay(frame: &mut Frame, ecs_world: &mut World, cursor: usize) {
    let size = frame.area();
    let panel_width = 42u16.min(size.width);
    let panel_height = 20u16.min(size.height.saturating_sub(4));

    let x = size.width.saturating_sub(panel_width) / 2;
    let y = 2;

    let area = Rect::new(x, y, panel_width, panel_height);

    let mut lines: Vec<Line> = Vec::new();

    let mut quest_query = ecs_world.query::<&ActiveQuest>();
    let quests: Vec<ActiveQuest> = quest_query.iter(ecs_world).cloned().collect();

    let quest_log = ecs_world.get_resource::<QuestLog>().cloned()
        .unwrap_or_default();

    let active_quests: Vec<&ActiveQuest> = quests.iter()
        .filter(|q| q.state == QuestState::Active || q.state == QuestState::Complete)
        .collect();

    lines.push(Line::from(Span::styled(
        " Active Quests:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if active_quests.is_empty() {
        lines.push(Line::from(Span::styled(
            " No active quests",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, quest) in active_quests.iter().enumerate() {
            let is_complete = quest.state == QuestState::Complete;
            let progress = quest.objective.progress_text();
            let selector = if i == cursor { ">" } else { " " };

            let status_str = if is_complete { " DONE!" } else { "" };

            let (name_style, progress_style) = if is_complete {
                (
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Green),
                )
            } else if i == cursor {
                (
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::White),
                )
            } else {
                (
                    Style::default().fg(Color::White),
                    Style::default().fg(Color::DarkGray),
                )
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", selector), Style::default().fg(Color::Yellow)),
                Span::styled(&quest.name, name_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("     ", Style::default()),
                Span::styled(format!("[{}]{}", progress, status_str), progress_style),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(" Completed: ({})", quest_log.turned_in.len()),
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Up/Down: Navigate | Enter: Turn In | Esc: Close",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING)
        .title(" Journal ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
