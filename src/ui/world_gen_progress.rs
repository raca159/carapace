use bevy::prelude::*;
use crate::ui::UiEntities;

pub const STAGE_NAMES: &[&str] = &[
    "Loading Configurations",
    "Generating Terrain",
    "Placing Locations",
    "Computing Economy & Trade",
    "Loading Game Data",
    "Finalizing World",
    "Spawning Creatures",
    "Preparing Player",
];

pub const TOTAL_STAGES: usize = STAGE_NAMES.len();

#[derive(Resource)]
pub struct WorldGenProgress {
    pub current_stage: usize,
    pub done: bool,
    pub stage_timer: f32,
}

impl Default for WorldGenProgress {
    fn default() -> Self {
        Self {
            current_stage: 0,
            done: false,
            stage_timer: 0.0,
        }
    }
}

#[derive(Component)]
pub struct ProgressDisplay;

pub fn spawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
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
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text(String::new()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                ProgressDisplay,
            ));
        })
        .id();
    ui.root = Some(root);
}

pub fn despawn_ui(mut commands: Commands, mut ui: ResMut<UiEntities>) {
    if let Some(root) = ui.root.take() {
        commands.entity(root).despawn_recursive();
    }
}

pub fn update_progress_text(
    progress: Res<WorldGenProgress>,
    time: Res<Time>,
    mut query: Query<&mut Text, With<ProgressDisplay>>,
) {
    let mut text = match query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    let elapsed = time.elapsed_secs();
    let dots = match (elapsed * 2.0) as usize % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    };

    let pct = if TOTAL_STAGES > 0 {
        ((progress.current_stage as f32 / TOTAL_STAGES as f32) * 100.0).min(100.0) as usize
    } else {
        0
    };

    let filled = pct / 10;
    let empty = 10usize.saturating_sub(filled);
    let bar = format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty));

    let pulse = ((elapsed * 3.0).sin() * 0.5 + 0.5) > 0.6;

    let mut lines = Vec::new();
    lines.push(format!("GENERATING WORLD{}\n", dots));
    lines.push(format!(" {}  {}%\n", bar, pct));

    for (i, name) in STAGE_NAMES.iter().enumerate() {
        if i < progress.current_stage {
            lines.push(format!("  \u{2713} {}", name));
        } else if i == progress.current_stage && !progress.done {
            let arrow = if pulse { "\u{25b6}" } else { " " };
            lines.push(format!(" {} {}", arrow, name));
        } else {
            lines.push(format!("    {}", name));
        }
    }

    if progress.done {
        lines.push(String::new());
        lines.push("  Done!".to_string());
    }

    text.0 = lines.join("\n");
}
