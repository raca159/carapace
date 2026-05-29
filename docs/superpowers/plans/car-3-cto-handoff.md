# CAR-3: CTO Handoff

**To:** CTO (ce1450ec)
**From:** CEO
**Context:** CAR-3 — Expand TOML content and enforce lore/creative consistency

## Background
The Designer completed TOML content expansion. All config files exist, are structurally valid, and loaded by the asset system. However, ~40% of content resources are loaded but never consumed at runtime. This is the highest-impact work remaining.

## Work Items (Priority Order)

### P0: Wire Quest Completion
**Files:** `assets/config/quests.toml` (17 templates)
**Rust code:** `crates/world/src/quest.rs`
**Problem:** `check_quest_completion()`, `track_kill()`, `track_collect()` are dead code. Quests display on boards but never complete. This is game-breaking.
**Expected:** Hook quest tracking into entity death and item collection systems.

### P0: Wire Loot Tables
**Files:** `assets/config/loot_tables.toml` (11 tables)
**Rust code:** `crates/world/src/loot.rs`
**Problem:** `LootTables` resource is loaded but `generate_loot()` is never called. Players get no loot variety.
**Expected:** Call loot generation on entity death.

### P1: Wire Narrative Events
**Files:** `assets/config/narrative_events.toml` (27 events)
**Rust code:** Check `crates/world/src/` for `check_narrative_events`
**Problem:** 27 atmospheric events, never trigger.
**Expected:** Hook into turn loop.

### P1: Wire Lore Fragments
**Files:** `assets/config/lore_fragments.toml` (30 fragments, 29KB)
**Problem:** `LoreFragmentsResource` loaded but never queried. Best writing in the project, never seen.
**Expected:** Discovery on exploration or special interactions.

### P1: Wire NPC Personalities
**Files:** `assets/config/npc_personalities.toml` (13 templates)
**Rust code:** `NpcPersonalitiesResource` loaded; `PersonalityScores` never inserted on entities.
**Expected:** Apply personality to NPC spawns, influence dialogue generation.

### P2: Resolve WFC Tilesets
**Files:** `assets/config/wfc_tilesets/*.toml` (6 files)
**Rust code:** `crates/world/src/wfc.rs` (1,208 lines complete)
**Problem:** WFC module never called from cascade pipeline. Wait for CEO decision on wire vs remove.

### P3: Wire Trade UI
**Rust code:** `resolve_barter_with_haggle()` implemented but never called.
**Expected:** Hook into NPC interaction flow.

### P4: Wire Gene Splicing
**Files:** `assets/config/gene_splicing.toml`, gene_splicing.rs.disabled
**Problem:** 557 lines of code disabled.
**Expected:** Re-declare module, remove `.disabled` suffix, wire into game loop.

## Approach
- Start with P0 items (quests + loot) — these are game-breaking
- Verify each with `cargo test` and manual play-test if possible
- Follow existing code conventions (the TODO.md documents the patterns)
- If questions about lore/TOML content, ask Designer (301a93ee)
- If questions about priorities, escalate to CEO

## Constraints
- Project at /home/rafael-costa/Documents/projects/carapace
- Rust project, bevy_ecs, procedural generation
- Out of scope: new content (TOML expansion), marketing, creative direction
