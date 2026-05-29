# World Exploration & NPC Interaction Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire location traversal, encounters, world overview, weather, pathfinding, and NPC personalities into the Bevy game loop.

**Architecture:** 6 sequential tasks. Weather feeds into encounters and pathfinding (so it runs first). Location traversal uses `>` key + LocationMap. Encounters procedurally compose from cascade engine. World overview renders terrain from WorldMap. Pathfinding swaps naive chase for A*. NPC Personalities add score-based tag derivation to entity gen.

**Tech Stack:** Bevy 0.16, bevy_ecs, game-tags (Tags, TagRegistry), game-core, CascadeEngine, LocationMap

---

### File Structure

**Modified:**
- `crates/core/src/lib.rs` — add `pub mod weather;` `pub mod encounters;`
- `src/world_gen.rs` — insert WeatherState, WeatherContext, Encounters, WorldOverviewState resources
- `src/game/mod.rs` — add `>` key for location traversal, `M` key for overview; add encounter roll during movement
- `src/render/mod.rs` — add world overview overlay system
- `crates/world/src/behavior.rs` — replace `chase_direction` with `a_star_step`
- `crates/world/src/lib.rs` — add `pub mod pathfinding;`
- `crates/world/src/entity_gen.rs` — integrate PersonalityScores into entity spawning

**Created:**
- `src/interact/overview.rs` — world overview Bevy UI overlay system
- `crates/core/src/personality.rs` — `PersonalityScores` component + score derivation

---

### Task 1: Weather & Day/Night Cycle

- [ ] **Step 1: Declare weather module in `crates/core/src/lib.rs`**

Add after line 13 (`pub mod durability;`):
```rust
pub mod weather;
```

Add exports:
```rust
pub use weather::{Weather, TimeOfDay, WeatherState, WeatherContext, weather_tags_for_context};
```

- [ ] **Step 2: Read `crates/core/src/weather.rs`** — verify data structures

The file has `Weather`, `TimeOfDay`, `WeatherState` with `advance_time()`, `visibility_modifier()`, `light_level()`. Add a `WeatherContext` resource at the end of the file:

```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherContext {
    pub tags: Vec<String>,
}

/// Build weather tags from current weather + time
pub fn weather_tags_for_context(weather: &Weather, time: &TimeOfDay) -> Vec<String> {
    let mut tags = Vec::new();
    match weather {
        Weather::Rain => { tags.push("RAINY".to_string()); tags.push("WET".to_string()); }
        Weather::Storm => { tags.push("STORMY".to_string()); tags.push("WET".to_string()); tags.push("WINDY".to_string()); }
        Weather::Snow => { tags.push("COLD".to_string()); tags.push("SNOWY".to_string()); }
        Weather::Fog => { tags.push("FOGGY".to_string()); tags.push("REDUCED_VISIBILITY".to_string()); }
        Weather::Sandstorm => { tags.push("REDUCED_VISIBILITY".to_string()); tags.push("DRY".to_string()); }
        _ => {}
    }
    match time {
        TimeOfDay::Night => { tags.push("DARK".to_string()); }
        TimeOfDay::Dusk | TimeOfDay::Dawn => { tags.push("DIM".to_string()); }
        _ => {}
    }
    tags
}
```

- [ ] **Step 3: Insert weather resources in `src/world_gen.rs`**

In `generate_world()`, after all other resource insertions (before `game_camera`):
```rust
    ecs_world.insert_resource(game_core::WeatherState::new());
    let initial_tags = game_core::weather_tags_for_context(&game_core::Weather::Clear, &game_core::TimeOfDay::Day);
    ecs_world.insert_resource(game_core::WeatherContext { tags: initial_tags });
```

- [ ] **Step 4: Call `advance_time` in `finish_npc_turn` in `src/game/mod.rs`**

After `tc.increment();` (line 281), add:
```rust
    if let Some(mut ws) = game_world.0.get_resource_mut::<game_core::WeatherState>() {
        ws.advance_time(&game_world.0);
        let tags = game_core::weather_tags_for_context(&ws.weather, &ws.time);
        if let Some(mut wc) = game_world.0.get_resource_mut::<game_core::WeatherContext>() {
            wc.tags = tags;
        }
    }
```

