# Weather & Environmental Scores Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans.

**Goal:** Replace hardcoded weather enum with TOML-driven weather templates + environmental score system that produces tags through thresholds.

**Architecture:** Weather types become TOML files with score modifiers (light, temperature, moisture) + arbitrary modifier tags. Three core axes produce tags via threshold ranges. Tags flow through the existing interaction pipeline. WeatherSensitive component gates which entities are affected.

**Tech Stack:** Bevy ECS, serde/toml for config loading, existing tag system (TagRegistry, Tags component)

---

### Task 1: Add `threshold` field to tag system + Register environmental tags in tags.toml

**Files:**
- Modify: `crates/tags/src/definition.rs` — add `threshold` field to `TagToml`
- Modify: `crates/tags/src/registry.rs` — add `threshold` field to `TagDef` and `TagRegistryBuilder::add_tag`
- Modify: `crates/tags/src/loader.rs` — pass `threshold` through in `load_tag_registry`
- Modify: `assets/config/tags.toml` — add `light` archetype, `weather` archetype, `threshold` fields on temperature/moisture tags

#### Step 1a: Add `threshold` to TagToml, TagDef, and loader

`crates/tags/src/definition.rs` — add to `TagToml`:
```rust
#[derive(Debug, Deserialize)]
pub struct TagToml {
    pub id: String,
    #[serde(default)]
    pub implies: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub default_magnitude: Option<f32>,
    #[serde(default)]
    pub ticks: Option<[u32; 2]>,
    #[serde(default)]
    pub multiplier: Option<f32>,
    #[serde(default)]
    pub move_cost: Option<f32>,
    #[serde(default)]
    pub range: Option<u32>,
    #[serde(default)]
    pub threshold: Option<[u32; 2]>,
    #[serde(default)]
    pub tile_occupancy: Option<f32>,
    #[serde(default)]
    pub hp_mult: Option<f32>,
}
```

`crates/tags/src/registry.rs` — add `threshold` to `TagDef`:
```rust
#[derive(Debug, Clone)]
pub struct TagDef {
    pub id: TagId,
    pub name: String,
    pub archetype: ArchetypeId,
    pub implies: Vec<TagId>,
    pub conflicts: Vec<TagId>,
    pub bit_index: usize,
    pub default_magnitude: Option<f32>,
    pub ticks_range: Option<[u32; 2]>,
    pub multiplier: Option<f32>,
    pub move_cost: Option<f32>,
    pub range: Option<u32>,
    pub threshold: Option<[u32; 2]>,
    pub tile_occupancy: Option<f32>,
    pub hp_mult: Option<f32>,
}
```

Add `threshold` parameter to `TagRegistryBuilder::add_tag`:
```rust
    #[allow(clippy::too_many_arguments)]
    pub fn add_tag(
        &mut self,
        archetype: ArchetypeId,
        name: &str,
        implies_strs: Vec<String>,
        conflicts_strs: Vec<String>,
        default_magnitude: Option<f32>,
        ticks_range: Option<[u32; 2]>,
        multiplier: Option<f32>,
        move_cost: Option<f32>,
        range: Option<u32>,
        threshold: Option<[u32; 2]>,
        tile_occupancy: Option<f32>,
        hp_mult: Option<f32>,
    ) -> Result<TagId, RegistryError> {
```

And in the body of `add_tag`, update the `TagDef` construction:
```rust
        self.tags.push(TagDef {
            id,
            name: name.to_string(),
            archetype,
            implies: Vec::new(),
            conflicts: Vec::new(),
            bit_index,
            default_magnitude,
            ticks_range,
            multiplier,
            move_cost,
            range,
            threshold,
            tile_occupancy,
            hp_mult,
        });
```

`crates/tags/src/loader.rs` — pass `threshold` in `load_tag_registry`:
```rust
            builder
                .add_tag(
                    archetype_id,
                    &tag.id,
                    tag.implies.clone(),
                    tag.conflicts.clone(),
                    tag.default_magnitude,
                    tag.ticks,
                    tag.multiplier,
                    tag.move_cost,
                    tag.range,
                    tag.threshold,
                    tag.tile_occupancy,
                    tag.hp_mult,
                )
                .map_err(LoadError::Registry)?;
```

#### Step 1b: Update existing `add_tag` call sites in tests

Every test that calls `builder.add_tag(...)` currently passes 11 arguments. Add `None` for `threshold` (new 10th argument) after `range`.

In `crates/tags/src/registry.rs` tests, every `add_tag` call gains a `None` after the `range` argument:
```rust
// Before (example):
builder.add_tag(elem, "FIRE", vec![], vec![], Some(1.0), None, Some(2.0), None, None, None, Some(1.5)).unwrap();
// After:
builder.add_tag(elem, "FIRE", vec![], vec![], Some(1.0), None, Some(2.0), None, None, None, None, Some(1.5)).unwrap();
```

Update ALL `add_tag` calls in `registry.rs` tests the same way (add `None` after the `range` arg — position 10).

In `crates/tags/src/component.rs` tests, every `add_tag` call also needs the extra `None`:
```rust
// Before:
builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None).unwrap();
// After:
builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
```

#### Step 1c: Add environmental archetypes to `assets/config/tags.toml`

Add after the `moisture` archetype block (after line 182, `id = "SOAKED"`):

```toml
[[archetype]]
id = "light"
name = "Light Level"
exclusivity = "mutual"

[[archetype.tags]]
id = "DARK"
threshold = [0, 20]

[[archetype.tags]]
id = "DIM"
threshold = [20, 40]

[[archetype.tags]]
id = "BRIGHT"
threshold = [60, 100]

[[archetype]]
id = "weather"
name = "Weather State"
exclusivity = "any"

[[archetype.tags]]
id = "RAINY"

[[archetype.tags]]
id = "STORMY"

[[archetype.tags]]
id = "SNOWY"

[[archetype.tags]]
id = "FOGGY"

[[archetype.tags]]
id = "WINDY"

[[archetype.tags]]
id = "REDUCED_VISIBILITY"
```

Add `threshold` fields to existing temperature tags:
```toml
[[archetype]]
id = "temperature"
name = "Temperature"
exclusivity = "mutual"

[[archetype.tags]]
id = "FREEZING"
threshold = [0, 15]

[[archetype.tags]]
id = "COLD"
threshold = [15, 35]

[[archetype.tags]]
id = "NEUTRAL"
threshold = [35, 65]

[[archetype.tags]]
id = "WARM"
threshold = [65, 85]

[[archetype.tags]]
id = "HOT"
threshold = [85, 100]
```

Add `threshold` fields to existing moisture tags:
```toml
[[archetype]]
id = "moisture"
name = "Moisture"
exclusivity = "mutual"

[[archetype.tags]]
id = "DRY"
threshold = [0, 20]

[[archetype.tags]]
id = "DAMP"
threshold = [20, 40]

[[archetype.tags]]
id = "WET"
threshold = [40, 70]

[[archetype.tags]]
id = "SOAKED"
threshold = [70, 100]
```

#### Tests

Run: `cargo test -p game-tags`
Expected: All existing tests pass. New tags (DARK, DIM, BRIGHT, RAINY, STORMY, etc.) are loadable. Threshold values present on temperature/moisture/light tags.

Add a test to `crates/tags/src/loader.rs` integration tests:
```rust
    #[test]
    fn test_threshold_fields_loaded() {
        let registry = full_registry();
        let freezing = registry.tag_by_name("FREEZING").unwrap();
        assert_eq!(freezing.threshold, Some([0, 15]));
        let dark = registry.tag_by_name("DARK").unwrap();
        assert_eq!(dark.threshold, Some([0, 20]));
        let rainy = registry.tag_by_name("RAINY").unwrap();
        assert!(rainy.threshold.is_none());
        let wet = registry.tag_by_name("WET").unwrap();
        assert_eq!(wet.threshold, Some([40, 70]));
    }

    #[test]
    fn test_new_archetypes_loaded() {
        let registry = full_registry();
        assert!(registry.archetype_by_name("light").is_some());
        assert!(registry.archetype_by_name("weather").is_some());
        assert!(registry.tag_by_name("DARK").is_some());
        assert!(registry.tag_by_name("BRIGHT").is_some());
        assert!(registry.tag_by_name("RAINY").is_some());
        assert!(registry.tag_by_name("REDUCED_VISIBILITY").is_some());
    }
```

