use bevy_ecs::entity::Entity;

use crate::component::Tags;
use crate::id::TagId;

pub struct TagQuery;

impl TagQuery {
    pub fn with_tag<'a>(
        iter: impl Iterator<Item = (Entity, &'a Tags)>,
        tag: TagId,
    ) -> Vec<(Entity, &'a Tags)> {
        iter.filter(|(_, tags)| tags.has(tag)).collect()
    }

    pub fn with_any<'a>(
        iter: impl Iterator<Item = (Entity, &'a Tags)>,
        tags: &[TagId],
    ) -> Vec<(Entity, &'a Tags)> {
        iter.filter(|(_, t)| t.has_any(tags)).collect()
    }

    pub fn with_all<'a>(
        iter: impl Iterator<Item = (Entity, &'a Tags)>,
        tags: &[TagId],
    ) -> Vec<(Entity, &'a Tags)> {
        iter.filter(|(_, t)| t.has_all(tags)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::{Tags, TagValue, load_tag_registry};

    fn build_test_registry() -> crate::TagRegistry {
        let toml = r#"
[[archetype]]
id = "element"
name = "Element"
exclusivity = "mutual"

[[archetype.tags]]
id = "FIRE"

[[archetype.tags]]
id = "WATER"

[[archetype]]
id = "property"
name = "Property"
exclusivity = "any"

[[archetype.tags]]
id = "HOT"

[[archetype]]
id = "status"
name = "Status"
exclusivity = "any"

[[archetype.tags]]
id = "BURNING"

[[archetype.tags]]
id = "FLAMMABLE"
"#;
        load_tag_registry(toml).unwrap()
    }

    #[test]
    fn with_tag_filters_correctly() {
        let registry = build_test_registry();
        let mut world = World::new();
        let fire = registry.tag_id("FIRE").unwrap();
        let water = registry.tag_id("WATER").unwrap();
        let hot = registry.tag_id("HOT").unwrap();

        let mut tags1 = Tags::new(registry.tag_count());
        tags1.add_tag(fire, TagValue::None, &registry);
        let e1 = world.spawn(tags1).id();

        let mut tags2 = Tags::new(registry.tag_count());
        tags2.add_tag(water, TagValue::None, &registry);
        let _e2 = world.spawn(tags2).id();

        let tags3 = Tags::new(registry.tag_count());
        let _e3 = world.spawn(tags3).id();

        let mut q = world.query::<(Entity, &Tags)>();
        let result = TagQuery::with_tag(q.iter(&world), fire);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, e1);

        let result = TagQuery::with_tag(q.iter(&world), water);
        assert_eq!(result.len(), 1);

        let result = TagQuery::with_tag(q.iter(&world), hot);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn with_any_matches_any_of() {
        let registry = build_test_registry();
        let mut world = World::new();
        let fire = registry.tag_id("FIRE").unwrap();
        let water = registry.tag_id("WATER").unwrap();
        let burning = registry.tag_id("BURNING").unwrap();

        let mut tags1 = Tags::new(registry.tag_count());
        tags1.add_tag(fire, TagValue::None, &registry);
        world.spawn(tags1);

        let mut tags2 = Tags::new(registry.tag_count());
        tags2.add_tag(water, TagValue::None, &registry);
        world.spawn(tags2);

        let mut tags3 = Tags::new(registry.tag_count());
        tags3.add_tag(burning, TagValue::None, &registry);
        world.spawn(tags3);

        let tags4 = Tags::new(registry.tag_count());
        world.spawn(tags4);

        let mut q = world.query::<(Entity, &Tags)>();
        let result = TagQuery::with_any(q.iter(&world), &[fire, water]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn with_all_requires_all_tags() {
        let registry = build_test_registry();
        let mut world = World::new();
        let fire = registry.tag_id("FIRE").unwrap();
        let hot = registry.tag_id("HOT").unwrap();
        let burning = registry.tag_id("BURNING").unwrap();

        let mut tags1 = Tags::new(registry.tag_count());
        tags1.add_tag(fire, TagValue::None, &registry);
        tags1.add_tag(hot, TagValue::None, &registry);
        world.spawn(tags1);

        let mut tags2 = Tags::new(registry.tag_count());
        tags2.add_tag(burning, TagValue::None, &registry);
        world.spawn(tags2);

        let mut q = world.query::<(Entity, &Tags)>();
        let result = TagQuery::with_all(q.iter(&world), &[fire, hot]);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn with_tag_empty_iterator() {
        let registry = build_test_registry();
        let mut world = World::new();
        let fire = registry.tag_id("FIRE").unwrap();
        let mut q = world.query::<(Entity, &Tags)>();
        let result = TagQuery::with_tag(q.iter(&world), fire);
        assert!(result.is_empty());
    }
}
