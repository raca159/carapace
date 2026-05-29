# Gene-Splicing System — Implementation Design

**Issue:** CAR-302
**Status:** Design draft
**Date:** 2026-05-26

---

## 1. Overview

Replace the unused `magic` tag archetype with a **gene-splicing system** — the player extracts Viable Tissue Samples from slain Carapace creatures and Sanguine nobles, then uses Gene-Splicing Pods (pre-collapse biotech terminals) to force biological adaptations into their genome.

The design follows the existing architecture described in `docs/architecture.md §5.3` and the lore-realignment plan (`CAR-164`).

---

## 2. Design Decisions

### Q1 Resolution: New overlay (option a)

Gene-splicing gets its own keybinding (`G` by default) and overlay, modeled on the crafting system (`crafting.rs` / `crafting_avail` flow). Rationale:
- It has different requirements (proximity to a GeneSplicer pod, not tools/fire)
- It needs a distinct failure/risk UX (humanity cost, debuff on fail)
- The crafting system doesn't support risk-based outcomes

### Q2 Resolution: Tags + component (option c)

Mutations are stored as:
- **Tags** on the player entity for gameplay effects (e.g., BIO_ELECTRIC → passive shock damage)
- **Mutations component** for metadata (source sample name, splice turn number, whether active/suppressed)

### Q3 Resolution: Permanent debuff tags (option a)

Failed splices apply a permanent malapty tag from the new `malapty` archetype (e.g., MALFORMED, WEAKENED_IMMUNE, CHROMATIC_INSTABILITY). These stack and have persistent gameplay effects.

---

## 3. Module Architecture

### 3.1 New: `crates/core/src/gene_splicing.rs`

Core splicing logic, following `crafting.rs` patterns:

```rust
pub struct SplicingRecipe {
    pub name: String,
    pub input_sample_tag: String,    // Tag the tissue sample must have
    pub output_mutation_tag: String, // Tag applied on success
    pub output_mutation_name: String, // Display name of adaptation
    pub success_chance: f64,         // Base success probability (0.0–1.0)
    pub humanity_cost: u32,          // Humanity points lost on success
    pub failure_tags: Vec<String>,   // Malapty tags applied on failure
    pub description: String,         // Lore text for the adaptation
}
```

Loading from `gene_splicing.toml`, same pattern as `load_crafting_recipes`.

**Key functions:**
- `load_splicing_recipes(toml_str) -> Vec<SplicingRecipe>`
- `find_available_recipes(recipes, inventory, player_pos, world, registry) -> Vec<RecipeAvailability>`
- `execute_splice(recipe, player_entity, world, registry) -> SpliceOutcome`

### 3.2 New: `SpliceOutcome` enum

```rust
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
```

### 3.3 New Components/Resources (in `components.rs`)

```rust
#[derive(Component)]
pub struct GeneSplicer;  // Marker for Gene-Splicing Pod world objects

#[derive(Resource)]
pub struct Humanity {
    pub value: u32,   // 0–100
    pub max: u32,     // always 100
}

#[derive(Component)]
pub struct Mutations {
    pub active: Vec<MutationEntry>,
    pub total_splices: u32,
    pub failed_splices: u32,
}

pub struct MutationEntry {
    pub tag_id: TagId,
    pub source_sample: String,
    pub splice_turn: u64,
    pub suppressed: bool,
}
```

### 3.4 New Game Events (in `events.rs`)

```rust
GeneSpliced {
    sample_name: String,
    adaptation_name: String,
    success: bool,
    humanity_after: u32,
}

GeneSplicingFailed {
    sample_name: String,
    malapty_name: String,
    humanity_after: u32,
}
```

### 3.5 New Tags (in `tags.toml`)

**New archetypes:**

```toml
[[archetype]]
id = "biological"
name = "Biological Property"
exclusivity = "any"

[[archetype.tags]]
id = "VIABLE_TISSUE"       # Item tag: extractable tissue sample
[[archetype.tags]]
id = "GENE_SPLICED"         # Player tag: has undergone at least one splice
[[archetype.tags]]
id = "PRE_COLLAPSE"         # Already added by lore-realignment
[[archetype.tags]]
id = "TECHDEPENDENT"        # Already added by lore-realignment

# Active mutations (player tags)
[[archetype.tags]]
id = "SONIC_CAVITATION"     # Ranged stunning shockwave (pistol shrimp)
[[archetype.tags]]
id = "ANHYDRO_CHITIN"       # Extreme damage resistance at low HP (tardigrade)
[[archetype.tags]]
id = "BIO_ELECTRIC"         # Passive melee shock damage (electric eel)
[[archetype.tags]]
id = "EXOTHERMIC_SPRAY"     # Acid spray, armor degradation (bombardier beetle)
[[archetype.tags]]
id = "CHROMATOPHORIC"       # High evasion, tile-meld stealth (cuttlefish)
```

