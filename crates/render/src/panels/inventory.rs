use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use bevy_ecs::prelude::World;
use bevy_ecs::query::With;

use game_core::{Equipment, Glyph, Inventory, Player};
use game_tags::{TagRegistry, Tags};

use crate::world::{PANEL_PADDING, UNICODE_BORDER_SET};

pub const INVENTORY_PANEL_WIDTH: u16 = 30;

pub fn draw_inventory_overlay(frame: &mut Frame, ecs_world: &mut World, cursor: usize) {
    let size = frame.area();
    let panel_width = INVENTORY_PANEL_WIDTH.min(size.width);
    let panel_height = size.height.saturating_sub(4);

    let x = size.width.saturating_sub(panel_width) / 2;
    let y = 2;

    let area = Rect::new(x, y, panel_width, panel_height);

    let mut player_query = ecs_world.query_filtered::<(&Inventory, &Equipment), With<Player>>();
    let (inventory, equipment) = match player_query.single(ecs_world) {
        Ok((inv, equip)) => (inv.clone(), equip.clone()),
        Err(_) => return,
    };

    let registry = ecs_world.get_resource::<TagRegistry>().cloned();

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " Equipment:",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));

    let weapon_name = equipment.weapon.and_then(|e| {
        ecs_world.get::<game_core::Name>(e).map(|n| n.0.clone())
            .or_else(|| ecs_world.get::<Glyph>(e).map(|g| g.char.to_string()))
    }).unwrap_or_else(|| "(empty)".to_string());

    let armor_name = equipment.armor.and_then(|e| {
        ecs_world.get::<game_core::Name>(e).map(|n| n.0.clone())
            .or_else(|| ecs_world.get::<Glyph>(e).map(|g| g.char.to_string()))
    }).unwrap_or_else(|| "(empty)".to_string());

    let acc_name = equipment.accessory.and_then(|e| {
        ecs_world.get::<game_core::Name>(e).map(|n| n.0.clone())
            .or_else(|| ecs_world.get::<Glyph>(e).map(|g| g.char.to_string()))
    }).unwrap_or_else(|| "(empty)".to_string());

    let weapon_color = if equipment.weapon.is_some() { Color::Green } else { Color::DarkGray };
    let armor_color = if equipment.armor.is_some() { Color::Green } else { Color::DarkGray };
    let acc_color = if equipment.accessory.is_some() { Color::Green } else { Color::DarkGray };

    lines.push(Line::from(vec![
        Span::styled(" W: ", Style::default().fg(Color::Cyan)),
        Span::styled(weapon_name, Style::default().fg(weapon_color)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" A: ", Style::default().fg(Color::Cyan)),
        Span::styled(armor_name, Style::default().fg(armor_color)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" C: ", Style::default().fg(Color::Cyan)),
        Span::styled(acc_name, Style::default().fg(acc_color)),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(" Backpack ({}/{}):", inventory.items.len(), inventory.capacity),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if inventory.items.is_empty() {
        lines.push(Line::from(Span::styled(
            " (empty)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let mut glyph_query = ecs_world.query::<(&Glyph, Option<&Tags>)>();
        for (idx, &item_entity) in inventory.items.iter().enumerate() {
            if let Ok((glyph, tags)) = glyph_query.get(ecs_world, item_entity) {
                let color = Color::Rgb(glyph.color.0, glyph.color.1, glyph.color.2);
                let name = if let Some(name_comp) = ecs_world.get::<game_core::Name>(item_entity) {
                    name_comp.0.clone()
                } else if let Some(tags) = tags {
                    if let Some(ref reg) = registry {
                        let names: Vec<String> = tags
                            .iter_present()
                            .filter_map(|id| {
                                let n = &reg.tag_by_id(id).name;
                                if n.starts_with("ORE_") || n.starts_with("HERB_") || n.starts_with("GEM_") {
                                    Some(n.replace("_", " "))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if names.is_empty() {
                            format!("{}", glyph.char)
                        } else {
                            names.join(", ")
                        }
                    } else {
                        format!("{}", glyph.char)
                    }
                } else {
                    format!("{}", glyph.char)
                };
                let cursor_marker = if idx == cursor { ">" } else { " " };
                let name_style = if idx == cursor {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{} {} ", cursor_marker, glyph.char), Style::default().fg(color)),
                    Span::styled(name, name_style),
                ]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " [I] close  [E] equip  [U] unequip wpn",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(UNICODE_BORDER_SET)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(PANEL_PADDING)
        .title(" Inventory ");

    frame.render_widget(Clear, area);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
