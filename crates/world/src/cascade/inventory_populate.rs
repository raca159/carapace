use bevy_ecs::prelude::*;
use rand::Rng;
use rand::SeedableRng;

use game_core::{Glyph, Inventory, Item, Name};
use game_tags::{TagRegistry, Tags, TagValue};

use crate::cascade::{CascadeEngine, TaggedWeight};
use crate::cascade::inventory::{self, InventoryItem};
use crate::loot::{LootTables, roll_loot_for_table};

pub struct PopulateContext {
    pub dungeon_type: Option<String>,
    pub depth: Option<u32>,
    pub dungeon_seed: Option<u64>,
}

fn resolve_capacity(tags: &Tags, registry: &TagRegistry) -> usize {
    for tag_name in &["INVENTORY_HUGE", "INVENTORY_LARGE", "INVENTORY_MEDIUM", "INVENTORY_SMALL", "INVENTORY_TINY"] {
        if let Some(tid) = registry.tag_id(tag_name) {
            if tags.has(tid) {
                if let Some(mag) = registry.tag_by_id(tid).default_magnitude {
                    return mag as usize;
                }
            }
        }
    }
    8
}

fn has_content_tag(tags: &Tags, tag_name: &str, registry: &TagRegistry) -> bool {
    registry.tag_id(tag_name).is_some_and(|id| tags.has(id))
}

fn create_item_entity(
    world: &mut World,
    item_def: &crate::cascade::ItemDef,
    registry: &TagRegistry,
) -> Entity {
    let mut item_tags = Tags::new(registry.tag_count());
    for t in &item_def.tags {
        if let Some(tid) = registry.tag_id(t) {
            item_tags.add_tag(tid, TagValue::None, registry);
        }
    }
    world.spawn((
        Glyph { char: item_def.glyph, color: (item_def.color[0], item_def.color[1], item_def.color[2]) },
        item_tags,
        Name(item_def.name.clone()),
        Item,
    )).id()
}

fn fill_trade(
    entity_tags: &Tags,
    faction_id: Option<game_tags::TagId>,
    location_supply: Option<&[TaggedWeight]>,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    inventory::roll_inventory(entity_tags, 3, faction_id, location_supply, engine, registry, rng)
}

fn fill_equipment(
    _entity_tags: &Tags,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    let equip_tag_ids: Vec<game_tags::TagId> = ["EQUIP_WEAPON", "EQUIP_ARMOR", "EQUIP_ACCESSORY"]
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();

    let candidates: Vec<&crate::cascade::ItemDef> = engine.items.iter()
        .filter(|item| {
            item.tags.iter()
                .filter_map(|t| registry.tag_id(t))
                .any(|id| equip_tag_ids.contains(&id))
        })
        .collect();

    if candidates.is_empty() { return vec![]; }

    let roll_count = 1 + (rng.random::<f32>() * 2.0) as u32;
    let mut items = Vec::new();
    for _ in 0..roll_count {
        let total: f32 = candidates.iter().map(|c| c.weight).sum();
        if total <= 0.0 { break; }
        let roll = rng.random::<f32>() * total;
        let mut accum = 0.0f32;
        if let Some(selected) = candidates.iter().find(|c| { accum += c.weight; roll < accum }) {
            items.push(InventoryItem {
                item_id: selected.id.clone(),
                quantity: 1,
                trade_only: false,
            });
        }
    }
    items
}

fn fill_fallback(
    entity_tags: &Tags,
    faction_id: Option<game_tags::TagId>,
    location_supply: Option<&[TaggedWeight]>,
    engine: &CascadeEngine,
    registry: &TagRegistry,
    rng: &mut impl Rng,
) -> Vec<InventoryItem> {
    inventory::roll_inventory(entity_tags, 2, faction_id, location_supply, engine, registry, rng)
}