**New malapty archetype:**

```toml
[[archetype]]
id = "malapty"
name = "Genetic Malapty"
exclusivity = "any"

[[archetype.tags]]
id = "MALFORMED"            # -1 sight range, -1 move speed
[[archetype.tags]]
id = "WEAKENED_IMMUNE"      # -20% poison/disease resistance
[[archetype.tags]]
id = "CHROMATIC_INSTABILITY" # Cannot benefit from stealth/evasion
[[archetype.tags]]
id = "NEURAL_FRAGILE"       # -2 max HP per level
[[archetype.tags]]
id = "BIO_RECESSION"        # -5 humanity per splice instead of -3
```

---

## 4. Gameplay Flow

### 4.1 Acquiring Tissue Samples

Tissue samples are added to `loot_tables.toml` as new entries linked to Carapace and Sanguine entity types:

```toml
[[loot_entry]]
name = "Carapace Tissue Sample"
templates = ["carapace_spawn", "carapace_brute", "carapace_horror"]
tags = ["VIABLE_TISSUE", "TISSUE_PISTOL_SHRIMP"]
glyph = "t"
color = [220, 120, 80]
drop_chance = 0.8
quality = "RARE"
```

- Carapace creatures (Carapace Spawn, Brute, Horror) drop `VIABLE_TISSUE` items on death via loot tables
- Sanguine nobles and enforcers also drop tissue samples
- Sample quality correlates to creature tier: COMMON/UNCOMMON/RARE/EPIC/LEGENDARY
- Each entity template's loot table entry specifies which tissue tag (e.g., `TISSUE_PISTOL_SHRIMP`, `TISSUE_TARDIGRADE`) it drops, and the drop chance (typically 60-90%)
- Boss-tier entities (Carapace Horror, Sanguine Enforcer) guarantee a drop

### 4.2 Finding a Gene-Splicing Pod

- GeneSplicer entities spawn in `PreCollapseFacility` and `SubterraneanRift` dungeon types, and in `cryo_vault` WFC locations
- Placement: one pod per dungeon floor, positioned in a predefined room (tech room / lab room)
- Pod spawn rules added to `spawn_rules.toml`:

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

- The pod is a world object with `GeneSplicer` marker component, rendered as a cyan `P` glyph
- Player must be adjacent to use (same proximity model as crafting stations / repair stations)
- Pods are indestructible and persist for the entire game session

### 4.3 Splicing Interface

1. Player presses `G` while adjacent to a GeneSplicing Pod
2. Overlay shows available tissue samples and their corresponding adaptations:
   ```
   ┌─ Gene-Splicing Interface ───────────────────┐
   │                                              │
   │  Available Samples:                          │
   │  > Pistol Shrimp Gland (SONIC_CAVITATION)    │
   │    [65%] -3 Humanity | Extends sight range   │
   │                                              │
   │    Tardigrade Tissue (ANHYDRO_CHITIN)        │
   │    [45%] -5 Humanity | Dmg resist @ low HP   │
   │                                              │
   │  Gene-Splicing Pod Status: OPERATIONAL       │
   │  Humanity: ████████░░ 82/100                 │
   └──────────────────────────────────────────────┘
   ```
3. Select sample → confirmation prompt showing success chance, humanity cost, failure risk
4. Execute: deterministic roll using sample quality tier + player's total splice count
5. Result: success (adaptation tag) or failure (malapty tag) + humanity change

### 4.4 Success/Failure Formula

```
base_chance = recipe.success_chance
quality_mult = match quality_tag: COMMON=0.8, UNCOMMON=1.0, RARE=1.2, EPIC=1.4, LEGENDARY=1.6
splice_penalty = 0.03 * total_splices  # Each splice makes future ones harder
humanity_mod = if humanity < 30: +0.1, if humanity < 10: +0.15  # Desperate gamblers get a boost
final_chance = (base_chance * quality_mult - splice_penalty + humanity_mod).clamp(0.05, 0.95)
```

Roll: RNG < final_chance → success, else failure.

### 4.5 Humanity Meter

