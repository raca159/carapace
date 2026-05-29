use bevy_ecs::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default, Component)]
pub struct BiomeEnvironment {
    #[serde(default)]
    pub light: u32,
    #[serde(default)]
    pub temperature: u32,
    #[serde(default)]
    pub moisture: u32,
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

#[derive(Clone)]
pub struct BiomeClassifier {
    rules: Vec<BiomeRule>,
}

impl BiomeClassifier {
    pub fn new(mut rules: Vec<BiomeRule>) -> Self {
        rules.sort_by_key(|b| std::cmp::Reverse(b.priority));
        Self { rules }
    }

    pub fn classify(&self, elevation: f32, moisture: f32, temperature: f32) -> Option<&BiomeRule> {
        self.rules.iter().find(|rule| {
            let elev_ok = rule
                .elevation
                .is_none_or(|(min, max)| elevation >= min && elevation <= max);
            let moist_ok = rule
                .moisture
                .is_none_or(|(min, max)| moisture >= min && moisture <= max);
            let temp_ok = rule
                .temperature
                .is_none_or(|(min, max)| temperature >= min && temperature <= max);
            elev_ok && moist_ok && temp_ok
        })
    }

    pub fn rules(&self) -> &[BiomeRule] {
        &self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ocean_rule() -> BiomeRule {
        BiomeRule {
            biome: "OCEAN".into(),
            glyph: '~',
            color: [0, 0, 128],
            elevation: Some((0.0, 0.20)),
            moisture: None,
            temperature: None,
            tags: vec![],
            priority: 100,
            environment: BiomeEnvironment::default(),
        }
    }

    fn desert_rule() -> BiomeRule {
        BiomeRule {
            biome: "DESERT".into(),
            glyph: '.',
            color: [237, 201, 175],
            elevation: Some((0.35, 0.70)),
            moisture: Some((0.0, 0.3)),
            temperature: Some((0.7, 1.0)),
            tags: vec![],
            priority: 50,
            environment: BiomeEnvironment::default(),
        }
    }

    fn fallback_rule() -> BiomeRule {
        BiomeRule {
            biome: "GRASSLAND".into(),
            glyph: '.',
            color: [144, 238, 144],
            elevation: Some((0.35, 0.70)),
            moisture: None,
            temperature: None,
            tags: vec![],
            priority: 1,
            environment: BiomeEnvironment::default(),
        }
    }

    #[test]
    fn test_classify_ocean() {
        let classifier = BiomeClassifier::new(vec![ocean_rule()]);
        let result = classifier.classify(0.1, 0.5, 0.5);
        assert_eq!(result.unwrap().biome, "OCEAN");
    }

    #[test]
    fn test_classify_priority() {
        let classifier = BiomeClassifier::new(vec![fallback_rule(), desert_rule()]);
        let result = classifier.classify(0.5, 0.1, 0.9);
        assert_eq!(result.unwrap().biome, "DESERT");
    }

    #[test]
    fn test_classify_fallback() {
        let classifier = BiomeClassifier::new(vec![ocean_rule(), fallback_rule()]);
        let result = classifier.classify(0.5, 0.5, 0.5);
        assert_eq!(result.unwrap().biome, "GRASSLAND");
    }

    #[test]
    fn test_no_match() {
        let classifier = BiomeClassifier::new(vec![ocean_rule()]);
        let result = classifier.classify(0.5, 0.5, 0.5);
        assert!(result.is_none());
    }
}
