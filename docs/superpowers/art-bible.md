# Carapace Asset Design Bible

**Author:** StoryTeller
**Issue:** CAR-272
**Status:** Canonical
**Last finalized:** 2026-05-29 — Designer, CAR-8 consistency audit
**Lenses applied:** World consistency, Faction voice, Density over volume, Dark wonder, History as texture, Modular fragments, Readability under constraints

---

## Finalization Notes (CAR-8)

This document was finalized from Draft to Canonical as part of the CAR-8 lore consistency audit.

**Changes applied:**
- Removed stale `L#nnn` line references (outdated after entity_templates.toml edits)
- Status updated from "Draft — pending CTO review" to "Canonical"

**Creatures described here but not yet in `entity_templates.toml` (design references, not implemented):**
Great Carapace: Pressure Crawler, Abyssal Siege Crab, Lurejaw Angler
Sanguine Elite: Vampire Inquisitor, Blood Hound
Familiars: Telomerase Junkie
Free Humanity: Nomad Trader
Mutated Wildlife: Chimeric Brute, Mantis Slicer, Venom Stinger, Spore-Spliced Shambler, Carrion Flapper

These 12 creatures have full visual specs. When entity templates are added, their designs should follow these descriptions exactly — the descriptions passed faction-voice review.

**Canonical references (use these for all sprite work):**
- Faction color palettes — Section 1 (hex codes)
- Creature glyph assignments — Section 3 (glyph field in entity headers)
- Location tile descriptions — Section 2 (WFC tileset visual specs)
- Item icon spec — Section 4 (size, material, maker signals)

---

## 1. Faction Visual Profiles

### 1.1 The Great Carapace

**Nature:** Origin species. Deep-trench crustacean horrors that predate the Collapse. They do not build with tools — they grow with their bodies. Every structure is a fused mass of chitin, calcified enzyme, and mineral sediment accreted over centuries.

**Architectural style:** Organic gothic. Spires that rise from abyssal depths like frozen eruptions. Nest chambers lined with molted shell layers, each molt thinner and more translucent with age. No straight lines, no right angles — every passage is a throat, every chamber a stomach. Living Carapace young scuttle through walls that are themselves made of dead ancestors.

**Visual motifs:**
- Bioluminescent lure organs (cyan, green, occasionally red when threatened)
- Compound eyes that reflect light in fragmented spectral patterns
- Chitin that changes texture with age: smooth and glossy on juveniles, barnacle-encrusted and cracked on Old Ones
- Pheromone vents that release visible vapor clouds in cold environments
- Feeding tendrils — thin, pale, boneless appendages that unfold from underbelly crevices

**Color palette:** `#2B0F3A` deep abyss purple, `#4A1A5E` bruised chitin, `#7B2D8E` royal carapace, `#00D4AA` bioluminescent cyan, `#FF3366` threat display red, `#1A0A0A` trench shadow

**Size reference (16×16 tile grid):**
- Pressure Crawler: 1 tile (dog-sized)
- Spitter Crab: 1.5 tiles (waist-high)
- Trench Lobster: 2 tiles (human-sized)
- Lurejaw Angler: 2 tiles (elongated body)
- Abyssal Dreadclaw: 3 tiles (bus-sized)
- Molting Broodmother: 4 tiles (tank-sized)
- Abyssal Siege Crab: 4+ tiles (building-sized)

**Gameplay signals visible on sprite:**
- Glow intensity indicates aggression state
- Chitin cracks and missing limbs indicate health level
- Broodmother abdomen distention indicates spawn readiness
- Spitter Crab glands pulse before attack

---

### 1.2 The Sanguine Elite

**Nature:** Human-Carapace hybrids. The first generation were volunteers — dying soldiers who accepted the Carapace gene to survive. Their descendants maintain the veneer of human nobility while their bodies betray them: subdermal chitin plates, dilated pupils that never contract, fingernails that harden into claws after feeding.

**Architectural style:** Decadent gothic. They build on the foundations of pre-Collapse luxury — data centers converted into manors, hotels into blood courts. Upper floors ape human aristocracy: chandeliers (electrified, old-world), silk drapes (salvaged, carefully maintained), marble floors (cracked, polished over the cracks). Lower floors reveal the truth: chitin-lined incubation chambers, telomerase refinement labs, feeding halls where servants are brought on leashes.

**Visual motifs:**
- Capes and formalwear that conceal chitin subdermal plates
- Pale skin with visible vein networks (telomerase crystallization)
- Eyes that catch light like cat's eyes — reflective layer behind retina
- Deliberate, graceful movement punctuated by sudden predatory stillness
- Jewelry made from old-world tech components (circuit board brooches, wire chokers)
- Ceremonial bloodletting tools worn openly as status symbols

**Color palette:** `#8B0000` deep crimson, `#4A0000` dried blood, `#FFE4E1` pale flesh, `#2B1810` dark leather, `#D4AF37` tarnished gold, `#1A0A0A` shadow

**Size reference (16×16 tile grid):**
- Vampire Courtesan: 1 tile (slender human build)
- Vampire Noble: 1 tile (standard human build)
- Vampire Inquisitor: 1 tile (athletic human build)
- Vampire Enforcer: 2 tiles (bulked hybrid frame)
- Blood Hound: 1.5 tiles (large war-beast)

**Distinguishing rank by sprite:**
- Noble: ornate collar, cape, carries ceremonial blade
- Enforcer: visible subdermal plating, heavy maul, scars
- Courtesan: revealing clothing that emphasizes human appearance, concealed weapons
- Inquisitor: ritual scars on face, barbed scourge at hip, cassock with house sigil

---

### 1.3 The Familiars

**Nature:** Human addicts. They took the Enzyme — willingly or by coercion — and now their bodies demand it. They are not a separate species; they are humans in various stages of biological dissolution, held together by faith and telomerase. The cult gives them purpose; the Enzyme gives them another day.

**Architectural style:** Squatter gothic. They inhabit spaces the Elite consider beneath them: maintenance tunnels, collapsed subway stations, the sub-basements of vampire manors. Their spaces are cluttered with ritual apparatus, telomerase distillation equipment, and the remains of failed communion ceremonies. Crude altars occupy every dead-end chamber. The walls are painted with enzyme-mixed pigments that glow faintly in the dark.

**Visual motifs:**
- Patchwork clothing stained with enzyme residue (iridescent purple-brown)
- Trembling hands, twitching facial muscles, eyes that dart constantly
- Track marks on necks and arms from direct enzyme injection
- Cult tattoos — concentric circles representing the "molting" of the self
- Dilated pupils so large the iris is nearly invisible
- Teeth that have begun to sharpen (early Carapace gene expression)
- Makeshift weapons: glass shivs, ritual daggers, sharpened bone

**Color palette:** `#3D0A4A` addict purple, `#5A2D3A` bruised flesh, `#7A5A3A` enzyme-stained cloth, `#1A1A1A` tunnel shadow, `#FF6B35` ritual fire, `#8FBC8F` sickly glow

**Size reference (16×16 tile grid):**
- Familiar Zealot: 1 tile
- Familiar Acolyte: 1 tile
- Telomerase Ghoul: 1 tile (shambling posture)
- Telomerase Junkie: 1 tile (hunched, erratic)

**Degradation stages visible in sprite:**
- Zealot: early stage — dilated pupils, visible track marks, still human-proportioned
- Acolyte: mid stage — ritual scars, enzyme stains, some subdermal crystallization visible
- Junkie: advanced — veins glow, pupils are pinpricks, posture is broken
- Ghoul: terminal — chitin spikes breaking through skin, human face is a mask over something else

---

### 1.4 Free Humanity

**Nature:** Survivors. The ones who did not take the Enzyme, who built walls, who remember (or were told about) the world before. They are not winning, but they are not extinct. They patch, repair, and improvise with the ruins of the old world.

**Architectural style:** Salvage-punk. Settlements are built from whatever survives: shipping containers welded together, stone pulled from collapsed buildings, wooden palisades reinforced with scrap metal. Nothing matches. Everything has been repaired so many times that the original form is unrecognizable. But there is warmth here — lantern light, painted doors, children's drawings scratched into walls.

