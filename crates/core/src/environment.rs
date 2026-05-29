use std::collections::HashMap;

use game_tags::{TagId, TagRegistry, Tags, TagValue};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EnvironmentalScores {
    pub light: u32,
    pub temperature: u32,
    pub moisture: u32,
}

impl EnvironmentalScores {
    pub fn compute(
        base: EnvironmentalScores,
        weather_modifiers: &HashMap<String, i32>,
        time: &crate::TimeOfDay,
    ) -> Self {
        let light_mod = *weather_modifiers.get("light").unwrap_or(&0);
        let temp_mod = *weather_modifiers.get("temperature").unwrap_or(&0);
        let moist_mod = *weather_modifiers.get("moisture").unwrap_or(&0);

        let time_light_mod = match time {
            crate::TimeOfDay::Night => -80i32,
            crate::TimeOfDay::Dusk | crate::TimeOfDay::Dawn => -40i32,
            crate::TimeOfDay::Day => 0i32,
        };

        Self {
            light: (base.light as i32 + light_mod + time_light_mod).clamp(0, 100) as u32,
            temperature: (base.temperature as i32 + temp_mod).clamp(0, 100) as u32,
            moisture: (base.moisture as i32 + moist_mod).clamp(0, 100) as u32,
        }
    }

    pub fn resolve_tags(&self, registry: &TagRegistry) -> Vec<TagId> {
        let mut result = Vec::new();
        for tag_def in registry.all_tags() {
            if let Some([low, high]) = tag_def.threshold {
                let arch = registry.archetype_by_id(tag_def.archetype);
                let score = match arch.name.as_str() {
                    "light" => self.light,
                    "temperature" => self.temperature,
                    "moisture" => self.moisture,
                    _ => continue,
                };
                if score >= low && score <= high {
                    result.push(tag_def.id);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::{Exclusivity, TagRegistryBuilder};

    fn make_registry() -> TagRegistry {
        let mut builder = TagRegistryBuilder::new();
        let light_arch = builder.add_archetype("light", "Light", Exclusivity::Mutual);
        let temp_arch = builder.add_archetype("temperature", "Temperature", Exclusivity::Mutual);
        let moist_arch = builder.add_archetype("moisture", "Moisture", Exclusivity::Mutual);

        builder.add_tag(light_arch, "DARK", vec![], vec![], None, None, None, None, None, Some([0, 20]), None, None).unwrap();
        builder.add_tag(light_arch, "DIM", vec![], vec![], None, None, None, None, None, Some([20, 40]), None, None).unwrap();
        builder.add_tag(light_arch, "BRIGHT", vec![], vec![], None, None, None, None, None, Some([60, 100]), None, None).unwrap();

        builder.add_tag(temp_arch, "FREEZING", vec![], vec![], None, None, None, None, None, Some([0, 15]), None, None).unwrap();
        builder.add_tag(temp_arch, "COLD", vec![], vec![], None, None, None, None, None, Some([15, 35]), None, None).unwrap();
        builder.add_tag(temp_arch, "NEUTRAL", vec![], vec![], None, None, None, None, None, Some([35, 65]), None, None).unwrap();
        builder.add_tag(temp_arch, "WARM", vec![], vec![], None, None, None, None, None, Some([65, 85]), None, None).unwrap();
        builder.add_tag(temp_arch, "HOT", vec![], vec![], None, None, None, None, None, Some([85, 100]), None, None).unwrap();

        builder.add_tag(moist_arch, "DRY", vec![], vec![], None, None, None, None, None, Some([0, 20]), None, None).unwrap();
        builder.add_tag(moist_arch, "DAMP", vec![], vec![], None, None, None, None, None, Some([20, 40]), None, None).unwrap();
        builder.add_tag(moist_arch, "WET", vec![], vec![], None, None, None, None, None, Some([40, 70]), None, None).unwrap();
        builder.add_tag(moist_arch, "SOAKED", vec![], vec![], None, None, None, None, None, Some([70, 100]), None, None).unwrap();

        builder.build().unwrap()
    }

    fn make_tag_names(registry: &TagRegistry, tags: &[TagId]) -> Vec<String> {
        tags.iter().map(|t| registry.tag_by_id(*t).name.clone()).collect()
    }

    #[test]
    fn test_desert_day_scores() {
        let base = EnvironmentalScores { light: 85, temperature: 85, moisture: 5 };
        let modifiers = HashMap::new();
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Day);
        assert_eq!(scores.light, 85);
        assert_eq!(scores.temperature, 85);
        assert_eq!(scores.moisture, 5);
    }

    #[test]
    fn test_rain_modifier() {
        let base = EnvironmentalScores { light: 70, temperature: 50, moisture: 30 };
        let mut modifiers = HashMap::new();
        modifiers.insert("moisture".to_string(), 50);
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Day);
        assert_eq!(scores.moisture, 80);
        assert_eq!(scores.light, 70);
    }

    #[test]
    fn test_night_reduces_light() {
        let base = EnvironmentalScores { light: 70, temperature: 50, moisture: 50 };
        let modifiers = HashMap::new();
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Night);
        assert_eq!(scores.light, 0);
    }

    #[test]
    fn test_clamp_lower_bound() {
        let base = EnvironmentalScores { light: 10, temperature: 10, moisture: 10 };
        let mut modifiers = HashMap::new();
        modifiers.insert("temperature".to_string(), -40);
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Night);
        assert_eq!(scores.temperature, 0);
        assert_eq!(scores.light, 0);
    }

