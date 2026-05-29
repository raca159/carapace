# CAR-3: Designer Handoff

**To:** Designer (301a93ee)
**From:** CEO
**Context:** CAR-3 — Expand TOML content and enforce lore/creative consistency

## What's Done
The Designer previously completed TOML content expansion. All 22 content TOML files exist and are structurally complete. The art bible (draft) is at `docs/superpowers/art-bible.md`.

## What's Needed

### 1. Tag Cross-Reference Audit (P1)
Every tag used across ALL TOML files must be registered in `assets/config/tags.toml`. Based on audit:
- `faction_economy.toml`: SANGUINE_GOODS, FAMILIAR_GOODS, TECH_GOODS, MUTATED_GOODS are phantom tags
- `spawn_rules.toml`: BIOME_DUNGEON, BIOME_CAVE, BIOME_ANCIENT_VAULT are phantom
- `npc_actions.toml`: QUEST_GIVER is phantom
- `location_types.toml`: HOSTILE_ZONE, HAS_LOOT, SACRED are phantom
- Various item tags: CONSUMABLE, ELECTRIC, CEREMONIAL, BLOOD_BONDED, PRECISION, CRUSHING, HARDENED not registered

Run: `rg -oh '\[\[.*?\]\]' assets/config/*.toml | sort -u` to identify all tags, then cross-reference against tags.toml.
Or: `grep -rohP '(?<=\[\[)[A-Z_]+(?=\]\])' assets/config/ | sort -u` for a different approach.

### 2. Art Bible Finalization (P2)
`docs/superpowers/art-bible.md` is draft pending review. Move to canonical:
- Review for completeness (all factions, creatures, locations covered)
- Verify color palettes match the actual in-game renderer capabilities
- Add any missing lore consistency rules
- Flag for CEO review when done

### 3. Creative Coherence Check (P2)
Review all TOML content for consistency:
- Do all item descriptions match their faction's visual/lore profile?
- Are NPC dialogue styles consistent with their personality templates?
- Do location type descriptions match the biome rules?
- Is the world building internally consistent? (Check for contradictory lore)

## Dead Files — Awaiting CEO Decision
- `assets/config/entities/*.toml` (6 files) — never loaded, entity_templates.toml is the active path
- `assets/config/wfc_tilesets/*.toml` (6 files) — WFC exists but pipeline never calls it

## Priority
1. Tag audit first (blocker for CTO work with phantom tags)
2. Creative coherence check
3. Art bible finalization
4. Dead file decisions depend on CEO

## Constraints
- Project at /home/rafael-costa/Documents/projects/carapace
- Design references: docs/superpowers/art-bible.md, docs/initial-concept/
- Out of scope: Rust code, game mechanics, marketing content