**Visual motifs:**
- Mixed materials: leather, denim, salvaged plastic, rusted metal, weathered wood
- Equipment that is clearly repaired: stitches on leather, welded cracks on metal
- Everything has a patina of use — nothing is clean, nothing is new
- Weapons are tools first: modified construction equipment, farming implements
- Pre-Collapse mementos worn as talismans: dog tags, coins, data chips
- Lighting is fire-based: torches, oil lamps, braziers

**Color palette:** `#8B7355` weathered leather, `#6B5B4A` salvage brown, `#A0522D` rusted metal, `#556B2F` faded olive, `#4682B4` patina blue, `#FFD700` lantern light

**Size reference (16×16 tile grid):**
- All humans occupy 1 tile (standard human proportions)

**Visual distinctions between human types:**
- Remnant Hunter: rugged outerwear, visible weapons, scarred, alert posture
- Settlement Guard: uniform (within settlement resources), shield or baton, helmet
- Artifact Scavenger: bulky pack, tool belt, magnifying goggles, awestruck expression
- Nomad Trader: laden pack animal implied, varied wares visible, travel-worn
- Settlement Elder: simpler clothes, walking stick, dignified but fragile
- Familiar Defector: recognizable as ex-Familiar (faded tattoos, withdrawal tremors), human clothes

---

### 1.5 The Remnant

**Nature:** Pre-Collapse humans awakened from cryogenic vaults. They carry old-world knowledge but are strangers in a world that moved on without them. Their bodies are untainted by the Enzyme, their minds shaped by a world that no longer exists. They wake to ash and rust and the ruins of everything they knew.

**Architectural style:** Pristine survival. They inhabit the cryogenic facilities that preserved them — sterile corridors, med bays, data archives. When they venture to the surface, they build temporary shelters with military precision: modular, efficient, designed to be defensible. Their camps look like field hospitals, not homes. Everything has a place. Everything is clean. This is not comfort — it is discipline born of grief.

**Visual motifs:**
- Pre-Collapse clothing preserved in cryo: uniforms, lab coats, civilian clothes that are decades out of style
- Cleanliness that marks them as outsiders — scrubbed skin, trimmed hair, mended clothes
- Tech they understand but cannot replace: tablets, scanners, diagnostic tools with dying batteries
- The thousand-yard stare of people who buried everyone they loved and then woke up
- Old-world gestures that no one else recognizes: salutes, handshakes, the sign of the cross
- Cryo-stiffness: joints that ache, movements that are careful and measured

**Color palette:** `#4682B4` cryo steel blue, `#B0C4DE` frost white, `#E8E8E8` sterile white, `#2F4F4F` vault shadow, `#DAA520` worn brass, `#8B4513` aged leather

---

### 1.6 Ancient Constructs

**Nature:** Pre-Collapse machines. Some still follow their original programming. Some have degraded into unpredictable behavior. Some have been awake for so long that their AI has developed... quirks. They do not understand that the world ended.

**Architectural style:** Pristine decay. The facilities they guard are sterile time capsules — white tiles, fluorescent lighting (if power remains), automated doors that still slide open. But the decay is everywhere: rust streaks, dead plant matter piled in corners, water damage spreading across ceilings. The machines maintain order in a morgue.

**Visual motifs:**
- Sharp geometric shapes contrasting with organic chaos of the post-Collapse world
- Lights that still function: LEDs, status indicators, targeting lasers
- Corrosion and wear on otherwise clean, functional surfaces
- Old-world writing: warning labels, serial numbers, manufacturing dates
- Movement is mechanical: precise, repetitive, slightly unsettling

**Color palette:** `#C0C0C0` aged steel, `#708090` slate grey, `#00BFFF` electric blue, `#FF8C00` warning orange, `#2F4F4F` dark panel, `#FF0000` error red

**Size reference (16×16 tile grid):**
- Security Drone: 1 tile (hovering, compact)
- Nanite Repair Swarm: 0.5 tiles (cloud form, multiple sprites in swarm)
- Cryo-Security Sentinel: 2 tiles (bulky, stationary-capable)
- Trench Maintenance Unit: 3 tiles (industrial, heavy)
- Plasma Guardian: 2 tiles (humanoid-ish, imposing)

**Function indicators visible on sprite:**
- Security Drone: weapon visible, optical sensor glows
- Nanite Swarm: cloud pattern, spark-like glints
- Cryo Sentinel: coolant pipes, frost vents, blue glow
- Maintenance Unit: industrial tools as weapons, treads or heavy legs
- Plasma Guardian: cannon barrel, heat shimmer around weapon port

---

### 1.7 Mutated Wildlife

**Nature:** Accidental byproducts of the telomerase cascade. Animals that absorbed Carapace DNA through the food chain, water table, or direct exposure. Each is a unique evolutionary experiment — some failed, some terrifyingly successful.

**Visual motifs:**
- Asymmetrical mutations: one normal leg, one chitinous claw
- Body parts that do not match: mammal fur over insect chitin
- Growths, tumors, and extraneous organs that twitch independently
- Eyes that do not track together, mouths with mixed dentition (mammal + crustacean)
- Color patterns that are trying to mimic something (threat display, camouflage) but getting it wrong

**Color palette:** Varies wildly. Dominant tones: `#8B4513` mutated brown, `#556B2F` sickly green, `#DAA520` aberrant gold, `#800020` blood rust

---

## 2. Location Visual Themes

### 2.1 Trench Nest (Great Carapace Dungeon)

**Tileset file:** `wfc_tilesets/trench_nest.toml`

**Atmosphere:** A flooded pre-Collapse facility that the Carapace have colonized. The waterline fluctuates with tides nobody on the surface tracks. Machines hum in the dark long after their operators died. The Carapace have reshaped the spaces — walls dissolved and regrown as chitin, floors carpeted with molted shell fragments.

| Tile | Visual Description |
|------|-------------------|
| WALL | Crumbling pre-Collapse concrete, overgrown with chitinous moss that pulses faintly. Cracks leak dark water. |
| WALL_RUIN | Collapsed sections where the Carapace dissolved through — fused chitin-slag edges, twisted rebar exposed. |
| FLOOR | Water-stained concrete tiles, partially submerged. Bioluminescent algae grow in the grout lines. |
| WATER | Dark, almost black water with an oil-slick iridescence from dissolved telomerase. Occasional bubbles rise from deep vents. |
| CORRIDOR | Narrow passages where the walls close in — conduits and pipes exposed, dripping condensation, the air thick and warm. |
| DOOR | Rusted security doors, some forced open, some sealed by calcified enzyme deposits. |
| TECH_TERMINAL | A flickering pre-Collapse console. Screen displays system diagnostics from a century ago. Keyboard keys are crusted with salt. |
| CARAPACE_NEST | A wall of fused chitin hexagons, each chamber containing a gestating Carapace spawn. The surface is tacky to the touch and warm. |

**Lighting:** Near-total darkness punctuated by:
- Console status LEDs (cyan, amber, red)
- Carapace bioluminescence (cyan-green, pulsing)
- Emergency strip lighting along corridor ceilings (flickering, half-dead)

**Color dominance (tile palette):** Deep blues (submerged concrete), muted purples (chitin), cyan highlights (bioluminescence), rust orange (degrading metal).

---

### 2.2 Sanguine Manse (Vampire Dungeon)

**Tileset file:** `wfc_tilesets/sanguine_manse.toml`

**Atmosphere:** A pre-Collapse luxury building — hotel, corporate headquarters, or data center — converted into a vampire noble's residence. The upper floors maintain the fiction of civilization. The basement levels abandoned that fiction generations ago.

