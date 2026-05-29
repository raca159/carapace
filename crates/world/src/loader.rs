use std::fs;

use serde::Deserialize;

use crate::biome::{BiomeClassifier, BiomeRule};
use crate::noise_gen::NoiseLayerConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct WorldGenConfig {
    pub width: u32,
    pub height: u32,
    pub latitude_weight: f32,
    pub elevation: NoiseLayerConfig,
    pub moisture: NoiseLayerConfig,
    pub temperature: NoiseLayerConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct BiomeRulesToml {
    rule: Vec<BiomeRule>,
}

pub fn load_world_config(path: &str) -> Result<WorldGenConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: WorldGenConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn load_world_config_str(content: &str) -> Result<WorldGenConfig, Box<dyn std::error::Error>> {
    let config: WorldGenConfig = toml::from_str(content)?;
    Ok(config)
}

pub fn load_biome_rules(
    content: &str,
) -> Result<BiomeClassifier, Box<dyn std::error::Error>> {
    let parsed: BiomeRulesToml = toml::from_str(content)?;
    Ok(BiomeClassifier::new(parsed.rule))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORLD_TOML: &str = r#"
width = 200
height = 200
latitude_weight = 0.4

[elevation]
name = "elevation"
frequency = 0.008
octaves = 6
persistence = 0.5
lacunarity = 2.0

[moisture]
name = "moisture"
frequency = 0.012
octaves = 5
persistence = 0.5
lacunarity = 2.0

[temperature]
name = "temperature"
frequency = 0.005
octaves = 4
persistence = 0.6
lacunarity = 2.0
"#;

    const BIOME_TOML: &str = r#"
[[rule]]
biome = "OCEAN_DEEP"
glyph = "~"
color = [0, 0, 128]
elevation = [0.0, 0.20]
priority = 100
tags = ["BIOME_DEEP_OCEAN"]

[[rule]]
biome = "GRASSLAND"
glyph = "."
color = [144, 238, 144]
elevation = [0.35, 0.70]
priority = 1
tags = ["BIOME_GRASSLAND"]
"#;

    #[test]
    fn test_load_world_config() {
        let config = load_world_config_str(WORLD_TOML).unwrap();
        assert_eq!(config.width, 200);
        assert_eq!(config.height, 200);
        assert_eq!(config.latitude_weight, 0.4);
        assert_eq!(config.elevation.octaves, 6);
    }

    #[test]
    fn test_load_biome_rules() {
        let classifier = load_biome_rules(BIOME_TOML).unwrap();
        let rule = classifier.classify(0.1, 0.5, 0.5);
        assert_eq!(rule.unwrap().biome, "OCEAN_DEEP");
    }

    #[test]
    fn test_biome_rules_sorted_by_priority() {
        let classifier = load_biome_rules(BIOME_TOML).unwrap();
        let rules = classifier.rules();
        assert_eq!(rules[0].biome, "OCEAN_DEEP");
        assert_eq!(rules[1].biome, "GRASSLAND");
    }
}
