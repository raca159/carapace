# Gene-Splicing System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a gene-splicing system that replaces the unused magic tag archetype with biological adaptations, including tissue sample drops, GeneSplicer pod world objects, splicing UI, humanity meter, and mutation tags.

**Architecture:** New `gene_splicing.rs` module in `game-core` follows existing `crafting.rs` patterns. Single `GeneSplicing` component on player holds adaptations list + humanity + splice count. TOML-driven recipes in `gene_splicing.toml`. Dedicated overlay + `G` keybinding in the binary crate.

**Tech Stack:** Rust, bevy_ecs, ratatui, TOML config

---

## File Inventory

| Action | File | Purpose |
|--------|------|---------|
| Create | `assets/config/gene_splicing.toml` | Splicing recipes (5 initial) |
| Modify | `assets/config/tags.toml` | Add `biological` + `malapty` archetypes with tags |
| Modify | `crates/core/src/components.rs` | Add `GeneSplicing`, `GeneSplicer`, `Adaptation`, `SpliceSlot` |
| Modify | `crates/core/src/events.rs` | Add `GeneSpliced`, `GeneSplicingFailed` events + formatting |
| Create | `crates/core/src/gene_splicing.rs` | Core logic: load, find, execute splice |
| Modify | `crates/core/src/lib.rs` | Export `gene_splicing` module + public types |
| Modify | `crates/core/src/input.rs` | Add `GeneSplice` input event |
| Modify | `src/app.rs` | Add gene-splicing overlay state + `G` key handling |
| Modify | `src/project.rs` | Add gene-splicing overlay rendering |
| Modify | `crates/core/src/save.rs` | Add `GeneSplicing` to save/load serialization |
| Modify | `assets/config/loot_tables.toml` | Add tissue sample loot entries |
| Modify | `assets/config/spawn_rules.toml` | Add GeneSplicer pod spawn rule |

---

### Task 1: Add gene-splicing tags to tags.toml

**Files:**
- Modify: `assets/config/tags.toml`

- [ ] **Step 1: Add `biological` archetype with tissue/mutation tags**

Open `assets/config/tags.toml`. After the last archetype (`armor_type`), add:

```toml
[[archetype]]
id = "biological"
name = "Biological Property"
exclusivity = "any"

[[archetype.tags]]
id = "VIABLE_TISSUE"

[[archetype.tags]]
id = "GENE_SPLICED"

[[archetype.tags]]
id = "TISSUE_PISTOL_SHRIMP"
implies = ["VIABLE_TISSUE"]

[[archetype.tags]]
id = "TISSUE_TARDIGRADE"
implies = ["VIABLE_TISSUE"]

[[archetype.tags]]
id = "TISSUE_ELECTRIC_EEL"
implies = ["VIABLE_TISSUE"]

[[archetype.tags]]
id = "TISSUE_BOMBARDIER"
implies = ["VIABLE_TISSUE"]

[[archetype.tags]]
id = "TISSUE_CUTTLEFISH"
implies = ["VIABLE_TISSUE"]

[[archetype.tags]]
id = "SONIC_CAVITATION"
implies = ["GENE_SPLICED"]

[[archetype.tags]]
id = "ANHYDRO_CHITIN"
implies = ["GENE_SPLICED"]

[[archetype.tags]]
id = "BIO_ELECTRIC"
implies = ["GENE_SPLICED"]

[[archetype.tags]]
id = "EXOTHERMIC_SPRAY"
implies = ["GENE_SPLICED"]

[[archetype.tags]]
id = "CHROMATOPHORIC"
implies = ["GENE_SPLICED"]
```

- [ ] **Step 2: Add `malapty` archetype with failure tags**

After the `biological` archetype, add:

```toml
[[archetype]]
id = "malapty"
name = "Genetic Malapty"
exclusivity = "any"

[[archetype.tags]]
id = "MALFORMED"

[[archetype.tags]]
id = "WEAKENED_IMMUNE"

[[archetype.tags]]
id = "CHROMATIC_INSTABILITY"

[[archetype.tags]]
id = "NEURAL_FRAGILE"

[[archetype.tags]]
id = "BIO_RECESSION"
```

- [ ] **Step 3: Verify tags parse**

Run: `cargo test -p game-tags`
Expected: All tag tests pass, including new tags in registry

---

### Task 2: Create gene_splicing.toml recipe file

**Files:**
- Create: `assets/config/gene_splicing.toml`

- [ ] **Step 1: Write 5 splicing recipes**

Create `assets/config/gene_splicing.toml`:

```toml
[[recipe]]
name = "Sonic Cavitation"
input_sample_tag = "TISSUE_PISTOL_SHRIMP"
output_mutation_tag = "SONIC_CAVITATION"
output_mutation_name = "Sonic Cavitation"
success_chance = 0.65
humanity_cost = 3
failure_tags = ["NEURAL_FRAGILE"]
description = "Pistol Shrimp gland — focused sonic blast extends your shockwave range."

[[recipe]]
name = "Anhydro-Chitin"
input_sample_tag = "TISSUE_TARDIGRADE"
output_mutation_tag = "ANHYDRO_CHITIN"
output_mutation_name = "Anhydro-Chitin Carapace"
success_chance = 0.45
humanity_cost = 5
failure_tags = ["MALFORMED"]
description = "Tardigrade DNA — extreme damage resistance when near death."

[[recipe]]
name = "Bio-Electric Surge"
input_sample_tag = "TISSUE_ELECTRIC_EEL"
output_mutation_tag = "BIO_ELECTRIC"
output_mutation_name = "Bio-Electric Surge"
success_chance = 0.55
humanity_cost = 4
failure_tags = ["CHROMATIC_INSTABILITY"]
description = "Electric Eel nodes — passive melee shock damage."

[[recipe]]
name = "Exothermic Spray"
input_sample_tag = "TISSUE_BOMBARDIER"
output_mutation_tag = "EXOTHERMIC_SPRAY"
output_mutation_name = "Exothermic Spray Glands"
success_chance = 0.50
humanity_cost = 4
failure_tags = ["WEAKENED_IMMUNE"]
description = "Bombardier Beetle glands — acid spray degrades enemy armor."

[[recipe]]
name = "Chromatophoric Shift"
input_sample_tag = "TISSUE_CUTTLEFISH"
output_mutation_tag = "CHROMATOPHORIC"
output_mutation_name = "Chromatophoric Shift"
success_chance = 0.60
humanity_cost = 3
failure_tags = ["BIO_RECESSION"]
description = "Cuttlefish chromatophores — high evasion with tile-meld stealth."
```

- [ ] **Step 2: Verify file loads**

Run: `cargo build`
Expected: Clean compile (file is data-only, loaded at runtime)

---

### Task 3: Add GeneSplicing component, GeneSplicer marker, and related types to game-core

**Files:**
- Modify: `crates/core/src/components.rs`

- [ ] **Step 1: Add GeneSplicer, GeneSplicing, Adaptation, SpliceSlot types**

Open `crates/core/src/components.rs`. After the `Animation` struct (ends around line 177), add:

```rust
#[derive(Component, Debug, Clone)]
pub struct GeneSplicer;

#[derive(Component, Debug, Clone)]
pub struct GeneSplicing {
    pub adaptations: Vec<Adaptation>,
    pub humanity: u32,
    pub splice_points: u32,
}

impl GeneSplicing {
    pub fn new() -> Self {
        Self {
            adaptations: Vec::new(),
            humanity: 100,
            splice_points: 0,
        }
    }

    pub fn humanity_rank(&self) -> &'static str {
        match self.humanity {
            100 => "Full Human",
            70..=99 => "Minor Mutations",
            40..=69 => "Significant Chitin",
            10..=39 => "Mostly Monster",
            0..=9 => "Carapace-Mind",
        }
    }
}

impl Default for GeneSplicing {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adaptation {
    pub id: String,
    pub source: String,
    pub tags_granted: Vec<u16>,
    pub humanity_cost: u32,
    pub slot: SpliceSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpliceSlot {
    Arms,
    Body,
    Organs,
    Sense,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 4: Add gene-splicing events to events.rs

**Files:**
- Modify: `crates/core/src/events.rs`

- [ ] **Step 1: Add GeneSpliced and GeneSplicingFailed event variants**

In `crates/core/src/events.rs`, in the `GameEvent` enum (after `LootDropped` variant, around line 79), add:

```rust
    GeneSpliced {
        sample_name: String,
        adaptation_name: String,
        humanity_after: u32,
        humanity_rank: String,
    },
    GeneSplicingFailed {
        sample_name: String,
        malapty_name: String,
        humanity_after: u32,
        humanity_rank: String,
    },
```

- [ ] **Step 2: Add formatting for gene-splicing events**

In the `format_event` function, in the match arms (before `GameEvent::Message`), add:

```rust
        GameEvent::GeneSpliced { sample_name, adaptation_name, humanity_after, humanity_rank } => {
            format!("Gene splice successful! {} adapted from {}. Humanity: {} ({})", adaptation_name, sample_name, humanity_after, humanity_rank)
        }
        GameEvent::GeneSplicingFailed { sample_name, malapty_name, humanity_after, humanity_rank } => {
            format!("Gene splice FAILED! {} rejected. Genetic malapty: {}. Humanity: {} ({})", sample_name, malapty_name, humanity_after, humanity_rank)
        }