**Commit:** `feat: add threshold field to tag system, register light/weather archetypes`

---

### Task 2: Add base environmental scores to biome_rules.toml

**Files:**
- Modify: `crates/world/src/biome.rs` — add `environment` field to `BiomeRule`
- Modify: `assets/config/biome_rules.toml` — add `environment` to each rule

#### Step 2a: Add `EnvironmentScores` struct and `environment` field to BiomeRule

`crates/world/src/biome.rs`:
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct EnvironmentScores {
    #[serde(default)]
    pub light: i32,
    #[serde(default)]
    pub temperature: i32,
    #[serde(default)]
    pub moisture: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeRule {
    pub biome: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub elevation: Option<(f32, f32)>,
    pub moisture: Option<(f32, f32)>,
    pub temperature: Option<(f32, f32)>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub priority: u32,
    #[serde(default)]
    pub environment: EnvironmentScores,
}
```

#### Step 2b: Add environment scores to biome_rules.toml

Update each `[[rule]]` in `assets/config/biome_rules.toml` with an `environment` field:

```toml
[[rule]]
biome = "BIOME_DEEP_OCEAN"
glyph = "~"
color = [0, 0, 128]
elevation = [0.0, 0.20]
priority = 100
tags = ["BIOME_DEEP_OCEAN", "BLOCKED", "SWIMMABLE"]
environment = { light = 60, temperature = 30, moisture = 100 }

[[rule]]
biome = "BIOME_OCEAN"
glyph = "~"
color = [0, 0, 255]
elevation = [0.20, 0.30]
priority = 90
tags = ["BIOME_OCEAN", "BLOCKED", "SWIMMABLE"]
environment = { light = 70, temperature = 40, moisture = 95 }

[[rule]]
biome = "BIOME_BEACH"
glyph = "."
color = [255, 215, 0]
elevation = [0.30, 0.35]
priority = 80
tags = ["BIOME_BEACH", "WALKABLE"]
environment = { light = 85, temperature = 55, moisture = 30 }

[[rule]]
biome = "BIOME_MOUNTAIN_PEAK"
glyph = "^"
color = [64, 64, 64]
elevation = [0.85, 1.0]
priority = 70
tags = ["BIOME_MOUNTAIN_PEAK", "BLOCKED", "OPAQUE"]
environment = { light = 90, temperature = 5, moisture = 20 }

[[rule]]
biome = "BIOME_MOUNTAIN"
glyph = "^"
color = [128, 128, 128]
elevation = [0.70, 0.85]
priority = 60
tags = ["BIOME_MOUNTAIN", "WALKABLE", "SLOW", "CLIMBABLE"]
environment = { light = 70, temperature = 20, moisture = 40 }

[[rule]]
biome = "BIOME_SWAMP"
glyph = "~"
color = [85, 107, 47]
elevation = [0.30, 0.40]
moisture = [0.7, 1.0]
temperature = [0.5, 1.0]
priority = 55
tags = ["BIOME_SWAMP", "WALKABLE", "SLOW", "WET", "SOAKED"]
environment = { light = 40, temperature = 60, moisture = 90 }

[[rule]]
biome = "BIOME_DESERT"
glyph = "."
color = [237, 201, 175]
elevation = [0.35, 0.70]
temperature = [0.7, 1.0]
moisture = [0.0, 0.3]
priority = 50
tags = ["BIOME_DESERT", "WALKABLE", "DRY", "HOT"]
environment = { light = 80, temperature = 80, moisture = 10 }

[[rule]]
biome = "BIOME_SAVANNA"
glyph = "."
color = [210, 180, 140]
elevation = [0.35, 0.70]
temperature = [0.7, 1.0]
moisture = [0.3, 0.6]
priority = 49
tags = ["BIOME_SAVANNA", "WALKABLE", "DRY"]
environment = { light = 80, temperature = 70, moisture = 35 }

[[rule]]
biome = "BIOME_TROPICAL_FOREST"
glyph = "T"
color = [0, 100, 0]
elevation = [0.35, 0.70]
temperature = [0.7, 1.0]
moisture = [0.6, 1.0]
priority = 48
tags = ["BIOME_TROPICAL_FOREST", "WALKABLE", "WET", "FLAMMABLE"]
environment = { light = 50, temperature = 70, moisture = 80 }

[[rule]]
biome = "BIOME_SHRUBLAND"
glyph = "."
color = [218, 165, 32]
elevation = [0.35, 0.70]
temperature = [0.4, 0.7]
moisture = [0.0, 0.3]
priority = 45
tags = ["BIOME_SHRUBLAND", "WALKABLE", "DRY"]
environment = { light = 75, temperature = 50, moisture = 25 }

[[rule]]
biome = "BIOME_GRASSLAND"
glyph = "."
color = [144, 238, 144]
elevation = [0.35, 0.70]
temperature = [0.4, 0.7]
moisture = [0.3, 0.6]
priority = 44
tags = ["BIOME_GRASSLAND", "WALKABLE"]
environment = { light = 70, temperature = 50, moisture = 50 }

[[rule]]
biome = "BIOME_TEMPERATE_FOREST"
glyph = "T"
color = [34, 139, 34]
elevation = [0.35, 0.70]
temperature = [0.4, 0.7]
moisture = [0.6, 1.0]
priority = 43
tags = ["BIOME_TEMPERATE_FOREST", "WALKABLE", "FLAMMABLE"]
environment = { light = 55, temperature = 50, moisture = 60 }

[[rule]]
biome = "BIOME_BOREAL_FOREST"
glyph = "T"
color = [0, 128, 128]
elevation = [0.35, 0.70]
temperature = [0.2, 0.4]
moisture = [0.3, 0.7]
priority = 40
tags = ["BIOME_BOREAL_FOREST", "WALKABLE", "COLD"]
environment = { light = 50, temperature = 25, moisture = 55 }

[[rule]]
biome = "BIOME_ICE_SHEET"
glyph = "."
color = [255, 255, 255]
elevation = [0.35, 0.70]
temperature = [0.0, 0.1]
moisture = [0.0, 0.3]
priority = 36
tags = ["BIOME_ICE_SHEET", "WALKABLE", "FREEZING", "SLIPPERY"]
environment = { light = 65, temperature = 5, moisture = 15 }

[[rule]]
biome = "BIOME_TUNDRA"
glyph = "."
color = [192, 192, 192]
elevation = [0.35, 0.70]
temperature = [0.0, 0.2]
priority = 35
tags = ["BIOME_TUNDRA", "WALKABLE", "COLD", "SLOW"]
environment = { light = 60, temperature = 10, moisture = 30 }

[[rule]]
biome = "BIOME_VOLCANIC"
glyph = "^"
color = [200, 50, 0]
elevation = [0.80, 1.0]
temperature = [0.9, 1.0]
priority = 75
tags = ["BIOME_VOLCANIC", "WALKABLE", "HOT", "DANGEROUS"]
environment = { light = 60, temperature = 95, moisture = 5 }

[[rule]]
biome = "BIOME_GRASSLAND"
glyph = "."
color = [144, 238, 144]
elevation = [0.35, 0.70]
priority = 1
tags = ["BIOME_GRASSLAND", "WALKABLE"]
environment = { light = 70, temperature = 50, moisture = 50 }
```

#### Tests

Add to `crates/world/src/biome.rs` tests:
```rust
    #[test]
    fn test_environment_scores_default() {
        let scores = EnvironmentScores::default();
        assert_eq!(scores.light, 0);
        assert_eq!(scores.temperature, 0);
        assert_eq!(scores.moisture, 0);
    }

    #[test]
    fn test_biome_rule_with_environment() {
        let rule: BiomeRule = toml::from_str(r#"
            biome = "TEST"
            glyph = "."
            color = [0, 0, 0]
            priority = 1
            environment = { light = 80, temperature = 70, moisture = 10 }
        "#).unwrap();
        assert_eq!(rule.environment.light, 80);
        assert_eq!(rule.environment.temperature, 70);
        assert_eq!(rule.environment.moisture, 10);
    }
```

Run: `cargo test -p game-world -- biome`
Expected: All tests pass.

**Commit:** `feat: add base environmental scores to biome rules`

---

### Task 3: Create WeatherDef struct and TOML loader

**Files:**
- Modify: `crates/core/src/weather.rs` — add `WeatherDef` struct, loader function

#### Step 3a: Add WeatherDef and loader

Add to `crates/core/src/weather.rs` after the imports:
```rust
use std::collections::HashMap;

use bevy_ecs::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::turn::TurnCounter;

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherDef {
    pub name: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default = "default_duration")]
    pub duration: [u32; 2],
    #[serde(default = "default_visibility")]
    pub visibility: f32,
    #[serde(default)]
    pub modifiers: HashMap<String, i32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_weight() -> u32 { 10 }
fn default_duration() -> [u32; 2] { [5, 25] }
fn default_visibility() -> f32 { 1.0 }

pub fn load_weather_defs(toml_str: &str) -> Result<Vec<WeatherDef>, toml::de::Error> {
    #[derive(Deserialize)]
    struct WeatherFile {
        #[serde(rename = "weather")]
        weathers: Vec<WeatherDef>,
    }
    let file: WeatherFile = toml::from_str(toml_str)?;
    Ok(file.weathers)
}

pub fn load_weather_def(toml_str: &str) -> Result<WeatherDef, toml::de::Error> {
    toml::from_str(toml_str)
}
```

#### Step 3b: Add tests for parsing

Add to the `#[cfg(test)] mod tests` block:
```rust
    #[test]
    fn parse_weather_def() {
        let toml = r#"
            name = "Rain"
            weight = 10
            duration = [5, 15]
            visibility = 0.6
            [modifiers]
            moisture = 50
            [tags]
            values = ["RAINY"]
        "#;
        // Note: tags is a Vec<String> not a table
        let toml = r#"
            name = "Rain"
            weight = 10
            duration = [5, 15]
            visibility = 0.6

            [modifiers]
            moisture = 50
        "#;
        let def: WeatherDef = toml::from_str(toml).unwrap();
        assert_eq!(def.name, "Rain");
        assert_eq!(def.weight, 10);
        assert_eq!(def.visibility, 0.6);
        assert_eq!(def.modifiers.get("moisture"), Some(&50));
    }

    #[test]
    fn parse_weather_def_with_tags() {
        let toml = r#"
            name = "Storm"
            weight = 5
            duration = [3, 8]
            visibility = 0.3
            tags = ["STORMY", "WINDY"]

            [modifiers]
            moisture = 60
        "#;
        let def: WeatherDef = toml::from_str(toml).unwrap();
        assert_eq!(def.tags, vec!["STORMY", "WINDY"]);
    }

    #[test]
    fn parse_weather_def_defaults() {
        let toml = r#"
            name = "Clear"
        "#;
        let def: WeatherDef = toml::from_str(toml).unwrap();
        assert_eq!(def.weight, 10);
        assert_eq!(def.duration, [5, 25]);
        assert!((def.visibility - 1.0).abs() < 0.01);
        assert!(def.modifiers.is_empty());
        assert!(def.tags.is_empty());
    }

    #[test]
    fn load_multiple_weather_defs() {
        let toml = r#"
            [[weather]]
            name = "Clear"
            weight = 30
            visibility = 1.0

            [[weather]]
            name = "Rain"
            weight = 10
            visibility = 0.6

            [weather.modifiers]
            moisture = 50
        "#;
        let defs = load_weather_defs(toml).unwrap();
        assert_eq!(defs.len(), 2);
    }
```

Run: `cargo test -p game-core -- weather::tests`
Expected: All tests pass.

**Commit:** `feat: add WeatherDef struct and TOML loader`

---

### Task 4: Create 8 weather TOML files

**Files:**
- Create: `assets/config/weather/weather_clear.toml`
- Create: `assets/config/weather/weather_cloudy.toml`
- Create: `assets/config/weather/weather_fog.toml`
- Create: `assets/config/weather/weather_rain.toml`
- Create: `assets/config/weather/weather_storm.toml`
- Create: `assets/config/weather/weather_snow.toml`
- Create: `assets/config/weather/weather_sandstorm.toml`
- Create: `assets/config/weather/weather_ashfall.toml`

First create the directory:
```bash
mkdir -p assets/config/weather
```

`assets/config/weather/weather_clear.toml`:
```toml
name = "Clear"
weight = 30
duration = [8, 25]
visibility = 1.0
```

`assets/config/weather/weather_cloudy.toml`:
```toml
name = "Cloudy"
weight = 20
duration = [5, 20]
visibility = 0.9

[modifiers]
light = -10
```

`assets/config/weather/weather_fog.toml`:
```toml
name = "Fog"
weight = 5
duration = [3, 10]
visibility = 0.3
tags = ["FOGGY", "REDUCED_VISIBILITY"]

[modifiers]
moisture = 20
```

`assets/config/weather/weather_rain.toml`:
```toml
name = "Rain"
weight = 10
duration = [5, 15]
visibility = 0.6
tags = ["RAINY"]

[modifiers]
moisture = 50
```

`assets/config/weather/weather_storm.toml`:
```toml
name = "Storm"
weight = 3
duration = [3, 8]
visibility = 0.3
tags = ["STORMY", "WINDY"]

[modifiers]
moisture = 60
```

`assets/config/weather/weather_snow.toml`:
```toml
name = "Snow"
weight = 5
duration = [5, 15]
visibility = 0.5
tags = ["SNOWY"]

[modifiers]
temperature = -40
moisture = 30
```

`assets/config/weather/weather_sandstorm.toml`:
```toml
name = "Sandstorm"
weight = 2
duration = [3, 8]
visibility = 0.3
tags = ["REDUCED_VISIBILITY"]

[modifiers]
moisture = -40
```

`assets/config/weather/weather_ashfall.toml`:
```toml
name = "AshFall"
weight = 2
duration = [3, 10]
visibility = 0.4
tags = ["REDUCED_VISIBILITY"]

[modifiers]
light = -50
```

#### Tests

No Rust code changes. Verify files parse:
```bash
for f in assets/config/weather/weather_*.toml; do
    python3 -c "import tomllib; tomllib.load(open('$f', 'rb'))" && echo "OK: $f"
done
```
Expected: All 8 files print "OK".

**Commit:** `feat: add 8 weather TOML template files`

---

### Task 5: Refactor WeatherState to use WeatherDef

**Files:**
- Modify: `crates/core/src/weather.rs` — replace `Weather` enum with loaded `WeatherDef`, update `WeatherState` and `WeatherContext`

#### Step 5a: Replace Weather enum with WeatherDef-based system

Remove the `Weather` enum entirely. Update `WeatherState`:
```rust
use std::collections::HashMap;

use bevy_ecs::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::turn::TurnCounter;
use game_tags::TagId;

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherDef {
    pub name: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default = "default_duration")]
    pub duration: [u32; 2],
    #[serde(default = "default_visibility")]
    pub visibility: f32,
    #[serde(default)]
    pub modifiers: HashMap<String, i32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_weight() -> u32 { 10 }
fn default_duration() -> [u32; 2] { [5, 25] }
fn default_visibility() -> f32 { 1.0 }

pub fn load_weather_def(toml_str: &str) -> Result<WeatherDef, toml::de::Error> {
    toml::from_str(toml_str)
}

pub fn load_weather_defs(toml_str: &str) -> Result<Vec<WeatherDef>, toml::de::Error> {
    #[derive(Deserialize)]
    struct WeatherFile {
        #[serde(rename = "weather")]
        weathers: Vec<WeatherDef>,
    }
    let file: WeatherFile = toml::from_str(toml_str)?;
    Ok(file.weathers)
}

fn glyph_for_name(name: &str) -> char {
    match name {
        "Clear" => '\u{2600}',
        "Cloudy" => '\u{2601}',
        "Fog" => '\u{2261}',
        "Rain" => '\u{2502}',
        "Storm" => '\u{26A1}',
        "Snow" => '\u{2744}',
        "Sandstorm" => '\u{2592}',
        "AshFall" => '\u{2022}',
        _ => '?',
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub fn name(&self) -> &'static str {
        match self {
            TimeOfDay::Dawn => "Dawn",
            TimeOfDay::Day => "Day",
            TimeOfDay::Dusk => "Dusk",
            TimeOfDay::Night => "Night",
        }
    }

    pub fn light_modifier(&self) -> i32 {
        match self {
            TimeOfDay::Dawn => -40,
            TimeOfDay::Day => 0,
            TimeOfDay::Dusk => -40,
            TimeOfDay::Night => -80,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct WeatherState {
    pub active_weather: Option<WeatherDef>,
    pub all_weathers: Vec<WeatherDef>,
    pub time: TimeOfDay,
    pub turn_count: u64,
    pub weather_turns_remaining: u32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            active_weather: None,
            all_weathers: Vec::new(),
            time: TimeOfDay::Day,
            turn_count: 0,
            weather_turns_remaining: 20,
        }
    }
}

impl WeatherState {
    pub fn new(weathers: Vec<WeatherDef>) -> Self {
        let active = weathers.first().cloned();
        Self {
            active_weather: active,
            all_weathers: weathers,
            time: TimeOfDay::Day,
            turn_count: 0,
            weather_turns_remaining: 20,
        }
    }

    pub fn weather_name(&self) -> &str {
        self.active_weather.as_ref().map(|w| w.name.as_str()).unwrap_or("Clear")
    }

    pub fn weather_glyph(&self) -> char {
        self.active_weather.as_ref().map(|w| glyph_for_name(&w.name)).unwrap_or('\u{2600}')
    }

    pub fn weather_visibility(&self) -> f32 {
        self.active_weather.as_ref().map(|w| w.visibility).unwrap_or(1.0)
    }

    pub fn weather_modifiers(&self) -> &HashMap<String, i32> {
        self.active_weather.as_ref().map(|w| &w.modifiers).unwrap_or(&EMPTY_MODIFIERS)
    }

    pub fn weather_tags(&self) -> &[String] {
        self.active_weather.as_ref().map(|w| w.tags.as_slice()).unwrap_or(&[])
    }

    pub fn advance_time(&mut self) {
        self.turn_count += 1;

        let day_length = 50;
        match self.time {
            TimeOfDay::Dawn => { if self.turn_count >= day_length / 4 { self.time = TimeOfDay::Day; self.turn_count = 0; } }
            TimeOfDay::Day => { if self.turn_count >= day_length / 2 { self.time = TimeOfDay::Dusk; self.turn_count = 0; } }
            TimeOfDay::Dusk => { if self.turn_count >= day_length / 4 { self.time = TimeOfDay::Night; self.turn_count = 0; } }
            TimeOfDay::Night => { if self.turn_count >= day_length { self.time = TimeOfDay::Dawn; self.turn_count = 0; } }
        }

        if self.weather_turns_remaining > 0 {
            self.weather_turns_remaining -= 1;
        } else {
            self.roll_new_weather();
        }
    }

    fn roll_new_weather(&mut self) {
        if self.all_weathers.is_empty() { return; }
        let total: u32 = self.all_weathers.iter().map(|w| w.weight).sum();
        if total == 0 { return; }
        let mut rng = rand::rng();
        let mut roll = rng.random_range(0..total);
        for weather in &self.all_weathers {
            if roll < weather.weight {
                self.active_weather = Some(weather.clone());
                self.weather_turns_remaining = rng.random_range(weather.duration[0]..=weather.duration[1]);
                return;
            }
            roll -= weather.weight;
        }
        self.active_weather = self.all_weathers.last().cloned();
        let dur = self.active_weather.as_ref().map(|w| w.duration).unwrap_or([5, 25]);
        self.weather_turns_remaining = rng.random_range(dur[0]..=dur[1]);
    }

    pub fn effective_visibility(&self) -> f32 {
        let base = self.weather_visibility();
        let light_mod = match self.time {
            TimeOfDay::Night => 0.15,
            TimeOfDay::Dawn => 0.6,
            TimeOfDay::Dusk => 0.4,
            TimeOfDay::Day => 1.0,
        };
        base * light_mod
    }
}

static EMPTY_MODIFIERS: HashMap<String, i32> = HashMap::new();

#[derive(Resource, Debug, Clone, Default)]
pub struct WeatherContext {
    pub tags: Vec<String>,
    pub applied_tags: Vec<TagId>,
    pub active_modifiers: HashMap<String, i32>,
}

pub fn weather_tags_for_context(state: &WeatherState) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(ref weather) = state.active_weather {
        tags.extend(weather.tags.iter().cloned());
    }
    match state.time {
        TimeOfDay::Night => tags.push("DARK".to_string()),
        TimeOfDay::Dusk | TimeOfDay::Dawn => tags.push("DIM".to_string()),
        _ => {}
    }
    tags
}
```

#### Step 5b: Update lib.rs exports

`crates/core/src/lib.rs` — update the weather re-export line:
```rust
pub use weather::{WeatherDef, TimeOfDay, WeatherState, WeatherContext, weather_tags_for_context, load_weather_def, load_weather_defs};
```

Remove `Weather` from the export.

#### Step 5c: Update call sites that reference `Weather` enum

In `src/game/mod.rs` (around line 485-498), the weather advancement code currently calls `ws.advance_time(&dummy)` and `weather_tags_for_context(&ws.weather, &ws.time)`. Update:

```rust
    // Advance weather
    let weather_tags = {
        if let Some(mut ws) = game_world.0.get_resource_mut::<WeatherState>() {
            ws.advance_time();
            Some(weather_tags_for_context(&ws))
        } else {
            None
        }
    };
    if let Some(tags) = weather_tags {
        if let Some(mut wc) = game_world.0.get_resource_mut::<WeatherContext>() {
            wc.tags = tags;
        }
    }
```

Remove the `let dummy = World::new();` and `&world` parameter from `advance_time` — it no longer takes a `&World` parameter.

Any other file that references `Weather::Clear`, `Weather::Storm`, etc. — search and replace with string-based or WeatherDef references. The primary consumer is the weather.rs itself, which is fully rewritten.

#### Tests

Update weather tests:
```rust
    #[test]
    fn weather_state_defaults() {
        let ws = WeatherState::default();
        assert!(ws.active_weather.is_none());
        assert_eq!(ws.time, TimeOfDay::Day);
    }

    #[test]
    fn weather_state_with_defs() {
        let defs = vec![
            WeatherDef { name: "Clear".into(), weight: 30, duration: [8, 25], visibility: 1.0, modifiers: HashMap::new(), tags: vec![] },
            WeatherDef { name: "Rain".into(), weight: 10, duration: [5, 15], visibility: 0.6, modifiers: HashMap::from([("moisture".into(), 50)]), tags: vec!["RAINY".into()] },
        ];
        let ws = WeatherState::new(defs);
        assert_eq!(ws.weather_name(), "Clear");
        assert_eq!(ws.all_weathers.len(), 2);
    }

    #[test]
    fn advance_time_changes_day_to_dusk() {
        let mut ws = WeatherState::default();
        ws.time = TimeOfDay::Day;
        ws.turn_count = 24;
        ws.advance_time();
        assert_eq!(ws.time, TimeOfDay::Dusk);
    }

    #[test]
    fn visibility_never_zero() {
        let defs = vec![
            WeatherDef { name: "Clear".into(), weight: 30, duration: [8, 25], visibility: 1.0, modifiers: HashMap::new(), tags: vec![] },
        ];
        let ws = WeatherState::new(defs);
        assert!(ws.effective_visibility() > 0.0);
    }

    #[test]
    fn weather_changes_after_turns_elapsed() {
        let mut ws = WeatherState::default();
        ws.weather_turns_remaining = 1;
        ws.all_weathers = vec![
            WeatherDef { name: "Clear".into(), weight: 30, duration: [8, 25], visibility: 1.0, modifiers: HashMap::new(), tags: vec![] },
        ];
        ws.advance_time();
    }

    #[test]
    fn weather_roll_by_weight() {
        let defs = vec![
            WeatherDef { name: "Clear".into(), weight: 100, duration: [8, 25], visibility: 1.0, modifiers: HashMap::new(), tags: vec![] },
            WeatherDef { name: "Rain".into(), weight: 1, duration: [5, 15], visibility: 0.6, modifiers: HashMap::from([("moisture".into(), 50)]), tags: vec![] },
        ];
        let mut ws = WeatherState::new(defs);
        ws.weather_turns_remaining = 0;
        ws.roll_new_weather();
        assert!(ws.active_weather.is_some());
        assert_eq!(ws.weather_name(), "Clear");
    }

    #[test]
    fn time_of_day_light_modifier() {
        assert_eq!(TimeOfDay::Night.light_modifier(), -80);
        assert_eq!(TimeOfDay::Dawn.light_modifier(), -40);
        assert_eq!(TimeOfDay::Day.light_modifier(), 0);
        assert_eq!(TimeOfDay::Dusk.light_modifier(), -40);
    }
```

Run: `cargo test -p game-core -- weather::tests`
Expected: All tests pass.

Run: `cargo build` to verify all call sites compile.
Expected: Build succeeds.

**Commit:** `refactor: replace Weather enum with TOML-driven WeatherDef system`

---

### Task 6: Implement score computation and threshold resolution

**Files:**
- Modify: `crates/core/src/weather.rs` — add `compute_environmental_scores` and `resolve_scores_to_tags` functions
- Modify: `crates/core/src/lib.rs` — export new functions

#### Step 6a: Add score computation and resolution functions

Add to `crates/core/src/weather.rs`:
```rust
use game_tags::{TagRegistry, TagId, Tags, TagValue};
use crate::world_overview::EnvironmentScores as BiomeEnvironment;

#[derive(Debug, Clone, Copy, Default)]
pub struct EnvironmentalScores {
    pub light: i32,
    pub temperature: i32,
    pub moisture: i32,
}

impl EnvironmentalScores {
    pub fn clamp(&self) -> Self {
        Self {
            light: self.light.clamp(0, 100),
            temperature: self.temperature.clamp(0, 100),
            moisture: self.moisture.clamp(0, 100),
        }
    }
}

pub fn compute_environmental_scores(
    base: &BiomeEnvironment,
    weather_modifiers: &HashMap<String, i32>,
    time_of_day: TimeOfDay,
) -> EnvironmentalScores {
    let mut scores = EnvironmentalScores {
        light: base.light,
        temperature: base.temperature,
        moisture: base.moisture,
    };

    for (key, &value) in weather_modifiers {
        match key.as_str() {
            "light" => scores.light += value,
            "temperature" => scores.temperature += value,
            "moisture" => scores.moisture += value,
            _ => {}
        }
    }

    scores.light += time_of_day.light_modifier();

    scores.clamp()
}

pub fn resolve_scores_to_tags(
    scores: &EnvironmentalScores,
    registry: &TagRegistry,
) -> Vec<TagId> {
    let mut result = Vec::new();

    for tag_def in registry.all_tags() {
        if let Some([min, max]) = tag_def.threshold {
            let score = match tag_def.name.as_str() {
                "DARK" | "DIM" | "BRIGHT" => scores.light,
                "FREEZING" | "COLD" | "NEUTRAL" | "WARM" | "HOT" => scores.temperature,
                "DRY" | "DAMP" | "WET" | "SOAKED" => scores.moisture,
                _ => continue,
            };
            if score >= min as i32 && score < max as i32 {
                result.push(tag_def.id);
            }
        }
    }

    result
}
```

Note: We need `EnvironmentScores` from `crates/world/src/biome.rs` to be accessible from `game-core`. Since `game-core` doesn't depend on `game-world` (only the reverse), we should re-define `EnvironmentScores` in `game-core` or use a shared type. The simplest approach: define `BiomeEnvironment` as a simple struct in `game-core`:

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct BiomeEnvironment {
    pub light: i32,
    pub temperature: i32,
    pub moisture: i32,
}
```

And the `compute_environmental_scores` function takes `&BiomeEnvironment` instead. Then in `biome.rs`, the `EnvironmentScores` struct will need a conversion:
```rust
impl From<EnvironmentScores> for game_core::weather::BiomeEnvironment {
    fn from(e: EnvironmentScores) -> Self {
        Self { light: e.light, temperature: e.temperature, moisture: e.moisture }
    }
}
```

Actually, to avoid circular dependencies, the cleanest approach is to define `BiomeEnvironment` in `game-core` and have `game-world`'s `BiomeRule` use `game_core::weather::BiomeEnvironment` as its `environment` field. This requires `game-world` to depend on `game-core` (which it already does via `use game_core::...` in behavior.rs and spawner.rs).

Update `crates/world/src/biome.rs`:
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct BiomeEnvironment {
    #[serde(default)]
    pub light: i32,
    #[serde(default)]
    pub temperature: i32,
    #[serde(default)]
    pub moisture: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeRule {
    pub biome: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub elevation: Option<(f32, f32)>,
    pub moisture: Option<(f32, f32)>,
    pub temperature: Option<(f32, f32)>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub priority: u32,
    #[serde(default)]
    pub environment: BiomeEnvironment,
}
```

And `game-core` re-exports it. In `crates/core/src/weather.rs`, import:
```rust
pub use crate::world_overview::BiomeEnvironment;
```

Wait — `world_overview` is in `game-core` already. Let's just put `BiomeEnvironment` in `crates/core/src/weather.rs` itself and have `game-world` use it from `game_core`.

Update `crates/world/Cargo.toml` to ensure it depends on `game-core` (it likely already does — confirmed: `spawner.rs` has `use game_core::...`):
```toml
game-core = { path = "../core" }
```

Then in `crates/world/src/biome.rs`:
```rust
use game_core::weather::BiomeEnvironment;

#[derive(Debug, Clone, Deserialize)]
pub struct BiomeRule {
    pub biome: String,
    pub glyph: char,
    pub color: [u8; 3],
    pub elevation: Option<(f32, f32)>,
    pub moisture: Option<(f32, f32)>,
    pub temperature: Option<(f32, f32)>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub priority: u32,
    #[serde(default)]
    pub environment: BiomeEnvironment,
}
```

And in `crates/world/src/lib.rs`, export it:
```rust
pub use biome::{BiomeClassifier, BiomeRule};
```

And `game-core` re-exports from weather:
```rust
pub use weather::{..., BiomeEnvironment, EnvironmentalScores, compute_environmental_scores, resolve_scores_to_tags};
```

#### Step 6b: Tests

```rust
    #[test]
    fn compute_scores_basic() {
        let base = BiomeEnvironment { light: 70, temperature: 50, moisture: 50 };
        let mods = HashMap::from([("moisture".into(), 50)]);
        let scores = compute_environmental_scores(&base, &mods, TimeOfDay::Day);
        assert_eq!(scores.light, 70);
        assert_eq!(scores.temperature, 50);
        assert_eq!(scores.moisture, 100);
    }

    #[test]
    fn compute_scores_night_reduces_light() {
        let base = BiomeEnvironment { light: 70, temperature: 50, moisture: 50 };
        let mods = HashMap::new();
        let scores = compute_environmental_scores(&base, &mods, TimeOfDay::Night);
        assert_eq!(scores.light, 0);
    }

    #[test]
    fn compute_scores_clamped() {
        let base = BiomeEnvironment { light: 50, temperature: 80, moisture: 10 };
        let mods = HashMap::from([("temperature".into(), 50), ("moisture".into(), -20)]);
        let scores = compute_environmental_scores(&base, &mods, TimeOfDay::Day);
        let clamped = scores.clamp();
        assert_eq!(clamped.temperature, 100);
        assert_eq!(clamped.moisture, 0);
    }

    #[test]
    fn resolve_scores_dark() {
        let scores = EnvironmentalScores { light: 10, temperature: 50, moisture: 50 };
        let reg = make_test_registry();
        let tags = resolve_scores_to_tags(&scores, &reg);
        let dark_id = reg.tag_id("DARK").unwrap();
        assert!(tags.contains(&dark_id));
    }

    #[test]
    fn resolve_scores_hot() {
        let scores = EnvironmentalScores { light: 70, temperature: 90, moisture: 10 };
        let reg = make_test_registry();
        let tags = resolve_scores_to_tags(&scores, &reg);
        let hot_id = reg.tag_id("HOT").unwrap();
        let dry_id = reg.tag_id("DRY").unwrap();
        assert!(tags.contains(&hot_id));
        assert!(tags.contains(&dry_id));
    }

    fn make_test_registry() -> TagRegistry {
        let mut builder = TagRegistryBuilder::new();
        let light = builder.add_archetype("light", "Light", Exclusivity::Mutual);
        builder.add_tag(light, "DARK", vec![], vec![], None, None, None, None, None, Some([0, 20]), None, None).unwrap();
        builder.add_tag(light, "DIM", vec![], vec![], None, None, None, None, None, Some([20, 40]), None, None).unwrap();
        builder.add_tag(light, "BRIGHT", vec![], vec![], None, None, None, None, None, Some([60, 100]), None, None).unwrap();
        let temp = builder.add_archetype("temperature", "Temperature", Exclusivity::Mutual);
        builder.add_tag(temp, "FREEZING", vec![], vec![], None, None, None, None, None, Some([0, 15]), None, None).unwrap();
        builder.add_tag(temp, "COLD", vec![], vec![], None, None, None, None, None, Some([15, 35]), None, None).unwrap();
        builder.add_tag(temp, "NEUTRAL", vec![], vec![], None, None, None, None, None, Some([35, 65]), None, None).unwrap();
        builder.add_tag(temp, "WARM", vec![], vec![], None, None, None, None, None, Some([65, 85]), None, None).unwrap();
        builder.add_tag(temp, "HOT", vec![], vec![], None, None, None, None, None, Some([85, 100]), None, None).unwrap();
        let moist = builder.add_archetype("moisture", "Moisture", Exclusivity::Mutual);
        builder.add_tag(moist, "DRY", vec![], vec![], None, None, None, None, None, Some([0, 20]), None, None).unwrap();
        builder.add_tag(moist, "DAMP", vec![], vec![], None, None, None, None, None, Some([20, 40]), None, None).unwrap();
        builder.add_tag(moist, "WET", vec![], vec![], None, None, None, None, None, Some([40, 70]), None, None).unwrap();
        builder.add_tag(moist, "SOAKED", vec![], vec![], None, None, None, None, None, Some([70, 100]), None, None).unwrap();
        builder.build().unwrap()
    }
```

Need imports in test module:
```rust
    use game_tags::{TagRegistryBuilder, Exclusivity};
```

And at the top of the file, import:
```rust
use game_tags::{TagRegistry, TagId};
```

Run: `cargo test -p game-core -- weather::tests`
Expected: All tests pass.

**Commit:** `feat: implement environmental score computation and threshold resolution`

---

### Task 7: Implement apply_environmental_tags pipeline

**Files:**
- Modify: `crates/core/src/weather.rs` — add `apply_environmental_tags` function
- Modify: `src/game/mod.rs` — call `apply_environmental_tags` in `finish_npc_turn`

#### Step 7a: Add apply_environmental_tags function

Add to `crates/core/src/weather.rs`:
```rust
use bevy_ecs::prelude::*;

pub fn apply_environmental_tags(
    world: &mut World,
    registry: &TagRegistry,
) {
    let (weather_modifiers, time_of_day, weather_descriptive_tags, applied_last_turn) = {
        let ws = world.get_resource::<WeatherState>();
        let wc = world.get_resource::<WeatherContext>();
        let (mods, time, desc_tags) = match ws {
            Some(s) => (
                s.active_weather.as_ref().map(|w| w.modifiers.clone()).unwrap_or_default(),
                s.time,
                s.active_weather.as_ref().map(|w| w.tags.clone()).unwrap_or_default(),
            ),
            None => (HashMap::new(), TimeOfDay::Day, Vec::new()),
        };
        let applied = wc.map(|c| c.applied_tags.clone()).unwrap_or_default();
        (mods, time, desc_tags, applied)
    };

    let new_tags: Vec<TagId> = {
        let mut resolved = Vec::new();
        for name in &weather_descriptive_tags {
            if let Some(tid) = registry.tag_id(name) {
                resolved.push(tid);
            }
        }
        resolved
    };

    let blocks_weather_id = registry.tag_id("BLOCKS_WEATHER");
    let indoors_id = registry.tag_id("INDOORS");

    // Remove stale tags from last turn
    for &tag_id in &applied_last_turn {
        let mut query = world.query::<&mut Tags>();
        for mut tags in query.iter_mut(world) {
            tags.remove_tag(tag_id, registry);
        }
    }

    // Compute scores and resolve for tiles
    let tile_data: Vec<(Entity, Tags)> = {
        let map = world.get_resource::<crate::WorldMap>();
        match map {
            Some(_map) => {
                let mut q = world.query::<(Entity, &Tags, &crate::Position)>();
                q.iter(world)
                    .map(|(e, t, _)| (e, t.clone()))
                    .collect()
            }
            None => Vec::new(),
        }
    };

    // For now, apply weather-derived tags to all entities with WeatherSensitive
    // Tile-specific biome base scores will come in a follow-up pass
    let mut all_applied: Vec<TagId> = new_tags.clone();

    // Apply descriptive weather tags to WeatherSensitive entities
    let mut query = world.query_filtered::<&mut Tags, With<crate::components::Creature>>();
    for mut tags in query.iter_mut(world) {
        for &tag_id in &new_tags {
            tags.add_tag(tag_id, TagValue::None, registry);
        }
    }

    // Store applied tags for next turn cleanup
    if let Some(mut wc) = world.get_resource_mut::<WeatherContext>() {
        wc.applied_tags = all_applied.clone();
        wc.active_modifiers = weather_modifiers.clone();
    }
}
```

Wait — we need to import `With` and `Creature`. The function references `crate::components::Creature` and `crate::Position` but `weather.rs` is in `game-core` which has those. But it also references `WorldMap` from `game-world`, creating a circular dependency. We need to avoid that.

The solution: `apply_environmental_tags` should live in the binary crate (`src/game/mod.rs` or a new `src/weather_pipeline.rs`), not in `game-core`, because it needs access to both `game-core` and `game-world` types.

Create `src/weather_pipeline.rs`:

```rust
use bevy_ecs::prelude::*;
use game_core::weather::{
    WeatherState, WeatherContext, BiomeEnvironment, EnvironmentalScores,
    compute_environmental_scores, resolve_scores_to_tags,
};
use game_core::{Creature, Position};
use game_tags::{TagRegistry, Tags, TagId, TagValue};
use game_world::WorldMap;

pub fn apply_environmental_tags(world: &mut World) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };

    let (weather_modifiers, time_of_day, weather_descriptive_tags, applied_last_turn) = {
        let ws = world.get_resource::<WeatherState>();
        let wc = world.get_resource::<WeatherContext>();
        let (mods, time, desc_tags) = match ws {
            Some(s) => (
                s.active_weather.as_ref().map(|w| w.modifiers.clone()).unwrap_or_default(),
                s.time,
                s.active_weather.as_ref().map(|w| w.tags.clone()).unwrap_or_default(),
            ),
            None => (std::collections::HashMap::new(), game_core::TimeOfDay::Day, Vec::new()),
        };
        let applied = wc.map(|c| c.applied_tags.clone()).unwrap_or_default();
        (mods, time, desc_tags, applied)
    };

    let mut new_desc_tag_ids: Vec<TagId> = Vec::new();
    for name in &weather_descriptive_tags {
        if let Some(tid) = registry.tag_id(name) {
            new_desc_tag_ids.push(tid);
        }
    }

    // Remove stale tags from last turn
    for &tag_id in &applied_last_turn {
        let mut q = world.query::<&mut Tags>();
        for mut tags in q.iter_mut(world) {
            tags.remove_tag(tag_id, &registry);
        }
    }

    // Apply descriptive weather tags to Creature entities (they are WeatherSensitive)
    let mut all_applied: Vec<TagId> = new_desc_tag_ids.clone();
    let mut query = world.query_filtered::<&mut Tags, With<Creature>>();
    for mut tags in query.iter_mut(world) {
        for &tag_id in &new_desc_tag_ids {
            tags.add_tag(tag_id, TagValue::None, &registry);
        }
    }

    // Store applied tags for next turn cleanup
    if let Some(mut wc) = world.get_resource_mut::<WeatherContext>() {
        wc.applied_tags = all_applied;
        wc.active_modifiers = weather_modifiers;
    }
}
```

And in `src/game/mod.rs`, add `mod weather_pipeline;` and call it:

```rust
mod weather_pipeline;
```

Update `finish_npc_turn` (around line 469-498):
```rust
fn finish_npc_turn(
    mut game_world: ResMut<GameWorld>,
    mut turn_state: ResMut<GameTurnState>,
) {
    if !turn_state.processing_npcs {
        return;
    }
    process_npc_turns(&mut game_world.0);
    crate::status::process_status_effects(&mut game_world.0);

    turn_state.processing_npcs = false;
    if let Some(mut tc) = game_world.0.get_resource_mut::<TurnCounter>() {
        tc.increment();
    }

    // Advance weather
    let weather_tags = {
        if let Some(mut ws) = game_world.0.get_resource_mut::<WeatherState>() {
            ws.advance_time();
            Some(weather_tags_for_context(&ws))
        } else {
            None
        }
    };
    if let Some(tags) = weather_tags {
        if let Some(mut wc) = game_world.0.get_resource_mut::<WeatherContext>() {
            wc.tags = tags;
        }
    }

    // Apply environmental tags to entities
    crate::weather_pipeline::apply_environmental_tags(&mut game_world.0);
}
```

#### Tests

Create `src/weather_pipeline.rs` with a test module:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use game_core::weather::{WeatherDef, WeatherState};
    use game_tags::load_tag_registry;

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");

    fn make_world() -> World {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        world.insert_resource(registry);
        world.insert_resource(WeatherState::default());
        world.insert_resource(WeatherContext::default());
        world
    }

    #[test]
    fn apply_tags_with_no_weather_no_crash() {
        let mut world = make_world();
        apply_environmental_tags(&mut world);
    }

    #[test]
    fn apply_tags_with_rain_applies_rainy() {
        let mut world = make_world();
        let registry = world.resource::<TagRegistry>().clone();
        let rainy_id = registry.tag_id("RAINY").unwrap();

        let rain = WeatherDef {
            name: "Rain".into(),
            weight: 10,
            duration: [5, 15],
            visibility: 0.6,
            modifiers: std::collections::HashMap::from([("moisture".into(), 50)]),
            tags: vec!["RAINY".into()],
        };
        world.insert_resource(WeatherState::new(vec![rain]));

        let entity = world.spawn((
            Creature,
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Health { current: 50, max: 50 },
            game_core::Name("Test".into()),
            Tags::new(registry.tag_count()),
        )).id();

        apply_environmental_tags(&mut world);

        let tags = world.get::<Tags>(entity).unwrap();
        assert!(tags.has(rainy_id), "Creature should have RAINY tag after weather application");
    }

    #[test]
    fn stale_tags_removed_on_next_apply() {
        let mut world = make_world();
        let registry = world.resource::<TagRegistry>().clone();
        let rainy_id = registry.tag_id("RAINY").unwrap();

        let rain = WeatherDef {
            name: "Rain".into(),
            weight: 10,
            duration: [5, 15],
            visibility: 0.6,
            modifiers: std::collections::HashMap::new(),
            tags: vec!["RAINY".into()],
        };
        world.insert_resource(WeatherState::new(vec![rain]));

        let entity = world.spawn((
            Creature,
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Health { current: 50, max: 50 },
            game_core::Name("Test".into()),
            Tags::new(registry.tag_count()),
        )).id();

        apply_environmental_tags(&mut world);
        assert!(world.get::<Tags>(entity).unwrap().has(rainy_id));

        // Change weather to clear
        let clear = WeatherDef {
            name: "Clear".into(),
            weight: 30,
            duration: [8, 25],
            visibility: 1.0,
            modifiers: std::collections::HashMap::new(),
            tags: vec![],
        };
        world.insert_resource(WeatherState::new(vec![clear]));

        apply_environmental_tags(&mut world);
        assert!(!world.get::<Tags>(entity).unwrap().has(rainy_id), "RAINY should be removed when weather changes");
    }
}
```

Run: `cargo test -- weather_pipeline`
Expected: All tests pass.

**Commit:** `feat: implement apply_environmental_tags pipeline`

---

### Task 8: Add WeatherSensitive component

**Files:**
- Modify: `crates/core/src/components.rs` — add `WeatherSensitive` component
- Modify: `crates/core/src/lib.rs` — export `WeatherSensitive`
- Modify: `crates/world/src/spawner.rs` — add `WeatherSensitive` to spawned creatures
- Modify: `src/weather_pipeline.rs` — gate tag application to `WeatherSensitive` entities

#### Step 8a: Add component

In `crates/core/src/components.rs`:
```rust
#[derive(Component)]
pub struct WeatherSensitive;
```

In `crates/core/src/lib.rs`, add to exports:
```rust
pub use components::{
    ArmorProtection, Creature, Equipment, EquipmentSlot, Glyph, Health, Inventory, Item,
    ItemEffects, LootContainer, MessageLog, Name, Player, Position, WeaponDamage,
    WeatherSensitive,
};
```

#### Step 8b: Add to spawner

In `crates/world/src/spawner.rs`, add `WeatherSensitive` to creature entity spawns.

In `spawn_location_entities` (around line 250-259), add to the spawn tuple:
```rust
            let creature_entity = world.spawn((
                pos,
                Glyph { char: rule.glyph, color: (rule.color[0], rule.color[1], rule.color[2]) },
                Health { current: hp, max: hp },
                entity_tags.clone(),
                Name(rule.name.clone()),
                Creature,
                BehaviorState { home_pos: Some(pos) },
                NpcEmotionalState::default(),
                WeatherSensitive,
            )).id();
