use bevy::prelude::*;
use bevy::window::WindowResized;
use game_core::screen::AppScreen;
use game_core::components::{Player, Position, Glyph, Health, Name};
use game_core::{MessageLog, ExamineMode};
use game_core::color_theme::desaturate_color;
use game_world::{Tile, TilePos, WorldMap};
use game_render::spritesheet::{self, SpriteAtlas};
use crate::interact::overview::{self, OverviewOverlay};

const TILE_SIZE: f32 = 16.0;
const MINIMAP_TILE_SIZE: f32 = 5.0;
const MINIMAP_W: u32 = 20;
const MINIMAP_H: u32 = 15;

#[derive(Resource, Default)]
pub struct GameCamera {
    pub x: u32,
    pub y: u32,
    #[allow(dead_code)]
    pub z: u32,
}

#[derive(Resource)]
pub struct GameWorld(pub bevy_ecs::prelude::World);

impl Default for GameWorld {
    fn default() -> Self {
        Self(bevy_ecs::prelude::World::new())
    }
}

#[derive(Resource, Default)]
pub struct TileSprites(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct EntitySprites(pub Vec<Entity>);

#[derive(Resource)]
pub struct HudEntities {
    pub hp_text: Entity,
    pub pos_text: Entity,
    pub biome_text: Entity,
    pub msg_text: Entity,
}

#[derive(Resource, Default)]
pub struct MinimapTiles(pub Vec<Entity>);

#[derive(Resource, Default)]
pub struct ExaminePanel(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct PauseOverlay(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct DeathOverlay(pub Option<Entity>);

#[derive(Resource, Default)]
pub struct DisambiguationPanel(pub Option<Entity>);

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AppScreen>()
            .init_resource::<GameCamera>()
            .init_resource::<GameWorld>()
            .init_resource::<TileSprites>()
            .init_resource::<EntitySprites>()
            .init_resource::<ExaminePanel>()
            .init_resource::<PauseOverlay>()
            .init_resource::<DeathOverlay>()
            .init_resource::<DisambiguationPanel>()
            .init_resource::<MinimapTiles>()
            .init_resource::<crate::interact::consume::ConsumeOverlay>()
            .init_resource::<crate::interact::talk::TalkPanel>()
            .init_resource::<crate::interact::throw::ThrowOverlay>()
            .init_resource::<crate::interact::craft::CraftPanel>()
            .init_resource::<crate::interact::quest_board::QuestBoardPanel>()
            .init_resource::<crate::interact::loot::LootPanel>()
            .init_resource::<OverviewOverlay>()
            .add_systems(OnEnter(AppScreen::PauseMenu), spawn_pause_overlay)
            .add_systems(OnExit(AppScreen::PauseMenu), despawn_pause_overlay)
            .add_systems(OnEnter(AppScreen::Dead), spawn_death_screen)
            .add_systems(OnExit(AppScreen::Dead), despawn_death_screen)
            .add_systems(OnEnter(AppScreen::WorldOverview), overview::spawn_world_overview)
            .add_systems(OnExit(AppScreen::WorldOverview), overview::despawn_world_overview)
            .add_systems(Startup, setup_atlas)
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(AppScreen::Boot), boot_sequence)
            .add_systems(Startup, setup_hud)
            .add_systems(Update, handle_window_resize)
            .add_systems(Update, (
                boot_to_main_menu,
                render_terrain,
                render_entities,
                lerp_camera,
                update_hud.run_if(in_state(AppScreen::InWorld)),
                render_examine_panel.run_if(in_state(AppScreen::InWorld)),
                render_disambiguation_panel.run_if(in_state(AppScreen::InWorld)),
                overview::handle_world_overview_input.run_if(in_state(AppScreen::WorldOverview)),
            ).chain());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_atlas(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let config = spritesheet::load_atlas_config("assets/sprites/atlas.toml");
    let atlas = spritesheet::build_sprite_atlas(&config, images, layouts);
    commands.insert_resource(atlas);
}

fn boot_sequence(mut next_state: ResMut<NextState<AppScreen>>) {
    next_state.set(AppScreen::Boot);
}

fn handle_window_resize(
    mut resize_events: EventReader<WindowResized>,
) {
    for ev in resize_events.read() {
        // Bevy's flexbox layout recalculates automatically for UI children
        // with Val::Percent, so this is mainly a hook for future use.
        let _ = ev;
    }
}

fn boot_to_main_menu(
    screen: Res<State<AppScreen>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppScreen>>,
    mut timer: Local<f32>,
) {
    if screen.get() != &AppScreen::Boot {
        return;
    }
    *timer += time.delta_secs();
    if *timer >= 1.0 {
        if AppScreen::transition_allowed(&AppScreen::Boot, &AppScreen::MainMenu) {
            next_state.set(AppScreen::MainMenu);
        }
    }
}

fn render_terrain(
    mut commands: Commands,
    game_world: ResMut<GameWorld>,
    atlas: Option<Res<SpriteAtlas>>,
    mut tile_sprites: ResMut<TileSprites>,
    camera: Res<GameCamera>,
    windows: Query<&Window>,
) {
    let atlas = match atlas {
        Some(a) => a,
        None => return,
    };

    let gw = &game_world.0;
    let map = match gw.get_resource::<WorldMap>() {
        Some(m) => m,
        None => {
            for e in tile_sprites.0.drain(..) { commands.entity(e).despawn(); }
            return;
        }
    };

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    // Count visible tiles (viewport in tile units, plus 1 tile padding)
    let vw = (window.width() / TILE_SIZE) as u32 + 2;
    let vh = (window.height() / TILE_SIZE) as u32 + 2;
    let total = (vw * vh) as usize;

    // Resize sprite pool
    while tile_sprites.0.len() > total {
        if let Some(e) = tile_sprites.0.pop() { commands.entity(e).despawn(); }
    }
    while tile_sprites.0.len() < total {
        let e = commands.spawn((
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas { layout: atlas.layout.clone(), index: 0 },
            ),
            Transform::default(),
            Visibility::Visible,
        )).id();
        tile_sprites.0.push(e);
    }

    let cx = camera.x;
    let cy = camera.y;

    for sy in 0..vh {
        for sx in 0..vw {
            let idx = (sy * vw + sx) as usize;
            if idx >= tile_sprites.0.len() { continue; }
            let entity = tile_sprites.0[idx];

            let wx = cx + sx;
            let wy = cy + sy;

            let px = wx as f32 * TILE_SIZE;
            let py = -(wy as f32) * TILE_SIZE;

            if wx >= map.width || wy >= map.height {
                commands.entity(entity).insert((
                    Transform::from_xyz(px, py, -1.0),
                    Visibility::Hidden,
                ));
                continue;
            }

            let te = map.get_unchecked(TilePos::new(wx, wy));
            let tile = match gw.get::<Tile>(te) {
                Some(t) => t,
                None => {
                    commands.entity(entity).insert((
                        Transform::from_xyz(px, py, -1.0),
                        Visibility::Hidden,
                    ));
                    continue;
                }
            };

            let dim = desaturate_color(tile.color, 0.35);
            let gi = atlas.glyph_index(tile.glyph);
            let sc = Color::srgb(
                dim.0 as f32 / 255.0,
                dim.1 as f32 / 255.0,
                dim.2 as f32 / 255.0,
            );

            commands.entity(entity).insert((
                Transform::from_xyz(px, py, 0.0),
                Visibility::Visible,
                Sprite {
                    color: sc,
                    ..Sprite::from_atlas_image(
                        atlas.texture.clone(),
                        TextureAtlas { layout: atlas.layout.clone(), index: gi },
                    )
                },
            ));
        }
    }
}

fn render_entities(
    mut commands: Commands,
    mut game_world: ResMut<GameWorld>,
    atlas: Option<Res<SpriteAtlas>>,
    mut entity_sprites: ResMut<EntitySprites>,
    camera: Res<GameCamera>,
    windows: Query<&Window>,
) {
    let atlas = match atlas {
        Some(a) => a,
        None => return,
    };

    let gw = &mut game_world.0;
    if gw.get_resource::<WorldMap>().is_none() {
        for e in entity_sprites.0.drain(..) { commands.entity(e).despawn(); }
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let vw = (window.width() / TILE_SIZE) as i64 + 2;
    let vh = (window.height() / TILE_SIZE) as i64 + 2;
    let cx = camera.x as i64;
    let cy = camera.y as i64;

    // Collect visible entities with render data
    let mut screen_entities: Vec<(f32, f32, u32, usize, Color, bool)> = Vec::new();
    {
        let mut entity_query = gw.query::<(bevy_ecs::entity::Entity, &Position, &Glyph)>();
        let mut player_query = gw.query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>();
        let player_entity = player_query.iter(gw).next();

        for (entity, pos, glyph) in entity_query.iter(gw) {
            let sx = pos.x as i64 - cx;
            let sy = pos.y as i64 - cy;
            if sx >= 0 && sx < vw && sy >= 0 && sy < vh {
                let is_player = Some(entity) == player_entity;
                let z = if is_player { 100 } else { 50 + pos.z };
                let gi = atlas.glyph_index(glyph.char);
                let sc = Color::srgb(
                    glyph.color.0 as f32 / 255.0,
                    glyph.color.1 as f32 / 255.0,
                    glyph.color.2 as f32 / 255.0,
                );
                screen_entities.push((
                    pos.x as f32 * TILE_SIZE,
                    -(pos.y as f32) * TILE_SIZE,
                    z, gi, sc, is_player,
                ));
            }
        }
    }

    // Sort: non-players first by z, then player last (renders on top)
    screen_entities.sort_by_key(|e| (e.5, e.2));

    // Resize sprite pool
    while entity_sprites.0.len() > screen_entities.len() {
        if let Some(e) = entity_sprites.0.pop() { commands.entity(e).despawn(); }
    }
    while entity_sprites.0.len() < screen_entities.len() {
        let e = commands.spawn((
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas { layout: atlas.layout.clone(), index: 0 },
            ),
            Transform::default(),
            Visibility::Visible,
        )).id();
        entity_sprites.0.push(e);
    }

    for (i, (px, py, z, gi, sc, _)) in screen_entities.iter().enumerate() {
        if i >= entity_sprites.0.len() { break; }
        commands.entity(entity_sprites.0[i]).insert((
            Transform::from_xyz(*px, *py, *z as f32),
            Visibility::Visible,
            Sprite {
                color: *sc,
                ..Sprite::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas { layout: atlas.layout.clone(), index: *gi },
                )
            },
        ));
    }
}

