use bevy::prelude::*;
use game_core::{CraftingRecipe, Inventory, Player};
use game_tags::TagRegistry;
use crate::interact::{InteractState, InteractMode};

#[derive(Resource, Default)]
pub struct CraftPanel(pub Option<Entity>);

#[derive(Resource)]
pub struct CraftingRecipesResource {
    pub recipes: Vec<CraftingRecipe>,
}

pub fn update_craft_panel(
    mut commands: Commands,
    interact: Res<InteractState>,
    mut panel: ResMut<CraftPanel>,
    mut game_world: ResMut<crate::render::GameWorld>,
) {
    if let Some(old) = panel.0.take() { commands.entity(old).despawn(); }

    if !matches!(&interact.active, Some(InteractMode::Crafting)) {
        return;
    }

    let ecs_world = &mut game_world.0;

    let player = match ecs_world
        .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(e) => e,
        Err(_) => return,
    };

    let registry = match ecs_world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    let inventory = match ecs_world.get::<Inventory>(player) {
        Some(i) => i.clone(),
        None => return,
    };

    let player_pos = match ecs_world
        .query_filtered::<&game_core::Position, bevy_ecs::query::With<Player>>()
        .single(ecs_world)
    {
        Ok(p) => (p.x, p.y),
        Err(_) => return,
    };

    let recipes = match ecs_world.get_resource::<CraftingRecipesResource>() {
        Some(r) => r.recipes.clone(),
        None => return,
    };

    let available = game_core::crafting::find_available_recipes(
        &recipes, &inventory, ecs_world, player_pos, &registry,
    );

    let mut lines = vec!["┌─ Crafting ────────────────────────┐".to_string()];
    for (i, ra) in available.iter().enumerate() {
        let status = if ra.available {
            "✓".to_string()
        } else {
            let mut reasons = vec![];
            if !ra.missing_inputs.is_empty() {
                reasons.push(format!("need: {}", ra.missing_inputs.join(", ")));
            }
            if !ra.env_met {
                reasons.push("need workbench/fire".to_string());
            }
            format!("✗ {}", reasons.join("; "))
        };
        if i < 9 {
            lines.push(format!("│ {}. {}  {}", i + 1, ra.recipe.name, status));
        }
    }
    lines.push("│".to_string());
    lines.push("│  [1-9] Select  |  Esc".to_string());
    lines.push("└──────────────────────────────────┘".to_string());
    let text = if available.is_empty() {
        "┌─ Crafting ────────────────────────┐\n│  No recipes available.              │\n│  [Esc]                               │\n└──────────────────────────────────┘".to_string()
    } else {
        lines.join("\n")
    };

    let root = commands.spawn((
        Text(text),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.92)),
    )).id();
    panel.0 = Some(root);
}
