use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use game_core::crafting::RecipeAvailability;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub fn draw_crafting_overlay(
    frame: &mut Frame,
    avail: &[RecipeAvailability],
    cursor: usize,
) {
    let size = frame.area();
    let panel_width = 36u16.min(size.width);
    let panel_height = (avail.len() as u16 + 4).min(size.height.saturating_sub(4));

    let x = size.width.saturating_sub(panel_width) / 2;
    let y = 2;

    let area = Rect::new(x, y, panel_width, panel_height);

    let mut lines: Vec<Line> = Vec::new();

    if avail.is_empty() {
        lines.push(Line::from(Span::styled(
            " No recipes known",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, ra) in avail.iter().enumerate() {
            let selector = if i == cursor { ">" } else { " " };
            let input_str = ra.recipe.inputs.join(", ");
            let style = if ra.available {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let env_note = if ra.env_met || ra.recipe.requires_env.is_empty() {
                String::new()
            } else {
                format!(" [need {}]", ra.recipe.requires_env.join(","))
            };
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", selector), Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:20}", ra.recipe.name), style),
                Span::styled(format!("[{}]{}", input_str, env_note), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Up/Down: Select | Enter: Craft | Esc: Close",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING)
        .title(" Crafting ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