pub fn setup_hud(mut commands: Commands) {
    let mut hp_text = Entity::from_raw(0);
    let mut pos_text = Entity::from_raw(0);
    let mut biome_text = Entity::from_raw(0);
    let mut msg_text = Entity::from_raw(0);
    let mut tile_entities = Vec::with_capacity((MINIMAP_W * MINIMAP_H) as usize);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Visibility::Visible,
        ))
        .with_children(|parent| {
            // Top bar panel — flex row with semi-transparent background
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(24.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
                BorderColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
            ))
            .with_children(|bar| {
                hp_text = bar.spawn((
                    Text("HP: 100/100".to_string()),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    Node { margin: UiRect::right(Val::Px(24.0)), ..default() },
                )).id();

                pos_text = bar.spawn((
                    Text("(0, 0)".to_string()),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                    Node { margin: UiRect::right(Val::Px(24.0)), ..default() },
                )).id();

                biome_text = bar.spawn((
                    Text("".to_string()),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.0, 1.0)),
                    Node { ..default() },
                )).id();
            });

            // Message log panel — bottom-left with semi-transparent background
            msg_text = parent.spawn((
                Text("".to_string()),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(6.0),
                    left: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    max_width: Val::Px(420.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                BorderColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
            )).id();

            // Minimap panel — bottom-right with semi-transparent background
            let minimap_w = MINIMAP_W as f32 * MINIMAP_TILE_SIZE + 2.0;
            let minimap_h = MINIMAP_H as f32 * MINIMAP_TILE_SIZE + 2.0;
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(6.0),
                    right: Val::Px(8.0),
                    width: Val::Px(minimap_w),
                    height: Val::Px(minimap_h),
                    padding: UiRect::all(Val::Px(1.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                BorderColor(Color::srgba(0.3, 0.3, 0.3, 0.6)),
            ))
            .with_children(|minimap| {
                for y in 0..MINIMAP_H {
                    for x in 0..MINIMAP_W {
                        let tile = minimap.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(1.0 + x as f32 * MINIMAP_TILE_SIZE),
                                top: Val::Px(1.0 + y as f32 * MINIMAP_TILE_SIZE),
                                width: Val::Px(MINIMAP_TILE_SIZE),
                                height: Val::Px(MINIMAP_TILE_SIZE),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                        )).id();
                        tile_entities.push(tile);
                    }
                }
            });
        });

    commands.insert_resource(HudEntities { hp_text, pos_text, biome_text, msg_text });
    commands.insert_resource(MinimapTiles(tile_entities));
}

