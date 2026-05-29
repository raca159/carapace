use crate::definition::{InteractionsToml, TagsToml};
use crate::id::TagId;
use crate::interaction::{InteractionRule, InteractionRules};
use crate::registry::{Exclusivity, RegistryError, TagRegistry, TagRegistryBuilder};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("registry error: {0}")]
    Registry(#[from] RegistryError),
    #[error("unknown exclusivity: {0}")]
    UnknownExclusivity(String),
    #[error("unknown tag: {0}")]
    UnknownTag(String),
    #[error("interaction rule references unknown tag: {0}")]
    InteractionUnknownTag(String),
    #[error("empty archetype: {0} has no tags")]
    EmptyArchetype(String),
}

pub fn load_tag_registry(toml_content: &str) -> Result<TagRegistry, LoadError> {
    let parsed: TagsToml = toml::from_str(toml_content)?;
    let mut builder = TagRegistryBuilder::new();

    for arch in &parsed.archetypes {
        let exclusivity = match arch.exclusivity.as_str() {
            "mutual" => Exclusivity::Mutual,
            "any" => Exclusivity::Any,
            other => return Err(LoadError::UnknownExclusivity(other.to_string())),
        };
        let archetype_id = builder.add_archetype(&arch.id, &arch.name, exclusivity);

        for tag in &arch.tags {
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
        }
    }

    builder.build().map_err(LoadError::Registry)
}

pub fn load_interaction_rules(
    toml_content: &str,
    registry: &TagRegistry,
) -> Result<InteractionRules, LoadError> {
    let parsed: InteractionsToml = toml::from_str(toml_content)?;
    let mut rules = Vec::new();

    for rule_toml in &parsed.interactions {
        let tag_a = registry
            .tag_id(&rule_toml.tag_a)
            .ok_or_else(|| LoadError::InteractionUnknownTag(rule_toml.tag_a.clone()))?;
        let tag_b = registry
            .tag_id(&rule_toml.tag_b)
            .ok_or_else(|| LoadError::InteractionUnknownTag(rule_toml.tag_b.clone()))?;

        let produces: Vec<TagId> = rule_toml
            .produces
            .iter()
            .filter_map(|name| registry.tag_id(name))
            .collect();

        let consumes: Vec<TagId> = rule_toml
            .consumes
            .iter()
            .filter_map(|name| registry.tag_id(name))
            .collect();

        rules.push(InteractionRule {
            tag_a,
            tag_b,
            produces,
            consumes,
            priority: rule_toml.priority,
            description: rule_toml.description.clone(),
        });
    }

    rules.sort_by_key(|b| std::cmp::Reverse(b.priority));

    Ok(InteractionRules { rules })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_TOML: &str = r#"
[[archetype]]
id = "element"
name = "Element"
exclusivity = "mutual"

[[archetype.tags]]
id = "FIRE"
implies = ["HOT"]

[[archetype.tags]]
id = "HOT"
"#;

    #[test]
    fn test_load_minimal() {
        let reg = load_tag_registry(MINIMAL_TOML).unwrap();
        assert_eq!(reg.tag_count(), 2);
        assert!(reg.tag_by_name("FIRE").is_some());
        assert!(reg.tag_by_name("HOT").is_some());
    }

    #[test]
    fn test_implication_chain() {
        let reg = load_tag_registry(MINIMAL_TOML).unwrap();
        let fire_id = reg.tag_id("FIRE").unwrap();
        let hot_id = reg.tag_id("HOT").unwrap();
        let fire_def = reg.tag_by_id(fire_id);
        assert!(fire_def.implies.contains(&hot_id));
    }

    #[test]
    fn test_duplicate_tag() {
        let toml = r#"
[[archetype]]
id = "a"
name = "A"
exclusivity = "mutual"

[[archetype.tags]]
id = "X"

[[archetype]]
id = "b"
name = "B"
exclusivity = "any"

[[archetype.tags]]
id = "X"
"#;
        assert!(load_tag_registry(toml).is_err());
    }

    #[test]
    fn test_unknown_implies() {
        let toml = r#"
[[archetype]]
id = "a"
name = "A"
exclusivity = "any"

[[archetype.tags]]
id = "X"
implies = ["NONEXISTENT"]
"#;
        assert!(load_tag_registry(toml).is_err());
    }

    #[test]
    fn test_self_implication() {
        let toml = r#"
[[archetype]]
id = "a"
name = "A"
exclusivity = "any"

[[archetype.tags]]
id = "X"
implies = ["X"]
"#;
        assert!(load_tag_registry(toml).is_err());
    }

    #[test]
    fn test_load_interactions() {
        let reg = load_tag_registry(MINIMAL_TOML).unwrap();
        let toml = r#"
[[interaction]]
tag_a = "FIRE"
tag_b = "HOT"
produces = []
consumes = []
priority = 10
"#;
        let rules = load_interaction_rules(toml, &reg).unwrap();
        assert_eq!(rules.rules.len(), 1);
    }

    #[test]
    fn test_interaction_unknown_tag() {
        let reg = load_tag_registry(MINIMAL_TOML).unwrap();
        let toml = r#"
[[interaction]]
tag_a = "FIRE"
tag_b = "NONEXISTENT"
produces = []
consumes = []
priority = 10
"#;
        assert!(load_interaction_rules(toml, &reg).is_err());
    }

    #[test]
    fn test_unknown_exclusivity() {
        let toml = r#"
[[archetype]]
id = "a"
name = "A"
exclusivity = "invalid"

[[archetype.tags]]
id = "X"
"#;
        assert!(load_tag_registry(toml).is_err());
    }

    #[test]
    fn test_load_error_empty_archetype_variant() {
        let err = LoadError::EmptyArchetype("my_arch".to_string());
        let msg = err.to_string();
        assert!(msg.contains("my_arch"));
        assert!(msg.contains("empty archetype"));
    }

    #[test]
    fn test_load_error_unknown_tag_variant() {
        let err = LoadError::UnknownTag("MYSTERY".to_string());
        let msg = err.to_string();
        assert!(msg.contains("MYSTERY"));
        assert!(msg.contains("unknown tag"));
    }

    #[test]
    fn test_load_error_interaction_unknown_tag_variant() {
        let err = LoadError::InteractionUnknownTag("BAD_TAG".to_string());
        let msg = err.to_string();
        assert!(msg.contains("BAD_TAG"));
    }
}

