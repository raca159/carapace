use bevy_ecs::prelude::Resource;

use crate::component::Tags;
use crate::id::TagId;

#[derive(Debug, Clone, Resource)]
pub struct InteractionRules {
    pub rules: Vec<InteractionRule>,
}

#[derive(Debug, Clone)]
pub struct InteractionRule {
    pub tag_a: TagId,
    pub tag_b: TagId,
    pub produces: Vec<TagId>,
    pub consumes: Vec<TagId>,
    pub priority: u32,
    pub description: String,
}

impl InteractionRules {
    pub fn check_self_interactions(&self, tags: &Tags) -> Vec<&InteractionRule> {
        self.rules
            .iter()
            .filter(|rule| tags.has(rule.tag_a) && tags.has(rule.tag_b))
            .collect()
    }

    pub fn check_cross_interactions(
        &self,
        tags_a: &Tags,
        tags_b: &Tags,
    ) -> Vec<(&InteractionRule, bool)> {
        self.rules
            .iter()
            .filter_map(|rule| {
                if tags_a.has(rule.tag_a) && tags_b.has(rule.tag_b) {
                    Some((rule, false))
                } else if tags_a.has(rule.tag_b) && tags_b.has(rule.tag_a) {
                    Some((rule, true))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Exclusivity, TagRegistryBuilder, TagValue};

    fn build_registry_and_rules() -> (crate::TagRegistry, InteractionRules) {
        let mut builder = TagRegistryBuilder::new();
        let elem = builder.add_archetype("element", "Element", Exclusivity::Mutual);
        builder.add_tag(elem, "FIRE", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(elem, "WATER", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        let status = builder.add_archetype("status", "Status", Exclusivity::Any);
        builder.add_tag(status, "FLAMMABLE", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(status, "WET", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        let reg = builder.build().unwrap();

        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();
        let _water = reg.tag_id("WATER").unwrap();
        let wet = reg.tag_id("WET").unwrap();
        let rules = InteractionRules {
            rules: vec![
                InteractionRule {
                    tag_a: fire,
                    tag_b: flammable,
                    produces: vec![],
                    consumes: vec![],
                    priority: 10,
                    description: "fire meets flammable".to_string(),
                },
                InteractionRule {
                    tag_a: fire,
                    tag_b: wet,
                    produces: vec![],
                    consumes: vec![],
                    priority: 5,
                    description: "fire meets wet".to_string(),
                },
            ],
        };

        (reg, rules)
    }

    #[test]
    fn check_self_interactions_finds_matching() {
        let (reg, rules) = build_registry_and_rules();
        let mut tags = Tags::new(reg.tag_count());
        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();
        tags.add_tag(fire, TagValue::None, &reg);
        tags.add_tag(flammable, TagValue::None, &reg);
        let matched = rules.check_self_interactions(&tags);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].description, "fire meets flammable");
    }

    #[test]
    fn check_self_interactions_no_match() {
        let (reg, rules) = build_registry_and_rules();
        let mut tags = Tags::new(reg.tag_count());
        let water = reg.tag_id("WATER").unwrap();
        tags.add_tag(water, TagValue::None, &reg);
        let matched = rules.check_self_interactions(&tags);
        assert!(matched.is_empty());
    }

    #[test]
    fn check_self_interactions_multiple_matches() {
        let mut builder = TagRegistryBuilder::new();
        let elem = builder.add_archetype("element", "Element", Exclusivity::Mutual);
        builder.add_tag(elem, "FIRE", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        let status = builder.add_archetype("status", "Status", Exclusivity::Any);
        builder.add_tag(status, "FLAMMABLE", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.add_tag(status, "WET", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        let reg = builder.build().unwrap();

        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();
        let wet = reg.tag_id("WET").unwrap();

        let rules = InteractionRules {
            rules: vec![
                InteractionRule {
                    tag_a: fire,
                    tag_b: flammable,
                    produces: vec![],
                    consumes: vec![],
                    priority: 10,
                    description: "fire meets flammable".to_string(),
                },
                InteractionRule {
                    tag_a: fire,
                    tag_b: wet,
                    produces: vec![],
                    consumes: vec![],
                    priority: 5,
                    description: "fire meets wet".to_string(),
                },
            ],
        };

        let mut tags = Tags::new(reg.tag_count());
        tags.add_tag(fire, TagValue::None, &reg);
        tags.add_tag(flammable, TagValue::None, &reg);
        tags.add_tag(wet, TagValue::None, &reg);
        let matched = rules.check_self_interactions(&tags);
        assert_eq!(matched.len(), 2, "FIRE + FLAMMABLE + WET should match both rules since all are in different archetypes");
    }

    #[test]
    fn check_cross_interactions_forward() {
        let (reg, rules) = build_registry_and_rules();
        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();

        let mut tags_a = Tags::new(reg.tag_count());
        tags_a.add_tag(fire, TagValue::None, &reg);

        let mut tags_b = Tags::new(reg.tag_count());
        tags_b.add_tag(flammable, TagValue::None, &reg);

        let matched = rules.check_cross_interactions(&tags_a, &tags_b);
        assert_eq!(matched.len(), 1);
        assert!(!matched[0].1, "swapped flag should be false for forward match");
    }

    #[test]
    fn check_cross_interactions_reversed() {
        let (reg, rules) = build_registry_and_rules();
        let fire = reg.tag_id("FIRE").unwrap();
        let flammable = reg.tag_id("FLAMMABLE").unwrap();

        let mut tags_a = Tags::new(reg.tag_count());
        tags_a.add_tag(flammable, TagValue::None, &reg);

        let mut tags_b = Tags::new(reg.tag_count());
        tags_b.add_tag(fire, TagValue::None, &reg);

        let matched = rules.check_cross_interactions(&tags_a, &tags_b);
        assert_eq!(matched.len(), 1);
        assert!(matched[0].1, "swapped flag should be true for reversed match");
    }

    #[test]
    fn check_cross_interactions_no_match() {
        let (reg, rules) = build_registry_and_rules();
        let water = reg.tag_id("WATER").unwrap();

        let mut tags_a = Tags::new(reg.tag_count());
        tags_a.add_tag(water, TagValue::None, &reg);

        let tags_b = Tags::new(reg.tag_count());

        let matched = rules.check_cross_interactions(&tags_a, &tags_b);
        assert!(matched.is_empty());
    }

    #[test]
    fn interaction_rule_fields_accessible() {
        let rule = InteractionRule {
            tag_a: crate::TagId(0),
            tag_b: crate::TagId(1),
            produces: vec![crate::TagId(2)],
            consumes: vec![],
            priority: 42,
            description: "test rule".to_string(),
        };
        assert_eq!(rule.priority, 42);
        assert_eq!(rule.produces.len(), 1);
        assert!(rule.consumes.is_empty());
        assert_eq!(rule.description, "test rule");
    }
}