```

- [ ] **Step 3: Add category mapping for gene-splicing events**

In the `event_category` function, add to the `GameEvent::Interaction { .. }` match arm's list (or create a new pattern):

```rust
        GameEvent::GeneSpliced { .. } | GameEvent::GeneSplicingFailed { .. } => MessageCategory::System,
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 5: Create gene_splicing.rs core module

**Files:**
- Create: `crates/core/src/gene_splicing.rs`

- [ ] **Step 1: Write the file header, structs, and imports**

Create `crates/core/src/gene_splicing.rs`:

```rust
use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::Deserialize;

use crate::components::{Adaptation, GeneSplicing, SpliceSlot};
use crate::{Glyph, Inventory, Item, Name, Position};
use game_tags::{TagId, TagRegistry, TagValue, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct SplicingRecipe {
    pub name: String,
    pub input_sample_tag: String,
    pub output_mutation_tag: String,
    pub output_mutation_name: String,
    pub success_chance: f64,
    pub humanity_cost: u32,
    pub failure_tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SplicingToml {
    #[serde(rename = "recipe")]
    recipes: Vec<SplicingRecipe>,
}

pub fn load_splicing_recipes(toml_str: &str) -> Vec<SplicingRecipe> {
    toml::from_str::<SplicingToml>(toml_str)
        .map(|f| f.recipes)
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct RecipeAvailability {
    pub recipe: SplicingRecipe,
    pub available: bool,
    pub reason: String,
    pub success_chance: f64,
    pub humanity_cost: u32,
    pub failure_tags: Vec<String>,
}

pub fn find_available_recipes(
    recipes: &[SplicingRecipe],
    player_entity: Entity,
    world: &World,
    registry: &TagRegistry,
) -> Vec<RecipeAvailability> {
    let viable_tissue = registry.tag_id("VIABLE_TISSUE");
    let Some(viable_tissue) = viable_tissue else {
        return recipes.iter().map(|r| RecipeAvailability {
            recipe: r.clone(),
            available: false,
            reason: "Tag registry not loaded".to_string(),
            success_chance: 0.0,
            humanity_cost: 0,
            failure_tags: vec![],
        }).collect();
    };

    let player_has_splicer = {
        let mut query = world.query_filtered::<Entity, (With<Position>, With<crate::components::GeneSplicer>)>();
        let player_pos = world.get::<Position>(player_entity).map(|p| (p.x, p.y));
        query.iter(world).any(|e| {
            world.get::<Position>(e).map(|p| {
                let dx = (p.x as i32 - player_pos.unwrap_or((0,0)).0 as i32).unsigned_abs();
                let dy = (p.y as i32 - player_pos.unwrap_or((0,0)).1 as i32).unsigned_abs();
                dx <= 1 && dy <= 1
            }).unwrap_or(false)
        })
    };

    let inventory = world.get::<Inventory>(player_entity);
    let player_tags = world.get::<Tags>(player_entity);
    let gene_splicing = world.get::<GeneSplicing>(player_entity);

    let total_splices = gene_splicing.map(|g| g.splice_points).unwrap_or(0);
    let humanity = gene_splicing.map(|g| g.humanity).unwrap_or(100);

    let inventory_sample_tags: Vec<TagId> = inventory.map(|inv| {
        inv.items.iter().filter_map(|&item_entity| {
            world.get::<Tags>(item_entity).map(|tags| {
                tags.iter_present().collect::<Vec<_>>()
            })
        }).flatten().collect()
    }).unwrap_or_default();

    recipes.iter().map(|recipe| {
        let sample_tag_id = registry.tag_id(&recipe.input_sample_tag);
        let has_sample = sample_tag_id.map(|id| inventory_sample_tags.contains(&id)).unwrap_or(false);
        let all_failure_ids: Vec<TagId> = recipe.failure_tags.iter()
            .filter_map(|t| registry.tag_id(t))
            .collect();

        if !player_has_splicer {
            return RecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: "Not near a Gene-Splicing Pod".to_string(),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        if !has_sample {
            return RecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: format!("Missing tissue: {}", recipe.input_sample_tag),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        if humanity == 0 {
            return RecipeAvailability {
                recipe: recipe.clone(),
                available: false,
                reason: "Humanity depleted — cannot splice further.".to_string(),
                success_chance: 0.0,
                humanity_cost: 0,
                failure_tags: vec![],
            };
        }

        let quality_mult = sample_tag_id.and_then(|id| {
            let item_with_tag = inventory.iter().and_then(|inv| {
                inv.items.iter().find(|&&e| {
                    world.get::<Tags>(e).map(|t| t.has(id)).unwrap_or(false)
                })
            });
            item_with_tag.and_then(|e| {
                world.get::<Tags>(e).map(|t| {
                    if t.has(registry.tag_id("LEGENDARY").unwrap_or(TagId(65535))) { 1.6 }
                    else if t.has(registry.tag_id("EPIC").unwrap_or(TagId(65535))) { 1.4 }
                    else if t.has(registry.tag_id("RARE").unwrap_or(TagId(65535))) { 1.2 }
                    else if t.has(registry.tag_id("UNCOMMON").unwrap_or(TagId(65535))) { 1.0 }
                    else { 0.8 }
                })
            })
        }).unwrap_or(1.0);

        let splice_penalty = 0.03 * total_splices as f64;
        let humanity_mod = if humanity < 10 { 0.15 } else if humanity < 30 { 0.1 } else { 0.0 };
        let final_chance = (recipe.success_chance * quality_mult - splice_penalty + humanity_mod)
            .clamp(0.05, 0.95);

        let failure_tag_names: Vec<String> = recipe.failure_tags.iter()
            .filter_map(|t| registry.tag_id(t).map(|id| registry.tag_by_id(id).name.clone()))
            .collect();

        RecipeAvailability {
            recipe: recipe.clone(),
            available: true,
            reason: format!("{:.0}% chance", final_chance * 100.0),
            success_chance: final_chance,
            humanity_cost: recipe.humanity_cost,
            failure_tags: failure_tag_names,
        }
    }).collect()
}

pub enum SpliceOutcome {
    Success {
        mutation_name: String,
        new_tag: TagId,
    },
    Failure {
        malapty_name: String,
        applied_tags: Vec<TagId>,
    },
}

pub fn execute_splice(
    recipe: &SplicingRecipe,
    player_entity: Entity,
    world: &mut World,
    registry: &TagRegistry,
) -> SpliceOutcome {
    let sample_tag_id = match registry.tag_id(&recipe.input_sample_tag) {
        Some(id) => id,
        None => return SpliceOutcome::Failure {
            malapty_name: "Unknown".to_string(),
            applied_tags: vec![],
        },
    };

    let mutation_tag_id = match registry.tag_id(&recipe.output_mutation_tag) {
        Some(id) => id,
        None => return SpliceOutcome::Failure {
            malapty_name: "Unknown".to_string(),
            applied_tags: vec![],
        },
    };

    let (quality_mult, sample_entity) = {
        let inventory = world.get::<Inventory>(player_entity).cloned();
        match inventory {
            Some(inv) => {
                let found = inv.items.iter().find(|&&e| {
                    world.get::<Tags>(e).map(|t| t.has(sample_tag_id)).unwrap_or(false)
                });
                match found {
                    Some(&e) => {
                        let qual = {
                            let tags = world.get::<Tags>(e);
                            let legendary = registry.tag_id("LEGENDARY");
                            let epic = registry.tag_id("EPIC");
                            let rare = registry.tag_id("RARE");
                            let uncommon = registry.tag_id("UNCOMMON");
                            if let Some(t) = &tags {
                                if legendary.map_or(false, |lid| t.has(lid)) { 1.6 }
                                else if epic.map_or(false, |eid| t.has(eid)) { 1.4 }
                                else if rare.map_or(false, |rid| t.has(rid)) { 1.2 }
                                else if uncommon.map_or(false, |uid| t.has(uid)) { 1.0 }
                                else { 0.8 }
                            } else { 1.0 }
                        };
                        (qual, Some(e))
                    }
                    None => (1.0, None),
                }
            }
            None => (1.0, None),
        }
    };

    let total_splices = world.get::<GeneSplicing>(player_entity)
        .map(|g| g.splice_points).unwrap_or(0);
    let humanity = world.get::<GeneSplicing>(player_entity)
        .map(|g| g.humanity).unwrap_or(100);

    let splice_penalty = 0.03 * total_splices as f64;
    let humanity_mod = if humanity < 10 { 0.15 } else if humanity < 30 { 0.1 } else { 0.0 };
    let final_chance = (recipe.success_chance * quality_mult - splice_penalty + humanity_mod)
        .clamp(0.05, 0.95);

    let roll: f64 = fastrand::f64();

    if roll < final_chance {
        if let Some(mut gs) = world.get_mut::<GeneSplicing>(player_entity) {
            let human_cost = recipe.humanity_cost.min(gs.humanity);
            gs.humanity = gs.humanity.saturating_sub(human_cost);
            gs.splice_points += 1;
            gs.adaptations.push(Adaptation {
                id: recipe.output_mutation_tag.clone(),
                source: recipe.name.clone(),
                tags_granted: vec![mutation_tag_id.0],
                humanity_cost: recipe.humanity_cost,
                slot: SpliceSlot::Body,
            });
        }

        if let Some(mut tags) = world.get_mut::<Tags>(player_entity) {
            tags.add_tag(mutation_tag_id, TagValue::None, registry);
            let spliced_id = registry.tag_id("GENE_SPLICED");
            if let Some(sid) = spliced_id {
                tags.add_tag(sid, TagValue::None, registry);
            }
        }

        if let Some(sample_entity) = sample_entity {
            if let Some(mut inv) = world.get_mut::<Inventory>(player_entity) {
                inv.items.retain(|&e| e != sample_entity);
            }
            let _ = world.despawn(sample_entity);
        }

        SpliceOutcome::Success {
            mutation_name: recipe.output_mutation_name.clone(),
            new_tag: mutation_tag_id,
        }
    } else {
        let failure_ids: Vec<TagId> = recipe.failure_tags.iter()
            .filter_map(|t| registry.tag_id(t))
            .collect();

        if let Some(mut tags) = world.get_mut::<Tags>(player_entity) {
            for &fid in &failure_ids {
                tags.add_tag(fid, TagValue::None, registry);
            }
        }

        if let Some(mut gs) = world.get_mut::<GeneSplicing>(player_entity) {
            gs.splice_points += 1;
        }

        if let Some(sample_entity) = sample_entity {
            if let Some(mut inv) = world.get_mut::<Inventory>(player_entity) {
                inv.items.retain(|&e| e != sample_entity);
            }
            let _ = world.despawn(sample_entity);
        }

        let malapty_name = failure_ids.first()
            .map(|id| registry.tag_by_id(*id).name.clone())
            .unwrap_or_else(|| "Unknown Malapty".to_string());

        SpliceOutcome::Failure {
            malapty_name,
            applied_tags: failure_ids,
        }
    }
}
```