| Tile | Visual Description |
|------|-------------------|
| WALL | Marble-veined stone, darkened by age and smoke. Tapestries hang at intervals — hunting scenes rendered in shades of red. |
| FLOOR | Polished stone tiles, originally white, now stained with a patina of use that no cleaning removes. Blood-traces in the grout. |
| HALL | Long galleries lined with portraits of ancestors — each one looks progressively less human as the series descends the timeline. |
| DOOR | Heavy oak banded with iron. Each door has a blood-seal — a handprint in dried sanguine wax — that must be matched to open. |
| THRONE | An ornate chair built from salvaged pre-Collapse materials: titanium framing, velvet upholstery, circuit-board inlays. The arms have blood-drain channels carved into the wood. |
| BLOOD_FOUNTAIN | A marble basin fed by a slow trickle from above. The liquid within is dark and viscous. The air around it is noticeably cooler. |
| COFFIN | Rosewood or black oak, lined with silk — or something that used to be silk. The interior is shaped to accommodate subdermal chitin ridges. |

**Lighting:** Candlelit. Chandeliers with real flame, wall sconces with tallow candles, braziers in the great hall. The light is warm, unsteady, and casts long shadows.

**Color dominance:** Rich reds, warm golds, pale marble, deep browns of aged wood. The palette of a royal court designed by predators.

---

### 2.3 Familiar Den (Cultist Dungeon)

**Tileset file:** `wfc_tilesets/familiar_den.toml`

**Atmosphere:** A repurposed underground space — storm drain, subway tunnel, natural cave system — claimed by the cult. The smell is the first thing: telomerase residue, unwashed bodies, stale blood, candle wax. The sound is the second: chanting, always chanting, from somewhere deeper in.

| Tile | Visual Description |
|------|-------------------|
| WALL | Rough-hewn stone and packed earth. Cult symbols painted in enzyme-mixed pigment glow faintly in the dark. |
| FLOOR | Hard-packed dirt scattered with bone fragments, wax drippings, and discarded injection needles. |
| CORRIDOR | Tunnels so narrow two people cannot pass. The walls are slick with condensation and human oils from constant traffic. |
| DOOR | Crude barriers — scrap wood, corrugated metal, anything that can be moved to block a passage. Function over form. |
| ALTAR | A slab of scavenged stone or metal, stained almost black. Behind it, the wall is painted with a fresco of the Enzyme's "blessings": bodies dissolving into light. |
| BRAZIER | A metal bowl on a tripod, burning low and slow. The fuel is rendered fat mixed with enzyme dregs — the flame burns purple-green. |
| SYMBOL | The cult's mark: concentric circles representing molting, painted or carved into every surface. Sometimes fresh — still wet. |
| TANK | A glass or metal vessel containing raw telomerase brew. The liquid inside is milky, iridescent, and occasionally moves on its own. |

**Lighting:** Firelight from braziers and candles. The enzyme symbols on walls add a faint, sourceless purple glow. Visibility is poor — the cult prefers it that way.

**Color dominance:** Sickly purples, dirty browns, rust reds, muted orange from flame. The palette of desperation.

---

### 2.4 Cryo Vault (Ancient Construct Dungeon)

**Tileset file:** `wfc_tilesets/cryo_vault.toml`

**Atmosphere:** A pre-Collapse cryogenic facility. The air is cold, dry, and still. The machines hum at a frequency that vibrates in the teeth. Rows of stasis pods stretch into darkness, their occupants long dead — or waiting.

| Tile | Visual Description |
|------|-------------------|
| WALL | White metal panels, now streaked with rust and frost. Warning stripes (yellow-black) at intervals. |
| FLOOR | Diamond-plate metal flooring, cold to the touch. Some panels are missing, revealing cable-choked service shafts below. |
| CORRIDOR | Long, straight, fluorescent-lit. The lights flicker in sequence, creating a moving shadow effect. Emergency systems mark exit routes that lead nowhere. |
| DOOR | Heavy pressure-sealed doors, some still functional. They hiss when they open. The labels next to them say things like "LABORATORY 7 — RESTRICTED" and "CRYO WARD B — AUTHORIZED PERSONNEL ONLY." |
| CRYO_CHAMBER | An upright pod with a frosted viewing window. Most show only dark shapes within. A few have their green "VIABLE" lights still glowing after a century. |
| CONSOLE | A data terminal, still powered. The screen displays system status, pod occupancy charts, and error logs from the year the world ended. |
| PIPE | Ceiling-height pipes wrapped in insulation, running the length of corridors. Some leak a fluid that has frozen into stalactite formations. |
| GENERATOR | A massive industrial generator, still running. The vibration is constant. Warning lights indicate it is operating at 23% of original capacity. |

**Lighting:** Fluorescent. Harsh, cold, unforgiving. Emergency red lighting in damaged sections. The cryo chambers emit a soft blue-green glow from their status indicators.

**Color dominance:** Cold greys, clinical whites, frost blue, emergency red, the sickly green of old display screens.

---

### 2.5 Human Settlement (Surface Town)

**Tileset file:** `wfc_tilesets/human_settlement.toml`

**Atmosphere:** A walled community held together by mutual need and shared fear. The streets are narrow, the buildings are short (two stories max — watchtowers excepted), and every window is shuttered by nightfall. But there is life here: cooking smoke, children's voices, a blacksmith's hammer.

| Tile | Visual Description |
|------|-------------------|
| WALL | The outer wall: salvaged stone and concrete, reinforced with scrap metal. Patched so many times it has no original surface. Guard posts at intervals. |
| FLOOR | Interior floors: salvaged wooden planks, uneven, worn smooth by generations of feet. |
| COBBLE | Main streets: uneven cobblestones salvaged from pre-Collapse roads, fitted loosely. Rain pools in the gaps. |
| DOOR | Heavy wooden doors, banded with scrap metal. Each house has its own lock. Most have a viewing slit at eye level. |
| MARKET_STALL | Canvass awnings over wooden tables. Goods displayed: preserved food, salvaged tools, cloth, candles, the occasional artifact. The traders watch everyone who passes. |
| SHRINE | A small altar to... something. The settlements vary. Some pray to the old world's saints. Some to abstract concepts like "The Dawn" or "The Return." A few pray to nothing — the shrine is just a place to sit in silence. |
| RUIN_PATCH | A section of the settlement that has not been rebuilt. Rubble, weeds, a collapsed roof. The children play here and the elders pretend not to see. |

**Lighting:** Daylight through windows, lantern light after dark. The settlement is brighter and warmer than any dungeon — deliberately so. Light is a defense against fear.

**Color dominance:** Warm browns, faded reds, muted greens, sky blue (if outdoors). The palette of handmade life.

---

### 2.6 Vampire City (Surface Settlement)

**Tileset file:** `wfc_tilesets/vampire_city.toml`

**Atmosphere:** A settlement ruled by the Sanguine Elite. The architecture is human-scale but the vibe is not. The streets are clean — too clean. The citizens walk with their heads down. The nobles' carriages are curtained. Everyone knows who feeds here and who is fed upon.

| Tile | Visual Description |
|------|-------------------|
| WALL | Dark stone, fitted precisely. The mortar has a reddish tint. Ivy grows in controlled patterns — trimmed, not wild. |
| FLOOR | Interior floors: polished dark wood or stone. Runners of crimson fabric in noble quarters. |
| ROAD | Cobblestone streets, well-maintained. The cobbles are dark grey river stones, fitted tightly. Gutters run along both sides. |
| DOOR | Arched wooden doors, ironwork depicting hunting scenes (predator and prey, always). Nobles' doors have house crests inlaid in brass. |
| THRONE | Throne of the city's ruler: dark wood, crimson upholstery, armrests carved into the shape of feeding serpents. Raised on a dais of three steps. |
| GATE | The city gate: iron, intimidating, decorated with the skulls of creatures (and perhaps people) who tried to force entry. |
| COFFIN | Above-ground crypts in the noble quarter. Each house has a mausoleum shaped like a miniature manor. The doors are sealed with blood-wax. |

**Lighting:** Gas lamps in the streets, lit at dusk and extinguished at dawn. The city is beautiful in low light. The shadows are long and purposeful.

**Color dominance:** Deep greys, muted reds, dark wood, pale stone, the occasional gold accent. Aristocratic restraint with an undercurrent of threat.

---

## 3. Creature Visual Descriptions (Entity Templates)

### 3.1 The Great Carapace

