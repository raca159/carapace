# World Exploration & NPC Interaction Design

> **Status:** Spec — pending review
> **Goal:** Wire location traversal, encounters, world overview, weather, pathfinding, and NPC personalities into a cohesive system using the cascade engine's tag-based infrastructure

**Architecture:** Six systems. Location traversal and world overview handle the player's spatial awareness. Encounters, weather, and pathfinding form a runtime feedback loop (weather → encounter probability → NPC detection range → pathfinding cost). NPC personalities are the bridge between world generation and runtime behavior.

**Integration surface:** Every system reads from the cascade engine's outputs (LocationMap, RegionEconomies, terrain tags, entity tags) and feeds into NPC AI (detection, chase, flee, dialogue). The runtime loop is: **player moves → weather ticks → encounter rolls → NPCs detect/think/move using pathfinding + personality**.

---

## Cluster 1: World Exploration & Navigation

### 1. Unified Location Traversal

**Problem:** Dungeon entry/exit code exists but has no key binding. 6 location types from cascade engine are placed on the map but only the player walking onto their tile has any gameplay effect (none currently). Different location types need different entry behaviors (dungeon → BSP interior, city → zone shift, ruin → narrative/loot).

**Design:**

A single `>` key that checks the player's position against LocationMap and switches context:

```
Player presses > on:
  ├── Dungeon entrance tile → try_enter_dungeon() → BSP dungeon, loot, stairs
  ├── City/village tile → enter_settlement() → zone shift (different spawn rules, NPCs)
  ├── Cave/ruin tile → generate_interior() → small WFC or predefined layout
  └── Within a dungeon on:
        ├── DeeperStair → try_go_deeper() → deeper level
        └── EntranceStair → try_exit_dungeon() → overworld
```

Key mapping: `>` or `Enter` — single key, context-sensitive routing.

**Integration with LocationMap:**
- `LocationMap.locations` already has type, position, zone_radius for every placed location
- On `>` press, query `location_at(player_x, player_y)` from locations.rs
- Match on `location_type` to route to handler:
  - `"dungeon"`, `"cave"` → existing `try_enter_dungeon` (with BSP generation)
  - `"city"`, `"village"`, `"outpost"` → `enter_settlement` (future: zone with special spawn rules, barter access)
  - `"ruin"`, `"shrine"` → `enter_poi` (future: small interior, loot, narrative event)

**What exists:**
- `try_enter_dungeon` — biome→type mapping, BSP gen, loot, MapLayer state
- `try_go_deeper` / `try_exit_dungeon` — depth progression, overworld return
- `MapLayer` resource — active dungeon + overworld position tracking
- `cascade::locations::location_at` — position→location lookup
- `location_types.toml` — 7 types with pass/radius/tags

**Complexity:** Medium. Key binding + routing is small. New settlement/POI entry handlers are stubs that can grow later. The hard part (BSP gen, loot, state management) already exists.

---

### 2. Encounters as Procedural Event System

**Problem:** Encounters module is completely dead (not declared, not loaded, not triggered). Currently hardcoded 15% flat chance with no context awareness.

**Design:**

Encounters are **procedurally composed from context** rather than statically defined. The existing `encounters.toml` becomes a set of **component templates** — entity types, loot types, and effects — that are assembled based on the player's environment:

```
Player moves one tile
  → Calculate encounter_chance from:
      Biome tags (wilderness > settled)
      Location proximity (near city → lower wild animal chance)
      Weather/time (night → +20%, storm → +15%)
      Faction territory (in great_carapace zone → carapace encounters)
  → If triggered:
      Determine encounter "mood" from context:
        Aggressive zone → hostile entities
        Near city → merchant/trader
        Settlement zone → guard patrol
        Night → nocturnal creatures
      → Compose encounter from mood:
        Pick entity type from context-eligible pool
        Roll count from mood × region density
        Roll equipment using cascade engine (entity tags × location economy)
        Attach loot using cascade loot tables
      → spawn_encounter() at player position
```