```

In `spawn_wild_entities` (around lines 374-378), add to each spawn tuple variant:
```rust
                        (Some(f), true) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, BehaviorState { home_pos: Some(position) }, *f, QuestGiver, NpcEmotionalState::default(), WeatherSensitive)).id(),
                        (Some(f), false) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, BehaviorState { home_pos: Some(position) }, *f, NpcEmotionalState::default(), WeatherSensitive)).id(),
                        (None, true) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, BehaviorState { home_pos: Some(position) }, QuestGiver, NpcEmotionalState::default(), WeatherSensitive)).id(),
                        (None, false) => world.spawn((position, glyph, Health { current: max_hp, max: max_hp }, entity_tags.clone(), Name(rule.name.clone()), Creature, BehaviorState { home_pos: Some(position) }, NpcEmotionalState::default(), WeatherSensitive)).id(),
```

Add import at top of spawner.rs:
```rust
use game_core::{..., WeatherSensitive};
```

#### Step 8c: Update weather_pipeline to use WeatherSensitive

In `src/weather_pipeline.rs`, change the creature query:
```rust
    let mut query = world.query_filtered::<&mut Tags, (With<Creature>, With<game_core::WeatherSensitive>)>();
```

And update the test entities to include `WeatherSensitive`:
```rust
        let entity = world.spawn((
            Creature,
            game_core::WeatherSensitive,
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Health { current: 50, max: 50 },
            game_core::Name("Test".into()),
            Tags::new(registry.tag_count()),
        )).id();