**Trench Lobster** (`entity_templates.toml` glyph `L`)
- **Silhouette:** Bipedal crustacean form, hunched forward. Two massive claws held at different heights. Six segmented legs, folded against the body when walking, extending for stability when attacking. Tail segments drag behind.
- **Head:** Two stalks with compound eyes, swiveling independently. Mandibles click audibly. Antennae sweep the air for chemical signatures.
- **Texture:** Smooth glossy chitin on the carapace, slightly translucent at the joints where soft tissue shows through as pale pink.
- **Color:** Deep crimson body fading to black at the limb tips. Eyes are jet black with a pinprick of cyan reflection.
- **Sprite notes:** Claws should be the focal point — one slightly larger than the other. Animation idea: claw opening/closing idle. Eye stalks should twitch.

**Abyssal Dreadclaw** (`entity_templates.toml` glyph `D`)
- **Silhouette:** Larger, wider, lower to the ground than Trench Lobster. Claws are disproportionately massive — one designed for crushing, one for gripping. Carapace is thicker, almost armored-vehicle proportions.
- **Head:** Recessed into the body — barely visible between the shoulder plates. Small eyes, multiple pairs, arranged along the brow ridge. Bioluminescent lure organs on the underside of the carapace, flashing in patterns.
- **Texture:** Barnacle-encrusted chitin. The carapace is a palimpsest of past wounds healed over.
- **Color:** Dark purple-black body. Bioluminescence in cyan and occasionally red. The claws have a slightly iridescent sheen.
- **Sprite notes:** This is a boss-tier creature. Needs to convey weight. Should occupy 3 tiles. Animation idea: slow, deliberate claw movements followed by explosive speed.

**Spitter Crab** (`entity_templates.toml` glyph `C`)
- **Silhouette:** Quadrupedal crab form with an enlarged abdomen housing chemical glands. Two small pincers for manipulation, one large modified limb ending in a spray nozzle.
- **Head:** Eyes on long stalks that can retract into the carapace. The spray gland is visible on the back — a translucent sac that pulses when full.
- **Texture:** Bumpy chitin, heat-cracked around the spray gland. The gland pulses with internal pressure.
- **Color:** Amber-orange body with dark brown highlights. The chemical gland is visible as a yellow-green sac. When full, it glows from within.
- **Sprite notes:** The gland should be the visual focus. When the creature is about to attack, the gland should appear swollen and brighter. Add acid-drip sprite effect on attack.

**Molting Broodmother** (`entity_templates.toml` glyph `M`)
- **Silhouette:** Massive, stationary or slow-moving. Abdomen is hugely distended — translucent enough to see the shapes of gestating spawn within. The front half is armored, the back half is soft and vulnerable.
- **Head:** Small in proportion to body. Mandibles are delicate tools, not weapons — she is not the fighter. She is the factory.
- **Texture:** The forward carapace is thick, cracked, and ancient — hundreds of molts layered. The abdomen is smooth, stretched thin, veined.
- **Color:** Muted purple-brown body. The abdomen glows with internal warmth — a soft orange-pink like embers. Eyes are pale and almost blind.
- **Sprite notes:** This is a "don't get close" creature. Should convey vulnerability and horror. Surround with smaller spawn sprites. Animation idea: slow abdominal pulse, spawn occasionally emerging.

**Pressure Crawler** (`entity_templates.toml` glyph `p`)
- **Silhouette:** Dog-sized, many-legged. Six to eight jointed legs splayed outward like a spider-crab hybrid. Low to the ground. Tail is a thin sensory filament that drags behind.
- **Head:** Enlarged mandible cluster — the face is mostly mouth. Small eyes on short stalks. Sensory pits along the jawline detect vibrations in water.
- **Texture:** Thin, flexible chitin. Joints are vulnerable and unarmored — a survival tradeoff for speed.
- **Color:** Mottled grey-purple, matches deep-trench sediment. Underside is pale cream.
- **Sprite notes:** Pack animal — sprites should be designed to look good in groups of 3-6. Variation in size within the pack. Fast, skittering movement.

**Abyssal Siege Crab** (`entity_templates.toml` glyph `S`)
- **Silhouette:** A building with legs. The carapace is a dome of mineral-encrusted chitin so thick that it has become geological. Legs are columns — four massive, tree-trunk limbs that plant with each step.
- **Head:** Microscopic in proportion. Two tiny eye stalks that are almost vestigial. It does not need to see well — it crushes everything in its path.
- **Texture:** The carapace is not just chitin — it has fused with seabed minerals, forming a rock-like surface. Coral, barnacles, and tube worms grow on the shell of the oldest individuals.
- **Color:** Stone grey with purple-chitin undertones. The living tissue (leg joints, underbelly) is a deep bruise purple. Eyes are pale blue.
- **Sprite notes:** Ultra-boss. 4+ tiles. Should inspire awe and "do not engage." Movement should be terrifyingly slow but inexorable.

**Lurejaw Angler** (`entity_templates.toml` glyph `A`)
- **Silhouette:** Eel-like body, serpentine. The front half is all jaw — a mouth that unhinges to swallow prey larger than itself. A bioluminescent lure extends from the forehead on a flexible stalk.
- **Head:** The lure is the key feature: a glowing orb that pulses in rhythmic patterns designed to mesmerize. Below it, a mouth lined with translucent needle-teeth that fold inward (prey can only go in, not out).
- **Texture:** Smooth, almost slimy chitin. The body is flexible, allowing it to coil in confined spaces.
- **Color:** Deep abyss black with cyan bioluminescence. The lure is bright cyan fading to white at the center. Inside the mouth: pale pink tissue, dark throat.
- **Sprite notes:** The lure needs an animation cycle — pulsing glow, occasional swing. The mouth should be visible even when closed (the teeth protrude slightly). Designed to ambush from WATER tiles.

---

### 3.2 The Sanguine Elite

**Vampire Noble** (`entity_templates.toml` glyph `V`)
- **Silhouette:** Human-proportioned but wrong — shoulders slightly too broad, neck slightly too long, fingers slightly too many joints. Wears formal attire that conceals the worst of the hybrid tells.
- **Head:** Aristocratic features, pale skin, high cheekbones. Eyes are the giveaway: pupils that contract to pinpricks in light, expand to black pools in dark. Canines are visibly elongated but often filed down to pass as human.
- **Texture:** Skin is smooth and cold-looking. Beneath it, at the jawline and wrists, the faint outline of subdermal chitin plates is visible as a darker shadow under the skin.
- **Color:** Pale flesh, dark hair (often black or deep red), crimson clothing. Eyes are pale grey or red. The chitin plates have a faint purple tint visible only in certain light.
- **Sprite notes:** Distinction from humans should be subtle but present. The cape/collar is an important silhouette differentiator. When feeding or enraged, the chitin plates should be shown emerging (sprite swap or overlay).

**Vampire Enforcer** (`entity_templates.toml` glyph `E`)
- **Silhouette:** Bulked humanoid — the hybrid gene expression was pushed toward physical mass rather than social grace. Shoulders broad, neck thick, arms slightly too long. Subdermal plates are visible as ridges pushing against the skin, especially across the shoulders and forearms.
- **Head:** Heavy brow, flat nose, small eyes. Expression is perpetually aggressive. Chitin growth along the jawline looks like a bone beard.
- **Texture:** Skin is thick and scarred. The chitin plates that push through at the knuckles, elbows, and spine are dark and rough.
- **Color:** Dark grey-purple skin (higher chitin density than nobles), wearing functional dark leather with metal reinforcement. Eyes are solid red.
- **Sprite notes:** Bigger sprite than nobles. Weapons should look heavy and brutal. No capes — armor and function.

**Vampire Courtesan** (`entity_templates.toml` glyph `C`)
- **Silhouette:** Deliberately human-convincing. Slender, graceful, often shown in motion — a dancer's posture. The clothes are designed to reveal and conceal simultaneously: bare shoulders, high collars, long gloves.
- **Head:** Beautiful by human standards. The only tell is the eyes — they track too precisely, and the smile does not quite reach them. Hair is always immaculate.
- **Texture:** Skin is perfect — too perfect. No pores, no scars, no imperfections. This is the most recent and expensive genetic maintenance the Elite can buy.
- **Color:** Pale skin, dark or brightly dyed hair, elaborate clothing in deep reds and blacks. Gold jewelry accents.
- **Sprite notes:** Most human-looking of the vampire types. The threat should come through in the pose — coiled grace, a hand resting near a concealed weapon. Chromatophoric skin shifts could be shown as subtle color ripples on the sprite.

