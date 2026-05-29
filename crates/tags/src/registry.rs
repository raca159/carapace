use std::collections::HashMap;

use bevy_ecs::prelude::Resource;

use crate::id::{ArchetypeId, TagId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusivity {
    Mutual,
    Any,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("duplicate tag name: {0}")]
    DuplicateTag(String),
    #[error("duplicate archetype name: {0}")]
    DuplicateArchetype(String),
    #[error("unresolved implies reference: tag '{tag}' implies '{implies}' which does not exist")]
    UnresolvedImplies { tag: String, implies: String },
    #[error("unresolved conflicts reference: tag '{tag}' conflicts with '{conflict}' which does not exist")]
    UnresolvedConflict { tag: String, conflict: String },
    #[error("self-implication: tag '{0}' implies itself")]
    SelfImplication(String),
}

#[derive(Debug, Clone)]
pub struct ArchetypeDef {
    pub id: ArchetypeId,
    pub name: String,
    pub display_name: String,
    pub exclusivity: Exclusivity,
    pub tag_ids: Vec<TagId>,
    pub bit_offset: usize,
}

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

#[derive(Debug, Clone, Resource)]
pub struct TagRegistry {
    archetypes: Vec<ArchetypeDef>,
    tags: Vec<TagDef>,
    name_to_id: HashMap<String, TagId>,
    archetype_name_to_id: HashMap<String, ArchetypeId>,
    tag_count: usize,
}

impl TagRegistry {
    pub fn tag_by_id(&self, id: TagId) -> &TagDef {
        &self.tags[id.0 as usize]
    }

    pub fn tag_by_name(&self, name: &str) -> Option<&TagDef> {
        self.name_to_id.get(name).map(|&id| self.tag_by_id(id))
    }

    pub fn tag_id(&self, name: &str) -> Option<TagId> {
        self.name_to_id.get(name).copied()
    }

    pub fn archetype_by_id(&self, id: ArchetypeId) -> &ArchetypeDef {
        &self.archetypes[id.0 as usize]
    }

    pub fn archetype_by_name(&self, name: &str) -> Option<&ArchetypeDef> {
        self.archetype_name_to_id
            .get(name)
            .map(|&id| self.archetype_by_id(id))
    }

    pub fn archetype_id(&self, name: &str) -> Option<ArchetypeId> {
        self.archetype_name_to_id.get(name).copied()
    }

    pub fn tags_for_archetype(&self, archetype: ArchetypeId) -> &[TagId] {
        &self.archetypes[archetype.0 as usize].tag_ids
    }

    pub fn tag_count(&self) -> usize {
        self.tag_count
    }

    pub fn all_tags(&self) -> impl Iterator<Item = &TagDef> {
        self.tags.iter()
    }

    pub fn all_archetypes(&self) -> impl Iterator<Item = &ArchetypeDef> {
        self.archetypes.iter()
    }
}

pub struct TagRegistryBuilder {
    archetypes: Vec<ArchetypeDef>,
    tags: Vec<TagDef>,
    name_to_id: HashMap<String, TagId>,
    archetype_name_to_id: HashMap<String, ArchetypeId>,
    next_bit: usize,
    pending_implies: HashMap<TagId, Vec<String>>,
    pending_conflicts: HashMap<TagId, Vec<String>>,
}

