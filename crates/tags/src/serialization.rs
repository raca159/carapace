use serde::{Deserialize, Serialize};

use crate::component::{TagValue, Tags};
use crate::registry::TagRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsSnapshot {
    pub tags: Vec<(String, TagValueSnapshot)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagValueSnapshot {
    None,
    Magnitude(f32),
    Ticks { remaining: u32, max: u32 },
    MagnitudeAndTicks { magnitude: f32, remaining: u32, max: u32 },
}

impl From<&TagValue> for TagValueSnapshot {
    fn from(value: &TagValue) -> Self {
        match value {
            TagValue::None => TagValueSnapshot::None,
            TagValue::Magnitude(m) => TagValueSnapshot::Magnitude(*m),
            TagValue::Ticks { remaining, max } => TagValueSnapshot::Ticks {
                remaining: *remaining,
                max: *max,
            },
            TagValue::MagnitudeAndTicks {
                magnitude,
                remaining,
                max,
            } => TagValueSnapshot::MagnitudeAndTicks {
                magnitude: *magnitude,
                remaining: *remaining,
                max: *max,
            },
        }
    }
}

pub fn snapshot_to_tags(snapshot: &TagsSnapshot, registry: &TagRegistry) -> Tags {
    let mut tags = Tags::new(registry.tag_count());
    for (name, value) in &snapshot.tags {
        if let Some(tag_id) = registry.tag_id(name) {
            let tag_value = match value {
                TagValueSnapshot::None => TagValue::None,
                TagValueSnapshot::Magnitude(m) => TagValue::Magnitude(*m),
                TagValueSnapshot::Ticks { remaining, max } => TagValue::Ticks { remaining: *remaining, max: *max },
                TagValueSnapshot::MagnitudeAndTicks { magnitude, remaining, max } => TagValue::MagnitudeAndTicks { magnitude: *magnitude, remaining: *remaining, max: *max },
            };
            tags.add_tag(tag_id, tag_value, registry);
        }
    }
    tags
}

pub fn tags_to_snapshot(tags: &Tags, registry: &TagRegistry) -> TagsSnapshot {
    let entries = tags
        .iter_present()
        .map(|tag_id| {
            let name = registry.tag_by_id(tag_id).name.clone();
            let value = tags
                .get_value(tag_id)
                .map(TagValueSnapshot::from)
                .unwrap_or(TagValueSnapshot::None);
            (name, value)
        })
        .collect();
    TagsSnapshot { tags: entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_tag_registry;

    fn test_registry() -> TagRegistry {
        load_tag_registry(
            r#"
[[archetype]]
id = "element"
name = "Element"
exclusivity = "mutual"

[[archetype.tags]]
id = "FIRE"
implies = ["HOT"]

[[archetype]]
id = "temperature"
name = "Temperature"
exclusivity = "mutual"

[[archetype.tags]]
id = "HOT"

[[archetype]]
id = "status"
name = "Status"
exclusivity = "any"

[[archetype.tags]]
id = "BURNING"
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_round_trip() {
        let registry = test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let fire_id = registry.tag_id("FIRE").unwrap();
        let burning_id = registry.tag_id("BURNING").unwrap();
        tags.add_tag(fire_id, TagValue::None, &registry);
        tags.add_tag(
            burning_id,
            TagValue::Ticks { remaining: 5, max: 10 },
            &registry,
        );

        let snapshot = tags_to_snapshot(&tags, &registry);

        assert!(snapshot.tags.iter().any(|(n, _)| n == "FIRE"));
        assert!(snapshot.tags.iter().any(|(n, _)| n == "HOT"));
        assert!(snapshot.tags.iter().any(|(n, _)| n == "BURNING"));
    }

    #[test]
    fn test_snapshot_magnitude_value() {
        let registry = test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let hot_id = registry.tag_id("HOT").unwrap();
        tags.add_tag(hot_id, TagValue::Magnitude(7.5), &registry);
        let snapshot = tags_to_snapshot(&tags, &registry);
        let entry = snapshot.tags.iter().find(|(n, _)| n == "HOT").unwrap();
        assert!(matches!(entry.1, TagValueSnapshot::Magnitude(7.5)));
    }

    #[test]
    fn test_snapshot_ticks_value() {
        let registry = test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let burning_id = registry.tag_id("BURNING").unwrap();
        tags.add_tag(burning_id, TagValue::Ticks { remaining: 5, max: 10 }, &registry);
        let snapshot = tags_to_snapshot(&tags, &registry);
        let entry = snapshot.tags.iter().find(|(n, _)| n == "BURNING").unwrap();
        assert!(matches!(entry.1, TagValueSnapshot::Ticks { remaining: 5, max: 10 }));
    }

    #[test]
    fn test_snapshot_magnitude_and_ticks_value() {
        let registry = test_registry();
        let mut tags = Tags::new(registry.tag_count());
        let burning_id = registry.tag_id("BURNING").unwrap();
        tags.add_tag(burning_id, TagValue::MagnitudeAndTicks { magnitude: 3.0, remaining: 4, max: 8 }, &registry);
        let snapshot = tags_to_snapshot(&tags, &registry);
        let entry = snapshot.tags.iter().find(|(n, _)| n == "BURNING").unwrap();
        assert!(matches!(entry.1, TagValueSnapshot::MagnitudeAndTicks { magnitude: 3.0, remaining: 4, max: 8 }));
    }

    #[test]
    fn test_snapshot_empty_tags() {
        let registry = test_registry();
        let tags = Tags::new(registry.tag_count());
        let snapshot = tags_to_snapshot(&tags, &registry);
        assert!(snapshot.tags.is_empty());
    }
}