| Range | Effect |
|-------|--------|
| 100 | Full human — standard NPC reactions |
| 70–99 | Minor mutations — some NPCs uneasy, Sanguine dialogue opens |
| 40–69 | Significant chitin — human factions wary, Familiars deferential |
| 10–39 | Mostly monster — attacked on sight by humans, Sanguine treat as kin |
| 0–9 | Carapace-mind — narrative endgame state |

**Loss:** Decreases on each successful splice (per recipe config, typically 3–8).
**Gain:** Rare narrative events (acts of genuine compassion, specific purified compounds).

---

## 5. Integration Points

### 5.1 New Input `InputEvent::GeneSplice`

Add to `input.rs` in game-core. Default key: `G`.

### 5.2 Game Loop Integration

In `src/app.rs`, add gene-splicing overlay handling alongside crafting:
- Toggle with `G` key (only when adjacent to GeneSplicer pod)
- Load recipes from `gene_splicing.toml`
- Compute availability based on inventory samples + proximity to pod
- Execute splice with outcome display

### 5.3 Status Effect Integration

Active mutation tags can interact with existing interaction rules in `interactions.toml`:
- `BIO_ELECTRIC + FLESH → SHOCK_DAMAGE` (cross-entity)
- `ANHYDRO_CHITIN + HEALTH_LOW → DAMAGE_RESIST` (self-interaction, using magnitude)

### 5.4 Quest Integration

Quests can reference gene-splicing:
- "Extract a tissue sample from a Carapace Horror"
- "Use a Gene-Splicing Pod" (objective type)
- "Reach humanity below 30" (fail condition for certain human faction quests)

### 5.5 LLM Narrative Integration

The LLM prompt builder (`prompt.rs`) should include:
- Player's current humanity value and range description
- Any active mutations in the narrative context
- Recent gene-splicing events

### 5.6 Save/Load

- `Humanity` resource serialized in SaveGame
- `Mutations` component serialized per-player
- `GeneSplicer` entities don't need special handling (marker component, no state)

---

## 6. TOML Config: `gene_splicing.toml`

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

---

## 7. Implementation Order

### Step 1: Foundation (data layer)
- Add new tags to `tags.toml`: VIABLE_TISSUE, mutation tags, malapty tags
- Create `gene_splicing.toml` with initial 5 recipes
- Add `GeneSplicer` component, `Humanity` resource, `Mutations` component to `components.rs`
- Add gene-splicing event variants to `events.rs`

### Step 2: Core logic module
- Create `crates/core/src/gene_splicing.rs`
- Implement `load_splicing_recipes`, `find_available_recipes`, `execute_splice`
- Implement success/failure formula
- Write unit tests (at least 5 tests)

### Step 3: UI integration (binary crate)
- Add `InputEvent::GeneSplice` to input module
- Add gene-splicing overlay handling in `src/app.rs`
- Add gene-splicing rendering in `project.rs` / renderer

### Step 4: World spawning
- Add spawn rules for tissue sample drops on Carapace/Sanguine entities
- Add GeneSplicer entity spawning in PreCollapseFacility dungeons
- Ensure deterministic spawning (seeded RNG)

### Step 5: System integration
- Add humanity to save/load serialization
- Add LLM narrative context for mutations/humanity
- Add narrative events for gene-splicing outcomes
- Add quest objectives for tissue collection / splicing

### Step 6: Polish
- Humanity meter display in HUD
- Mutation status display in character panel
- Event formatting for gene-splicing messages
- Visual feedback (glyph changes on mutation)

---

## 8. Test Plan

| Test | Coverage |
|------|----------|
| Recipe loading from TOML | 2 tests: valid/corrupt |
| Availability checking | 2 tests: has sample + pod, missing sample |
| Splice execution (success) | 1 test: verifies tag applied, humanity decreased |
| Splice execution (failure) | 1 test: verifies malapty tag applied, humanity not decreased |
| Humanity clamping | 1 test: 0-100 bounds |
| Save/load round-trip | 1 test: humanity + mutations survive serialization |
| No pod adjacency | 1 test: recipe not available without GeneSplicer nearby |
| Mutation interaction tests | 1 test per mutation tag (verifies tag works in interactions) |

---

## 9. Open Items / Future

- Mutation suppression mechanic (temporary disable a mutation)
- Hybrid mutations (combine two samples for unique outcome)
- Reversed mutations (intentional malapty for roleplay)
- NPC reactions to visible mutations (dialogue filtering by humanity + mutation count)
- Pre-collapse lore documents about gene-splicing (data drive content)