```

Add the player entity WeatherSensitive in the world gen or game setup. In `src/game/mod.rs`, where the player is spawned (search for `Player` component insertion), add `WeatherSensitive`. Find the player spawn site:

```bash
grep -n "Player" src/world_gen.rs | head -20
```

Add `WeatherSensitive` to the player entity spawn.

#### Tests

Run: `cargo test -p game-world -- spawner`
Expected: All existing tests pass (spawner tests create creatures without WeatherSensitive in test spawn rules, but the test queries don't check for it, so they should still pass).

Run: `cargo test -- weather_pipeline`
Expected: All tests pass.

**Commit:** `feat: add WeatherSensitive component to creatures and player`

---

### Task 9: Wire get_sense_range to visibility

**Files:**
- Modify: `crates/world/src/behavior.rs` — update `get_sense_range` to accept visibility multiplier, handle DARKVISION and THERMAL_SENSE
- Modify: `crates/world/src/behavior.rs` — update call sites of `get_sense_range`

#### Step 9a: Update get_sense_range signature

In `crates/world/src/behavior.rs`, change:
```rust
fn get_sense_range(tags: &Tags, registry: &TagRegistry) -> u32 {
    if let Some(id) = registry.tag_id("SIGHT")
        && tags.has(id)
        && let Some(val) = tags.get_value(id)
        && let Some(mag) = val.magnitude()
    {
        return mag as u32;
    }
    8
}
```

To:
```rust
fn get_sense_range(tags: &Tags, registry: &TagRegistry, visibility: f32) -> u32 {
    if let Some(id) = registry.tag_id("THERMAL_SENSE") && tags.has(id) {
        if let Some(val) = tags.get_value(id) {
            if let Some(mag) = val.magnitude() {
                return mag as u32;
            }
        }
        if let Some(r) = registry.tag_by_id(id).range {
            return r;
        }
        return 4;
    }

    if let Some(id) = registry.tag_id("SIGHT")
        && tags.has(id)
        && let Some(val) = tags.get_value(id)
        && let Some(mag) = val.magnitude()
    {
        let darkvision_id = registry.tag_id("DARKVISION");
        let has_darkvision = darkvision_id.is_some_and(|dv| tags.has(dv));
        let effective_visibility = if has_darkvision { visibility.max(0.5) } else { visibility };
        return (mag as f32 * effective_visibility).max(1.0) as u32;
    }

    let darkvision_id = registry.tag_id("DARKVISION");
    let has_darkvision = darkvision_id.is_some_and(|dv| tags.has(dv));
    let effective_visibility = if has_darkvision { visibility.max(0.5) } else { visibility };
    (8.0 * effective_visibility).max(1.0) as u32
}
```

#### Step 9b: Find all call sites of get_sense_range

Search for `get_sense_range` calls in behavior.rs. Each call needs the visibility parameter. The visibility comes from `WeatherState.effective_visibility()` which needs to be threaded through.

The function `process_npc_turns` takes `&mut World`, so `WeatherState` can be read from the world resource. Find where `get_sense_range` is called and thread the visibility.

Read the behavior.rs file to find all call sites of `get_sense_range`:
```rust
// The function is called inside process_npc_turns
// It's a private function, so all calls are within behavior.rs
```

At the top of `process_npc_turns` or the relevant function, extract visibility:
```rust
let visibility = world.get_resource::<game_core::WeatherState>()
    .map(|ws| ws.effective_visibility())
    .unwrap_or(1.0);
