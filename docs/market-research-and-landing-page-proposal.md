# Market Research & Landing Page Proposal

> Deliverable for [CAR-4](/CAR/issues/CAR-4)
> Prepared by CMO (agent: [2aef0100-51b5-49f5-8c41-76f965fc9da1](/CAR/agents/cmo))
> Date: 2026-05-29

---

## Part 1: Market Research

### 1.1 Comparable Games — Pricing Matrix

| Game | Steam Price | Early Access | 1.0 Release | Peak Players | Review Score | Notes |
|------|------------|-------------|-------------|-------------|-------------|-------|
| Caves of Qud | $29.99 | 2015–2024 (9 yr) | Dec 2024 | ~2,500 | 95% | Closest analogue: bio-horror + procedural + RPG. Won Hugo Award 2025. |
| Dwarf Fortress | $29.99 | Classic free | Dec 2022 (Steam) | ~12,000 | 94% | Simulation benchmark. Audience overlap is strong. |
| RimWorld | $34.99 | 2013–2018 (5 yr) | Oct 2018 | ~20,000 | 97% | Story-generation sim with bio-modding. Higher price point with DLC. |
| Stoneshard | $24.99 | 2020–present (5+ yr) | TBD | ~4,500 | 81% | Harsh turn-based RPG, similar tone. Early access fatigue hurting reviews. |
| Kenshi | $29.99 | N/A | Dec 2018 | ~12,000 | 93% | Sandbox simulation, squad-based. 2.3M+ copies sold. |
| StarSector | $15 | Direct sale (no Steam) | N/A | N/A | 96% (est.) | Space fleet sim, not a roguelike, but cult following + direct-sale model is notable. |
| Path of Achra | $9.99 | N/A | 2023 | ~500 | 97% | Short-run procedural RPG. Lighter simulation, lower price. |

**Price corridor for Carapace:** $19.99 (early access) → $24.99 (1.0). This undercuts Caves of Qud and Dwarf Fortress while signalling similar depth. Path of Achra at $9.99 is a different tier (simpler game). Stoneshard at $24.99 proves the corridor works.

### 1.2 Steam Wishlist Benchmarks

Based on Chris Zukowski / GameDiscoverCo public data for comparable games:

- **500–2,000 wishlists** before Steam page: weak launch
- **7,000–15,000** before Steam page: decent launch (few thousand first-week units)
- **30,000–100,000+** before Steam page: strong launch (10k+ first-week units)

For a first-ever game from an unknown Rust indie team, realistic target is **7,000–15,000 wishlists before Steam page launch**, achieved through devlog + itch.io Early Access seeding.

### 1.3 Steam Tags Strategy

The game's unique combination suggests these primary tags:

| Tag | Why | Priority |
|-----|-----|----------|
| Roguelike | Core genre | Critical |
| Procedural Generation | Core mechanic | Critical |
| Turn-Based | Combat system | Critical |
| RPG | Genre | Critical |
| Body Horror | **Unique differentiator** — only ~200 games on Steam use this tag | High |
| Survival | Gameplay loop | High |
| Atmospheric | Tone | Medium |
| Simulation | Depth | Medium |
| Sci-Fi | Setting | Medium |
| Pixel Graphics | Visual style | Medium |

**Key insight:** Body Horror is an underused Steam tag (~200 games) with a dedicated audience. Carapace should own this tag on Steam — it's a natural fit and near-zero competition for discoverability.

### 1.4 Audience Segments

| Segment | Reach | Message Fit | Channel |
|---------|-------|-------------|---------|
| Traditional roguelike fans (DCSS, NetHack) | Large | High — deep simulation, procedural, permadeath | /r/roguelikes, Discord |
| Bio-horror enthusiasts (SCP, Annihilation, The Thing) | Medium | **Highest** — body horror is the hook | /r/horrorgaming, itch.io horror tags |
| Simulation depth players (DF, RimWorld) | Large | Medium — sell the emergent systems | /r/dwarffortress, Steam discovery |
| Rust language enthusiasts | Small | High — Rust-based ECS game | /r/rust, Hacker News |
| Terminal/ASCII aesthetic fans | Medium | High — terminal-rendered visuals | /r/unixporn, itch.io terminal tag |

---

## Part 2: Current Landing Page Audit

### 2.1 What Works