#[cfg(test)]
mod integration {
    use super::*;
    use crate::component::{TagValue, Tags};

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
    const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");

    fn full_registry() -> TagRegistry {
        load_tag_registry(TAGS_TOML).expect("tags.toml should parse")
    }

    fn full_rules(registry: &TagRegistry) -> InteractionRules {
        load_interaction_rules(INTERACTIONS_TOML, registry).expect("interactions.toml should parse")
    }

    #[test]
    fn test_full_registry_loads() {
        let registry = full_registry();
        assert!(registry.tag_count() > 100, "expected 150+ tags, got {}", registry.tag_count());
        assert_eq!(registry.all_archetypes().count(), 26);
    }

    #[test]
    fn test_full_interactions_load() {
        let registry = full_registry();
        let rules = full_rules(&registry);
        assert!(rules.rules.len() >= 20, "expected 20+ interaction rules, got {}", rules.rules.len());
    }

    #[test]
    fn test_implication_fire_implies_hot_luminescent() {
        let registry = full_registry();
        let mut tags = Tags::new(registry.tag_count());

        let fire = registry.tag_id("FIRE").unwrap();
        let hot = registry.tag_id("HOT").unwrap();
        let luminescent = registry.tag_id("LUMINESCENT").unwrap();

        let added = tags.add_tag(fire, TagValue::None, &registry);

        assert!(tags.has(fire));
        assert!(tags.has(hot));
        assert!(tags.has(luminescent));
        assert!(added.contains(&hot));
        assert!(added.contains(&luminescent));
    }