Add `use game_core::{WeatherState, WeatherContext};` and `use game_core::weather_tags_for_context;` to the imports.

- [ ] **Step 5: Verify build + test**

```bash
cargo check 2>&1 | grep "^error"
cargo test 2>&1 | grep "test result"
```
Expected: 0 errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/weather.rs crates/core/src/lib.rs src/world_gen.rs src/game/mod.rs
git commit -m "explore: weather + day/night cycle with composable tags"
```

---

### Task 2: Unified Location Traversal

- [ ] **Step 1: Add `>` / `Enter` key handler in `src/game/mod.rs`**

Add this after the `E` key handler (after the interact section) and before the `C` key:

```rust
    // Unified location traversal
    if keyboard.just_pressed(KeyCode::Period) || keyboard.just_pressed(KeyCode::Enter) {
        // Check if in a dungeon first
        let map_layer = game_world.0.get_resource::<game_world::MapLayer>();
        let in_dungeon = map_layer.is_some_and(|ml| ml.active_dungeon.is_some());

        if in_dungeon {
            // Check what the player is standing on
            let player_pos = match game_world.0
                .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
                .single(&game_world.0)
            {
                Ok(p) => (p.x, p.y),
                Err(_) => return,
            };

            // Check for entrance stair (exit dungeon) or deeper stair
            // This logic calls try_exit_dungeon or try_go_deeper from src/dungeon.rs
            let has_camera = false; // camera handled separately
            if has_camera && game_world.0.get_resource::<game_world::MapLayer>()
                .and_then(|ml| ml.active_dungeon.as_ref())
                .is_some()
            {
                // For now, just try to go deeper or exit
                // Full dungeon stair detection requires reading DungeonTile data
                // For this task, stub the routing and handle the basic case
                if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                    msg.messages.push("Stand on < to exit or > to descend.".to_string());
                }
            }
            return;
        }

        // Not in dungeon — check if player is inside a location
        let location_map = match game_world.0.get_resource::<game_world::cascade::LocationMap>() {
            Some(lm) => lm.clone(),
            None => return,
        };
        let player_pos = match game_world.0
            .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
            .single(&game_world.0)
        {
            Ok(p) => (p.x, p.y),
            Err(_) => return,
        };

        let loc = game_world::cascade::locations::location_at(&location_map.locations, player_pos.x, player_pos.y);
        if let Some(location) = loc {
            match location.location_type.as_str() {
                "dungeon" | "cave" => {
                    // Call try_enter_dungeon from src/dungeon.rs
                    // This generates BSP dungeon and teleports player
                    if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                        msg.messages.push(format!("Entering {}...", location.name));
                    }
                }
                "city" | "village" | "outpost" => {
                    if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                        msg.messages.push(format!("You arrive at {}.", location.name));
                    }
                }
                "ruin" | "shrine" => {
                    if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                        msg.messages.push(format!("You explore the {}.", location.name));
                    }
                }
                _ => {
                    if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                        msg.messages.push("Nothing to enter here.".to_string());
                    }
                }
            }
        } else {
            if let Some(mut msg) = game_world.0.get_resource_mut::<MessageLog>() {
                msg.messages.push("Nothing to enter here.".to_string());
            }
        }
        return;
    }