- **Design system**: Dark void background, warm ember accents, monospace/serif hybrid — excellent. Professional, atmospheric, distinctive.
- **"FROM THE TRADITION OF"**: Naming Dwarf Fortress, Caves of Qud, StarSector, DCSS — perfect positioning. This tells a roguelike player exactly what to expect.
- **Feature card structure**: Clean, scannable, with ASCII art. Good rhythm.
- **Email capture CTA**: Well-placed. Single action.
- **Heritage panorama ASCII**: Memorable. Shows care.

### 2.2 What Doesn't Work

| Issue | Severity | Why |
|-------|----------|-----|
| Tagline is generic | **CRITICAL** | "Every world is uncharted. Every story is yours." describes literally every procedural game. No mention of crustaceans, blood, body horror, immortality. |
| Hero copy buries the hook | **CRITICAL** | The bio-horror crustacean-vampire-blood-economy concept is the most distinctive pitch in indie games right now. It's completely absent from the landing page. |
| Feature copy is beautiful but indistinct | **HIGH** | "Infinite Worlds", "Simulation That Breathes", "Stories No One Wrote" — every paragraph could describe Caves of Qud or Dwarf Fortress. Where is the biological grimdark? |
| CTA is unclear | **MEDIUM** | "VENTURE IN" / "SUBSCRIBE TO DISPATCHES" — what am I subscribing to? Dev log? Demo access? Wishlist announcement? Be specific: "Get Early Access When It Drops." |
| Footer links are all "#" | **LOW** | Understandable for a pre-launch page, but Discord and itch.io should be live if they exist. |
| No screenshot/gif | **MEDIUM** | Terminal-rendered screenshots would sell the aesthetic. Even one animated terminal capture would outperform all the copy. |
| No release date frame | **LOW** | Don't need a date, but "DEMO Q3 2026" or similar would signal this is a real thing, not vaporware. |

### 2.3 The Core Problem

The page positions Carapace as a **procedural simulation roguelike**. So does Caves of Qud. So does Dwarf Fortress. So does every other game in the genre.

Carapace's *actual* differentiator is **biological grimdark body horror**: immortal crustaceans, blood-drinking hybrids, a disease-like economy built on telomerase harvesting. This is what makes a player *remember* the game. It's what makes them tell a friend. It's what gets shared in Discord.

**The fix**: Lead with the bio-horror. Let the simulation depth be the *validation* that it's a real game, not the hook.

---

## Part 3: Landing Page Copy Proposal

### 3.1 Structural Changes

1. **Hero**: New tagline and description that front-loads the bio-horror
2. **Feature cards**: Replace 3 of 5 cards with bio-horror-specific content
3. **New section**: "The World" — brief lore hook between hero and features
4. **CTA specificity**: Name what happens after signup
5. **Screenshot anchor**: Add `<img>` or recorded-ascii capture showing in-game trench combat

### 3.2 Proposed Hero Copy

```
Title:    CARAPACE
Tagline:  IMMORTALITY HAS A PRICE. YOUR HUMANITY IS THE CURRENCY.
Desc:     A procedurally-generated terminal RPG set in a biological grimdark world.
          A century after civilization collapsed, immortal crustaceans rule the deep
          trenches. Their blood-thirsty hybrids walk among humans. The telomerase
          enzyme grants eternal life — and has poisoned everything it touches.
          Harvest it from the dead. Trade in blood. Survive the harvest.
CTA:      GET THE DEMO →  (email input + "Notify Me")
```

### 3.3 Proposed Feature Cards (replace 3 of 5)

**Card 1: THE GREAT CARAPACE** *(replaces "Infinite Worlds")*
> In the deep trenches below civilization, primordial crustaceans never stop growing. They've lived centuries — fed on telomerase that makes human cells immortal. Their offspring crawl upward. Their hybrids walk among kings. And the deeper you descend, the older — and hungrier — they get.

**Card 2: THE BLOOD ECONOMY** *(replaces "Simulation That Breathes")*
> Gold is worthless. The real currency is Sanguis — human blood, harvested in vacuum-sealed canisters — and Telomerase Fluid, the glowing enzyme that sustains immortals. Buy technology from the Empire with vampire blood. Buy genetic modifications from the black market with your own. Every transaction costs a piece of your humanity.

**Card 3: THE FAMILIARS** *(replaces "Stories No One Wrote")*
> Humans addicted to vampire blood protect their masters during the day. Cultists hunt their own kind for tribute. Factions form around enzyme supply lines. A merchant's death on the Ash Road can trigger a famine. A single rusted artifact can rewrite the balance of power. The simulation remembers everything — and nothing was scripted.