pub fn populate_inventories(
    world: &mut World,
    ctx: Option<&PopulateContext>,
) {
    let registry = match world.get_resource::<TagRegistry>() {
        Some(r) => r.clone(),
        None => return,
    };
    let cascade = match world.get_resource::<CascadeEngine>() {
        Some(c) => c.clone(),
        None => return,
    };
    let loot_tables = world.get_resource::<LootTables>().cloned();

    let has_inv_id = match registry.tag_id("HAS_INVENTORY") {
        Some(id) => id,
        None => return,
    };

    let entities: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Tags)>();
        q.iter(world)
            .filter(|(_, tags)| tags.has(has_inv_id))
            .map(|(e, _)| e)
            .collect()
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(
        world.get_resource::<crate::map::WorldMap>()
            .map(|m| m.seed.0.wrapping_add(0x4E56))
            .unwrap_or(0xDEAD)
    );

    for entity in entities {
        if world.get::<Inventory>(entity).is_some() {
            continue;
        }

        let entity_tags = match world.get::<Tags>(entity) {
            Some(t) => t.clone(),
            None => continue,
        };

        let capacity = resolve_capacity(&entity_tags, &registry);
        let is_loot = has_content_tag(&entity_tags, "INVENTORY_LOOT", &registry);
        let is_trade = has_content_tag(&entity_tags, "INVENTORY_TRADE", &registry);
        let is_equipment = has_content_tag(&entity_tags, "INVENTORY_EQUIPMENT", &registry);

        let faction_id = world.get::<crate::faction::Faction>(entity)
            .and_then(|f| {
                let faction_rels = world.get_resource::<crate::faction::FactionRelationships>()?;
                let name = faction_rels.faction_name(f.faction_id)?;
                registry.tag_id(&name)
            });

        let location_supply = {
            let pos = world.get::<game_core::Position>(entity);
            let loc_map = world.get_resource::<crate::cascade::LocationMap>();
            let economies = world.get_resource::<crate::cascade::RegionEconomies>();
            match (pos, loc_map, economies) {
                (Some(p), Some(lm), Some(econ)) => {
                    let loc = crate::cascade::locations::location_at(&lm.locations, p.x, p.y);
                    loc.and_then(|l| econ.economies.get(&l.id))
                        .map(|pc| pc.location_supply.as_slice())
                }
                _ => None,
            }
        };

        let mut all_items: Vec<InventoryItem> = Vec::new();

        if is_loot {
            if let (Some(ref tables), Some(ref dctx)) = (loot_tables.as_ref(), ctx) {
                let dtype = dctx.dungeon_type.as_deref().unwrap_or("standard");
                let depth = dctx.depth.unwrap_or(1);
                let matching = tables.tables_for_dungeon(dtype, depth);
                for table in matching {
                    let drops = roll_loot_for_table(table, &mut rng);
                    for drop in drops {
                        all_items.push(InventoryItem {
                            item_id: drop.name.clone(),
                            quantity: drop.quantity,
                            trade_only: false,
                        });
                    }
                }
            }
        }

        if is_trade {
            let trade_items = fill_trade(
                &entity_tags, faction_id, location_supply,
                &cascade, &registry, &mut rng,
            );
            all_items.extend(trade_items);
        }

        if is_equipment {
            let equip_items = fill_equipment(
                &entity_tags, &cascade, &registry, &mut rng,
            );
            all_items.extend(equip_items);
        }

        if !is_loot && !is_trade && !is_equipment {
            let fallback_items = fill_fallback(
                &entity_tags, faction_id, location_supply,
                &cascade, &registry, &mut rng,
            );
            all_items.extend(fallback_items);
        }

        let mut item_entities: Vec<Entity> = Vec::new();
        for inv_item in all_items.iter().take(capacity) {
            if let Some(item_def) = cascade.item_by_id.get(&inv_item.item_id) {
                for _ in 0..inv_item.quantity {
                    if item_entities.len() >= capacity { break; }
                    let ie = create_item_entity(world, item_def, &registry);
                    item_entities.push(ie);
                }
            }
        }

        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(Inventory {
                items: item_entities,
                capacity,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::load_tag_registry;

    use crate::map::WorldMap;
    use crate::seed::WorldSeed;

    const TAGS_TOML: &str = include_str!("../../../../assets/config/tags.toml");
    const ITEMS_TOML: &str = include_str!("../../../../assets/config/items.toml");
    const BIOMES_TOML: &str = include_str!("../../../../assets/config/region_biomes.toml");
    const FACTIONS_TOML: &str = include_str!("../../../../assets/config/faction_economy.toml");
    const LOCATIONS_TOML: &str = include_str!("../../../../assets/config/location_types.toml");

    fn empty_world_map() -> WorldMap {
        WorldMap {
            width: 0,
            height: 0,
            depth: 0,
            current_z: 0,
            seed: WorldSeed(0),
            tiles: Vec::new(),
        }
    }

    fn setup() -> (World, TagRegistry, CascadeEngine) {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let cascade = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        let mut world = World::new();
        world.insert_resource(registry.clone());
        world.insert_resource(cascade.clone());
        world.insert_resource(empty_world_map());
        (world, registry, cascade)
    }

    #[test]
    fn test_resolve_capacity_tiny() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("INVENTORY_TINY").unwrap(), TagValue::None, &registry);
        assert_eq!(resolve_capacity(&tags, &registry), 4);
    }

    #[test]
    fn test_resolve_capacity_medium() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("INVENTORY_MEDIUM").unwrap(), TagValue::None, &registry);
        assert_eq!(resolve_capacity(&tags, &registry), 12);
    }

    #[test]
    fn test_resolve_capacity_default() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let tags = Tags::new(registry.tag_count());
        assert_eq!(resolve_capacity(&tags, &registry), 8);
    }

    #[test]
    fn test_has_inventory_gets_inventory_component() {
        let (mut world, registry, _cascade) = {
            let reg = load_tag_registry(TAGS_TOML).unwrap();
            let eng = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
            let mut w = World::new();
            w.insert_resource(reg.clone());
            w.insert_resource(eng.clone());
            w.insert_resource(empty_world_map());
            (w, reg, eng)
        };

        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("HAS_INVENTORY").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("INVENTORY_TINY").unwrap(), TagValue::None, &registry);

        let entity = world.spawn((
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Glyph { char: 'M', color: (255, 200, 50) },
            tags,
            game_core::Name("Test Merchant".to_string()),
            game_core::Creature,
        )).id();

        populate_inventories(&mut world, None);

        let inv = world.get::<Inventory>(entity);
        assert!(inv.is_some(), "Entity with HAS_INVENTORY should get Inventory component");
        let inv = inv.unwrap();
        assert_eq!(inv.capacity, 4, "INVENTORY_TINY should give 4 slots");
    }

    #[test]
    fn test_no_has_inventory_no_component() {
        let registry = load_tag_registry(TAGS_TOML).unwrap();
        let cascade = CascadeEngine::load(ITEMS_TOML, BIOMES_TOML, FACTIONS_TOML, LOCATIONS_TOML).unwrap();
        let mut world = World::new();
        world.insert_resource(registry.clone());
        world.insert_resource(cascade);
        world.insert_resource(empty_world_map());

        let tags = Tags::new(registry.tag_count());
        let entity = world.spawn((
            game_core::Position { x: 5, y: 5, z: 0 },
            game_core::Glyph { char: 'x', color: (100, 100, 100) },
            tags,
            game_core::Name("No Inventory Entity".to_string()),
            game_core::Creature,
        )).id();

        populate_inventories(&mut world, None);

        let inv = world.get::<Inventory>(entity);
        assert!(inv.is_none(), "Entity without HAS_INVENTORY should not get Inventory component");
    }
}
