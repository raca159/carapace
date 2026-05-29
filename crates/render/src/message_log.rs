use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use bevy_ecs::world::World;

use game_core::MessageLog;

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

const MESSAGE_LOG_HEIGHT: u16 = 5;

pub fn draw_message_log(frame: &mut Frame, ecs_world: &mut World) {
    let log = match ecs_world.get_resource::<MessageLog>() {
        Some(l) => l.clone(),
        None => return,
    };

    let messages = log.recent(3);
    if messages.is_empty() {
        return;
    }

    let size = frame.area();
    let log_height = (messages.len() as u16 + 2).min(MESSAGE_LOG_HEIGHT);
    let y = size.height.saturating_sub(log_height + 1);
    let area = Rect::new(0, y, size.width, log_height);

    let lines: Vec<Line> = messages
        .iter()
        .map(|msg| Line::from(Span::styled(format!(" {}", msg), Style::default().fg(Color::White))))
        .collect();

    let block = Block::default()
        .borders(Borders::TOP)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Messages ")
        .title_style(Style::default().fg(Color::DarkGray))
        .padding(PANEL_PADDING);

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
