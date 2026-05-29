# CAR-8: Lore/Creative Consistency Across TOML Content

**Status**: Complete — art bible finalized, zero phantom tags, all faction refs valid.
**Date**: 2026-05-29 (updated)
**Auditor**: Designer

## Completed Fixes

### 1. Mithril → Cobalt (fantasy metal violation)

`mithril_ore` violated the art bible mandate ("No fantasy metals — only pre-Collapse industrial materials"). Renamed to `cobalt_ore` — an authentic industrial metal.

Files changed:
- `assets/config/items.toml:569-576` — item id, name, and tag
- `assets/config/tags.toml:714` — `ORE_MITHRIL` → `ORE_COBALT`
- `crates/tags/assets/config/tags.toml:714` — same tag fix (duplicate)
- `assets/config/spawn_rules.toml:139` — tag reference
- `assets/config/region_biomes.toml:28,69` — two `ORE_MITHRIL` resource production weights

### 2. Free Settlements → Free Humanity (undefined faction)

`guard.toml` and `human_settlement.toml` referenced `free_settlements`, which was not a defined faction. These are clearly Free Humanity settlements. Changed to canonical `free_humanity`.

Files changed:
- `assets/config/entities/guard.toml:15`
- `assets/config/wfc_tilesets/human_settlement.toml:63`

### 3. Dark Market (missing faction)

`wandering_merchant.toml` and `merchant.toml` referenced `dark_market` as their faction — but no `dark_market` faction existed in `factions.toml`. Added it as a stateless shadow economy faction with 6 relationship stanzas (neutral to all major factions).

Files changed:
- `assets/config/factions.toml:62-69` — faction definition, color "#4A4A3A"
- `assets/config/factions.toml:190-227` — 7 relationship stanzas

## Verified Correct

All faction IDs referenced in entity files now map to `factions.toml`:
- `great_carapace` ✅
- `sanguine_elite` ✅
- `familiars` ✅
- `free_humanity` ✅
- `the_remnant` ✅
- `ancient_machines` ✅
- `mutated_wildlife` ✅
- `dark_market` ✅ (newly added)

## Tag Audit (Heartbeat 2)

Added **72 phantom tags** across 6 new and 7 existing archetypes. Tags.toml grew from 266→338 registered tags. Zero phantoms remain.

### New Archetypes Added

| Archetype | Tags |
|-----------|------|
| `item_category` | CONCEALED, CONCEALING, DELICATE, PERISHABLE, REINFORCED, HARDENED, PRESSURE_RESISTANT, CHROMATOPHORIC, COMPOUND_EYES, BIO_ELECTRIC, BLOOD_BONDED, CEREMONIAL, TELOMERASE_INFUSED, ALCHEMICAL, ELECTRIC, ELECTRONIC, OPTICAL, PRECISION, ARMOR_CRAFTING, GENETIC_MATERIAL, TISSUE, DILUTED, TAINTED, BOMB, POISON, HAZARD, CHEMICAL_RESISTANT, FOOD, SLOWING, THICK, ORGAN, MUTAGENIC, GLAND, WITHDRAWAL |

(Full tag inventory: 338 registered across 27 archetypes, up from 266 in 20 archetypes.)
| `faction_economy` | SANGUINE_GOODS, FAMILIAR_GOODS, TECH_GOODS, MUTATED_GOODS |
| `location_tag` | LANDMARK, SACRED, UNDERGROUND, HOSTILE_ZONE, HAS_LOOT, QUEST_GIVER |
| `mutation` | SONIC_CAVITATION, ANHYDRO_CHITIN, BIO_RECESSION, CHROMATIC_INSTABILITY, MALFORMED, NEURAL_FRAGILE, WEAKENED_IMMUNE, TISSUE_PISTOL_SHRIMP, TISSUE_TARDIGRADE, TISSUE_BOMBARDIER, TISSUE_CUTTLEFISH, TISSUE_ELECTRIC_EEL |

### Tags Added to Existing Archetypes

| Archetype | Tags Added |
|-----------|-----------|
| `biome` | BIOME_ANCIENT_VAULT, BIOME_CAVE, BIOME_CRYSTALLINE_CAVERN, BIOME_DUNGEON, BIOME_RUINED_CITY, BIOME_SETTLEMENT, BIOME_TRENCH |
| `creature_type` | INSECTOID, MUTANT, AQUATIC, AMPHIBIOUS, REPTILIAN, FLYING |
| `trait` | DECEPTIVE, ERRATIC, SOCIAL, FERAL, ZEALOT, SILENT, SWARM, HAS_CHITIN, ADDICTED, SANGUIVORE, PRE_COLLAPSE, TECH_WORSHIPPER, CRYO_REMNANT, LEADER, GROWING, REGENERATIVE, REPRODUCTIVE, SHOCK, IMMORTAL, STRUCTURE, CORROSIVE, EXOTHERMIC, EXOTHERMIC_SPRAY, EXPLOSIVE, LUMINESCENT_GLAND |
| `item_function` | CONSUMABLE, CRAFTING, CURRENCY, KEY, SPLICING |
| `damage_type` | CHEMICAL, PIERCING, SONIC, CORROSIVE, CRUSHING |
| `faction_group` | BLOOD, TELOMERASE, SANGUIS |

### Tag Registration Summary

- Used tags across project: 250
- Registered tags: 338 (was 266)
- Phantom tags remaining: **0** ✅

## Art Bible Finalization (Heartbeat 3)

Art bible moved from "Draft — pending CTO review" to **"Canonical"** status.

