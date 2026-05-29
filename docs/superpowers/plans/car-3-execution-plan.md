# CAR-3: Expand TOML Content & Enforce Lore/Creative Consistency

## Status
**Current:** Delegation phase (API unreachable - creating local artifacts)

## Objective
1. Wire all "loaded but dead" content TOML files into the game loop (CTO)
2. Enforce lore/creative consistency across all content (Designer)
3. Resolve dead file decisions (entities/, WFC tilesets) (CEO)

## Delegation Split

### Track A: CTO (ce1450ec) — Technical Wiring
**Priority order (P0 first):**

| Priority | Task | System | Status |
|----------|------|--------|--------|
| P0 | Wire quest completion | `check_quest_completion`, `track_kill`, `track_collect` | Dead code |
| P0 | Wire loot tables into death drops | `LootTables` resource → entity death | Loaded but never queried |
| P1 | Wire narrative events into turn loop | `check_narrative_events()` | Never called |
| P1 | Wire lore fragment discovery | `LoreFragmentsResource` | Never queried |
| P1 | Wire NPC personalities into dialogue | `PersonalityScores` → entity spawn | Never inserted |
| P2 | Resolve dead WFC tilesets (6 files) | `wfc.rs` → cascade pipeline | Or remove |
| P3 | Wire trade UI | `resolve_barter_with_haggle()` | Implemented but uncalled |
| P4 | Wire gene splicing | `gene_splicing.toml` + `.rs.disabled` | 557 lines disabled |

**Reference docs:**
- `docs/TODO.md` — system audit
- `crates/world/src/` — main game loop
- `assets/config/` — all TOML inputs

### Track B: Designer (301a93ee) — Lore & Creative Consistency

| Priority | Task | Status |
|----------|------|--------|
| P1 | Full tag cross-reference audit: validate every tag across all 58 TOML files is registered in `tags.toml` | ~40+ phantom tags |
| P2 | Art bible finalization: move from draft to canonical | Pending review |
| P2 | Creative coherence check: ensure every item, NPC, location feels like the same world | Continuous |
| P3 | Dead entity files: decide remove vs keep (6 files in `assets/config/entities/`) | CEO decision needed first |

### Track C: CEO (8bb450c2) — Strategy & Decisions

| Task | Status |
|------|--------|
| Approve art bible as canonical lore reference | Pending |
| Decide: remove or wire dead files (entities/, WFC tilesets) | Pending |
| Set overall priority for content wiring vs new features | Pending |

## Timeline
- **Immediate:** Wire P0 items (quest completion + loot tables) — game-breaking
- **Next:** Wire P1 items (narrative events, lore fragments, NPC personalities) — high value
- **Soon:** Resolve dead files, finalize art bible
- **Later:** Trade UI, gene splicing

## Key Files
- All TOML: `/home/rafael-costa/Documents/projects/carapace/assets/config/`
- Art bible: `docs/superpowers/art-bible.md`
- System audit: `docs/TODO.md`
- Entity templates: `assets/config/entity_templates.toml`
- Tags registry: `assets/config/tags.toml`

## API Blocker
Cannot reach Paperclip API (host `dotta-macbook-pro` unresolvable from NucBoxG3-Plus). Subtask creation blocked until network is fixed.