- [ ] **Step 2: Add `fastrand` to workspace dependencies**

If `fastrand` is not already in workspace deps, add to `Cargo.toml` workspace:
```toml
fastrand = "2"
```

And to `crates/core/Cargo.toml`:
```toml
fastrand.workspace = true
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 6: Wire gene_splicing into game-core lib.rs

**Files:**
- Modify: `crates/core/src/lib.rs`

- [ ] **Step 1: Add module declaration and public exports**

In `crates/core/src/lib.rs`, add after `pub mod durability;`:
```rust
pub mod gene_splicing;
```

Add after the durability/other exports, add:
```rust
pub use gene_splicing::{SplicingRecipe, RecipeAvailability, SpliceOutcome, load_splicing_recipes, find_available_recipes, execute_splice};
```

Add `GeneSplicing`, `GeneSplicer`, `Adaptation`, `SpliceSlot` to the components use line:
```rust
    Animation, BiomePreset, Creature, Equipment, EquipmentSlot, GeneSplicer, GeneSplicing, Glyph, Health, Inventory, Item,
    LootContainer, MessageCategory, MessageEntry, MessageLog, Name, Player, Position,
    RepairStation, WorldGenParams, WorldGenProgress, WorldGenStage, WorldSize,
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 7: Add GeneSplice input event to game-core input