```

Pass `visibility` to each `get_sense_range` call as the third argument.

#### Tests

```rust
    #[test]
    fn test_get_sense_range_full_visibility() {
        let registry = make_test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let sight_id = registry.tag_id("SIGHT").unwrap();
        tags.add_tag(sight_id, TagValue::Magnitude(10.0), &registry);
        let range = get_sense_range(&tags, &registry, 1.0);
        assert_eq!(range, 10);
    }

    #[test]
    fn test_get_sense_range_reduced_visibility() {
        let registry = make_test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let sight_id = registry.tag_id("SIGHT").unwrap();
        tags.add_tag(sight_id, TagValue::Magnitude(10.0), &registry);
        let range = get_sense_range(&tags, &registry, 0.5);
        assert_eq!(range, 5);
    }

    #[test]
    fn test_get_sense_range_darkvision_ignores_low_light() {
        let registry = make_test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let sight_id = registry.tag_id("SIGHT").unwrap();
        let dv_id = registry.tag_id("DARKVISION").unwrap();
        tags.add_tag(sight_id, TagValue::Magnitude(10.0), &registry);
        tags.add_tag(dv_id, TagValue::None, &registry);
        let range = get_sense_range(&tags, &registry, 0.1);
        assert!(range >= 5, "darkvision should guarantee at least 50% range, got {}", range);
    }

    #[test]
    fn test_get_sense_range_thermal_ignores_visibility() {
        let registry = make_test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let thermal_id = registry.tag_id("THERMAL_SENSE").unwrap();
        tags.add_tag(thermal_id, TagValue::None, &registry);
        let range = get_sense_range(&tags, &registry, 0.1);
        assert!(range > 0, "thermal sense should work regardless of visibility");
    }
