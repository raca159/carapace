use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use bevy_ecs::prelude::World;
use bevy_ecs::query::With;

use game_core::{ActiveQuest, Health, Player, Position, Equipment, QuestState};
use game_world::{Tile, TilePos, WorldMap};

pub fn draw_hud(frame: &mut Frame, ecs_world: &mut World) {
    let size = frame.area();

    let player_pos = {
        let mut pq = ecs_world.query_filtered::<&Position, With<Player>>();
        pq.single(ecs_world).ok().copied()
    };

    if let Some(pos) = player_pos {
        let health = {
            let mut hq = ecs_world.query_filtered::<&Health, With<Player>>();
            hq.single(ecs_world).ok().copied()
        };
        if let Some(hp) = health {
            let hp_text = format!(" HP: {}/{} ", hp.current, hp.max);
            let hp_color = if hp.current < hp.max / 4 {
                Color::Red
            } else if hp.current < hp.max / 2 {
                Color::Yellow
            } else {
                Color::Green
            };
            let hp_span = Span::styled(hp_text, Style::default().fg(hp_color).add_modifier(Modifier::BOLD));
            let hp_para = Paragraph::new(Line::from(hp_span));
            let hp_area = Rect::new(0, 0, 20, 1);
            frame.render_widget(hp_para, hp_area);
        }

        let pos_text = format!(" ({}, {}) ", pos.x, pos.y);
        let pos_width = pos_text.len() as u16 + 2;
        let pos_span = Span::styled(pos_text, Style::default().fg(Color::Cyan));
        let pos_para = Paragraph::new(Line::from(pos_span)).alignment(Alignment::Right);
        let pos_area = Rect::new(size.width.saturating_sub(pos_width), 0, pos_width, 1);
        frame.render_widget(pos_para, pos_area);

        let equipment = {
            let mut eq = ecs_world.query_filtered::<&Equipment, With<Player>>();
            eq.single(ecs_world).ok().cloned()
        };
        let mut line2_parts: Vec<Span> = Vec::new();

        if let Some(equip) = equipment {
            line2_parts.push(Span::styled(" EQ:", Style::default().fg(Color::Cyan)));

            let w_name = equip.weapon.and_then(|e| {
                ecs_world.get::<game_core::Name>(e).map(|n| n.0.clone())
            }).unwrap_or_else(|| "-".to_string());
            let a_name = equip.armor.and_then(|e| {
                ecs_world.get::<game_core::Name>(e).map(|n| n.0.clone())
            }).unwrap_or_else(|| "-".to_string());

            line2_parts.push(Span::styled(format!(" W:{}", w_name), Style::default().fg(
                if equip.weapon.is_some() { Color::Green } else { Color::DarkGray }
            )));
            line2_parts.push(Span::styled(format!(" A:{}", a_name), Style::default().fg(
                if equip.armor.is_some() { Color::Green } else { Color::DarkGray }
            )));
        }

        {
            let mut quest_query = ecs_world.query::<&ActiveQuest>();
            let active_quest = quest_query.iter(ecs_world).find(|q| q.state == QuestState::Active || q.state == QuestState::Complete)
                .cloned();
            if let Some(quest) = active_quest {
                let progress = quest.objective.progress_text();
                let quest_color = if quest.state == QuestState::Complete {
                    Color::Green
                } else {
                    Color::Yellow
                };
                line2_parts.push(Span::styled("  Quest:", Style::default().fg(Color::Cyan)));
                line2_parts.push(Span::styled(format!(" {} [{}]", quest.name, progress), Style::default().fg(quest_color)));
            }
        }

        if !line2_parts.is_empty() {
            let line2_para = Paragraph::new(Line::from(line2_parts));
            let line2_area = Rect::new(0, 1, size.width, 1);
            frame.render_widget(line2_para, line2_area);
        }

        let biome_name = {
            let map = ecs_world.get_resource::<WorldMap>().cloned();
            match map {
                Some(ref m) => {
                    let tile_entity = m.get(TilePos::new(pos.x, pos.y));
                    match tile_entity {
                        Some(e) => ecs_world.query::<&Tile>().get(ecs_world, e).ok().map(|t| t.biome_name.clone()),
                        None => None,
                    }
                }
                None => None,
            }
        };
        if let Some(biome) = biome_name {
            let biome_span = Span::styled(format!(" {} ", biome), Style::default().fg(Color::Magenta));
            let biome_para = Paragraph::new(Line::from(biome_span));
            let biome_area = Rect::new(0, size.height.saturating_sub(1), 30, 1);
            frame.render_widget(biome_para, biome_area);
        }
    }
}