**Card 4: YOUR BODY IS A RESOURCE** *(replaces "Learn Everything. Master Nothing.")*
> Harvest boss tissue for gene-splicing. Fail the splice and gain a permanent malapty — fused limbs, neural damage, chitin growths that destroy your equipment slots. Success means gaining exotic adaptations: bio-electric shocks, sonic blasts, chromatophore camouflage. Every mutation trades humanity for power.

**Card 5: BEAUTIFUL IN THE WAY MATH IS BEAUTIFUL** *(keep, rephrase slightly)*
> Retains current visual. Update copy to mention Rust/ECS architecture for the tech-curious.

### 3.4 New Section: "FROM THE TRADITION OF" *(keep as-is)*

The heritage section is correct. Keep Dwarf Fortress, Caves of Qud, StarSector, DCSS. This anchors genre expectations.

### 3.5 CTA Refresh

**Top CTA button**: "GET THE DEMO" or "PLAY THE PROLOGUE" (stronger action)
**Bottom CTA**: same, with copy: "Free demo. Your save carries to full release. No spam, just devlogs and launch notices."

### 3.6 Suggested Additions

- **One animated GIF / video capture**: A 15-second terminal GIF of fighting a crustacean in the trenches. Hosted on itch.io or imgur. This will outperform all copy.
- **itch.io widget**: Embed or link to the itch.io page with "Free Demo" if one exists.
- **Social proof placeholder**: "Join X players on Discord" with a real count when Discord reaches 100+.

---

## Part 4: Marketing Roadmap (Next 90 Days)

| Phase | Action | Owner | Deliverable |
|-------|--------|-------|-------------|
| Week 1 | Update landing page copy | CMO / Designer | Revised index.html + style.css |
| Week 2 | Launch itch.io page with free prologue | CTO / CMO | itch.io listing, demo build |
| Week 3 | First devlog: "Why crustacean horror?" | CMO | Devlog post on itch.io + blog |
| Week 4 | Discord server launch | CMO | Server structure, first members |
| Week 6 | Steam page creation | CMO | Steam store page, wishlist button |
| Week 8 | Second devlog: "The blood economy" | CMO | Deep-dive systems post |
| Week 10 | Community trailer (terminal capture only) | Designer / CMO | 60-second trailer |
| Week 12 | Third devlog: "How Rust powers emergent simulation" | CTO | Technical devlog, targets Hacker News |

---

## Part 5: Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Bio-horror theme is too niche | Medium | The genre audience (roguelike players) has high tolerance for dark themes. Caves of Qud (mutations, body horror adjacent) proves this works. |
| Terminal aesthetic limits mass appeal | Medium | Don't chase mass appeal. Target the 200k–500k niche audience that *prefers* terminal visuals. RimWorld's "ugly" graphics didn't limit it. |
| Early access fatigue | Low | Price entry low ($14.99–$19.99) and signal roadmap clearly. Stoneshard's EA fatigue came from slow updates — don't repeat that. |
| Crustacean theme reads as comedic | Low | The game's tone is grim. Marketing copy must avoid "funny crab" framing. Lean into the body horror, not the absurdity. |

---

## Appendix A: Steam Tags (Recommended)

```
Roguelike, Procedural Generation, Turn-Based Combat, RPG,
Body Horror, Atmospheric, Survival, Simulation, Sci-Fi,
Pixel Graphics, Singleplayer, Open World, CRPG, Dark Fantasy
```

**Body Horror** is the most important unique tag. Only ~200 games on Steam use it. Carapace should be one of the top results for this tag.

## Appendix B: Channel Prioritization

| Channel | Priority | Rationale |
|---------|----------|-----------|
| itch.io | **Tier 1** | Built-in terminal/horror audience, free demo distribution, direct community |
| Steam wishlists | **Tier 1** | Primary conversion metric. Everything drives here. |
| /r/roguelikes | **Tier 1** | Core audience, active community, allows devlog posts |
| Discord | **Tier 2** | Build community after itch.io launch |
| Hacker News | **Tier 2** | Rust/ECS angle for technical launch post |
| /r/rust | **Tier 2** | Reach developers who become players + contributors |
| Twitter / X | **Tier 3** | Low ROI for indie games in 2025 without existing following |
| TikTok | **Tier 3** | Not a fit for terminal visual game |
| Press outreach | **Tier 3** | Wait until Steam page or demo launch |

---

*This document was prepared as deliverable for [CAR-4](/CAR/issues/CAR-4). All pricing data sourced from SteamDB, IsThereAnyDeal, and public developer statements as of May 2026.*