```

Note: These tests require a registry with SIGHT, DARKVISION, and THERMAL_SENSE tags. Use the real tags.toml via `include_str!` or build a minimal test registry.

Run: `cargo test -p game-world -- behavior::tests`
Expected: All tests pass.

**Commit:** `feat: wire get_sense_range to weather visibility with darkvision/thermal override`

---

### Task 10: Convert string weather checks to TagId

**Files:**
- Modify: `crates/core/src/encounters.rs` — replace string checks with TagId
- Modify: `crates/core/src/npc_action.rs` — convert `EnvironmentContext.weather_tags` to `Vec<TagId>`
- Modify: `src/interact/talk.rs` — convert is_night check to TagId

#### Step 10a: encounters.rs

In `roll_encounter`, replace:
```rust
    if let Some(wc) = weather_context {
        if wc.tags.iter().any(|t| t == "DARK") { chance += 0.05; }
        if wc.tags.iter().any(|t| t == "STORMY") { chance += 0.03; }
        if wc.tags.iter().any(|t| t == "REDUCED_VISIBILITY") { chance += 0.02; }
    }
```

With:
```rust
    if let Some(wc) = weather_context {
        let registry = world.get_resource::<game_tags::TagRegistry>();
        if let Some(reg) = registry {
            let dark_id = reg.tag_id("DARK");
            let stormy_id = reg.tag_id("STORMY");
            let reduced_id = reg.tag_id("REDUCED_VISIBILITY");
            for tag_id in &wc.applied_tags {
                if dark_id.is_some_and(|d| *tag_id == d) { chance += 0.05; }
                if stormy_id.is_some_and(|s| *tag_id == s) { chance += 0.03; }
                if reduced_id.is_some_and(|r| *tag_id == r) { chance += 0.02; }
            }
        } else {
            if wc.tags.iter().any(|t| t == "DARK") { chance += 0.05; }
            if wc.tags.iter().any(|t| t == "STORMY") { chance += 0.03; }
            if wc.tags.iter().any(|t| t == "REDUCED_VISIBILITY") { chance += 0.02; }
        }
    }