**Vampire Inquisitor** (`entity_templates.toml` glyph `I`)
- **Silhouette:** Lean, wired, predatory. Wears a cassock or long coat over armor. Ritual scars are visible on the face and hands — deliberate cuts that healed with chitin inlay.
- **Head:** Clean-shaven or severely cropped hair. Scars form patterns — house sigils, doctrinal statements. Eyes are the coldest of all vampire types: they have seen every kind of heresy and are looking for the next.
- **Texture:** The scars are raised and slightly iridescent — chitin tissue that grew over the wounds. Hands are calloused from weapon practice.
- **Color:** Black cassock with blood-red accents. Pale grey skin. The ritual scars are a faint purple-silver.
- **Sprite notes:** Intimidating posture. The barbed scourge should be the most prominent visual element. The inquisitor's cassock should have a high collar and a severe silhouette.

**Blood Hound** (`entity_templates.toml` glyph `h`)
- **Silhouette:** Canine frame distorted by crustacean DNA. Longer body, more legs (six — the front two have become arm-like), chitin ridges along the spine that raise and lower with mood. Tail is a bare, whip-like appendage.
- **Head:** Wolf-like but the jaws open wider, and the teeth are layered — rows of chitin needles that replace themselves when lost. Nostrils are large, constantly sampling the air. No visible eyes — it does not need them.
- **Texture:** Patches of fur (patchy, molting) over chitin plating. The spine ridges are semi-translucent.
- **Color:** Dark red-brown fur fading to black. Chitin plates are a darker, glossier black. The mouth interior and ridged spine are visible pink when the mouth is open.
- **Sprite notes:** Beast. Should move in a low, stalking posture. The spine ridges rising is the aggression tell. Ideal for pack formations (2-3 per sprite group).

---

### 3.3 The Familiars

**Familiar Zealot** (`entity_templates.toml` glyph `f`)
- **Silhouette:** Human, but wrong. The posture is either hunched (in withdrawal) or unnaturally erect (when dosed). Movement is erratic — sudden bursts of energy followed by pauses. Clothing is mismatched cult robes.
- **Head:** Eyes are the first thing you notice: hugely dilated, unfocused, or locked on with disturbing intensity. Track marks visible on neck and temples. Teeth are starting to sharpen.
- **Texture:** Skin is sallow, clammy. Enzyme residue stains the fingers and around the mouth.
- **Color:** Pale grey skin, dark circles under eyes. Cult robes in muted purples and browns. Equipment is a mix of salvaged tools and ritual items.
- **Sprite notes:** The most important visual is the eyes — dilated pupils should be visible. Weapon visible but held poorly (they are not trained fighters). Multiple variants for different dosage states.

**Familiar Acolyte** (`entity_templates.toml` glyph `a`)
- **Silhouette:** Calmer, more centered than the Zealot. Stands straight. Wears better-maintained robes with cult rank markings — patterns of concentric circles in purple thread. Carries a ceremonial scepter or staff.
- **Head:** Serene expression, but the eyes betray the fanaticism. Older, more established in the cult. Tattoos cover the face — cult script, devotional markings.
- **Texture:** Skin is leathery from years of enzyme exposure. Fingertips are calloused from administering communion.
- **Color:** Deeper, richer purples in robes than the Zealots. Gold or brass ritual items. The tattoos are dark indigo against pale skin.
- **Sprite notes:** Should look like a priest — authoritative, calm, dangerous in a different way. The scepter/staff should have a visible enzyme reservoir (small vial or bulb at the top).

**Telomerase Ghoul** (`entity_templates.toml` glyph `G`)
- **Silhouette:** Shambling, barely human. The body is collapsing — joints swollen, spine curved, one shoulder higher than the other. Clothing hangs off a frame that is simultaneously wasting away and bulking in wrong places.
- **Head:** The face is a mask stretched over something else. Cheekbones prominent. The jaw unhinges slightly when it opens its mouth. Hair has fallen out in patches. Teeth are visibly chitinous and sharp.
- **Texture:** Skin is waxy, discolored, pulled tight over bone and the chitin plates that are beginning to emerge. Some ghouls have chitin spikes that have pierced through the skin at the elbows, knees, and spine.
- **Color:** Grey-white skin with purple undertones. The emerging chitin is a dark bruised purple. Eyes are milky - blind or nearly so.
- **Sprite notes:** The horror of the ghoul is that it was recently human. Keep recognizable human elements (a torn shirt, a wedding ring on a swollen finger) to sell the tragedy. Shambling animation, dragging one foot.

**Telomerase Junkie** (`entity_templates.toml` glyph `j`)
- **Silhouette:** Erratic, twitching, never still. The body is wired on unstable telomerase — muscles contract and release without conscious control. Clothing is disheveled, half-buttoned, torn.
- **Head:** Pinprick pupils, wild eyes, vein networks visible across the face and scalp glowing faintly purple with enzyme in the bloodstream. Grinning or grimacing — there is no in-between.
- **Texture:** Translucent skin at the temples and wrists where the enzyme glow is most visible. The veins themselves look like they are moving (actually the enzyme crystallizing and dissolving in real time).
- **Color:** Pale, almost translucent skin. The enzyme glow is a sickly purple-pink visible through the skin at pressure points. Dark circles, cracked lips, blood-flecked saliva at the corners of the mouth.
- **Sprite notes:** The most volatile-looking familiar. Animation should include tremors, head-jerking, and shaky hands. The glowing veins are the key visual — they should pulse.

---

### 3.4 Free Humanity

**Remnant Hunter** (`entity_templates.toml` glyph `H`)
- **Silhouette:** Lean, practical, loaded with gear. Wears a long coat or duster over salvaged armor. Weapons visible and accessible — belt knife, rifle or crossbow on the back, a hand-axe at the hip.
- **Head:** Weather-beaten, scarred. Eyes scan constantly — this is someone alive because they see threats before they arrive. Short hair or tied back. Practical.
- **Texture:** Leather and cloth that has been repaired many times. Metal parts are oiled but scratched.
- **Color:** Earth tones — browns, greys, faded greens. A splash of color from a scarf or bandana (personality choice). Skin is tanned and lined.
- **Sprite notes:** The most relatable human type for the player. Posture should be competent but not aggressive — a professional at rest. The gear loadout should be visibly useful (not decorative).

**Settlement Guard** (`entity_templates.toml` glyph `G`)
- **Silhouette:** Armored, standardized (within settlement resources). Wears a helm or cap with a visor, a reinforced jerkin or breastplate, and carries a weapon — spear, baton, or crossbow. Shield optional.
- **Head:** Hidden by helmet or cap. The visible lower face is neutral, professional. Eyes watch the gate.
- **Texture:** Leather and metal, well-maintained but not new. The settlement's emblem (if any) is painted or stitched onto the uniform.
- **Color:** Settlement colors — typically blues, greys, or browns. Metal is a muted silver with rust patina. 
- **Sprite notes:** Guard is about uniformity and readiness. Posture: standing watch, weapon at rest but hand on it. Different settlement styles could vary the uniform color.

**Artifact Scavenger** (`entity_templates.toml` glyph `S`)
- **Silhouette:** Carries more gear than seems reasonable. A pack overflowing with salvaged items, tools hanging from every belt loop, a scanner or probe in one hand. Posture is slightly hunched from the weight but eager — always looking at the next find.
- **Head:** Goggles pushed up on the forehead or worn over the eyes. Expression is curious, obsessive, delighted by discovery. Maybe a magnifying loupe on a headband.
- **Texture:** A mess of utility. Pockets on pockets. Straps and buckles everywhere. The gear is well-cared-for but eclectic.
- **Color:** Mix of whatever colors the salvaged gear came in — no coordination. The unifying element is the dirt and wear of the ruins.
- **Sprite notes:** The scavenger should look like they are on the verge of pulling out something interesting. Hands full, bag bulging. The tech-scanner tool should have a visible display or light.