**Key shift:** Instead of `roll_encounter()` picking from a static pool, encounters are **generated** at runtime using the cascade engine. The entity types, their equipment, and their loot all flow through the same pipeline as world gen — they're just smaller batches.

**What exists:**
- `roll_encounter` / `spawn_encounter` — function structure, entity/item spawning code
- Cascade engine — `generate_entity_equipment`, `roll_inventory`, items.toml
- LocationMap — determines "what zone is this" for encounter mood
- EncounterDef — spawn lists, loot, effects (becomes templates)

**What's needed:**
- Declare `pub mod encounters;` in `crates/core/src/lib.rs`
- Insert `Encounters` resource loaded from `encounters.toml`
- Replace 15% hardcoded chance with context-driven probability function
- Replace static weighted pick with procedural composition using cascade engine
- Call `roll_encounter` during movement in `handle_game_input`

**Complexity:** Medium-High. The procedural composition is new logic. The cascade engine already has all the building blocks (entity gen, equipment, inventory). The main work is connecting the encounter trigger to the cascade pipeline at runtime.

---

### 3. World Overview Map

**Problem:** `WorldOverviewState` is fully declared (82 lines, cursor/zoom/pan) but never activated.

**Design:**

`M` key toggles a Bevy UI overlay showing the full overworld map:
- Terrain rendered at reduced resolution (biome colors per tile)
- Location markers from LocationMap (city=V, dungeon=D, cave=O, etc.)
- Player position marker (@)
- Cursor for selecting locations → show location info (name, type, faction)
- Cursor at player position on open
- Arrow keys move cursor, Esc closes, Enter on location → could open location info

The map renders from the existing WorldMap tiles, not from a separate minimap. Since WorldMap is 200×200 tiles and each tile is 16px in game, the overview would render at something like 4×4 pixel per tile → 800×800 total, which fits in the window.

**What exists:**
- `WorldOverviewState` resource — cursor, zoom, pan, mode
- `LocationMap` — location positions, types, names
- `WorldMap` — tile entity references with biome colors
- `AppScreen::WorldOverview` — state machine entry exists

**What's needed:**
- Insert `WorldOverviewState` resource at world gen
- Bind `M` key in `handle_game_input` to toggle `WorldOverviewState.active`
- Bevy UI system that renders the map: iterate all tiles, draw colored rectangles
- Location markers on top of terrain
- Transition to/from `AppScreen::WorldOverview` or just show as overlay on InWorld

**Complexity:** Medium. The data exists. The main work is the Bevy UI rendering (up to 40,000 colored rectangles at 4×4px each). Needs performance consideration.

---

### 4. Weather & Day/Night Cycle

**Problem:** `WeatherState` (218 lines, 8 weather types, time of day, visibility modifiers) is a dead file — module not declared, resource never inserted.

**Design:**

Weather ticks each turn and broadcasts **composable tags** that other systems check:

```
Weather state:
  weather: Weather::Rain
  time: TimeOfDay::Night
  
→ Active weather tags: ["RAINY", "WET", "DARK", "COLD", "REDUCED_VISIBILITY"]
  (stored as a resource: WeatherContext { tags: Vec<String> })
```

Systems consuming weather tags:
- **Encounters** — DARK + RAINY = +25% encounter chance, different moods
- **NPC detection** — REDUCED_VISIBILITY → `SIGHT` range halved
- **HUD** — weather icon + time of day display
- **Movement** — future: SLIPPERY during rain/ice, SLOW in deep snow

Weather lifecycle:
```
finish_npc_turn:
  weather_state.advance_time(world)
    → turn_count++
    → every 50 turns: Dawn→Day→Dusk→Night cycle
    → when weather_turns_remaining expires: roll_new_weather()
      → weighted by time of day (night→fog, day→clear more likely)
      → set 5-25 turns until next weather change
  → update WeatherContext tags from current weather + time
```

**What exists:**
- `Weather` enum (8 types) + `TimeOfDay` enum (4 phases)
- `WeatherState` resource with `advance_time()`, `roll_new_weather()`
- `visibility_modifier()` + `light_level()` + `effective_visibility()`
- `WeatherState::new()` — initializes Clear + Day + 20 turns

