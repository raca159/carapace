use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use game_core::{
    BehaviorState, Creature, Equipment, Glyph, Health, Inventory, Item, Name, Position,
};
use game_tags::{snapshot_to_tags, tags_to_snapshot, TagRegistry, TagValueSnapshot, Tags};
use crate::faction::{Faction, FactionRelationships};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionExport {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthExport {
    pub current: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagExport {
    pub name: String,
    pub value: TagValueSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSlotExport {
    pub slot: String,
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub tags: Vec<TagExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemExport {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub tags: Vec<TagExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorExport {
    pub home_x: Option<u32>,
    pub home_y: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExport {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub position: PositionExport,
    pub health: HealthExport,
    pub tags: Vec<TagExport>,
    pub behavior: BehaviorExport,
    pub faction: Option<String>,
    pub equipment: Vec<EquipmentSlotExport>,
    pub inventory: Vec<InventoryItemExport>,
    pub is_creature: bool,
    pub is_item: bool,
}

pub fn export_entity(
    world: &World,
    entity: Entity,
    registry: &TagRegistry,
) -> Option<EntityExport> {
    let pos = world.get::<Position>(entity)?;
    let glyph = world.get::<Glyph>(entity)?;
    let health = world.get::<Health>(entity)?;
    let name = world.get::<Name>(entity)?;
    let tags = world.get::<Tags>(entity)?;
    let snapshot = tags_to_snapshot(tags, registry);

    let tag_exports: Vec<TagExport> = snapshot
        .tags
        .into_iter()
        .map(|(name, value)| TagExport { name, value })
        .collect();

    let behavior = world
        .get::<BehaviorState>(entity)
        .map(|b| BehaviorExport {
            home_x: b.home_pos.map(|p| p.x),
            home_y: b.home_pos.map(|p| p.y),
        })
        .unwrap_or(BehaviorExport {
            home_x: None,
            home_y: None,
        });

    let faction = world
        .get::<Faction>(entity)
        .and_then(|f| {
            world
                .get_resource::<FactionRelationships>()
                .and_then(|rels| {
                    rels.name_id_pairs()
                        .find(|(_, id)| **id == f.faction_id)
                        .map(|(name, _)| name.clone())
                })
        });

    let equipment = if let Some(eq) = world.get::<Equipment>(entity) {
        let mut slots = Vec::new();
        for (slot_name, slot_entity) in [
            ("weapon", eq.weapon),
            ("armor", eq.armor),
            ("accessory", eq.accessory),
        ] {
            if let Some(item_ent) = slot_entity
                && let Some(item_export) = export_item(world, item_ent, registry) {
                    slots.push(EquipmentSlotExport {
                        slot: slot_name.to_string(),
                        name: item_export.name,
                        glyph: item_export.glyph,
                        color: item_export.color,
                        tags: item_export.tags,
                    });
            }
        }
        slots
    } else {
        Vec::new()
    };

    let inventory = if let Some(inv) = world.get::<Inventory>(entity) {
        inv.items
            .iter()
            .filter_map(|&item_ent| export_item(world, item_ent, registry))
            .collect()
    } else {
        Vec::new()
    };

    Some(EntityExport {
        name: name.0.clone(),
        glyph: glyph.char,
        color: glyph.color,
        position: PositionExport {
            x: pos.x,
            y: pos.y,
        },
        health: HealthExport {
            current: health.current,
            max: health.max,
        },
        tags: tag_exports,
        behavior,
        faction,
        equipment,
        inventory,
        is_creature: world.get::<Creature>(entity).is_some(),
        is_item: world.get::<Item>(entity).is_some(),
    })
}

fn export_item(
    world: &World,
    entity: Entity,
    registry: &TagRegistry,
) -> Option<InventoryItemExport> {
    let glyph = world.get::<Glyph>(entity)?;
    let name = world
        .get::<Name>(entity)
        .map(|n| n.0.clone())
        .unwrap_or_default();
    let tags = world.get::<Tags>(entity).map(|t| {
        tags_to_snapshot(t, registry)
            .tags
            .into_iter()
            .map(|(name, value)| TagExport { name, value })
            .collect()
    }).unwrap_or_default();

    Some(InventoryItemExport {
        name,
        glyph: glyph.char,
        color: glyph.color,
        tags,
    })
}

pub fn import_entity(
    world: &mut World,
    export: &EntityExport,
    registry: &TagRegistry,
) -> Entity {
    let tags_snapshot = game_tags::TagsSnapshot {
        tags: export
            .tags
            .iter()
            .map(|t| (t.name.clone(), t.value.clone()))
            .collect(),
    };
    let tags = snapshot_to_tags(&tags_snapshot, registry);

    let home_pos = match (export.behavior.home_x, export.behavior.home_y) {
        (Some(x), Some(y)) => Some(Position { x, y, z: 0 }),
        _ => None,
    };

    let mut entity_cmds = world.spawn((
        Position { x: export.position.x, y: export.position.y, z: 0 },
        Glyph {
            char: export.glyph,
            color: export.color,
        },
        Health {
            current: export.health.current,
            max: export.health.max,
        },
        tags,
        Name(export.name.clone()),
        BehaviorState { home_pos },
    ));

    if export.is_creature {
        entity_cmds.insert(Creature);
    }
    if export.is_item {
        entity_cmds.insert(Item);
    }

    let entity = entity_cmds.id();

    if let Some(ref faction_name) = export.faction {
        let rels = world.get_resource::<FactionRelationships>().cloned();
        if let Some(rels) = rels
            && let Some(fid) = rels.faction_id(faction_name) {
                world.entity_mut(entity).insert(Faction { faction_id: fid });
        }
    }

    if !export.equipment.is_empty() {
        let mut equipment = Equipment::default();
        for eq in &export.equipment {
            let eq_tags_snapshot = game_tags::TagsSnapshot {
                tags: eq
                    .tags
                    .iter()
                    .map(|t| (t.name.clone(), t.value.clone()))
                    .collect(),
            };
            let eq_tags = snapshot_to_tags(&eq_tags_snapshot, registry);
            let item_ent = world
                .spawn((
                    Position { x: export.position.x, y: export.position.y, z: 0 },
                    Glyph {
                        char: eq.glyph,
                        color: eq.color,
                    },
                    eq_tags,
                    Name(eq.name.clone()),
                    Item,
                ))
                .id();
            match eq.slot.as_str() {
                "weapon" => equipment.weapon = Some(item_ent),
                "armor" => equipment.armor = Some(item_ent),
                "accessory" => equipment.accessory = Some(item_ent),
                _ => {}
            }
        }
        world.entity_mut(entity).insert(equipment);
    }

    if !export.inventory.is_empty() {
        let capacity = export.inventory.len();
        let items: Vec<Entity> = export
            .inventory
            .iter()
            .map(|item| {
                let item_tags_snapshot = game_tags::TagsSnapshot {
                    tags: item
                        .tags
                        .iter()
                        .map(|t| (t.name.clone(), t.value.clone()))
                        .collect(),
                };
                let item_tags = snapshot_to_tags(&item_tags_snapshot, registry);
                world
                    .spawn((
                        Position { x: export.position.x, y: export.position.y, z: 0 },
                        Glyph {
                            char: item.glyph,
                            color: item.color,
                        },
                        item_tags,
                        Name(item.name.clone()),
                        Item,
                    ))
                    .id()
            })
            .collect();
        world.entity_mut(entity).insert(Inventory { items, capacity });
    }

    entity
}

pub fn export_entity_to_toml(
    world: &World,
    entity: Entity,
    registry: &TagRegistry,
) -> Option<String> {
    let export = export_entity(world, entity, registry)?;
    toml::to_string_pretty(&export).ok()
}

pub fn import_entity_from_toml(
    world: &mut World,
    toml_str: &str,
    registry: &TagRegistry,
) -> Result<Entity, String> {
    let export: EntityExport =
        toml::from_str(toml_str).map_err(|e| format!("Failed to parse entity TOML: {}", e))?;
    Ok(import_entity(world, &export, registry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::load_tag_registry;
    use crate::faction::load_factions;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");
    const FACTIONS_TOML: &str = include_str!("../../../assets/config/factions.toml");

    fn setup_world() -> (World, TagRegistry) {
        let mut world = World::new();
        let registry = load_tag_registry(TAGS_TOML).expect("tags should load");
        world.insert_resource(registry.clone());
        let (_factions, rels) = load_factions(FACTIONS_TOML).expect("factions should load");
        world.insert_resource(rels);
        (world, registry)
    }

    fn create_test_entity(world: &mut World, registry: &TagRegistry) -> Entity {
        let mut tags = Tags::new(registry.tag_count());
        if let Some(humanoid) = registry.tag_id("HUMANOID") {
            tags.add_tag(humanoid, game_tags::TagValue::None, registry);
        }
        if let Some(medium) = registry.tag_id("MEDIUM") {
            tags.add_tag(medium, game_tags::TagValue::None, registry);
        }
        if let Some(aggressive) = registry.tag_id("AGGRESSIVE") {
            tags.add_tag(aggressive, game_tags::TagValue::None, registry);
        }

        let entity = world
            .spawn((
                Position { x: 5, y: 10, z: 0 },
                Glyph {
                    char: '@',
                    color: (255, 200, 150),
                },
                Health {
                    current: 55,
                    max: 55,
                },
                tags,
                Name("Test Human".to_string()),
                Creature,
                BehaviorState {
                    home_pos: Some(Position { x: 5, y: 10, z: 0 }),
                },
            ))
            .id();

        if registry.tag_id("UNDEAD").is_some() {
            let faction_id = world
                .get_resource::<FactionRelationships>()
                .and_then(|rels| rels.faction_id("sanguine_elite"));
            if let Some(fid) = faction_id {
                world.entity_mut(entity).insert(Faction { faction_id: fid });
            }
        }

        let mut eq_tags = Tags::new(registry.tag_count());
        if let Some(metal) = registry.tag_id("METAL") {
            eq_tags.add_tag(metal, game_tags::TagValue::None, registry);
        }
        if let Some(common) = registry.tag_id("COMMON") {
            eq_tags.add_tag(common, game_tags::TagValue::None, registry);
        }
        let sword = world
            .spawn((
                Position { x: 5, y: 10, z: 0 },
                Glyph {
                    char: '/',
                    color: (180, 180, 180),
                },
                eq_tags,
                Name("Metal Blade".to_string()),
                Item,
            ))
            .id();
        let equipment = Equipment { weapon: Some(sword), ..Default::default() };
        world.entity_mut(entity).insert(equipment);

        entity
    }

    #[test]
    fn test_export_entity_basic_components() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let export = export_entity(&world, entity, &registry).unwrap();

        assert_eq!(export.name, "Test Human");
        assert_eq!(export.glyph, '@');
        assert_eq!(export.color, (255, 200, 150));
        assert_eq!(export.position.x, 5);
        assert_eq!(export.position.y, 10);
        assert_eq!(export.health.current, 55);
        assert_eq!(export.health.max, 55);
        assert!(export.is_creature);
        assert!(!export.is_item);
    }

    #[test]
    fn test_export_entity_tags() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let export = export_entity(&world, entity, &registry).unwrap();
        let names: Vec<&str> = export.tags.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"HUMANOID"));
        assert!(names.contains(&"MEDIUM"));
        assert!(names.contains(&"AGGRESSIVE"));
    }

    #[test]
    fn test_export_entity_behavior() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let export = export_entity(&world, entity, &registry).unwrap();
        assert_eq!(export.behavior.home_x, Some(5));
        assert_eq!(export.behavior.home_y, Some(10));
    }

    #[test]
    fn test_export_entity_equipment() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let export = export_entity(&world, entity, &registry).unwrap();
        assert_eq!(export.equipment.len(), 1);
        assert_eq!(export.equipment[0].slot, "weapon");
        assert_eq!(export.equipment[0].name, "Metal Blade");
    }

    #[test]
    fn test_full_toml_round_trip() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let toml_str = export_entity_to_toml(&world, entity, &registry).unwrap();
        assert!(toml_str.contains("Test Human"));
        assert!(toml_str.contains("HUMANOID"));

        let mut import_world = World::new();
        import_world.insert_resource(registry.clone());
        let (_factions, rels) = load_factions(FACTIONS_TOML).unwrap();
        import_world.insert_resource(rels);

        let imported = import_entity_from_toml(&mut import_world, &toml_str, &registry).unwrap();

        assert_eq!(
            import_world.get::<Name>(imported).unwrap().0,
            "Test Human"
        );
        assert_eq!(
            import_world.get::<Position>(imported).unwrap().x,
            5
        );
        assert_eq!(
            import_world.get::<Health>(imported).unwrap().current,
            55
        );
        assert!(import_world.get::<Creature>(imported).is_some());

        let imported_tags = import_world.get::<Tags>(imported).unwrap();
        let humanoid = registry.tag_id("HUMANOID").unwrap();
        let medium = registry.tag_id("MEDIUM").unwrap();
        let aggressive = registry.tag_id("AGGRESSIVE").unwrap();
        assert!(imported_tags.has(humanoid));
        assert!(imported_tags.has(medium));
        assert!(imported_tags.has(aggressive));

        let imported_equip = import_world.get::<Equipment>(imported).unwrap();
        assert!(imported_equip.weapon.is_some());
        let weapon_entity = imported_equip.weapon.unwrap();
        assert_eq!(
            import_world.get::<Name>(weapon_entity).unwrap().0,
            "Metal Blade"
        );
    }

    #[test]
    fn test_export_nonexistent_entity() {
        let (mut world, registry) = setup_world();
        let bad_entity = world.spawn(()).id();
        let result = export_entity(&world, bad_entity, &registry);
        assert!(result.is_none());
    }

    #[test]
    fn test_export_import_preserves_faction() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let toml_str = export_entity_to_toml(&world, entity, &registry).unwrap();

        let mut import_world = World::new();
        import_world.insert_resource(registry.clone());
        let (_factions, rels) = load_factions(FACTIONS_TOML).unwrap();
        import_world.insert_resource(rels);

        let imported = import_entity_from_toml(&mut import_world, &toml_str, &registry).unwrap();
        let faction = import_world.get::<Faction>(imported);
        assert!(faction.is_some());
    }

    #[test]
    fn test_export_import_preserves_equipment_tags() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let toml_str = export_entity_to_toml(&world, entity, &registry).unwrap();

        let mut import_world = World::new();
        import_world.insert_resource(registry.clone());
        let (_factions, rels) = load_factions(FACTIONS_TOML).unwrap();
        import_world.insert_resource(rels);

        let imported = import_entity_from_toml(&mut import_world, &toml_str, &registry).unwrap();
        let equip = import_world.get::<Equipment>(imported).unwrap();
        let weapon = equip.weapon.unwrap();
        let weapon_tags = import_world.get::<Tags>(weapon).unwrap();
        let metal = registry.tag_id("METAL").unwrap();
        let common = registry.tag_id("COMMON").unwrap();
        assert!(weapon_tags.has(metal));
        assert!(weapon_tags.has(common));
    }

    #[test]
    fn test_export_to_toml_valid() {
        let (mut world, registry) = setup_world();
        let entity = create_test_entity(&mut world, &registry);

        let toml_str = export_entity_to_toml(&world, entity, &registry).unwrap();
        toml::from_str::<EntityExport>(&toml_str).expect("export must be valid TOML");
    }

    #[test]
    fn test_export_with_inventory() {
        let (mut world, registry) = setup_world();

        let mut tags = Tags::new(registry.tag_count());
        if let Some(humanoid) = registry.tag_id("HUMANOID") {
            tags.add_tag(humanoid, game_tags::TagValue::None, &registry);
        }

        let entity = world
            .spawn((
                Position { x: 0, y: 0, z: 0 },
                Glyph {
                    char: '@',
                    color: (255, 255, 255),
                },
                Health {
                    current: 10,
                    max: 10,
                },
                tags,
                Name("Merchant".to_string()),
                Creature,
                BehaviorState { home_pos: None },
            ))
            .id();

        let mut item_tags = Tags::new(registry.tag_count());
        if let Some(common) = registry.tag_id("COMMON") {
            item_tags.add_tag(common, game_tags::TagValue::None, &registry);
        }
        let item = world
            .spawn((
                Position { x: 0, y: 0, z: 0 },
                Glyph {
                    char: '$',
                    color: (255, 215, 0),
                },
                item_tags,
                Name("Chip".to_string()),
                Item,
            ))
            .id();
        world.entity_mut(entity).insert(Inventory {
            items: vec![item],
            capacity: 10,
        });

        let toml_str = export_entity_to_toml(&world, entity, &registry).unwrap();

        let mut import_world = World::new();
        import_world.insert_resource(registry.clone());
        let imported = import_entity_from_toml(&mut import_world, &toml_str, &registry).unwrap();

        let inv = import_world.get::<Inventory>(imported).unwrap();
        assert_eq!(inv.items.len(), 1);
        let inv_item = inv.items[0];
        assert_eq!(
            import_world.get::<Name>(inv_item).unwrap().0,
            "Chip"
        );
        assert!(import_world.get::<Item>(inv_item).is_some());
    }
}