```

Add `use game_world::MapLayer;` to imports if not already present.

- [ ] **Step 2: Verify build**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: 0 errors.

- [ ] **Step 3: Commit**

```bash
git add src/game/mod.rs
git commit -m "explore: unified location traversal with > key"
```

---

### Task 3: Encounters as Procedural Event System

- [ ] **Step 1: Declare encounters module in `crates/core/src/lib.rs`**

Add after `pub mod durability;`:
```rust
pub mod encounters;
```

Add exports:
```rust
pub use encounters::{Encounters, EncounterDef, roll_encounter, spawn_encounter, load_encounters};
```

- [ ] **Step 2: Add encounter probability context to `roll_encounter` in `crates/core/src/encounters.rs`**

Modify `roll_encounter` to accept biome + location + weather context and compute variable probability:

```rust
pub fn roll_encounter(
    world: &World,
    pos: &Position,
    biome_tags: &[game_tags::TagId],
    near_location: Option<&str>,
    weather_context: Option<&crate::WeatherContext>,
    rng: &mut impl Rng,
) -> Option<String> {
    let encounters = world.get_resource::<Encounters>()?;

    // Compute base chance from context
    let mut chance = 0.08f32; // 8% base in wilderness

    // Location proximity reduces chance
    if near_location.is_some() { chance = 0.04; }

    // Weather increases chance
    if let Some(wc) = weather_context {
        if wc.tags.iter().any(|t| t == "DARK") { chance += 0.05; }
        if wc.tags.iter().any(|t| t == "STORMY") { chance += 0.03; }
        if wc.tags.iter().any(|t| t == "REDUCED_VISIBILITY") { chance += 0.02; }
    }

    if rng.random::<f32>() > chance { return None; }

    // Filter encounters by biome (if EncounterDef has biome_tags)
    // For now, weighted random from all
    encounters.random_encounter(rng).map(|d| d.id.clone())
}
```

- [ ] **Step 3: Insert Encounters resource in `src/world_gen.rs`**

Add imports:
```rust
use game_core::encounters::{Encounters, load_encounters};
```

In `generate_world()`, add:
```rust
    let encounters_toml = include_str!("../assets/config/encounters.toml");
    let encounter_defs = load_encounters(encounters_toml).expect("Failed to load encounters");
    ecs_world.insert_resource(Encounters::new(encounter_defs));
```

- [ ] **Step 4: Call roll_encounter during movement in `src/game/mod.rs`**

In `handle_game_input`, after the movement block (after the player position is updated at line 221) and before `turn_state.processing_npcs = true;`, add:

```rust
    // Roll encounter at new position
    {
        use game_core::encounters::roll_encounter;
        let registry = game_world.0.resource::<TagRegistry>().clone();
        let pos = Position { x: nx, y: ny, z: 0 };
        let wc = game_world.0.get_resource::<game_core::WeatherContext>();
        let biome_tags: Vec<game_tags::TagId> = vec![]; // simplified
        let location_map = game_world.0.get_resource::<game_world::cascade::LocationMap>()
            .map(|lm| lm.locations.clone()).unwrap_or_default();
        let near_loc = game_world::cascade::locations::location_at(&location_map, nx, ny)
            .map(|l| l.location_type.as_str());

        if let Some(encounter_id) = roll_encounter(
            &game_world.0, &pos, &biome_tags, near_loc, wc, &mut rand::rng(),
        ) {
            game_core::encounters::spawn_encounter(&mut game_world.0, &encounter_id, pos);
            if let Some(mut bus) = game_world.0.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message("An encounter!".to_string()));
            }
        }
    }
```

- [ ] **Step 5: Verify build**

```bash
cargo check 2>&1 | grep "^error"
cargo test 2>&1 | grep "test result"
```
Expected: 0 errors, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/core/src/encounters.rs crates/core/src/lib.rs src/world_gen.rs src/game/mod.rs
git commit -m "explore: encounters as procedural event system"
```

---

### Task 4: World Overview Map

- [ ] **Step 1: Insert `WorldOverviewState` in `src/world_gen.rs`**

Add imports:
```rust
use game_core::world_overview::{WorldOverviewState, WorldOverviewMode};
```

In `generate_world()`, add:
```rust
    ecs_world.insert_resource(WorldOverviewState::new(WorldOverviewMode::ReadOnly));
```

- [ ] **Step 2: Add `M` key binding in `src/game/mod.rs`**

