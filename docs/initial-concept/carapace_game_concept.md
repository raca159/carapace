# SYSTEM ARCHITECTURE & DESIGN CODEX: PROJECT CARAPACE
## I. Executive Summary & Design Pillars
**Project Carapace** is a procedurally generated, terminal-style, dark sci-fi/gothic roguelike RPG set a century after the total collapse of a technologically advanced human civilization. The game combines the systemic simulation depth of *Dwarf Fortress* and *RimWorld*, the tactical loot/progression loops of *Diablo*, and real-time LLM API orchestration for hyper-reactive NPC interaction.
### Core Pillars
 * **Simulation-First Architecture:** Visuals are deliberately constrained to a retro terminal/pixel aesthetic, freeing 100% of computational overhead for complex ecological modeling, physical systems, and generative world-states.
 * **Biological Grimdark Reality:** Every narrative element, monster modifier, currency, and player superpower is grounded in realistic evolutionary biology and genetic mutation—specifically exploiting the science of cellular telomerase.
 * **The Translation Layer Pattern:** The underlying game engine is fully deterministic and mathematical. Artificial Intelligence does not make systemic decisions or dictate game state; instead, it acts as an interpretive translation layer, turning raw mathematical states and tokenized actions into atmospheric narrative.
## II. Lore & World Mythos
### The Narrative Setup: The Remnant
The player begins the game by waking up from an ancient, high-tech cryogenic storage chamber buried deep within subterranean ruins. You are a **Remnant**—a frozen relic of a forgotten, hyper-advanced human era. Upon crawling out of your vault, you emerge into a fragmented, neo-medieval world built directly on top of the decaying ruins of your past.
Society has devolved into feudal human kingdoms, hamlets, and castles. The highly advanced technology of your time still exists scattered throughout the world, but it is deeply misunderstood; citizens refer to functional machines, firearms, and power cells as holy or forbidden **"Artifacts."**
### The Biological Horrors
The primary threat to this new world stems from a twisted, cosmic manifestation of real-world biology: **Crustacean Immortality**. Lobsters inherently possess an active enzyme (telomerase) that indefinitely rejuvenates their DNA, preventing aging. In *Project Carapace*, an ancient horizontal gene transfer event from an alien lineage weaponized this trait, creating a terrifying predatory hierarchy.
```
+-------------------------------------------------------+
|            THE GREAT CARAPACE (Deep Dungeons)         |
| Pure alien lobsters. Cosmic horrors. Master producers  |
|               of raw Telomerase Fluid.                |
+-------------------------------------------------------+
                           |
                           v (Horizontal Gene Transfer)
+-------------------------------------------------------+
|             THE SANGUINE ELITE (Vampires)              |
| Human-Crustacean hybrids. Infiltrate human nobility.  |
| Crave human blood to process & maintain their longevity|
+-------------------------------------------------------+
                           |
                           v (The Infectious Bite / Pact)
+-------------------------------------------------------+
|               THE FAMILIARS (Cultists)                |
| Infected humans. Addicted to vampire blood/enzyme.   |
|   Worship the immortals for maintenance doses.       |
+-------------------------------------------------------+
                           |
                           v (The Feed Loop)
+-------------------------------------------------------+
|              FREE HUMANITY / THE REMNANT              |
| Live in feudal ruins. Target of the night-stalkers.  |
|   Hire Hunter factions to reclaim the frontier.      |
+-------------------------------------------------------+

```
 1. **The Great Carapace ("The Demons"):** Massive, multi-limbed prehistoric crustacean monsters that live in subterranean rifts and flooded ancient vaults ("The Trenches"). They never stop growing, mutating, and becoming heavier as the centuries pass. They are the apex producers of the pure telomerase enzyme.
 2. **The Sanguine Elite ("The Vampires"):** Failed or intentional genetic hybrids of humans and the Great Carapace. They look human but bear subtle physical anomalies (hardened subdermal plates, compound-flicker eyes). To process the alien telomerase and stop their own DNA from violently unraveling, they must consume uncorrupted human blood. They blend seamlessly into human cities, ruling as corrupt lords, merchants, or hidden puppet masters.
 3. **The Familiars ("The Cults"):** Pure humans who have been infected with a diluted strain. They cannot manufacture the longevity enzyme natively and are completely addicted to receiving maintenance doses from their Vampire lords. They form fanatical cults that protect the monsters during the day and hunt fellow humans to feed the Elite.