```

This provides a graceful fallback: if the registry is available, use TagId; otherwise fall back to string matching.

#### Step 10b: npc_action.rs

The `EnvironmentContext` struct uses `Vec<String>` for `weather_tags`. We'll add a parallel `weather_tag_ids: Vec<TagId>` field:

```rust
#[derive(Debug, Clone, Default)]
pub struct EnvironmentContext {
    pub weather_tags: Vec<String>,
    pub weather_tag_ids: Vec<game_tags::TagId>,
    pub near_location: Option<String>,
    pub is_night: bool,
}
```

In `score_action`, update the env modifier check to use tag IDs when available:
```rust
    for em in &def.env_modifiers {
        if !environment.weather_tag_ids.is_empty() {
            if let Some(tid) = registry.tag_id(&em.tag) {
                if environment.weather_tag_ids.contains(&tid) { score *= em.mult; }
            }
        } else {
            if environment.weather_tags.contains(&em.tag) { score *= em.mult; }
        }
    }
```

#### Step 10c: talk.rs

In `build_environment_context`, update the is_night detection:
```rust
pub fn build_environment_context(ecs_world: &World) -> EnvironmentContext {
    let weather_tags = ecs_world.get_resource::<WeatherContext>()
        .map(|wc| wc.tags.clone()).unwrap_or_default();
    let is_night = weather_tags.iter().any(|t| t == "DARK" || t == "DIM");
    let weather_tag_ids = ecs_world.get_resource::<WeatherContext>()
        .map(|wc| wc.applied_tags.clone()).unwrap_or_default();
    EnvironmentContext {
        weather_tags,
        weather_tag_ids,
        near_location: None,
        is_night,
    }
}
```

#### Tests

Run: `cargo test -p game-core -- encounters::tests`
Expected: All existing tests pass.

Run: `cargo test -p game-core -- npc_action::tests`
Expected: All existing tests pass.

Run: `cargo build`
Expected: Build succeeds.

**Commit:** `refactor: convert string weather checks to TagId-based lookups`

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Register environmental tags + threshold field | tags.toml, registry.rs, definition.rs, loader.rs |
| 2 | Base environmental scores for biomes | biome_rules.toml, biome.rs |
| 3 | WeatherDef struct + TOML loader | weather.rs |
| 4 | 8 weather TOML files | assets/config/weather/*.toml |
| 5 | Refactor WeatherState to use WeatherDef | weather.rs, lib.rs, game/mod.rs |
| 6 | Score computation + threshold resolution | weather.rs |
| 7 | Apply environmental tags pipeline | weather_pipeline.rs, game/mod.rs |
| 8 | WeatherSensitive component | components.rs, spawner.rs, weather_pipeline.rs |
| 9 | Wire get_sense_range to visibility | behavior.rs |
| 10 | Convert string checks to TagId | encounters.rs, npc_action.rs, talk.rs |