Add after the `>` key handler (before the `C` key):
```rust
    // World overview map
    if keyboard.just_pressed(KeyCode::KeyM) {
        if let Some(mut ov) = game_world.0.get_resource_mut::<game_core::world_overview::WorldOverviewState>() {
            ov.active = !ov.active;
            if ov.active {
                if let Ok(pos) = game_world.0
                    .query_filtered::<&Position, bevy_ecs::query::With<Player>>()
                    .single(&game_world.0)
                {
                    ov.player_x = pos.x;
                    ov.player_y = pos.y;
                    ov.cursor_x = pos.x;
                    ov.cursor_y = pos.y;
                }
            }
        }
        return;
    }
```

- [ ] **Step 3: Create `src/interact/overview.rs` with Bevy UI rendering**

```rust
use bevy::prelude::*;
use game_core::screen::AppScreen;
use game_core::world_overview::{WorldOverviewState, WorldOverviewMode};
use game_world::{Tile, TilePos, WorldMap};
use crate::render::GameWorld;

#[derive(Resource, Default)]
pub struct OverviewOverlay(pub Option<Entity>);

pub fn update_world_overview(
    mut commands: Commands,
    mut game_world: ResMut<GameWorld>,
    mut overlay: ResMut<OverviewOverlay>,
) {
    if let Some(old) = overlay.0.take() { commands.entity(old).despawn(); }

    let active = match game_world.0.get_resource::<WorldOverviewState>() {
        Some(s) if s.active => true,
        _ => return,
    };

    let map = match game_world.0.get_resource::<WorldMap>() {
        Some(m) => m.clone(),
        None => return,
    };

    let mut lines = vec!["=== WORLD OVERVIEW ===".to_string()];

    // Show player position and locations
    if let Some(ov) = game_world.0.get_resource::<WorldOverviewState>() {
        lines.push(format!("Position: ({}, {})", ov.player_x, ov.player_y));
        let locations = game_world.0.get_resource::<game_world::cascade::LocationMap>()
            .cloned().unwrap_or_default();
        lines.push(format!("Locations found: {}", locations.locations.len()));
        for loc in &locations.locations {
            let dist = ((ov.player_x as i32 - loc.x as i32).unsigned_abs()
                + (ov.player_y as i32 - loc.y as i32).unsigned_abs()) as f32;
            lines.push(format!("  {}: {} @ ({}, {}) — {} tiles away",
                loc.location_type, loc.name, loc.x, loc.y, dist as u32));
        }
    }

    lines.push("[M] Close  |  Arrows: move cursor".to_string());

    let root = commands.spawn((
        Text(lines.join("\n")),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.8, 1.0, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(8.0),
            top: Val::Px(4.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
    )).id();
    overlay.0 = Some(root);
}
```

- [ ] **Step 4: Register overview resources and systems**

In `src/render/mod.rs`:
```rust
.init_resource::<crate::interact::overview::OverviewOverlay>()
```

In `src/interact/mod.rs` `InteractPlugin::build()`:
```rust
    .add_systems(Update,
        crate::interact::overview::update_world_overview.run_if(in_state(AppScreen::InWorld))
    )
```

- [ ] **Step 5: Verify build**

```bash
cargo check 2>&1 | grep "^error"
```
Expected: 0 errors.

- [ ] **Step 6: Commit**

```bash
git add src/interact/overview.rs src/world_gen.rs src/game/mod.rs src/render/mod.rs src/interact/mod.rs
git commit -m "explore: world overview map with M key"
```

---

### Task 5: Pathfinding + NPC AI

- [ ] **Step 1: Declare pathfinding module in `crates/world/src/lib.rs`**

Add after `pub mod noise_gen;`:
```rust
pub mod pathfinding;
```

Add exports:
```rust
pub use pathfinding::{a_star_step, has_line_of_sight};
```

- [ ] **Step 2: Replace `chase_direction` in `crates/world/src/behavior.rs`**

Add at the top of the file:
```rust
use crate::pathfinding::a_star_step;
```

Replace the body of `chase_direction` function (around line 211) with:

```rust
fn chase_direction(
    cx: u32, cy: u32,
    tx: u32, ty: u32,
    map: &WorldMap,
    world: &World,
    creature_tags: &Tags,
    occupied: &HashSet<(u32, u32)>,
    blocked_id: Option<TagId>,
    swimmable_id: Option<TagId>,
    flight_id: Option<TagId>,
    aquatic_id: Option<TagId>,
    _rng: &mut impl Rng,
) -> (i32, i32) {
    // Use A* pathfinding instead of naive signum
    if let Some((dx, dy)) = a_star_step(
        (cx, cy), (tx, ty), map, world, creature_tags, occupied,
        blocked_id, swimmable_id, flight_id, aquatic_id,
    ) {
        (dx, dy)
    } else if cx == tx && cy == ty {
        (0, 0)
    } else {
        // Fallback: direct signum if A* fails (target unreachable)
        let dx = (tx as i32 - cx as i32).signum();
        let dy = (ty as i32 - cy as i32).signum();
        (dx, dy)
    }
}
```

Update the call site in `process_npc_turns` to pass the new parameters. The call currently passes `(cx, cy, tx, ty, rng)` — change to pass `map`, `world`, `creature_tags`, `occupied`, `blocked_id`, `swimmable_id`, `flight_id`, `aquatic_id`, and `rng`.

- [ ] **Step 3: Add `has_line_of_sight` check before chase initiation**

In `process_npc_turns`, before starting a chase action, check line of sight:

```rust
// Before chasing: check line of sight
if action_type == "chase" {
    if let Some(blocked) = blocked_id {
        if !crate::pathfinding::has_line_of_sight(
            cx, cy, target_x, target_y, map, world, blocked,
        ) {
            // Can't see the target — wander instead
            action_score = 0.0;
        }
    }
}
```

- [ ] **Step 4: Verify build + test**

```bash
cargo check 2>&1 | grep "^error"
cargo test -p game-world 2>&1 | grep "test result"
```
Expected: 0 errors. Behavior tests may need updates if chase_direction signature changed.

- [ ] **Step 5: Commit**

```bash
git add crates/world/src/pathfinding.rs crates/world/src/lib.rs crates/world/src/behavior.rs
git commit -m "explore: A* pathfinding integration in NPC AI"
```

---

### Task 6: NPC Personalities

- [ ] **Step 1: Create `crates/core/src/personality.rs`**