## III. Tokenized Tag & Engine State Architecture
To achieve organic, emergent simulation, every living entity within the C# engine is constructed entirely from bitwise Tags. This allows the game engine to compute physical logic instantly while preparing a rich semantic profile for the AI to read.
### 1. Token Definitions
```csharp
[Flags]
public enum PhysicalTag
{
    None = 0,
    CanTalk = 1 << 0,          // Capable of language
    HasChitin = 1 << 1,        // Natural armor plating
    CompoundEyes = 1 << 2,     // Extreme night vision / day blindness
    Sanguivore = 1 << 3,       // Dependent on human blood consumption
    BioElectric = 1 << 4,      // Generates internal voltage
    ExothermicSpray = 1 << 5,  // Spits boiling chemical compounds
    CryoRemnant = 1 << 6       // Emits pre-collapse tech signatures
}

[Flags]
public enum MentalTag
{
    None = 0,
    Predatory = 1 << 0,       // Views non-tagged species as resources
    Deceptive = 1 << 1,        // Prone to masks, social camouflage, and lies
    Feral = 1 << 2,            // Pure animalistic survival instinct
    TechWorshipper = 1 << 3,   // Highly compliant around ancient artifacts
    Addicted = 1 << 4          // Psychologically enslaved to Telomerase
}

```
### 2. The Deterministic Engine State Machine
Before any text is generated, the C# loop runs its own state updates. Entities evaluate their surroundings, matching environmental data against their mental and physical tags via a probability matrix.
```
       [ ENGINE TURN EVALUATION ]
                   |
     +-------------v-------------+
     |   Current State: IDLE     |
     +-------------+-------------+
                   |
    (Check Environment: Player nearby)
                   |
     +-------------v-------------+
     |    Evaluate Next State:   |
     | - If Predatory -> HUNT    |
     | - If Feral -> ATTACK      |
     | - If Deceptive -> SOCIAL  |
     +-------------+-------------+
                   |
                   v
  [ RUN DETERMINISTIC GAME ACTION ] 
  (Move grid, attack, or open dialogue)

```
The state machine dictates the mechanical boundaries of an encounter. If an entity rolls a HUNT or COMBAT state, the tactical grid layout triggers immediate turn-based combat. If it rolls a SOCIAL state, it halts combat and invokes the AI translation wrapper.
## IV. Dual-Agent State Translation Layer (The AI System)
To prevent breaking the simulation, the player does *not* interact via free-text input. Free text causes character drift (e.g., a terrifying monster suddenly speaking benevolently). Instead, user responses are **pre-computed options generated by the C# engine based on the player's underlying tags and stats.**
When an interaction occurs, both the player and the NPC are processed as objects. The outcome is rolled instantly by the engine, and the **AI Translation Layer** reads the data to narrate the event.
### The Automated AI Prompt Payload
When a player opts to execute a specific social action, the engine serializes the data into a strict JSON-to-Markdown prompt sent via an external API (Ollama/OpenAI/Anthropic):
```markdown
### SYSTEM INSTRUCTIONS
You are the narrative overlay for a dark fantasy roguelike. Translate the following game engine states and math rolls into a raw, atmospheric, concise script exchange. Do not change the calculated mathematical outcomes.

### SCENE METADATA
- Location: Kingdom Vault Gates (Stormy Night)
- State Event: Player chooses [DECEIVE via CHROMATOPHORES].
- Engine Roll Outcome: SUCCESS.

### ENTITY A: THE PLAYER (The Remnant)
- Physical Tags: [HasChitin, ChromatophoricShift, CryoRemnant]
- Mental History: Clinical, Pragmatic.

### ENTITY B: THE NPC (The Gate Guard)
- Physical Tags: [PureHuman]
- Mental Tags: [Paranoid, TechWorshipper]
- Post-Roll State: Fooled, Submissive.

### OUTPUT FORMAT
[Player Line]: Dialogue and physical action description.
[NPC Line]: Dialogue and reactive body language.
[World Context]: Text describing the mechanical room shift.

```
### Rendered Game Screen Output
The player clicks their pre-computed menu button, and the engine seamlessly populates the terminal frame:
> **[Player Line]:** Standing completely still in the freezing rain, your skin ripples violently, shifting its pigment to match the precise navy-and-gold tones of the High Kingdom's royal guard crest. You step forward, your voice completely devoid of human inflection: *"Stand down, Sentinel. This armor carries a security clearance signature your local warlord hasn't even crawled out of the ash to learn yet. Unlatch the gates."*
> **[NPC Line]:** The guard blinks, dropping his heavy crossbow as he stares at the glowing, shifting biological camouflage across your chest. He bows his head with a trembling sigh. *"M-my apologies, Commander. I didn't recognize the weave of the ancient uniform... Pass through in peace."*
> **[World Context]:** The iron bolts of the heavy oak gate slide back. The pathway to the Inner Citadel is now clear.
> 
## V. Procedural Mutation & Splicing Boss Systems
### 1. Procedural Boss Factory (Horizontal Gene Transfer)
Bosses and elites are generated procedurally by marrying real-world evolutionary biology with an **Age Multiplier**. The longer a monster or vampire lives via telomerase, the more foreign animal DNA its unstable body absorbs from the world.
The engine uses a global biological trait library:
 * **Pistol Shrimp (Sonic Cavitation):** Massive, asymmetrical crushing arm that snaps closed to unleash a long-range stunning shockwave.
 * **Tardigrade (Anhydro-Chitin):** Extreme cellular survival. When HP falls below 15%, the creature calcifies, gaining massive armor protection and health regeneration for a few turns.
 * **Electric Eel (Bio-Electric Surge):** Modified muscle layers that add passive lightning damage to melee attacks and shock attackers when hit.
 * **Bombardier Beetle (Chemical Exothermic):** Gland structures that spray pressurized lines of boiling, corrosive acid, destroying player armor over time.
 * **Cuttlefish (Chromatophoric Shift):** Wavelength-shifting flesh that grants high evasion and allows the entity to blend into terminal grid tiles to break player target locks.