pub fn update_hud(
    mut game_world: ResMut<GameWorld>,
    hud: Res<HudEntities>,
    minimap: Res<MinimapTiles>,
    mut text_query: Query<&mut Text>,
    mut bg_query: Query<&mut BackgroundColor>,
) {
    let gw = &mut game_world.0;

    let (hp, pos) = {
        let mut q =
            gw.query_filtered::<(&Health, &Position), bevy_ecs::query::With<Player>>();
        match q.iter(gw).next() {
            Some((h, p)) => (*h, *p),
            None => return,
        }
    };

    if let Ok(mut text) = text_query.get_mut(hud.hp_text) {
        text.0 = format!("HP: {}/{}", hp.current, hp.max);
    }
    if let Ok(mut text) = text_query.get_mut(hud.pos_text) {
        text.0 = format!("({}, {})", pos.x, pos.y);
    }

    let biome = gw.get_resource::<WorldMap>().and_then(|map| {
        let te = map.get(TilePos::new(pos.x, pos.y))?;
        gw.get::<Tile>(te).map(|t| t.biome_name.clone())
    }).unwrap_or_default();
    if let Ok(mut text) = text_query.get_mut(hud.biome_text) {
        text.0 = biome;
    }

    // Message log — show last 5 messages
    let msg_text_val = gw.get_resource::<MessageLog>()
        .map(|log| {
            let start = log.messages.len().saturating_sub(5);
            log.messages[start..].join("\n")
        })
        .unwrap_or_default();
    if let Ok(mut text) = text_query.get_mut(hud.msg_text) {
        text.0 = msg_text_val;
    }

    // Minimap — update tile colors from terrain around player
    let map = match gw.get_resource::<WorldMap>() {
        Some(m) => m,
        None => return,
    };

    let half_w = MINIMAP_W as i64 / 2;
    let half_h = MINIMAP_H as i64 / 2;
    let cx = pos.x as i64;
    let cy = pos.y as i64;

    for (i, entity) in minimap.0.iter().enumerate() {
        let mx = (i as u32 % MINIMAP_W) as i64 - half_w;
        let my = (i as u32 / MINIMAP_W) as i64 - half_h;
        let wx = cx + mx;
        let wy = cy + my;

        let color = if wx >= 0 && wy >= 0 && (wx as u32) < map.width && (wy as u32) < map.height {
            let te = map.get_unchecked(TilePos::new(wx as u32, wy as u32));
            gw.get::<Tile>(te).map(|t| {
                let (r, g, b) = t.color;
                Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
            }).unwrap_or(Color::srgb(0.05, 0.05, 0.05))
        } else {
            Color::srgb(0.02, 0.02, 0.02)
        };

        if let Ok(mut bg) = bg_query.get_mut(*entity) {
            bg.0 = color;
        }
    }
}