**Nomad Trader** (`entity_templates.toml` glyph `T`)
- **Silhouette:** Travel-worn but prosperous. Carries a pack but moves easily under it — experienced traveler. Wears a coat with many pockets, sturdy boots, a hat against the sun.
- **Head:** Friendly, weathered face. Eyes that assess value in everything they look at. Well-fed (compared to settlement-dwellers).
- **Texture:** Quality fabrics, well-maintained. Gear that has been chosen carefully for utility. A walking stick that doubles as a hidden blade.
- **Color:** Practical travel colors — browns and greys — with touches of color in the wares visible through open pack flaps.
- **Sprite notes:** Should look approachable but capable. The walking stick is the key prop. Visible trade goods dangling from the pack.

**Settlement Elder** (`npc_personalities.toml` only)
- **Silhouette:** Older, frailer, but not weak. Stands straight through will. Uses a walking stick for stability, not necessity. Simple, clean clothing — no armor.
- **Head:** Deeply lined face, white or grey hair, eyes that have seen loss and kept going. Speaks slowly, gestures sparingly.
- **Texture:** Worn, clean clothing. Hands that worked hard for decades and now rest.
- **Color:** Subdued natural tones. The elder has nothing to prove with their clothes.
- **Sprite notes:** Dignity in age. Should not look decrepit — should look like someone who survived everything the world threw at them.

**Familiar Defector** (`npc_personalities.toml` only)
- **Silhouette:** Trying to look human again. Wears settlement clothes (acquired or given) that do not quite fit. Posture is wary — expecting attack or rejection. Hands visible at all times to show they are unarmed.
- **Head:** Cult tattoos partially obscured with scarf or high collar. Withdrawal gives a haunted, sleepless look. Veins at the temples still show faint purple traces.
- **Texture:** Clean clothes on a body that is not used to being clean. Healing track marks on the neck. Nails bitten to the quick.
- **Color:** Trying to blend into settlement colors but the pallor of addiction marks them. Grey-pale skin, dark circles, faded tattoos.
- **Sprite notes:** The tragedy is visible. They are between worlds — not cult anymore, not truly accepted by humanity. Their posture should convey the effort of staying clean.

---

### 3.5 Mutated Wildlife

**Chitin-Rat Swarm** (`entity_templates.toml` glyph `r`)
- **Silhouette:** Not a single sprite — the swarm is a moving carpet of individual rats, each the size of a cat. The mass flows around obstacles, chitin plates clattering.
- **Individual rat:** Rat shape distorted by chitin plates that make it look armored, almost beetle-like. Eyes are multiple and black. Tail is bare, segmented, and whip-like.
- **Texture:** Fur patches over chitin sections. The chitin is smooth and glossy, the fur is matted.
- **Color:** Brown-grey fur, black chitin. The swarm has a rippling, shifting color as individuals move over each other.
- **Sprite notes:** Swarm needs to be a multi-tile effect. A roiling mass with individual rats visible at the edges. Movement animation is the key — flowing, surging, splitting around obstacles.

**Bio-Electric Eel Hound** (`entity_templates.toml` glyph `e`)
- **Silhouette:** Canine with an elongated, eel-like body. Legs are shorter, body is longer, neck is sinuous. Arcs of electricity dance between its ears and along its spine.
- **Head:** Dog-like but the mouth opens wider and the tongue is forked. Eyes are pale blue. Visible electrical organs along the jawline.
- **Texture:** Fur is damp-looking, slick. Patches of bare skin where electrical discharge has burned away the hair. The electrical organs are visible as blue-green patches under the skin.
- **Color:** Dark blue-black fur with bright cyan electrical arcs. The arcs are brighter when the creature is agitated.
- **Sprite notes:** Electrical arcs should be the primary animation — flickering, intensifying before an attack. The creature should look like it hurts to touch.

**Chromatophoric Stalker** (`entity_templates.toml` glyph `s`)
- **Silhouette:** Feline frame with cephalopod tissue grafts. The body is fluid, sinuous, and constantly in motion — not walking but flowing. The skin ripples with color even when "invisible."
- **Head:** Cuttlefish-shaped head with two large, complex eyes. Tentacle-like tendrils around the mouth. No visible ears or nose.
- **Texture:** Skin is smooth, wet-looking, and covered in chromatophores that shift color continuously. When camouflaged, it is not invisible — it is a shimmering distortion like heat haze.
- **Color:** Normally a mottled purple-green. The camouflage shift cycles through browns, greys, and greens. Threat display is bright red with eye-spot patterns.
- **Sprite notes:** The most visually complex creature to render. Recommend multiple sprite variants:
  - Visible form (purple-green)
  - Camouflaged form (transparency effect or partial outline)
  - Threat display (bright red with pattern)
  - Transition frames between states

**Bombardier Hog** (`entity_templates.toml` glyph `b`)
- **Silhouette:** Heavy, boar-shaped, but the back half is distorted by the chemical gland cluster. Two large glands protrude from the shoulders like humps, ending in nozzle-like orifices that can swivel independently.
- **Head:** Wild boar head, tusks, small angry eyes. The boar is visible but the mutation has given it an unsettling intelligence in the gaze.
- **Texture:** Bristly fur on the front half, leathery gland tissue on the back. The gland nozzles are chitinous. Chemical residue stains the fur around the gland openings.
- **Color:** Dark brown-black fur. The glands are a lighter, mottled orange-brown. The chemical spray vent shows as a darker opening.
- **Sprite notes:** The glands are the focal point. They should visibly swell before a spray attack. Steam or chemical mist rising from the gland openings adds atmosphere. The hog charges head-down — animation should convey mass and momentum.

**Chimeric Brute** (`entity_templates.toml` glyph `B`)
- **Silhouette:** Human frame expanded and distorted by Carapace muscle grafts. One arm is normal, the other is a massive chitinous club. The torso is asymmetrical — bulked on the grafted side. Head is human but the jaw has been displaced sideways.
- **Head:** Human face twisted by pain and confusion. One eye is human, one is a compound eye. The mouth is a rictus of mixed human and crustacean features.
- **Texture:** Patches of human skin stretch over bulging crustacean muscle. Chitin plates burst through the skin at the shoulders and hips. The graft line is visible as a ridge of scar tissue.
- **Color:** Pale human skin, dark purple chitin on the grafted regions. The compound eye is jet black. Blood from the forced growth stains the chitin seams.
- **Sprite notes:** The tragedy of the brute is visible humanity. Keep human elements dominant but wrong. The asymmetry of the body should be the key visual — it tells the story of the failed splicing.

**Mantis Slicer** (`entity_templates.toml` glyph `m`)
- **Silhouette:** Humanoid frame with mantis-like folded forelimbs that unfold to twice the arm length. The body is held in a low, predatory crouch. Head can rotate nearly 360 degrees.
- **Head:** Triangular, with large compound eyes and vestigial antennae. The mouth is small — this creature is a precision killer, not a biter.
- **Texture:** Smooth, almost polished chitin covering most of the body. The forelimbs are blade-sharp along the inner edge.
- **Color:** Bright verdant green with darker green stripes on the forelimbs. Eyes are large, black, and depthlessly reflective.
- **Sprite notes:** Speed embodied. The unfolded forelimbs should be the visual anchor — they extend past the rest of the body. Animation: the slow, deliberate sway of the mantis before the lightning strike.

**Venom Stinger** (`entity_templates.toml` glyph `v`)
- **Silhouette:** Humanoid torso with a scorpion-like lower body — four to six legs, a segmented tail that arches over the body, ending in a curved stinger dripping venom. Two pincer arms in front.
- **Head:** Human face, but the eyes are multiple (four, arranged vertically on each side of the head). The mouth has a proboscis that extends when feeding.
- **Texture:** Smooth chitin on the lower body, human skin on the torso — the transition line is visible at the waist. The tail segments are glossy.
- **Color:** Dark amber body, lighter underside. The stinger tip is black and visibly wet. Human torso is pale with purple veins.
- **Sprite notes:** Centaur-like composition. The tail should be a focus — it moves independently, threatening. The stinger drip is a nice detail. Animation: tail weaves, scorpion legs skitter.