**Files:**
- Modify: `crates/core/src/input.rs`

- [ ] **Step 1: Add GeneSplice variant to InputEvent enum**

Find the `InputEvent` enum in `crates/core/src/input.rs`. Add after `ThrowItem` (or at end):
```rust
    GeneSplice,
```

- [ ] **Step 2: Map the 'G' key to InputEvent::GeneSplice (terminal-input feature)**

In the `InputHandler` impl (or the key-to-event mapping function), find where key events are matched. Add:
```rust
    (KeyCode::Char('g'), KeyModifiers::NONE) => Some(InputEvent::GeneSplice),
    (KeyCode::Char('G'), KeyModifiers::SHIFT) => Some(InputEvent::GeneSplice),
```

Make sure it's placed where other letter-key mappings are (not conflicting with existing `G` usage).

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 8: Add GeneSplicing to save/load serialization

**Files:**
- Modify: `crates/core/src/save.rs`

- [ ] **Step 1: Find the save data structures and add GeneSplicing fields**

In `crates/core/src/save.rs`, find the `SaveGame` struct (or similar save data struct). Add:
```rust
    pub gene_splicing: Option<GeneSplicing>,
```

Make sure `GeneSplicing` is imported:
```rust
use crate::components::{GeneSplicing, Adaptation};
```

- [ ] **Step 2: Serialize GeneSplicing during save**