**What's needed:**
- Declare `pub mod weather;` in `crates/core/src/lib.rs`
- Insert `WeatherState` + `WeatherContext` resources in `world_gen.rs`
- Call `advance_time()` in `finish_npc_turn`
- Wire weather tags → encounter probability in roll_encounter
- Wire weather tags → NPC detection range in behavior.rs

**Complexity:** Low-Medium. Module declaration + resource insertion is trivial. Weather→encounter and weather→NPC detection wiring is a few function calls each. The tag-based composition approach makes it clean.

---

### 5. Pathfinding + NPC AI

**Problem:** NPC AI uses `chase_direction` (naive signum — walks into walls) and `flee_direction` (geometric only). A full A* implementation (655 lines) is a dead file.

**Design:**

This is the **final consumer of all cascade data** at runtime. The NPC AI already has a scoring system that evaluates actions (chase/flee/guard/approach/wander). Pathfinding integrates at the movement level:

```
process_npc_turns for each creature:
  1. Detection phase — use has_line_of_sight() + weather visibility modifier
     (can't chase what you can't see)
  
  2. Action scoring phase — unchanged (tag-based, weighted)
     → picks "chase" if AGGRESSIVE, "flee" if COWARDLY, etc.
  
  3. Movement phase — replaces signum with pathfinding:
     chase: a_star_step(start, target, map, world, creature_tags, occupied, ...)
       → returns first cardinal step toward goal
       → naturally avoids walls, blocked tiles, water (unless AQUATIC)
     flee: a_star_step(start, flee_target (away from threat), ...) 
       → pathfind away instead of geometric flee
     wander: stays random (already works)
     guard: a_star_step toward home if away from zone
     approach: a_star_step toward non-hostile target
```

The key insight: `a_star_step` takes **all the same arguments** that `behavior.rs` already computes for `try_move`. The integration is replacing `chase_direction` with `a_star_step` at one call site.

**What personality scores add to pathfinding:**
- Aggressive creatures (score > 70) → pursuit pathfinding (will A* around obstacles)
- Cowardly creatures (score < 30) → flee pathfinding (A* away from nearest threat)
- Territorial creatures → pathfind back to home zone
- Curious creatures → pathfind toward interesting entities/items

**What exists:**
- `a_star_step` — A* returning first cardinal step (g, h, f with Manhattan heuristic)
- `has_line_of_sight` — Bresenham's line algorithm
- `behavior.rs` — already computes all parameters that A* needs
- NPC action scoring — fully functional tag-based priorities

**What's needed:**
- Declare `pub mod pathfinding;` in `crates/world/src/lib.rs`
- Replace `chase_direction` internal logic with call to `a_star_step`
- Replace `flee_direction` with pathfinding toward best retreat direction
- Add `has_line_of_sight` check before chase initiation
- Weather → detection range wiring (visibility modifier × SIGHT magnitude)

**Complexity:** Medium. One function swap for chase, plus line-of-sight integration. Flee pathfinding is slightly more novel (need to pick a retreat target). The real value isn't in the code complexity — it's that A* makes ALL the cascade data (terrain tags, blocked tiles, swimmable areas, flight paths) actually matter to NPC behavior.

---

## Cluster 2: Economy & NPC Interaction

### 6. NPC Personalities

**Problem:** `NpcPersonalitiesResource` is loaded (12 archetypes) but no system reads it. Behavioral tags are hardcoded in `spawn_rules.toml` rather than derived from personality.

**Design:**

Every NPC entity gets a `PersonalityScores` component with 0-100 values for a fixed set of traits:
```
aggression, bravery, sociability, orderliness, curiosity,
industriousness, honesty, spirituality, gregariousness, volatility
```

Behavioral tags are **derived from scores**, not hardcoded:
```
aggression > 70 → AGGRESSIVE       |  aggression < 30 → PEACEFUL
bravery < 30    → COWARDLY         |  bravery > 70    → FEARLESS
curiosity > 60  → CURIOUS          |  orderliness > 70 → TERRITORIAL
sociability < 30 → MINDLESS (antisocial) 
```

