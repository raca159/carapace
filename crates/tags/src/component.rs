use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::Component;
use fixedbitset::FixedBitSet;

use crate::id::{ArchetypeId, TagId};
use crate::registry::{Exclusivity, TagRegistry};

#[derive(Debug, Clone, PartialEq)]
pub enum TagValue {
    None,
    Magnitude(f32),
    Ticks { remaining: u32, max: u32 },
    MagnitudeAndTicks { magnitude: f32, remaining: u32, max: u32 },
}

impl TagValue {
    pub fn is_none(&self) -> bool {
        matches!(self, TagValue::None)
    }

    pub fn magnitude(&self) -> Option<f32> {
        match self {
            TagValue::Magnitude(m) | TagValue::MagnitudeAndTicks { magnitude: m, .. } => Some(*m),
            _ => None,
        }
    }

    pub fn ticks_remaining(&self) -> Option<u32> {
        match self {
            TagValue::Ticks { remaining, .. }
            | TagValue::MagnitudeAndTicks { remaining, .. } => Some(*remaining),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Tags {
    present: FixedBitSet,
    values: HashMap<TagId, TagValue>,
    mutual_state: HashMap<ArchetypeId, TagId>,
}

impl Tags {
    pub fn new(tag_count: usize) -> Self {
        Tags {
            present: FixedBitSet::with_capacity(tag_count),
            values: HashMap::new(),
            mutual_state: HashMap::new(),
        }
    }

    #[inline]
    pub fn has(&self, tag: TagId) -> bool {
        self.present.contains(tag.bit_index())
    }

    pub fn has_all(&self, tags: &[TagId]) -> bool {
        tags.iter().all(|&t| self.has(t))
    }

    pub fn has_any(&self, tags: &[TagId]) -> bool {
        tags.iter().any(|&t| self.has(t))
    }

    pub fn get_value(&self, tag: TagId) -> Option<&TagValue> {
        self.values.get(&tag)
    }

    pub fn count(&self) -> usize {
        self.present.count_ones(..)
    }

    pub fn iter_present(&self) -> impl Iterator<Item = TagId> + '_ {
        self.present.ones().map(|bit| TagId(bit as u16))
    }

    pub fn mutual_tag(&self, archetype: ArchetypeId) -> Option<TagId> {
        self.mutual_state.get(&archetype).copied()
    }

    pub fn add_tag(
        &mut self,
        tag: TagId,
        value: TagValue,
        registry: &TagRegistry,
    ) -> Vec<TagId> {
        let tag_def = registry.tag_by_id(tag);
        let archetype_def = registry.archetype_by_id(tag_def.archetype);

        if archetype_def.exclusivity == Exclusivity::Mutual {
        if let Some(&old_tag) = self.mutual_state.get(&tag_def.archetype)
            && old_tag != tag
        {
            self.remove_tag_internal(old_tag, registry);
        }
            self.mutual_state.insert(tag_def.archetype, tag);
        }

        let mut added = vec![tag];
        self.present.insert(tag_def.bit_index);
        if !value.is_none() {
            self.values.insert(tag, value);
        }

        let mut seen = HashSet::new();
        seen.insert(tag);
        self.add_implications(tag, registry, &mut added, &mut seen);

        added
    }

    pub fn remove_tag(&mut self, tag: TagId, registry: &TagRegistry) -> bool {
        if !self.has(tag) {
            return false;
        }

        let tag_def = registry.tag_by_id(tag);
        if let Some(current) = self.mutual_state.get(&tag_def.archetype)
            && *current == tag
        {
            self.mutual_state.remove(&tag_def.archetype);
        }

        self.remove_tag_internal(tag, registry);
        true
    }

    fn remove_tag_internal(&mut self, tag: TagId, registry: &TagRegistry) {
        let tag_def = registry.tag_by_id(tag);
        if tag_def.bit_index < self.present.len() {
            self.present.set(tag_def.bit_index, false);
        }
        self.values.remove(&tag);
    }

    fn add_implications(
        &mut self,
        source: TagId,
        registry: &TagRegistry,
        added: &mut Vec<TagId>,
        seen: &mut HashSet<TagId>,
    ) {
        let implies: Vec<TagId> = registry.tag_by_id(source).implies.clone();

        for implied_id in implies {
            if seen.contains(&implied_id) || self.has(implied_id) {
                continue;
            }
            seen.insert(implied_id);

            let implied_def = registry.tag_by_id(implied_id);
            let archetype_def = registry.archetype_by_id(implied_def.archetype);

            if archetype_def.exclusivity == Exclusivity::Mutual {
                for &old_tag in &archetype_def.tag_ids {
                    if self.has(old_tag) && old_tag != implied_id {
                        let old_def = registry.tag_by_id(old_tag);
                        self.present.set(old_def.bit_index, false);
                        self.values.remove(&old_tag);
                    }
                }
                self.mutual_state.insert(implied_def.archetype, implied_id);
            }

            self.present.insert(implied_def.bit_index);
            added.push(implied_id);

            self.add_implications(implied_id, registry, added, seen);
        }
    }

    pub fn tick_status(&mut self, registry: &TagRegistry) -> Vec<TagId> {
        let mut expired = Vec::new();
        let tag_ids: Vec<TagId> = self.values.keys().copied().collect();

        for tag_id in tag_ids {
            if let Some(value) = self.values.get_mut(&tag_id) {
                let should_remove = match value {
                    TagValue::Ticks { remaining, .. } => {
                        *remaining = remaining.saturating_sub(1);
                        *remaining == 0
                    }
                    TagValue::MagnitudeAndTicks { remaining, .. } => {
                        *remaining = remaining.saturating_sub(1);
                        *remaining == 0
                    }
                    _ => false,
                };
                if should_remove {
                    expired.push(tag_id);
                }
            }
        }

        for tag_id in &expired {
            self.remove_tag(*tag_id, registry);
        }

        expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_value_is_none() {
        assert!(TagValue::None.is_none());
        assert!(!TagValue::Magnitude(1.0).is_none());
        assert!(!TagValue::Ticks { remaining: 1, max: 5 }.is_none());
        assert!(!TagValue::MagnitudeAndTicks { magnitude: 2.0, remaining: 3, max: 5 }.is_none());
    }

    #[test]
    fn tag_value_magnitude() {
        assert_eq!(TagValue::None.magnitude(), None);
        assert_eq!(TagValue::Magnitude(3.5).magnitude(), Some(3.5));
        assert_eq!(TagValue::Ticks { remaining: 1, max: 5 }.magnitude(), None);
        assert_eq!(TagValue::MagnitudeAndTicks { magnitude: 4.2, remaining: 2, max: 5 }.magnitude(), Some(4.2));
    }

    #[test]
    fn tag_value_ticks_remaining() {
        assert_eq!(TagValue::None.ticks_remaining(), None);
        assert_eq!(TagValue::Magnitude(1.0).ticks_remaining(), None);
        assert_eq!(TagValue::Ticks { remaining: 3, max: 5 }.ticks_remaining(), Some(3));
        assert_eq!(TagValue::MagnitudeAndTicks { magnitude: 1.0, remaining: 7, max: 10 }.ticks_remaining(), Some(7));
    }

    #[test]
    fn tags_get_value_returns_none_for_unset() {
        let tags = Tags::new(10);
        assert!(tags.get_value(TagId(0)).is_none());
    }

    #[test]
    fn tags_get_value_returns_set_value() {
        let mut tags = Tags::new(10);
        tags.present.insert(3);
        tags.values.insert(TagId(3), TagValue::Magnitude(5.0));
        let val = tags.get_value(TagId(3)).unwrap();
        assert_eq!(val.magnitude(), Some(5.0));
    }

    #[test]
    fn tags_iter_present_returns_set_tags() {
        let mut tags = Tags::new(10);
        tags.present.insert(2);
        tags.present.insert(5);
        tags.present.insert(7);
        let present: Vec<TagId> = tags.iter_present().collect();
        assert_eq!(present, vec![TagId(2), TagId(5), TagId(7)]);
    }

    #[test]
    fn tags_mutual_tag_returns_none_when_not_set() {
        let tags = Tags::new(10);
        assert!(tags.mutual_tag(ArchetypeId(0)).is_none());
    }

    #[test]
    fn tags_mutual_tag_returns_set_value() {
        let mut tags = Tags::new(10);
        tags.mutual_state.insert(ArchetypeId(2), TagId(5));
        assert_eq!(tags.mutual_tag(ArchetypeId(2)), Some(TagId(5)));
    }

    #[test]
    fn magnitude_and_ticks_tick_down() {
        let mut tags = Tags::new(10);
        tags.present.insert(0);
        tags.values.insert(TagId(0), TagValue::MagnitudeAndTicks { magnitude: 2.0, remaining: 2, max: 3 });

        let reg = {
            let mut builder = crate::TagRegistryBuilder::new();
            let arch = builder.add_archetype("test", "Test", Exclusivity::Any);
            builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
            builder.build().unwrap()
        };

        let expired = tags.tick_status(&reg);
        assert!(expired.is_empty());
        assert_eq!(tags.get_value(TagId(0)).unwrap().ticks_remaining(), Some(1));

        let expired = tags.tick_status(&reg);
        assert_eq!(expired, vec![TagId(0)]);
        assert!(!tags.has(TagId(0)));
    }

    #[test]
    fn tags_count_returns_present_count() {
        let mut tags = Tags::new(10);
        assert_eq!(tags.count(), 0);
        tags.present.insert(2);
        assert_eq!(tags.count(), 1);
        tags.present.insert(5);
        tags.present.insert(7);
        assert_eq!(tags.count(), 3);
    }

    #[test]
    fn tags_has_all_true_when_all_present() {
        let mut tags = Tags::new(10);
        tags.present.insert(1);
        tags.present.insert(2);
        tags.present.insert(3);
        assert!(tags.has_all(&[TagId(1), TagId(2), TagId(3)]));
    }

    #[test]
    fn tags_has_all_false_when_one_missing() {
        let mut tags = Tags::new(10);
        tags.present.insert(1);
        tags.present.insert(2);
        assert!(!tags.has_all(&[TagId(1), TagId(2), TagId(9)]));
    }

    #[test]
    fn tags_has_any_true_when_one_present() {
        let mut tags = Tags::new(10);
        tags.present.insert(5);
        assert!(tags.has_any(&[TagId(3), TagId(5), TagId(7)]));
    }

    #[test]
    fn tags_has_any_false_when_none_present() {
        let tags = Tags::new(10);
        assert!(!tags.has_any(&[TagId(1), TagId(2), TagId(3)]));
    }

    #[test]
    fn tags_remove_tag_returns_true_and_clears() {
        let reg = {
            let mut builder = crate::TagRegistryBuilder::new();
            let arch = builder.add_archetype("test", "Test", Exclusivity::Any);
            builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
            builder.add_tag(arch, "T1", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
            builder.build().unwrap()
        };
        let mut tags = Tags::new(reg.tag_count());
        let t0 = reg.tag_id("T0").unwrap();
        let t1 = reg.tag_id("T1").unwrap();
        tags.add_tag(t0, TagValue::None, &reg);
        tags.add_tag(t1, TagValue::Magnitude(3.0), &reg);
        assert!(tags.has(t0));
        assert!(tags.has(t1));
        assert!(tags.remove_tag(t0, &reg));
        assert!(!tags.has(t0));
        assert!(tags.has(t1));
    }

    #[test]
    fn tags_remove_tag_returns_false_for_absent() {
        let reg = {
            let mut builder = crate::TagRegistryBuilder::new();
            let arch = builder.add_archetype("test", "Test", Exclusivity::Any);
            builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
            builder.build().unwrap()
        };
        let mut tags = Tags::new(reg.tag_count());
        let t0 = reg.tag_id("T0").unwrap();
        assert!(!tags.remove_tag(t0, &reg));
    }

    #[test]
    fn tick_status_ticks_variant_expires() {
        let reg = {
            let mut builder = crate::TagRegistryBuilder::new();
            let arch = builder.add_archetype("test", "Test", Exclusivity::Any);
            builder.add_tag(arch, "T0", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
            builder.build().unwrap()
        };
        let mut tags = Tags::new(reg.tag_count());
        let t0 = reg.tag_id("T0").unwrap();
        tags.add_tag(t0, TagValue::Ticks { remaining: 2, max: 3 }, &reg);
        assert!(tags.has(t0));

        let expired = tags.tick_status(&reg);
        assert!(expired.is_empty());
        assert_eq!(tags.get_value(t0).unwrap().ticks_remaining(), Some(1));

        let expired = tags.tick_status(&reg);
        assert_eq!(expired, vec![t0]);
        assert!(!tags.has(t0));
    }
}