In the function that builds the save data (likely in `save_game` or similar), find where player data is extracted. Add:
```rust
    let player_entity = { /* existing player query */ };
    let gene_splicing = player_entity
        .and_then(|e| world.get::<GeneSplicing>(e).cloned());
```

And include it in the save struct:
```rust
    gene_splicing,
```

- [ ] **Step 3: Deserialize GeneSplicing during load**

In the deserialization function (`deserialize_to_world`), find where player components are inserted. Add:
```rust
    if let Some(gs) = save_data.gene_splicing.clone() {
        entity.insert(gs);
    }
```

Add `GeneSplicing::new()` as default on world creation:
```rust
    // In world init / new game creation code
    player_entity.insert(GeneSplicing::new());
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 9: Integrate gene-splicing into binary crate app.rs

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add overlay state variables**

In `src/app.rs`, in the `run_game` function where overlay state vars are declared (around line 63-95), add:
```rust
    let mut splice_open = false;
    let mut splice_cursor: usize = 0;
    let mut splice_recipes: Vec<game_core::SplicingRecipe> = Vec::new();
    let mut splice_avail: Vec<game_core::RecipeAvailability> = Vec::new();
```

- [ ] **Step 2: Add G key to toggle splice overlay**

Find the crafting toggle logic (the `OpenCrafting` event handler around line 578-613). After the crafting block, add similar logic for `GeneSplice`:

```rust
                if event == game_core::input::InputEvent::GeneSplice {
                    if splice_open {
                        splice_open = false;
                    } else {
                        let has_splicer_nearby = {
                            let mut sq = ecs_world.query_filtered::<&Position, bevy_ecs::query::With<game_core::GeneSplicer>>();
                            let player_pos = {
                                let mut pq = ecs_world.query_filtered::<&Position, bevy_ecs::query::With<Player>>();
                                pq.single(&ecs_world).ok().copied()
                            };
                            player_pos.map(|pp| {
                                sq.iter(&ecs_world).any(|sp| {
                                    let dx = (sp.x as i32 - pp.x as i32).unsigned_abs();
                                    let dy = (sp.y as i32 - pp.y as i32).unsigned_abs();
                                    dx <= 1 && dy <= 1
                                })
                            }).unwrap_or(false)
                        };

                        if !has_splicer_nearby {
                            if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                                bus.push(GameEvent::Message("No Gene-Splicing Pod nearby.".to_string()));
                            }
                        } else {
                            splice_open = true;
                            splice_cursor = 0;
                            if splice_recipes.is_empty() {
                                let splicing_toml = include_str!("../assets/config/gene_splicing.toml");
                                splice_recipes = game_core::load_splicing_recipes(splicing_toml);
                            }
                            let player_entity = {
                                let mut pq = ecs_world.query_filtered::<Entity, bevy_ecs::query::With<Player>>();
                                pq.single(&ecs_world).ok()
                            };
                            if let Some(pe) = player_entity {
                                let registry = ecs_world.resource::<TagRegistry>().clone();
                                splice_avail = game_core::find_available_recipes(
                                    &splice_recipes,
                                    pe,
                                    &ecs_world,
                                    &registry,
                                );
                            }
                        }
                    }
                }
```

- [ ] **Step 3: Add splice overlay navigation**

Find the crafting overlay input handling (the `crafting_open` match block around line 615-659). After it, add:

```rust
                if splice_open {
                    match event {
                        game_core::input::InputEvent::MoveUp => {
                            splice_cursor = splice_cursor.saturating_sub(1);
                        }
                        game_core::input::InputEvent::MoveDown
                            if splice_cursor + 1 < splice_avail.len() => {
                                splice_cursor += 1;
                            }
                        game_core::input::InputEvent::Activate
                            if splice_cursor < splice_avail.len() && splice_avail[splice_cursor].available => {
                                let recipe = splice_avail[splice_cursor].recipe.clone();
                                let player_entity = {
                                    let mut pq = ecs_world.query_filtered::<Entity, bevy_ecs::query::With<Player>>();
                                    pq.single(&ecs_world).ok()
                                };
                                if let Some(pe) = player_entity {
                                    let registry = ecs_world.resource::<TagRegistry>().clone();
                                    let outcome = game_core::execute_splice(&recipe, pe, &mut ecs_world, &registry);
                                    match outcome {
                                        game_core::SpliceOutcome::Success { mutation_name, new_tag } => {
                                            let humanity_after = ecs_world.get::<game_core::GeneSplicing>(pe)
                                                .map(|g| g.humanity).unwrap_or(100);
                                            let rank = ecs_world.get::<game_core::GeneSplicing>(pe)
                                                .map(|g| g.humanity_rank().to_string()).unwrap_or_default();
                                            if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                                                bus.push(GameEvent::GeneSpliced {
                                                    sample_name: recipe.input_sample_tag.clone(),
                                                    adaptation_name: mutation_name,
                                                    humanity_after,
                                                    humanity_rank: rank,
                                                });
                                            }
                                        }
                                        game_core::SpliceOutcome::Failure { malapty_name, .. } => {
                                            let humanity_after = ecs_world.get::<game_core::GeneSplicing>(pe)
                                                .map(|g| g.humanity).unwrap_or(100);
                                            let rank = ecs_world.get::<game_core::GeneSplicing>(pe)
                                                .map(|g| g.humanity_rank().to_string()).unwrap_or_default();
                                            if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
                                                bus.push(GameEvent::GeneSplicingFailed {
                                                    sample_name: recipe.input_sample_tag.clone(),
                                                    malapty_name,
                                                    humanity_after,
                                                    humanity_rank: rank,
                                                });
                                            }
                                        }
                                    }
                                    // Refresh availability
                                    let registry = ecs_world.resource::<TagRegistry>().clone();
                                    splice_avail = game_core::find_available_recipes(
                                        &splice_recipes,
                                        pe,
                                        &ecs_world,
                                        &registry,
                                    );
                                }
                            }
                        game_core::input::InputEvent::Cancel => {
                            splice_open = false;
                        }
                        _ => {}
                    }
                }
