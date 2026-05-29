use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_dialogue_overlay(frame: &mut Frame, text: &str, speaker: &str, quest_offer: bool) {
    let size = frame.area();
    let panel_width = 44u16.min(size.width);
    let line_count = text.len() as u16 / (panel_width - 4).max(1) + 1;
    let mut height = line_count + 5;
    if quest_offer {
        height += 3;
    }
    let panel_height = height.min(size.height.saturating_sub(4));

    let x = size.width.saturating_sub(panel_width) / 2;
    let y = 2;

    let area = Rect::new(x, y, panel_width, panel_height);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        format!(" {} says:", speaker),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let words: Vec<&str> = text.split_whitespace().collect();
    let max_line_len = (panel_width as usize).saturating_sub(4);
    let mut current_line = String::new();
    for word in &words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_line_len {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(Line::from(Span::styled(
                format!(" {}", current_line),
                Style::default().fg(Color::White),
            )));
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" {}", current_line),
            Style::default().fg(Color::White),
        )));
    }

    if quest_offer {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " I have a task for you...",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [Enter] Accept  [Esc] Decline",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " [Esc] Close",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING)
        .title(" Dialogue ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