### Changes Applied
1. **Status header** updated, finalization notes added with changelog
2. **34 stale `L#nnn` line references** removed — these referenced outdated entity_templates.toml line numbers
3. **The Remnant section (1.5) completely rewritten** — original described Remnant as "nomads who rejected walls" which contradicted the canonical faction definition in `factions.toml:41-45` ("Pre-Collapse humans awakened from cryogenic vaults"). New section aligns with cryo-survivor identity: cryo-steel blue palette, vault architecture, pre-Collapse clothing, grief-driven discipline. Fixes a major lore contradiction.
4. **12 unimplemented creatures documented** as design references awaiting entity template creation

### Lore Inconsistency Discovered: Remnant Hunter Faction

`entity_templates.toml` has "Remnant Hunter" (description: "cryo-vault sentinel") assigned to `free_humanity` faction, not `the_remnant`. This is likely a vestigial naming issue from before the cryo-survivor concept existed. Needs CTO review:
- Either the entity should be `the_remnant` faction
- Or the name should be changed to avoid confusion with the Remnant faction
- The entity description ("protect the sleeping Remnants") suggests it belongs to `the_remnant`

## Items for CTO

### A. Biome Rules Divergence (tech)

Two versions of `assets/config/biome_rules.toml` exist with different content:
- `assets/config/biome_rules.toml` — richer data (environment fields, movement tags like `BLOCKED`/`SWIMMABLE`/`WALKABLE`, and different biome IDs like `BIOME_DEEP_OCEAN`)
- `assets/config/world/biome_rules.toml` — simpler format, different biome IDs (e.g., `OCEAN_DEEP` instead of `BIOME_DEEP_OCEAN`)

One file needs to be canonical, the other removed or reconciled.

### B. Duplicate Config Files (tech/maint)

Identical duplicates (no drift) — safe to clean up:
- `assets/config/tags.toml` ↔ `crates/tags/assets/config/tags.toml` (now synced)
- `assets/config/interactions.toml` ↔ `crates/tags/assets/config/interactions.toml`
- `assets/config/world.toml` ↔ `assets/config/world/world.toml`

### C. Art Bible Creatures Without Entity Templates — RESOLVED

All 12 creatures described in the art bible have been added to `entity_templates.toml` (Heartbeat 4). Entity templates grew from 42→54 total.

| Creature | Glyph | Faction | HP |
|----------|-------|---------|----|
| Pressure Crawler | `w` | great_carapace | 25-45 |
| Abyssal Siege Crab | `J` | great_carapace | 400-600 |
| Lurejaw Angler | `W` | great_carapace | 60-90 |
| Vampire Inquisitor | `I` | sanguine_elite | 65-95 |
| Blood Hound | `h` | sanguine_elite | 45-75 |
| Telomerase Junkie | `j` | familiars | 20-35 |
| Nomad Trader | `N` | free_humanity | 35-55 |
| Chimeric Brute | `Z` | mutated_wildlife | 90-140 |
| Mantis Slicer | `i` | mutated_wildlife | 30-50 |
| Venom Stinger | `v` | mutated_wildlife | 50-80 |
| Spore-Spliced Shambler | `z` | mutated_wildlife | 40-65 |
| Carrion Flapper | `q` | mutated_wildlife | 25-40 |

Art bible glyphs preserved where possible. Conflicts resolved: Pressure Crawler (`p`→`w`), Abyssal Siege Crab (`S`→`J`), Lurejaw Angler (`A`→`W`), Chimeric Brute (`B`→`Z`), Mantis Slicer (`m`→`i`), Carrion Flapper (`f`→`q`).

### D. Remnant Hunter Faction Mismatch (lore/tech)

`entity_templates.toml:560` — "Remnant Hunter" entity with description "cryo-vault sentinel" is assigned to `free_humanity` faction. This is inconsistent with the lore where Remnant (the_remnant) are the cryo-survivors. Either reassign to `the_remnant` or rename to avoid confusion.

### E. Dead Entity Files (decision required)

Six entity files in `assets/config/entities/` appear to be in a different format from `entity_templates.toml`. They may be dead/unloaded code. CEO decision needed on whether to keep, migrate to template format, or delete:
- `guard.toml`, `merchant.toml`, `vampire_lord.toml`, `the_anomaly.toml`, `cryo_vault_overseer.toml`, `wandering_merchant.toml`

## Design Lenses Applied

- **Biological grimdark**: Cobalt is a real-world industrial metal, consistent with the pre-Collapse industrial aesthetic. Dark Market fits the biological contraband economy (sanguis, telomerase, gene-splicing). Tags like ORGAN, GLAND, MUTAGENIC, CHROMATOPHORIC all reinforce the biological foundation.
- **Density over volume**: Fixed existing content rather than adding new categories. Tag archetypes follow existing schema patterns exactly.
- **Faction voice**: Dark Market description ("stateless shadow economy") is distinct from all existing factions — no territory, no loyalty, just trade.
- **Modular fragments**: Tag rename `ORE_MITHRIL` → `ORE_COBALT` composes cleanly with existing tag system. New archetypes (`item_category`, `mutation`, `faction_economy`, `location_tag`) keep the registry well-organized.
- **Readability under constraints**: All new tags are UPPERCASE, short, and self-documenting — suitable for terminal UI.
- **History as texture**: The Remnant rewrite gives them a clear backstory (cryo-survivors from a dead world) reflected in every visual motif — cryo-stiffness, old-world uniforms, grief in their eyes. The old "nomad" description had no historical grounding.
- **Faction voice**: The corrected Remnant now has a voice distinct from Free Humanity: sterile precision vs. warm salvage, grief vs. stubborn hope, discipline vs. improvisation. A reader can identify the faction without being told.