```

- [ ] **Step 4: Block other actions when splice_open is true**

Find the conditionals that block actions when crafting_open, inventory_open, dialogue_open, etc. Add `splice_open` to these guards. For example, the dungeon entry/exit logic around lines 725-760:

```rust
                    && !splice_open
```

Similarly for quest board, talk, examine, overview, etc. Look for patterns like `!crafting_open && !journal_open` and add `&& !splice_open` to each.

- [ ] **Step 5: Pass splice state to render context**

Find the `RenderContext` construction (around line 1123). Add `splice_open` and `splice_avail` to the struct. 

Check if the snapshot/project needs a splice field. If using `RenderContext`, add:
```rust
                splice_open,
                splice_avail: &splice_avail,
                splice_cursor,
```

If using a separate snapshot struct, add the fields there.

- [ ] **Step 6: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 10: Add gene-splicing rendering to project.rs

**Files:**
- Modify: `src/project.rs`

- [ ] **Step 1: Add splice fields to RenderContext (or snapshot)**

Find the `RenderContext` struct definition in `src/project.rs`. Add:
```rust
    pub splice_open: bool,
    pub splice_avail: &'a [game_core::RecipeAvailability],
    pub splice_cursor: usize,
```

- [ ] **Step 2: Add splice overlay rendering**

In the rendering function, where crafting overlay is rendered, add a similar block for splice. For now, use a simple text-based overlay:

After the crafting rendering block, add:
```rust
    if ctx.splice_open {
        // Title
        let title = " Gene-Splicing Pod ";
        let x = (viewport_width - 40) / 2;
        let y = 2;
        render_rect(viewport, x, y, 40, 12 + ctx.splice_avail.len() as u16 * 2, Style::default().fg(Color::Cyan));

        // Title bar
        for (i, ch) in title.chars().enumerate() {
            viewport[(y, x + i as u16)] = Cell::new(ch).set_fg(Color::White).set_bg(Color::Blue);
        }

        // Available recipes
        for (i, avail) in ctx.splice_avail.iter().enumerate() {
            let ry = y + 2 + i as u16 * 2;
            let cursor = if i == ctx.splice_cursor { ">" } else { " " };
            let name = &avail.recipe.output_mutation_name;
            let chance = if avail.available {
                format!("{:.0}%", avail.success_chance * 100.0)
            } else {
                avail.reason.clone()
            };
            let humanity_str = if avail.available {
                format!("-{} Humanity", avail.humanity_cost)
            } else {
                String::new()
            };
            let line = format!(" {} {} [{}] {}", cursor, name, chance, humanity_str);
            for (j, ch) in line.chars().enumerate() {
                let color = if i == ctx.splice_cursor { Color::Yellow } else { Color::White };
                viewport[(ry, x + j as u16)] = Cell::new(ch).set_fg(color);
            }

            // Failure risk
            if !avail.failure_tags.is_empty() && avail.available {
                let risk = format!("   Risk: {}", avail.failure_tags.join(", "));
                let ry2 = ry + 1;
                for (j, ch) in risk.chars().enumerate() {
                    viewport[(ry2, x + j as u16)] = Cell::new(ch).set_fg(Color::Red);
                }
            }
        }

        // Humaniy meter at bottom
        if let Some(player_entity) = ctx.player_entity {
            if let Some(gs) = world.get::<GeneSplicing>(player_entity) {
                let meter_y = y + 3 + ctx.splice_avail.len() as u16 * 2;
                let meter = format!(" Humanity: {} ({})  ", gs.humanity, gs.humanity_rank());
                for (j, ch) in meter.chars().enumerate() {
                    viewport[(meter_y, x + j as u16)] = Cell::new(ch).set_fg(Color::Green);
                }
            }
        }

        // Footer
        let footer_y = y + 4 + ctx.splice_avail.len() as u16 * 2;
        let footer = " [Enter] Splice  [Esc] Close ";
        for (j, ch) in footer.chars().enumerate() {
            viewport[(footer_y, x + j as u16)] = Cell::new(ch).set_fg(Color::DarkGray);
        }
    }