    #[test]
    fn test_mutual_exclusivity_element() {
        let registry = full_registry();
        let mut tags = Tags::new(registry.tag_count());

        let fire = registry.tag_id("FIRE").unwrap();
        let water = registry.tag_id("WATER").unwrap();

        tags.add_tag(fire, TagValue::None, &registry);
        assert!(tags.has(fire));

        tags.add_tag(water, TagValue::None, &registry);
        assert!(tags.has(water));
        assert!(!tags.has(fire), "FIRE should be replaced by WATER (mutual archetype)");
    }

    #[test]
    fn test_mutual_exclusivity_biome() {
        let registry = full_registry();
        let mut tags = Tags::new(registry.tag_count());

        let desert = registry.tag_id("BIOME_DESERT").unwrap();
        let forest = registry.tag_id("BIOME_TEMPERATE_FOREST").unwrap();

        tags.add_tag(desert, TagValue::None, &registry);
        assert!(tags.has(desert));

        tags.add_tag(forest, TagValue::None, &registry);
        assert!(tags.has(forest));
        assert!(!tags.has(desert), "DESERT should be replaced by FOREST (mutual archetype)");
    }

    #[test]
    fn test_interaction_fire_flammable_produces_burning() {
        let registry = full_registry();
        let rules = full_rules(&registry);
        let mut tags = Tags::new(registry.tag_count());

        let fire = registry.tag_id("FIRE").unwrap();
        let flammable = registry.tag_id("FLAMMABLE").unwrap();
        let burning = registry.tag_id("BURNING").unwrap();

        tags.add_tag(fire, TagValue::None, &registry);
        tags.add_tag(flammable, TagValue::None, &registry);

        let matched = rules.check_self_interactions(&tags);
        let fire_flammable = matched.iter().find(|r| {
            (r.tag_a == fire && r.tag_b == flammable) || (r.tag_a == flammable && r.tag_b == fire)
        });
        assert!(fire_flammable.is_some(), "FIRE + FLAMMABLE should match an interaction rule");
        assert!(fire_flammable.unwrap().produces.contains(&burning));
    }

    #[test]
    fn test_temporary_tags_tick_down() {
        let registry = full_registry();
        let mut tags = Tags::new(registry.tag_count());

        let burning = registry.tag_id("BURNING").unwrap();
        tags.add_tag(burning, TagValue::Ticks { remaining: 3, max: 5 }, &registry);

        assert!(tags.has(burning));

        let expired = tags.tick_status(&registry);
        assert!(expired.is_empty());
        assert!(tags.has(burning));

        tags.tick_status(&registry);

        let expired = tags.tick_status(&registry);
        assert!(expired.contains(&burning));
        assert!(!tags.has(burning));
    }

    #[test]
    fn test_cross_entity_interaction() {
        let registry = full_registry();
        let rules = full_rules(&registry);

        let fire = registry.tag_id("FIRE").unwrap();
        let flammable = registry.tag_id("FLAMMABLE").unwrap();

        let mut tags_a = Tags::new(registry.tag_count());
        tags_a.add_tag(fire, TagValue::None, &registry);

        let mut tags_b = Tags::new(registry.tag_count());
        tags_b.add_tag(flammable, TagValue::None, &registry);

        let matched = rules.check_cross_interactions(&tags_a, &tags_b);
        assert!(!matched.is_empty(), "FIRE entity + FLAMMABLE entity should interact");
    }

    #[test]
    fn test_biome_implies_terrain() {
        let registry = full_registry();
        let mut tags = Tags::new(registry.tag_count());

        let desert = registry.tag_id("BIOME_DESERT").unwrap();
        let walkable = registry.tag_id("WALKABLE").unwrap();
        let dry = registry.tag_id("DRY").unwrap();
        let hot = registry.tag_id("HOT").unwrap();

        tags.add_tag(desert, TagValue::None, &registry);

        assert!(tags.has(walkable), "DESERT should imply WALKABLE");
        assert!(tags.has(dry), "DESERT should imply DRY");
        assert!(tags.has(hot), "DESERT should imply HOT");
    }

    #[test]
    fn test_tag_count_matches() {
        let registry = full_registry();
        let mut count = 0usize;
        for arch in registry.all_archetypes() {
            count += arch.tag_ids.len();
        }
        assert_eq!(count, registry.tag_count(), "sum of archetype tags should equal registry tag_count");
    }

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
}
