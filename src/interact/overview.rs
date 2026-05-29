use bevy::prelude::*;
use game_core::screen::AppScreen;
use game_core::world_overview::{WorldOverviewState, OverviewTab};
use game_core::TurnCounter;
use game_core::GameEventLog;
use game_world::{Tile, TilePos, WorldMap};
use game_world::cascade::LocationMap;
use game_world::faction::FactionRelationships;
use game_world::cascade::RegionEconomies;
use game_world::cascade::trade::TradeRoutes;
use crate::render::GameWorld;

const MINIMAP_MAX_PX: f32 = 380.0;

#[derive(Resource, Default)]
pub struct OverviewOverlay {
    pub root: Option<Entity>,
    pub cursor: Option<Entity>,
    pub panel: Option<Entity>,
    pub help: Option<Entity>,
    pub info: Option<Entity>,
    pub tab_text: Option<Entity>,
    pub last_zoom: u32,
    pub last_tab: OverviewTab,
    pub last_scroll: usize,
    pub cell_px: f32,
    pub cells_w: u32,
    pub cells_h: u32,
    pub last_cursor: (u32, u32),
}

pub fn spawn_world_overview(
    mut commands: Commands,
    mut game_world: ResMut<GameWorld>,
    mut overlay: ResMut<OverviewOverlay>,
) {
    if let Some(root) = overlay.root.take() {
        commands.entity(root).despawn_recursive();
    }
    if let Some(help) = overlay.help.take() {
        commands.entity(help).despawn();
    }

    let gw = &mut game_world.0;
    let ov = match gw.get_resource::<WorldOverviewState>() {
        Some(s) => s.clone(),
        None => return,
    };
    let map = match gw.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return,
    };

    let tpc = ov.tiles_per_cell();
    let cell_w = (map.width + tpc - 1) / tpc;
    let cell_h = (map.height + tpc - 1) / tpc;
    let cell_px = (MINIMAP_MAX_PX / cell_w.max(cell_h) as f32).floor().max(2.0);
    let minimap_w = cell_w as f32 * cell_px;
    let minimap_h = cell_h as f32 * cell_px;

    overlay.cell_px = cell_px;
    overlay.cells_w = cell_w;
    overlay.cells_h = cell_h;
    overlay.last_zoom = ov.zoom;
    overlay.last_tab = ov.tab;
    overlay.last_cursor = (ov.cursor_x, ov.cursor_y);

    let mut cursor_e = Entity::from_raw(0);
    let mut panel_e = Entity::from_raw(0);
    let mut info_e = Entity::from_raw(0);
    let mut tab_e = Entity::from_raw(0);

    let panel_content = build_panel_content(gw, &ov, &map);
    let info_text = build_info_text(gw, &ov, &map);

    let root_e = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
        Visibility::Visible,

    )).with_children(|root| {
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(minimap_w + 20.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        )).with_children(|left| {
            left.spawn((
                Text("WORLD OVERVIEW".to_string()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 1.0, 0.8)),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));

            left.spawn((
                Node {
                    width: Val::Px(minimap_w),
                    height: Val::Px(minimap_h),
                    position_type: PositionType::Relative,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            )).with_children(|grid| {
                for cy in 0..cell_h {
                    for cx in 0..cell_w {
                        let (r, g, b) = average_tile_color(gw, &map, cx, cy, tpc);
                        grid.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(cx as f32 * cell_px),
                                top: Val::Px(cy as f32 * cell_px),
                                width: Val::Px(cell_px),
                                height: Val::Px(cell_px),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(r, g, b)),
                        ));
                    }
                }
            });

            let pcx = (ov.player_x / tpc) as f32 * cell_px;
            let pcy = (ov.player_y / tpc) as f32 * cell_px;
            left.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(pcx),
                    top: Val::Px(pcy + 24.0),
                    width: Val::Px(cell_px.max(3.0)),
                    height: Val::Px(cell_px.max(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 1.0, 0.0)),
            ));

            let ccx = (ov.cursor_x / tpc) as f32 * cell_px;
            let ccy = (ov.cursor_y / tpc) as f32 * cell_px;
            cursor_e = left.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(ccx),
                    top: Val::Px(ccy + 24.0),
                    width: Val::Px(cell_px),
                    height: Val::Px(cell_px),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
            )).id();

            info_e = left.spawn((
                Text(info_text),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node { margin: UiRect::top(Val::Px(6.0)), ..default() },
            )).id();
        });

        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Px(340.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        )).with_children(|right| {
            tab_e = right.spawn((
                Text("[1] Map  |  [2] History  |  [3] Civ".to_string()),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.8, 0.9, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
            )).id();

            panel_e = right.spawn((
                Text(panel_content),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::srgb(0.7, 0.8, 1.0)),
                Node {
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            )).id();
        });
    }).id();

    let help_e = commands.spawn((
        Text("[M/Esc] Close  |  Arrows: Move cursor  |  +/-: Zoom  |  [1/2/3] Tabs".to_string()),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.6, 0.6, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(4.0),
            left: Val::Px(8.0),
            ..default()
        },

    )).id();

    overlay.root = Some(root_e);
    overlay.cursor = Some(cursor_e);
    overlay.panel = Some(panel_e);
    overlay.help = Some(help_e);
    overlay.info = Some(info_e);
    overlay.tab_text = Some(tab_e);
    overlay.last_scroll = 0;
}
pub fn handle_world_overview_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_world: ResMut<GameWorld>,
    mut next_screen: ResMut<NextState<AppScreen>>,
    mut overlay: ResMut<OverviewOverlay>,
    mut text_query: Query<&mut Text>,
    mut node_query: Query<&mut Node>,
) {
    let gw = &mut game_world.0;
    let map = match gw.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return,
    };
    let mut ov = match gw.get_resource::<WorldOverviewState>() {
        Some(s) => s.clone(),
        None => return,
    };
    let mut changed = false;

    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyM) {
        ov.active = false;
        *gw.get_resource_mut::<WorldOverviewState>().unwrap() = ov;
        if AppScreen::transition_allowed(&AppScreen::WorldOverview, &AppScreen::InWorld) {
            next_screen.set(AppScreen::InWorld);
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        ov.tab = OverviewTab::Minimap;
        changed = true;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        ov.tab = OverviewTab::History;
        changed = true;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        ov.tab = OverviewTab::CivSummary;
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        ov.zoom_in();
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        ov.zoom_out();
        changed = true;
    }

    let tpc = ov.tiles_per_cell();
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        ov.cursor_y = ov.cursor_y.saturating_sub(tpc);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        ov.cursor_y = (ov.cursor_y + tpc).min(map.height.saturating_sub(1));
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        ov.cursor_x = ov.cursor_x.saturating_sub(tpc);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        ov.cursor_x = (ov.cursor_x + tpc).min(map.width.saturating_sub(1));
        changed = true;
    }

    if keyboard.just_pressed(KeyCode::PageUp) {
        match ov.tab {
            OverviewTab::History => { ov.history_scroll = ov.history_scroll.saturating_sub(10); changed = true; }
            OverviewTab::CivSummary => { ov.civ_scroll = ov.civ_scroll.saturating_sub(10); changed = true; }
            _ => {}
        }
    }
    if keyboard.just_pressed(KeyCode::PageDown) {
        match ov.tab {
            OverviewTab::History => { ov.history_scroll += 10; changed = true; }
            OverviewTab::CivSummary => { ov.civ_scroll += 10; changed = true; }
            _ => {}
        }
    }

    if !changed {
        return;
    }

    let needs_respawn = ov.zoom != overlay.last_zoom;
    *gw.get_resource_mut::<WorldOverviewState>().unwrap() = ov.clone();

    if needs_respawn {
        overlay.last_zoom = ov.zoom;
        overlay.last_tab = ov.tab;
        overlay.last_cursor = (ov.cursor_x, ov.cursor_y);
        if let Some(root) = overlay.root.take() {
            commands.entity(root).despawn_recursive();
        }
        if let Some(help) = overlay.help.take() {
            commands.entity(help).despawn();
        }
        drop(map); drop(ov);
        spawn_world_overview(commands, game_world, overlay);
        return;
    }

    if (ov.cursor_x, ov.cursor_y) != overlay.last_cursor {
        overlay.last_cursor = (ov.cursor_x, ov.cursor_y);
        if let Some(cursor_e) = overlay.cursor {
            if let Ok(mut node) = node_query.get_mut(cursor_e) {
                let ccx = (ov.cursor_x / tpc) as f32 * overlay.cell_px;
                let ccy = (ov.cursor_y / tpc) as f32 * overlay.cell_px;
                node.left = Val::Px(ccx);
                node.top = Val::Px(ccy + 24.0);
            }
        }
    }

    if let Some(info_e) = overlay.info {
        if let Ok(mut text) = text_query.get_mut(info_e) {
            text.0 = build_info_text(gw, &ov, &map);
        }
    }

    if ov.tab != overlay.last_tab {
        overlay.last_tab = ov.tab;
        overlay.last_scroll = match ov.tab {
            OverviewTab::History => ov.history_scroll,
            _ => 0,
        };
        let content = build_panel_content(gw, &ov, &map);
        if let Some(panel_e) = overlay.panel {
            if let Ok(mut text) = text_query.get_mut(panel_e) {
                text.0 = content;
            }
        }
        if let Some(tab_e) = overlay.tab_text {
            if let Ok(mut text) = text_query.get_mut(tab_e) {
                text.0 = "[1] Map  |  [2] History  |  [3] Civ".to_string();
            }
        }
    }
}