pub fn render_examine_panel(
    mut game_world: ResMut<GameWorld>,
    mut panel: ResMut<ExaminePanel>,
    mut commands: Commands,
) {
    if let Some(e) = panel.0.take() {
        commands.entity(e).despawn();
    }

    let gw = &mut game_world.0;
    let cursor = match gw.get_resource::<ExamineMode>() {
        Some(e) if e.active => e.cursor,
        _ => return,
    };

    let mut lines = vec![format!("Examine ({}, {})", cursor.x, cursor.y)];

    if let Some(map) = gw.get_resource::<WorldMap>() {
        if let Some(te) = map.get(TilePos::new(cursor.x, cursor.y)) {
            if let Some(tile) = gw.get::<Tile>(te) {
                lines.push(format!("Biome: {}", tile.biome_name));
            }
        }
    }

    {
        let mut q = gw.query::<(Entity, &Position, &Glyph, Option<&Name>)>();
        for (_entity, pos, glyph, name) in q.iter(gw) {
            if pos.x == cursor.x && pos.y == cursor.y {
                let n = name.map(|n| n.0.as_str()).unwrap_or("???");
                lines.push(format!(" {} {} — {}", glyph.char, n, "(entity)"));
            }
        }
    }

    let text = lines.join("\n");
    let e = commands.spawn((
        Text(text),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(28.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(e);
}

fn lerp_camera(
    mut game_world: ResMut<GameWorld>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let gw = &mut game_world.0;
    let mut player_query = gw.query_filtered::<&Position, bevy_ecs::query::With<Player>>();
    let pos = match player_query.iter(gw).next() {
        Some(p) => p,
        None => return,
    };

    let mut cam_transform = match camera_query.single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    let tgt_x = pos.x as f32 * TILE_SIZE;
    let tgt_y = -(pos.y as f32) * TILE_SIZE;
    let speed = 8.0 * time.delta_secs();
    let t = speed.min(1.0);

    cam_transform.translation.x = cam_transform.translation.x.lerp(tgt_x, t);
    cam_transform.translation.y = cam_transform.translation.y.lerp(tgt_y, t);
    cam_transform.translation.z = 1000.0;
}

pub fn render_disambiguation_panel(
    mut commands: Commands,
    interact: Res<crate::interact::InteractState>,
    mut panel: ResMut<DisambiguationPanel>,
    game_world: Res<GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    let targets = match &interact.active {
        Some(crate::interact::InteractMode::Disambiguating(t)) => t.clone(),
        _ => return,
    };

    let mut lines = vec!["Who / What?".to_string()];
    for (i, &entity) in targets.iter().enumerate() {
        let name = game_world.0.get::<Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", i + 1));
        lines.push(format!("  {}. {}", i + 1, name));
    }
    lines.push("".to_string());
    lines.push("[1-9] Select  |  Esc Cancel".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            top: Val::Px(28.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(root);
}

pub fn spawn_pause_overlay(mut commands: Commands, mut overlay: ResMut<PauseOverlay>) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text("PAUSED".to_string()),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
            ));
            parent.spawn((
                Text("ESC — Resume  |  Q — Quit".to_string()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        })
        .id();
    overlay.0 = Some(root);
}

pub fn despawn_pause_overlay(mut commands: Commands, mut overlay: ResMut<PauseOverlay>) {
    if let Some(e) = overlay.0.take() { commands.entity(e).despawn(); }
}

pub fn spawn_death_screen(mut commands: Commands, mut overlay: ResMut<DeathOverlay>) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text("YOU HAVE DIED".to_string()),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
                Node { margin: UiRect::bottom(Val::Px(20.0)), ..default() },
            ));
            parent.spawn((
                Text("Press ENTER or ESC to return to the menu".to_string()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        })
        .id();
    overlay.0 = Some(root);
}

pub fn despawn_death_screen(mut commands: Commands, mut overlay: ResMut<DeathOverlay>) {
    if let Some(e) = overlay.0.take() { commands.entity(e).despawn(); }
}