    #[test]
    fn test_clamp_upper_bound() {
        let base = EnvironmentalScores { light: 90, temperature: 90, moisture: 90 };
        let mut modifiers = HashMap::new();
        modifiers.insert("moisture".to_string(), 50);
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Day);
        assert_eq!(scores.moisture, 100);
    }

    #[test]
    fn test_desert_day_tags() {
        let registry = make_registry();
        let base = EnvironmentalScores { light: 85, temperature: 85, moisture: 5 };
        let modifiers = HashMap::new();
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Day);
        let tags = scores.resolve_tags(&registry);
        let tag_names: Vec<String> = tags.iter()
            .filter_map(|t| Some(registry.tag_by_id(*t).name.clone()))
            .collect();
        assert!(tag_names.contains(&"BRIGHT".to_string()), "desert day should be BRIGHT, got {:?}", tag_names);
        assert!(tag_names.contains(&"HOT".to_string()), "desert should be HOT, got {:?}", tag_names);
        assert!(tag_names.contains(&"DRY".to_string()), "desert should be DRY, got {:?}", tag_names);
    }

    #[test]
    fn test_rain_forest_tags() {
        let registry = make_registry();
        let base = EnvironmentalScores { light: 45, temperature: 75, moisture: 80 };
        let mut modifiers = HashMap::new();
        modifiers.insert("moisture".to_string(), 50);
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Dusk);
        let tags = scores.resolve_tags(&registry);
        let tag_names: Vec<String> = tags.iter()
            .filter_map(|t| Some(registry.tag_by_id(*t).name.clone()))
            .collect();
        assert!(tag_names.contains(&"DARK".to_string()), "dusk rainforest should be DARK (light=45-40=5), got {:?}", tag_names);
        assert!(tag_names.contains(&"WARM".to_string()), "tropical should be WARM, got {:?}", tag_names);
        assert!(tag_names.contains(&"SOAKED".to_string()), "rainforest+rain should be SOAKED, got {:?}", tag_names);
    }

    #[test]
    fn test_tundra_night_tags() {
        let registry = make_registry();
        let base = EnvironmentalScores { light: 55, temperature: 10, moisture: 30 };
        let modifiers = HashMap::new();
        let scores = EnvironmentalScores::compute(base, &modifiers, &crate::TimeOfDay::Night);
        let tags = scores.resolve_tags(&registry);
        let tag_names: Vec<String> = tags.iter()
            .filter_map(|t| Some(registry.tag_by_id(*t).name.clone()))
            .collect();
        assert!(tag_names.contains(&"DARK".to_string()));
        assert!(tag_names.contains(&"FREEZING".to_string()), "tundra temp=10 should be FREEZING, got {:?}", tag_names);
    }
}