```

- [ ] **Step 3: Add GeneSplicing/GeneSplicer to the imports**

At the top of `src/project.rs`, add:
```rust
use game_core::{GeneSplicing, GeneSplicer};
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 11: Add GeneSplicer pod spawn rule to spawn_rules.toml

**Files:**
- Modify: `assets/config/spawn_rules.toml`

- [ ] **Step 1: Add Gene-Splicing Pod entry**

Add to `assets/config/spawn_rules.toml`:
```toml
[[spawn_rule]]
name = "Gene-Splicing Pod"
glyph = "P"
color = [0, 200, 255]
biome_tags = ["BIOME_CRYO_VAULT", "BIOME_SUBTERRANEAN_RIFT"]
tags = ["STATIONARY", "TECHDEPENDENT", "PRE_COLLAPSE"]
density = 0.01
is_station = true
```

- [ ] **Step 2: Add GeneSplicer component to spawned pod entities**

Find the entity spawning code in `crates/world/src/spawner.rs` where spawned entities are built. Find where `is_station` / stationary entities are spawned. Add:
```rust
if rule.is_station {
    entity.insert(game_core::GeneSplicer);
}
```

Or if the spawning already handles `is_station` entities, update to include the `GeneSplicer` component when the name matches "Gene-Splicing Pod".

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 12: Add tissue sample loot entries to loot_tables.toml

**Files:**
- Modify: `assets/config/loot_tables.toml`

- [ ] **Step 1: Add tissue sample loot entries**

Read the existing `loot_tables.toml` to understand its structure, then add entries like:
```toml
[[loot_entry]]
name = "Carapace Tissue Sample"
templates = ["carapace_spawn"]
tags = ["VIABLE_TISSUE", "TISSUE_PISTOL_SHRIMP"]
glyph = "t"
color = [220, 120, 80]
drop_chance = 0.6
quality = "RARE"

[[loot_entry]]
name = "Sanguine Tissue Sample"
templates = ["sanguine_noble"]
tags = ["VIABLE_TISSUE", "TISSUE_ELECTRIC_EEL"]
glyph = "t"
color = [200, 50, 50]
drop_chance = 0.7
quality = "EPIC"

[[loot_entry]]
name = "Tardigrade Infused Tissue"
templates = ["carapace_brute"]
tags = ["VIABLE_TISSUE", "TISSUE_TARDIGRADE"]
glyph = "t"
color = [120, 200, 120]
drop_chance = 0.5
quality = "RARE"

[[loot_entry]]
name = "Bombardier Gland Sample"
templates = ["carapace_horror"]
tags = ["VIABLE_TISSUE", "TISSUE_BOMBARDIER"]
glyph = "t"
color = [255, 150, 50]
drop_chance = 1.0
quality = "EPIC"

[[loot_entry]]
name = "Cuttlefish Chromatic Tissue"
templates = ["sanguine_enforcer"]
tags = ["VIABLE_TISSUE", "TISSUE_CUTTLEFISH"]
glyph = "t"
color = [100, 200, 255]
drop_chance = 0.6
quality = "RARE"
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Clean compile

---

### Task 13: Update architecture document

**Files:**
- Modify: `docs/architecture.md`

- [ ] **Step 1: Update Phase 9 section and gene-splicing section**

In `docs/architecture.md`, update the phase 9 section (or add a Phase 10 section). Update section 5.3 to reflect the implementation status. The architecture doc should mention that the gene-splicing system is now implemented with the tagged module structure.

No code changes needed — just documentation.

---

### Task 14: Integration verification

- [ ] **Step 1: Build entire workspace**

Run: `cargo build 2>&1`
Expected: Clean compile with no errors

- [ ] **Step 2: Run all tests**

Run: `cargo test 2>&1`
Expected: All ~519+ tests pass

- [ ] **Step 3: Run clippy**

Run: `cargo clippy 2>&1`
Expected: Zero warnings