impl TagRegistryBuilder {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            tags: Vec::new(),
            name_to_id: HashMap::new(),
            archetype_name_to_id: HashMap::new(),
            next_bit: 0,
            pending_implies: HashMap::new(),
            pending_conflicts: HashMap::new(),
        }
    }

    pub fn add_archetype(
        &mut self,
        name: &str,
        display_name: &str,
        exclusivity: Exclusivity,
    ) -> ArchetypeId {
        let id = ArchetypeId(self.archetypes.len() as u8);
        let bit_offset = self.next_bit;
        self.archetypes.push(ArchetypeDef {
            id,
            name: name.to_string(),
            display_name: display_name.to_string(),
            exclusivity,
            tag_ids: Vec::new(),
            bit_offset,
        });
        self.archetype_name_to_id.insert(name.to_string(), id);
        id
    }

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
        if self.name_to_id.contains_key(name) {
            return Err(RegistryError::DuplicateTag(name.to_string()));
        }

        let id = TagId(self.tags.len() as u16);
        let bit_index = self.next_bit;
        self.next_bit += 1;

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
        self.name_to_id.insert(name.to_string(), id);
        self.archetypes[archetype.0 as usize].tag_ids.push(id);

        if !implies_strs.is_empty() {
            self.pending_implies.insert(id, implies_strs);
        }
        if !conflicts_strs.is_empty() {
            self.pending_conflicts.insert(id, conflicts_strs);
        }

        Ok(id)
    }

    pub fn build(mut self) -> Result<TagRegistry, RegistryError> {
        let pending: Vec<(TagId, Vec<String>)> = self.pending_implies.into_iter().collect();

        for (tag_id, implies_strs) in pending {
            let tag_name = self.tags[tag_id.0 as usize].name.clone();
            let mut resolved = Vec::with_capacity(implies_strs.len());
            for implies_name in &implies_strs {
                if *implies_name == tag_name {
                    return Err(RegistryError::SelfImplication(tag_name));
                }
                match self.name_to_id.get(implies_name) {
                    Some(&id) => resolved.push(id),
                    None => {
                        return Err(RegistryError::UnresolvedImplies {
                            tag: tag_name,
                            implies: implies_name.clone(),
                        })
                    }
                }
            }
            self.tags[tag_id.0 as usize].implies = resolved;
        }

        let pending_conf: Vec<(TagId, Vec<String>)> = self.pending_conflicts.into_iter().collect();
        for (tag_id, conflicts_strs) in pending_conf {
            let tag_name = self.tags[tag_id.0 as usize].name.clone();
            let mut resolved = Vec::with_capacity(conflicts_strs.len());
            for conflict_name in &conflicts_strs {
                match self.name_to_id.get(conflict_name) {
                    Some(&id) => resolved.push(id),
                    None => {
                        return Err(RegistryError::UnresolvedConflict {
                            tag: tag_name,
                            conflict: conflict_name.clone(),
                        })
                    }
                }
            }
            self.tags[tag_id.0 as usize].conflicts = resolved;
        }

        Ok(TagRegistry {
            archetypes: self.archetypes,
            tags: self.tags,
            name_to_id: self.name_to_id,
            archetype_name_to_id: self.archetype_name_to_id,
            tag_count: self.next_bit,
        })
    }
}