The physical scaling is handled deterministically by a factory calculation:
### 2. Player Gene-Splicing Loop (Risk vs. Reward)
Players can butcher slain bosses to extract **Viable Tissue Samples**. By returning these samples to a functional pre-collapse **Gene-Splicing Pod**, the player can attempt to force these alien/animal adaptations into their own human genome.
```
                  [ INSERT SAMPLE IN POD ]
                              |
              +---------------+---------------+
              |                               |
       (Roll Success %)               (Roll Failure %)
              |                               |
              v                               v
    [ SUCCESSFUL SPLICE ]             [ GENETIC MALAPTY ]
  - Gain Active/Passive Skill.       - Gain Permanent Debuff.
  - DNA Stability Drops.             - Physical Disfigurement +5.
  - "Humanity" Decreases.            - Body parts warp into monsters.

```
#### Biological Malapties (Examples)
 * *Failed Pistol Shrimp Splice ->* **Calcified Rigidity:** The player's left arm fuses permanently into an oversized, unyielding chitin club. You lose the ability to equip two-handed weapons or firearms, and weapon swing speed drops by 30%.
 * *Failed Electric Eel Splice ->* **Neural Grounding:** Your nervous system arcs uncontrollably out of your skin. Wearing any metallic armor (iron, steel, alloy) causes you to shock and stun yourself every few movements on the grid.