pub fn despawn_world_overview(
    mut commands: Commands,
    mut overlay: ResMut<OverviewOverlay>,
) {
    if let Some(root) = overlay.root.take() {
        commands.entity(root).despawn_recursive();
    }
    if let Some(help) = overlay.help.take() {
        commands.entity(help).despawn();
    }
    overlay.cursor = None;
    overlay.panel = None;
    overlay.info = None;
    overlay.tab_text = None;
}

fn average_tile_color(gw: &bevy_ecs::prelude::World, map: &WorldMap, cx: u32, cy: u32, tpc: u32) -> (f32, f32, f32) {
    let mut r_sum = 0u32;
    let mut g_sum = 0u32;
    let mut b_sum = 0u32;
    let mut count = 0u32;
    for dy in 0..tpc {
        for dx in 0..tpc {
            let wx = cx * tpc + dx;
            let wy = cy * tpc + dy;
            if wx >= map.width || wy >= map.height { continue; }
            if let Some(te) = map.get(TilePos::new(wx, wy)) {
                if let Some(tile) = gw.get::<Tile>(te) {
                    r_sum += tile.color.0 as u32;
                    g_sum += tile.color.1 as u32;
                    b_sum += tile.color.2 as u32;
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        (r_sum as f32 / count as f32 / 255.0,
         g_sum as f32 / count as f32 / 255.0,
         b_sum as f32 / count as f32 / 255.0)
    } else {
        (0.0, 0.0, 0.0)
    }
}

fn get_biome_name(gw: &bevy_ecs::prelude::World, map: &WorldMap, x: u32, y: u32) -> String {
    if let Some(te) = map.get(TilePos::new(x, y)) {
        if let Some(tile) = gw.get::<Tile>(te) {
            return tile.biome_name.trim_start_matches("BIOME_").replace("_", " ").to_lowercase();
        }
    }
    "unknown".to_string()
}

fn location_near(x: u32, y: u32, locations: &[game_world::cascade::PlacedLocation]) -> Option<(String, u32)> {
    let search = 10u32;
    for loc in locations {
        let dx = (x as i32 - loc.x as i32).unsigned_abs();
        let dy = (y as i32 - loc.y as i32).unsigned_abs();
        let dist = dx + dy;
        if dist <= search {
            return Some((loc.name.clone(), dist));
        }
    }
    None
}

fn build_info_text(gw: &bevy_ecs::prelude::World, ov: &WorldOverviewState, map: &WorldMap) -> String {
    let biome = get_biome_name(gw, map, ov.cursor_x, ov.cursor_y);
    let nearby = gw.get_resource::<LocationMap>()
        .and_then(|lm| location_near(ov.cursor_x, ov.cursor_y, &lm.locations));
    let mut s = format!("Cursor: ({}, {})  biome: {}", ov.cursor_x, ov.cursor_y, biome);
    if let Some((name, dist)) = nearby {
        s.push_str(&format!("\nNear: {} ({} tiles)", name, dist));
    }
    s.push_str(&format!("\nPlayer: ({}, {})", ov.player_x, ov.player_y));
    s.push_str(&format!("\nZoom: {} ({}:{})", ov.zoom, ov.tiles_per_cell(), 1));
    s
}

fn build_panel_content(gw: &bevy_ecs::prelude::World, ov: &WorldOverviewState, map: &WorldMap) -> String {
    match ov.tab {
        OverviewTab::Minimap => build_minimap_tab_content(gw, ov),
        OverviewTab::History => build_history_content(gw, ov),
        OverviewTab::CivSummary => build_civ_summary_content(gw),
    }
}

fn build_minimap_tab_content(gw: &bevy_ecs::prelude::World, ov: &WorldOverviewState) -> String {
    let locations = gw.get_resource::<LocationMap>();
    let locs = match locations {
        Some(lm) => {
            let nearby: Vec<_> = lm.locations.iter()
                .filter(|l| {
                    let dx = (ov.cursor_x as i32 - l.x as i32).unsigned_abs();
                    let dy = (ov.cursor_y as i32 - l.y as i32).unsigned_abs();
                    dx + dy <= 30
                })
                .collect();
            if nearby.is_empty() {
                "No locations nearby.".to_string()
            } else {
                let mut s = format!("Locations near cursor ({}):", nearby.len());
                for loc in &nearby {
                    s.push_str(&format!("\n  [{}] {} @({},{})", loc.location_type, loc.name, loc.x, loc.y));
                }
                s
            }
        }
        None => "No location data.".to_string(),
    };
    format!("=== MAP INFO ===\n{}", locs)
}

fn build_history_content(gw: &bevy_ecs::prelude::World, ov: &WorldOverviewState) -> String {
    let turn = gw.get_resource::<TurnCounter>()
        .map(|t| t.current())
        .unwrap_or(0);
    let event_log = gw.get_resource::<GameEventLog>()
        .map(|el| {
            let start = ov.history_scroll.min(el.events.len().saturating_sub(1));
            let end = (start + 15).min(el.events.len());
            if start >= el.events.len() {
                vec!["(no more events)".to_string()]
            } else {
                el.events[start..end].iter().map(|e| {
                    let registry = gw.get_resource::<game_tags::TagRegistry>();
                    registry.as_ref()
                        .map(|r| game_core::events::format_event(e, r))
                        .unwrap_or_else(|| format!("{:?}", e))
                }).collect()
            }
        })
        .unwrap_or_else(|| vec!["No events recorded.".to_string()]);
    let mut s = format!("=== HISTORY === (Turn {})", turn);
    s.push_str("\n--- Events ---");
    for line in &event_log {
        s.push_str(&format!("\n  {}", line));
    }
    s.push_str("\n\n[PgUp/PgDn] Scroll");
    s
}

fn build_civ_summary_content(gw: &bevy_ecs::prelude::World) -> String {
    let factions = gw.get_resource::<FactionRelationships>();
    let locations = gw.get_resource::<LocationMap>();
    let economies = gw.get_resource::<RegionEconomies>();
    let trade = gw.get_resource::<TradeRoutes>();
    let mut s = "=== CIVILIZATION SUMMARY ===".to_string();
    if let Some(fr) = factions {
        let names: Vec<String> = fr.name_id_pairs()
            .map(|(name, _)| name.clone())
            .collect();
        s.push_str(&format!("\n\nFactions ({}):", names.len()));
        for name in &names {
            let fid = fr.faction_id(name).unwrap();
            let standing = fr.get_standing(game_world::faction::PLAYER_FACTION_ID, fid);
            s.push_str(&format!("\n  {} — {:?} with player", name, standing));
        }
    } else {
        s.push_str("\n\nNo faction data.");
    }
    if let Some(lm) = locations {
        let mut by_type: std::collections::BTreeMap<String, Vec<&game_world::cascade::PlacedLocation>> = std::collections::BTreeMap::new();
        for loc in &lm.locations {
            by_type.entry(loc.location_type.clone()).or_insert(Vec::new()).push(loc);
        }
        s.push_str(&format!("\n\nLocations ({} total):", lm.locations.len()));
        for (loc_type, locs) in &by_type {
            s.push_str(&format!("\n  {} ({}):", loc_type, locs.len()));
            for loc in locs.iter().take(5) {
                let faction = loc.faction.as_deref().unwrap_or("wild");
                s.push_str(&format!("\n    {} @({},{}) — {}", loc.name, loc.x, loc.y, faction));
            }
            if locs.len() > 5 {
                s.push_str(&format!("\n    ... and {} more", locs.len() - 5));
            }
        }
    }
    if let Some(regions) = economies {
        s.push_str(&format!("\n\nEconomy ({} active markets):", regions.economies.len()));
        for (id, pricing) in regions.economies.iter().take(8) {
            let loc_name = locations
                .and_then(|lm| lm.locations.iter().find(|l| l.id == *id))
                .map(|l| l.name.as_str())
                .unwrap_or("unknown");
            s.push_str(&format!("\n  {}: prosperity {:.1}", loc_name, pricing.prosperity));
        }
        if regions.economies.len() > 8 {
            s.push_str(&format!("\n  ... and {} more", regions.economies.len() - 8));
        }
    }
    if let Some(tr) = trade {
        s.push_str(&format!("\n\nTrade routes: {}", tr.0.len()));
    }
    s
}