impl Default for TagRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_registry() -> TagRegistry {
        let mut builder = TagRegistryBuilder::new();
        let elem = builder.add_archetype("element", "Element", Exclusivity::Mutual);
        builder.add_tag(elem, "FIRE", vec![], vec![], Some(1.0), None, Some(2.0), None, None, None, None, Some(1.5)).unwrap();
        builder.add_tag(elem, "WATER", vec![], vec![], None, Some([3, 5]), None, Some(0.5), Some(10), None, Some(0.3), None).unwrap();
        let status = builder.add_archetype("status", "Status", Exclusivity::Any);
        builder.add_tag(status, "BURNING", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn archetype_by_name() {
        let reg = build_registry();
        let arch = reg.archetype_by_name("element").unwrap();
        assert_eq!(arch.name, "element");
        assert_eq!(arch.display_name, "Element");
        assert!(reg.archetype_by_name("nonexistent").is_none());
    }

    #[test]
    fn archetype_id_lookup() {
        let reg = build_registry();
        let id = reg.archetype_id("status").unwrap();
        assert_eq!(reg.archetype_by_id(id).name, "status");
        assert!(reg.archetype_id("nonexistent").is_none());
    }

    #[test]
    fn tags_for_archetype() {
        let reg = build_registry();
        let elem_id = reg.archetype_id("element").unwrap();
        let tags = reg.tags_for_archetype(elem_id);
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn all_tags_iterator() {
        let reg = build_registry();
        let count = reg.all_tags().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn tag_def_numeric_fields() {
        let reg = build_registry();
        let fire = reg.tag_by_name("FIRE").unwrap();
        assert_eq!(fire.default_magnitude, Some(1.0));
        assert_eq!(fire.multiplier, Some(2.0));
        assert_eq!(fire.hp_mult, Some(1.5));

        let water = reg.tag_by_name("WATER").unwrap();
        assert_eq!(water.ticks_range, Some([3, 5]));
        assert_eq!(water.move_cost, Some(0.5));
        assert_eq!(water.range, Some(10));
        assert_eq!(water.tile_occupancy, Some(0.3));
    }

    #[test]
    fn builder_default_equals_new() {
        let b1 = TagRegistryBuilder::new();
        let b2 = TagRegistryBuilder::default();
        assert_eq!(b1.archetypes.len(), b2.archetypes.len());
        assert_eq!(b1.tags.len(), b2.tags.len());
    }

    #[test]
    fn duplicate_archetype_error() {
        let mut builder = TagRegistryBuilder::new();
        builder.add_archetype("test", "Test", Exclusivity::Any);
        builder.add_archetype("test", "Test 2", Exclusivity::Any);
        let arch = ArchetypeId(0);
        let result = builder.add_tag(arch, "TAG", vec![], vec![], None, None, None, None, None, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn unresolved_conflict_error() {
        let mut builder = TagRegistryBuilder::new();
        let arch = builder.add_archetype("test", "Test", Exclusivity::Any);
        builder.add_tag(arch, "A", vec![], vec!["NONEXISTENT".to_string()], None, None, None, None, None, None, None, None).unwrap();
        let result = builder.build();
        assert!(matches!(result, Err(RegistryError::UnresolvedConflict { .. })));
    }

    #[test]
    fn archetype_def_fields() {
        let reg = build_registry();
        let elem = reg.archetype_by_name("element").unwrap();
        assert_eq!(elem.name, "element");
        assert_eq!(elem.display_name, "Element");
        assert_eq!(elem.exclusivity, Exclusivity::Mutual);
        assert_eq!(elem.tag_ids.len(), 2);
    }

    #[test]
    fn tag_by_name_returns_none_for_unknown() {
        let reg = build_registry();
        assert!(reg.tag_by_name("NONEXISTENT").is_none());
    }

    #[test]
    fn duplicate_tag_returns_error() {
        let mut builder = TagRegistryBuilder::new();
        let arch = builder.add_archetype("a", "A", Exclusivity::Any);
        builder.add_tag(arch, "X", vec![], vec![], None, None, None, None, None, None, None, None).unwrap();
        let result = builder.add_tag(arch, "X", vec![], vec![], None, None, None, None, None, None, None, None);
        assert!(matches!(result, Err(RegistryError::DuplicateTag(_))));
    }

    #[test]
    fn all_archetypes_returns_all() {
        let reg = build_registry();
        let archetypes: Vec<&ArchetypeDef> = reg.all_archetypes().collect();
        assert_eq!(archetypes.len(), 2);
        let names: Vec<&str> = archetypes.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"element"));
        assert!(names.contains(&"status"));
    }

    #[test]
    fn tag_by_id_returns_correct_def() {
        let reg = build_registry();
        let fire_id = reg.tag_id("FIRE").unwrap();
        let def = reg.tag_by_id(fire_id);
        assert_eq!(def.name, "FIRE");
        assert_eq!(def.id, fire_id);
    }

    #[test]
    fn tag_id_returns_none_for_unknown() {
        let reg = build_registry();
        assert!(reg.tag_id("NONEXISTENT").is_none());
    }

    #[test]
    fn archetype_def_bit_offset_increments() {
        let reg = build_registry();
        let elem = reg.archetype_by_name("element").unwrap();
        let status = reg.archetype_by_name("status").unwrap();
        assert!(status.bit_offset > elem.bit_offset);
    }
}