## VI. Factional Biological Economics
Traditional fiat currencies (gold coins, paper money) are entirely worthless in *Project Carapace*. The economy is run strictly on the exchange of raw, highly volatile biological commodities that dictate life and death.
### The Twin Biological Currencies
 1. **Telomerase Fluid ("The Elixir" / "T-Fluid"):** A glowing, highly dense fluid harvested from the core organs of slain vampires and deep-trench lobster monstrosities.
 2. **Sanguis ("San" / Human Blood):** Pure, uncorrupted human blood, harvested and preserved inside vacuum-sealed, pressurized old-world medical canisters.
| Currency Type | Primary Consumer | Trading Value & Dynamics |
|---|---|---|
| **Telomerase Fluid** | Human Kingdoms & Familiars | Humans buy it to study ancient longevity or bribe regional vampire lords. Cultist Familiars are aggressively addicted to it; possessing it allows you to command them as expendable mercenary meat-shields or force them to open locked dungeon checkpoints. |
| **Sanguis** | The Sanguine Elite & Dark Markets | Vampires require it to catalyze and digest their longevity enzyme. It is the *only* recognized medium of exchange in hidden underground mutant black markets. It is used to purchase illegal black-market tissue samples, high-end old-world weapons, or pass through vampire fiefdoms without violence. |
### The Grim Harvest Mechanics
The biological economic loop forces players into deep moral compromises:
 * To buy advanced technology from the Human Empire, you must kill Vampires to harvest their Telomerase.
 * To buy exotic genetic modifications from the Dark Market, you must pay in human blood. You can obtain blood by looting human enemies, raiding corrupt "blood farms," or using the gene-pod to **drain your own blood**, permanently trading chunks of your maximum HP string for liquid capital.
## VII. Technical Architecture Implementation Blueprint
The game is structured cleanly in **C#**, enforcing a total separation of concerns between raw mathematical state management and formatting layers.
```
       +---------------------------------------------+
       |             CONFIGURATION LAYER             |
       |  Reads external JSON files (mutations, tags)|
       +--------------------+------------------------+
                            |
                            v
       +---------------------------------------------+
       |          DETERMINISTIC SIMULATION           |
       |  C# Turn Loop / ECS Engine / Math / State  |
       +--------------------+------------------------+
                            |
         (On Social Event)  |  (Every Turn Frame)
                            v  v
+------------------------------------+  +-----------------------------------+
|        AI MEDIATION LAYER          |  |          RENDERING LAYER          |
|  Serializes tags into system LLM   |  |  SadConsole / RogueSharp Matrix   |
|  prompts for text orchestration.   |  |  Draws ASCII grid characters (@,#)|
+------------------------------------+  +-----------------------------------+

```
### 1. Data-Driven Configuration Model
All mutations, attributes, and base behaviors are read dynamically via an external JSON file framework. This allows the game to expand its ecological systems without recompiling the underlying executable logic.
### 2. Standard Entity Core Engine Loop
```csharp
public class Entity
{
    public string Name { get; set; }
    public int Age { get; set; }
    public PhysicalTag PhysicalTags { get; set; }
    public MentalTag MentalTags { get; set; }
    public EntityState CurrentState { get; set; }
    public PlayerGenome Genome { get; set; }
    public PlayerInventory Inventory { get; set; }
    
    // Evaluates grid positioning and turns without AI dependency
    public void ProcessTurn(Entity player, Map currentMap)
    {
        if (this.MentalTags.HasFlag(MentalTag.Feral))
        {
            if (Map.CalculateDistance(this.Position, player.Position) <= 1)
            {
                this.CurrentState = EntityState.Combat;
                this.ExecuteAttack(player);
                return;
            }
        }
        // Additional modular AI-State calculations run here...
    }
}

```
### 3. Verification of LLM Isolation
If an external API model suffers an outage, times out, or experiences string hallucinations, the application loop remains completely stable. The C# game core processes physical grid steps, armor mitigation formulas, faction reputation variables, and inventory tracking through raw integer data. The AI text component functions strictly as a rich, reactive visual skin layered safely over an uncompromised, math-driven world engine.