**Spore-Spliced Shambler** (`entity_templates.toml` glyph `z`)
- **Silhouette:** Human shape obscured by fungal overgrowth. The body is bloated in places, withered in others. Spore caps push through the skin at irregular intervals. Posture is shambling, directionless — the human brain is mostly gone.
- **Head:** Face is partially obscured by fungal growth. One eye may be visible, staring without understanding. The mouth hangs open — breath visible as a spore cloud in cold air.
- **Texture:** Fungal bodies are fleshy, veined, and damp. Some have the texture of mushrooms (smooth cap, gills visible), others are more like slime molds.
- **Color:** Pale grey-green fungal tissue, dark brown or black spore caps, yellow-white mycelium networks visible under translucent skin. The overall impression is decay in progress.
- **Sprite notes:** Walking environmental hazard. The spore caps should look ready to burst. Mention animation: occasional spore puff released, body swaying slightly. Group them for maximum horror effect.

**Carrion Flapper** (`entity_templates.toml` glyph `f`)
- **Silhouette:** Vulture shape grafted with crustacean features. Leathery wings, a long naked neck, and a chitinous beak that is serrated on the inside. The feet are clawed and adapted to perch on ruins.
- **Head:** Bare, wrinkled, vulture-like. Eyes are large and adapted for long-distance carcass spotting. The beak is chitin, not bone — it grows continuously and is sharpened by use.
- **Texture:** Feathers are sparse and wiry, mixed with patches of chitinous scutes on the breast and wing joints.
- **Color:** Dirty black-grey feathers, pale naked head and neck, dark grey chitin beak. The feet are a lighter grey.
- **Sprite notes:** Flyer — should be shown in flight or perched on something high. In flight, the silhouette is the key. On the ground, it is ungainly, hopping. Packs of 3-5 circling above a battlefield.

---

### 3.6 Ancient Constructs

**Security Drone** (`entity_templates.toml` glyph `D`)
- **Silhouette:** Floating disk or sphere, 1 tile in size. No visible propulsion — it just hovers. A single optical sensor (camera lens) on the front. Weapon port on the underside.
- **Details:** The casing is scuffed and faded from a century of operation. Warning labels are still legible. A serial number is stenciled on the side. The optical sensor glows red when hostile, blue when neutral.
- **Color:** Faded white or grey casing, yellowed with age. Red status light. Warning stripes (yellow-black) around the weapon port.
- **Sprite notes:** The simplest construct sprite. Clean shapes, clear status indicator. The hover should be conveyed with a slight shadow offset and maybe a subtle glow underneath. 

**Trench Maintenance Unit** (`entity_templates.toml` glyph `U`)
- **Silhouette:** Industrial bulk on treads or heavy legs. The body is a rectangular chassis covered in tool mounts. Two or four manipulator arms, currently armed with improvised weapons (welding torch, circular saw, hydraulic clamp).
- **Details:** Painted industrial yellow originally, now faded and rusted. Scorch marks and chemical stains. A caution sign on the side reads "AUTOMATED — DO NOT APPROACH." The tool arms move continuously, even when idle — recalibrating, testing range of motion.
- **Color:** Faded safety yellow, rust orange, dark grey tool heads. The optical sensors glow a steady amber.
- **Sprite notes:** Heavy, slow, relentless. The movement should convey weight — each tread step is a thud. The tool arms should be animated, slowly pivoting and adjusting.

**Stasis Pod Guardian** (`entity_templates.toml` glyph `S`)
- **Silhouette:** Humanoid-ish, built on a security chassis. Rounded torso, sensor head, articulated legs. One arm has been replaced with a stasis field emitter. The other retains a manipulator hand.
- **Details:** Designed to look non-threatening to cryo-pod occupants — rounded edges, friendly color scheme (originally). The "face" is a screen that displays a simplified emoticon expression. After a century alone, the screen flickers between the assigned friendly face and error messages.
- **Color:** White and light blue originally, now scuffed and greyed. The stasis emitter glows a soft blue-white. The face screen is pale green.
- **Sprite notes:** The most humanoid construct. The friendly face that flickers into error state tells the story. Animation: head tilts, face expression changes, stasis field charging glow.

**Cryo-Security Sentinel** (`entity_templates.toml` glyph `C`)
- **Silhouette:** Bulky, stationary or slow-moving. Built around a cryogenic coolant system — pipes and tanks visible on the exterior. Two weapon barrels, side-mounted. A sensor array on top.
- **Details:** The coolant system is the key visual. Frost builds on the exterior pipes during operation. Vents release plumes of cold vapor. The weapon barrels glow faintly blue as they charge. A status panel displays: "COOLANT: 67% | WEAPONS: ARMED | TARGET: ACQUIRED."
- **Color:** Cold grey metal, ice blue highlights, frost white on the cooling elements. The status panel is amber text on black.
- **Sprite notes:** This is a turret with legs. The coolant system should be visibly active — frost spreading along pipes, vapor venting. Animation: barrel tracking, coolant pump sound implied by visual rhythm.

**Plasma Guardian** (`entity_templates.toml` glyph `P`)
- **Silhouette:** Heavy combat chassis. The defining feature is the plasma cannon — a large barrel assembly built into the torso or mounted on the shoulder. The body is armored in reactive plates that shift slightly with internal systems.
- **Details:** The plasma cannon requires visible charging: vents open, a whine builds (implied visually by glow intensity in the barrel), and the shot is a bright orange-white bolt. After firing, the barrel glows with residual heat. Warning labels everywhere: "THERMAL HAZARD. KEEP CLEAR."
- **Color:** Dark military green or grey, orange-white plasma glow, heat red on the barrel after firing. The status lights are warning orange.
- **Sprite notes:** The most dangerous construct. The plasma cannon charge cycle should be the core animation — vents opening, glow building, recoil on fire. Post-fire heat shimmer.

---

## 4. Item Design Briefs

### 4.1 Weapons

| Category | Base | Visual Keywords | Size (px) |
|----------|------|-----------------|-----------|
| Crushing Claw | Carapace chitin | Jagged, organic curve, rough surface, bioluminescent traces | 16×16 |
| Acid Spray Gland | Carapace organ | Translucent sac, yellow-green fluid, nozzle tip, veined | 16×8 |
| Rite-Blade | Vampire ceremonial | Curved blade, blood channel, gold hilt, darkened metal | 16×16 |
| Enforcer Maul | Heavy metal | Massive head, wrapped handle, spikes or flanges, crude | 16×24 |
| Sacrificial Dagger | Familiar ritual | Serrated edge, stained handle, cult symbols etched | 16×8 |
| Scrap Rifle | Salvaged pre-Collapse | Scoped, improvised stock, tape-wrapped grip, exposed mechanism | 24×8 |
| Guard Baton | Manufactured | Cylindrical, shock bands visible, rubberized grip | 16×8 |
| Laser Emitter | Construct tech | Metallic barrel, lens array, charge indicator light | 16×8 |

**Design principles:**
- Show the material: chitin weapons are organic and irregular, metal weapons are salvaged and worn
- Show the maker: human weapons are improvised, vampire weapons are ornate, Carapace weapons are grown
- Icons should be readable at 16×16: strong silhouette, minimal internal detail

### 4.2 Armor

| Category | Visual Keywords |
|----------|----------------|
| Carapace Plating | Chitin segments, organic curve, natural sheen |
| Subdermal Chitin Weave | Flesh-colored with darker chitin ridges, barely visible |
| Reinforced Jerkin | Leather patches, metal rivets, stitched repairs |
| Traveling Coat | Heavy fabric, many pockets, worn collar, belt |
| Insulated Chassis | Metal panels, warning labels, cable connections |

### 4.3 Consumables & Artifacts

| Item | Visual Description | Icon Notes |
|------|-------------------|------------|
| Telomerase Vial (T-Fluid) | Small glass vial, purple-pink liquid that glows faintly, rubber stopper | Glow effect important |
| Sanguis Canister | Metal canister, pressure valve, red-brown residue around the nozzle | Industrial medical design |
| Cell Battery | Rectangular, yellow-black warning stripes, terminal contacts at one end | Pre-Collapse industrial |
| Medical Bio-Scanner | Handheld device, screen, probe end, rubberized grip | Tech aesthetic |
| Cryo-Pod Keycard | White plastic card, magnetic strip, faded label "LEVEL 4 ACCESS" | Minimalist |