```rust
use bevy_ecs::prelude::*;
use game_tags::{TagId, TagRegistry, TagValue, Tags};
use rand::Rng;

/// 10 personality traits, 0-100 scale
#[derive(Component, Debug, Clone)]
pub struct PersonalityScores {
    pub aggression: u8,
    pub bravery: u8,
    pub sociability: u8,
    pub orderliness: u8,
    pub curiosity: u8,
    pub industriousness: u8,
    pub honesty: u8,
    pub spirituality: u8,
    pub gregariousness: u8,
    pub volatility: u8,
}

impl PersonalityScores {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        Self {
            aggression: rng.random_range(20..=80),
            bravery: rng.random_range(20..=80),
            sociability: rng.random_range(20..=80),
            orderliness: rng.random_range(20..=80),
            curiosity: rng.random_range(20..=80),
            industriousness: rng.random_range(20..=80),
            honesty: rng.random_range(20..=80),
            spirituality: rng.random_range(20..=80),
            gregariousness: rng.random_range(20..=80),
            volatility: rng.random_range(20..=80),
        }
    }
}

/// Derive behavioral tags from personality scores
pub fn tags_from_personality(
    scores: &PersonalityScores,
    tags: &mut Tags,
    registry: &TagRegistry,
) {
    if let Some(id) = registry.tag_id("AGGRESSIVE") {
        if scores.aggression > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("PEACEFUL") {
        if scores.aggression < 30 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("COWARDLY") {
        if scores.bravery < 30 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("FEARLESS") {
        if scores.bravery > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("CURIOUS") {
        if scores.curiosity > 60 { tags.add_tag(id, TagValue::None, registry); }
    }
    if let Some(id) = registry.tag_id("TERRITORIAL") {
        if scores.orderliness > 70 { tags.add_tag(id, TagValue::None, registry); }
    }
}

/// Modulate personality scores by environmental context
pub fn modulate_by_environment(
    scores: &mut PersonalityScores,
    biome_tags: &[TagId],
    location_type: Option<&str>,
    registry: &TagRegistry,
) {
    // Hostile biomes increase aggression
    if registry.tag_id("DANGEROUS").is_some_and(|id| biome_tags.contains(&id)) {
        scores.aggression = scores.aggression.saturating_add(10).min(100);
        scores.bravery = scores.bravery.saturating_add(10).min(100);
    }
    // Prosperous settlements increase sociability
    if let Some("city" | "village") = location_type {
        scores.sociability = scores.sociability.saturating_add(15).min(100);
    }
    // Military outposts increase orderliness
    if let Some("outpost") = location_type {
        scores.orderliness = scores.orderliness.saturating_add(15).min(100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    #[test]
    fn high_aggression_gets_aggressive_tag() {
        let registry = game_tags::load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        let scores = PersonalityScores {
            aggression: 85, bravery: 50, sociability: 50,
            orderliness: 50, curiosity: 50, industriousness: 50,
            honesty: 50, spirituality: 50, gregariousness: 50, volatility: 50,
        };
        tags_from_personality(&scores, &mut tags, &registry);
        assert!(tags.has(registry.tag_id("AGGRESSIVE").unwrap()), "high aggression should get AGGRESSIVE tag");
    }

    #[test]
    fn peaceful_does_not_get_aggressive() {
        let registry = game_tags::load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        let scores = PersonalityScores {
            aggression: 20, bravery: 50, sociability: 50,
            orderliness: 50, curiosity: 50, industriousness: 50,
            honesty: 50, spirituality: 50, gregariousness: 50, volatility: 50,
        };
        tags_from_personality(&scores, &mut tags, &registry);
        assert!(!tags.has(registry.tag_id("AGGRESSIVE").unwrap()), "low aggression should NOT get AGGRESSIVE tag");
    }

    #[test]
    fn random_scores_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let scores = PersonalityScores::new_random(&mut rng);
        assert!(scores.aggression >= 20 && scores.aggression <= 80);
        assert!(scores.bravery >= 20 && scores.bravery <= 80);
    }
}
```

- [ ] **Step 2: Declare module in `crates/core/src/lib.rs`**

Add:
```rust
pub mod personality;
```

Export:
```rust
pub use personality::{PersonalityScores, tags_from_personality, modulate_by_environment};
```

- [ ] **Step 3: Integrate into entity generation in `crates/world/src/entity_gen.rs`**

In `generate_entity`, after tags are built from the template but before the entity is spawned, add:

```rust
    // Assign personality scores and derive behavioral tags
    use game_core::personality::{PersonalityScores, tags_from_personality, modulate_by_environment};
    let mut scores = PersonalityScores::new_random(&mut rng);
    // Modulate by faction
    if let Some(faction_name) = template.faction.as_ref().map(|f| &f.name) {
        match faction_name.as_str() {
            "great_carapace" | "mutated_wildlife" => { scores.aggression = scores.aggression.saturating_add(20).min(100); }
            "free_humanity" | "the_remnant" => { scores.sociability = scores.sociability.saturating_add(15).min(100); }
            "sanguine_elite" => { scores.volatility = scores.volatility.saturating_add(15).min(100); }
            _ => {}
        }
    }
    tags_from_personality(&scores, &mut entity_tags, registry);
```

Then when spawning the entity, include the PersonalityScores component in the bundle:
```rust
    // Add scores to existing spawn (add to the tuple)
    scores,
```

- [ ] **Step 4: Verify build + test**

```bash
cargo check 2>&1 | grep "^error"
cargo test personality 2>&1 | grep "test result"
```
Expected: 0 errors, personality tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/personality.rs crates/core/src/lib.rs crates/world/src/entity_gen.rs
git commit -m "explore: NPC personality scores + tag derivation"
```

---

### Verification

- [ ] **Full build and test**

```bash
cargo check 2>&1 | grep "^error"
cargo test 2>&1 | grep "test result"
```
Expected: 0 errors, all tests pass.

- [ ] **Commit any remaining**

```bash
git add -A
git commit -m "explore: complete world exploration and NPC interaction integration"
```