Score derivation at entity spawn (Stage 3 of cascade):
```
1. Look up NpcPersonalitiesResource for matching archetype
   (match by entity tags + faction)
   
2. If archetype found:
     base_scores = archetype.score_profile (TOML-defined baselines)
   else:
     base_scores = faction_averages or species defaults
   
3. Apply environmental modulation:
     hostile biome → aggression +10, bravery +5
     prosperous location → sociability +10, honesty -5
     military outpost → orderliness +15, volatility +10
   
4. Apply random variance: ±15 for each score (deterministic seed)
   
5. Clamp 0-100, derive tags from score thresholds
   
6. Assign PersonalityScores component + derived tags to entity
```

The personality archetype TOML (`npc_personalities.toml`) already has this structure — `tags` field on each archetype maps to the DF-style scores. A `score_profile` field would be added to define baselines. The 12 existing archetypes become templates.

**What exists:**
- `NpcPersonalitiesResource` — 12 archetypes with tags, speech, values, fears, knowledge
- Behavior system — `process_npc_turns` reads behavioral tags for action scoring
- Dialogue system — `select_dialogue` uses tags for line selection
- Cascade entity gen — `generate_entity` in entity_gen.rs spawns with tags

**What's needed:**
- `PersonalityScores` component (10 scores × u8)
- Score profile field in `NpcPersonality` TOML definition
- Score derivation function (archetype × faction × environment × variance)
- Tag derivation function (scores → behavioral tags)
- Integration into cascade entity spawning (Stage 3)
- Personality → dialogue enrichment (pass scores into dialogue selection)

**Complexity:** High (but bounded — ~200 lines of new code). The structural pieces all exist. The new code is the `PersonalityScores` component, score derivation function, and tag derivation thresholds. Integration with entity gen is one extra step in the spawn pipeline.

---

## Integration Summary

```
              ┌─────────────────────┐
              │  World Generation    │
              │  (cascade engine)    │
              └──────┬──────┬───────┘
                     │      │
       ┌─────────────┘      └─────────────┐
       ▼                                  ▼
┌──────────────┐                   ┌──────────────┐
│ LocationMap   │                   │ Entity roster│
│ + economies   │                   │ + personalities│
│ + trade routes│                   │ + scores→tags│
└──────┬───────┘                   └──────┬───────┘
       │                                  │
       ▼                                  ▼
┌──────────────────────────────────────────────────┐
│              Runtime Game Loop                     │
│                                                    │
│  player moves → roll encounter → weather ticks    │
│       → NPC AI: detect + score + pathfind          │
│         (A* uses terrain, tags, scores, weather)   │
│                                                    │
│  > on location → enter context                    │
│  M key → world overview overlay                   │
│  E key → talk/barter (personality-aware dialogue)  │
└──────────────────────────────────────────────────┘
```

## Implementation Order

| Step | System | Complexity | What | Key Files |
|------|--------|-----------|------|-----------|
| 1 | **Weather** | Low | Module decl + resource + per-turn tick | `weather.rs` init, `world_gen.rs`, `finish_npc_turn` |
| 2 | **Location traversal** | Medium | `>` key binding + LocationMap routing | `game/mod.rs`, `dungeon.rs`, `cascade/locations.rs` |
| 3 | **Encounters** | Med-High | Module decl + procedural composition + movement hook | `encounters.rs` init, `game/mod.rs`, cascade integration |
| 4 | **World overview** | Medium | `M` key + Bevy UI terrain/location rendering | `render/mod.rs`, `WorldOverviewState` |
| 5 | **Pathfinding** | Medium | Module decl + A* swap in behavior.rs | `pathfinding.rs` init, `behavior.rs` |
| 6 | **NPC Personalities** | High | `PersonalityScores` component + derivation + entity gen integration | `personality.rs`, `entity_gen.rs`, `behavior.rs`, dialogue |

Note: Weather is ranked first because it feeds into encounters and pathfinding. Doing it first means those systems can consume weather from day one.