---

## 5. UI Design Brief

### 5.1 Panel Backgrounds
- **Sidebar/Info panels:** Semi-transparent dark overlay with a border of thin chitin-green lines. The corners have subtle organic motifs — a curve that suggests a claw or mandible.
- **Dialogue panels:** Cleaner, lighter. Parchment-like background with a border that suggests old-world paper forms. Subtle blood-spatter pattern at the very edge.

### 5.2 HUD Elements
- **Health bar:** A segmented bar (chitin segments). Color shifts from green (healthy) through amber (wounded) to red (critical). When damaged, segments crack and fall away.
- **Enzyme/Telomerase bar:** Similar segmentation but in purple-pink tones. A pulsing glow when the player has active telomerase in their system.
- **Faction reputation indicators:** Small icons per faction — a claw (Great Carapace), a blood drop (Sanguine Elite), a cult symbol (Familiars), a wall (Free Humanity), a gear (Constructs).

### 5.3 Cursor & Selection
- **Default cursor:** A crosshair or targeting reticle with subtle organic curves
- **Interact cursor:** A hand with elongated fingers (suggesting hybrid influence)
- **Attack cursor:** A claw or blade silhouette
- **Selection highlight:** A pulsing amber outline on the selected entity or tile

---

## 6. Biome Visual Themes

### Surface Biomes

| Biome | Ground | Flora | Lighting | Color Dominance | Key Atmosphere |
|-------|--------|-------|----------|-----------------|----------------|
| Grassland | Dry grass, cracked earth | Sparse shrubs, isolated trees | Bright, harsh | `#8FBC8F` `#D2B48C` | Exposed, dangerous, long sightlines |
| Temperate Forest | Dark soil, leaf litter | Dense trees, thick undergrowth | Dappled, dim | `#556B2F` `#2F4F2F` | Concealment, rustling, unseen watchers |
| Swamp | Mud, standing water, roots | Dead trees, reeds, fungus | Green-tinted gloom | `#4A5D23` `#3B3B1A` | Stagnation, decay, things moving under the surface |
| Tundra | Permafrost, scattered snow | Moss, stunted shrubs | Grey, flat, endless | `#B0C4DE` `#D3D3D3` | Exposure, cold, the horizon is a threat |
| Desert | Sand, rock outcroppings | Cacti, dry brush | Blazing, blinding | `#D2B48C` `#C4A882` | Thirst, heat shimmer, ruins half-buried in sand |
| Ice Sheet | Pack ice, pressure ridges | None | White, reflective, painful | `#E0FFFF` `#B0E0E6` | Isolation, the cold seeps through everything |
| Mountain | Stone, scree, thin soil | Stunted pines, alpine flowers | Sharp, shadowed | `#808080` `#A0522D` | Verticality, exposure, thin air |
| Settlement | Cobblestone, packed earth | Cultivated gardens, fruit trees | Warm, lantern-lit | `#8B7355` `#D2B48C` | Safety, community, the walls are never enough |

### Underground Biomes

| Biome | Ground | Features | Lighting | Color Dominance | Key Atmosphere |
|-------|--------|----------|----------|-----------------|----------------|
| Dungeon (generic) | Flagstone, packed earth | Walls, doors, chambers | Torchlight, variable | `#696969` `#8B4513` | Enclosure, the unknown in the next room |
| Cave | Rock, sand, pools | Stalactites, bioluminescent fungi | Near-total dark | `#2F2F2F` `#4A4A4A` | The weight of the earth above |
| Trench | Submerged concrete, sediment | Flooded chambers, chitin growths | Bioluminescent, dim | `#1A1A2E` `#2B0F3A` | Pressure, the deep, something vast nearby |
| Ancient Vault | Metal flooring, panels | Cryo pods, terminals, pipes | Fluorescent, harsh | `#C0C0C0` `#4682B4` | Sterility, time stopped, the machines never stopped |

---

## 7. Lore Consistency Notes

### Existing lore references that inform visual design:
1. **The Great Carapace's "no language"** (lore_fragments.toml:carapace_origin) → Visual communication through bioluminescence, pheromones, and chitin arrangement. Sprites should imply communication modes the player cannot decode.
2. **The Hybrid Condition** (lore_fragments.toml:vampire_biology) → Sanguine Elite sprites should show controlled instability — chitin plates barely contained under skin, the tension between human appearance and inhuman reality.
3. **Telomerase crystallization** (lore_fragments.toml:telomerase_crystallization) → Visual motif of amber-like crystals forming on surfaces, in wounds, around enzyme sources.
4. **The Red Flux** (lore_fragments.toml:the_red_flux) → Seasonal visual change: rust-colored sky, chitin growth acceleration, bioluminescence becoming more prominent.
5. **The Spire** (lore_fragments.toml:the_spire) → Carapace architecture reference: fused chitin that looks grown, not built. Organic curves, no straight lines.
6. **Chitin-rat ecology** (lore_fragments.toml:chitin_rat_ecology) → All post-Collapse creatures should show some degree of chitin incorporation, even subtle.

### What NOT to include (fantasy remnants — per CAR-164 cleanup):
- No mystical/magical auras (replace with bioluminescence, electrical discharge, or enzyme glow)
- No fantasy metals (mithril, adamantine) — only pre-Collapse industrial materials
- No elves, dwarves, or fantasy races — only humans, hybrids, and mutated creatures
- No magic circles or runes — only cult symbols, pre-Collapse signage, and biological patterns
- No potions — only chemical compounds, enzyme extractions, and pharmaceutical preparations
- No dragons or mythical beasts — only genetically modified or mutated real-world animals

---

## Appendix A: Visual Summary by Faction

| Faction | Architecture | Materials | Lighting | Mood | Key Detail |
|---------|------------|-----------|----------|------|------------|
| Great Carapace | Grown chitin, fused shells, no right angles | Chitin, calcified enzyme, mineral deposits | Bioluminescent, cyan-green | Alien, ancient, predatory | All structures pulse faintly — they are alive |
| Sanguine Elite | Converted pre-Collapse luxury + chitin basements | Marble, silk, old-world tech, chitin | Candlelight, warm but unsteady | Decadent, threatening, beautiful | The deeper you go, the less human it looks |
| Familiars | Squatted infrastructure, tunnel networks | Scrap wood, bone, enzyme-stained cloth | Firelight, dim, purple-tinged | Desperate, reverent, decaying | Everything is sticky, stained, or burning |
| Free Humanity | Salvage-built, patchwork, functional | Wood, stone, leather, scrap metal | Daylight + lanterns, warm | Hopeful, fragile, stubborn | Signs of repair everywhere — nothing is thrown away |
| Constructs | Pristine industrial, pre-Collapse preservation | Metal, plastic, glass, circuits | Fluorescent, cold, clinical | Lonely, precise, sad | They maintain a world that no longer exists |

## Appendix B: Recommended Sprite Creation Order

Per implementation priority from CAR-271:

1. **Tier 1 — Core tiles (18 sprites):** Grass, dirt, stone, sand, water, ice, swamp, forest floor, and lava variants × 2 each
2. **Tier 2 — Core entities (20 sprites):** Player, Trench Lobster, Abyssal Dreadclaw, Spitter Crab, Molting Broodmother, Vampire Noble, Vampire Enforcer, Vampire Courtesan, Familiar Zealot, Familiar Acolyte, Telomerase Ghoul, Remnant Hunter, Settlement Guard, Artifact Scavenger, Security Drone, Cryo-Security Sentinel, Plasma Guardian, Chimeric Brute, Mantis Slicer, Carrion Flapper
3. **Tier 3 — Items (12 sprites):** Weapons (×5), armor (×3), consumables (×4)
4. **Tier 4 — UI (14 sprites):** Panels (×2), HUD (×3), faction icons (×5), cursor (×3)
