use bevy_ecs::prelude::World;

use game_core::{Equipment, EquipmentSlot, EventBus, GameEvent, Glyph, Inventory, Name, Player};

pub fn handle_equip(ecs_world: &mut World, cursor: usize) {
    use bevy_ecs::query::With;

    let player_entity = {
        let mut pq = ecs_world.query_filtered::<bevy_ecs::entity::Entity, With<Player>>();
        pq.single(ecs_world).ok()
    };
    let Some(player_entity) = player_entity else { return };

    let item_entity = {
        let inv = ecs_world.get::<Inventory>(player_entity);
        match inv {
            Some(inv) if cursor < inv.items.len() => inv.items[cursor],
            _ => return,
        }
    };

    let registry = ecs_world.resource::<game_tags::TagRegistry>().clone();
    let equip_weapon_id = registry.tag_id("EQUIP_WEAPON");
    let equip_armor_id = registry.tag_id("EQUIP_ARMOR");
    let equip_acc_id = registry.tag_id("EQUIP_ACCESSORY");

    let item_tags = ecs_world.get::<game_tags::Tags>(item_entity).cloned();
    let Some(item_tags) = item_tags else { return };

    let slot = if equip_weapon_id.is_some_and(|id| item_tags.has(id)) {
        Some(EquipmentSlot::Weapon)
    } else if equip_armor_id.is_some_and(|id| item_tags.has(id)) {
        Some(EquipmentSlot::Armor)
    } else if equip_acc_id.is_some_and(|id| item_tags.has(id)) {
        Some(EquipmentSlot::Accessory)
    } else {
        None
    };

    let Some(slot) = slot else {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("That item cannot be equipped.".to_string()));
        }
        return;
    };

    let item_name = get_item_display_name(ecs_world, item_entity, &registry);

    {
        let mut inv = ecs_world.get_mut::<Inventory>(player_entity).unwrap();
        inv.items.retain(|&e| e != item_entity);
    }

    let old_equipped = {
        let mut equip = ecs_world.get_mut::<Equipment>(player_entity).unwrap();
        match slot {
            EquipmentSlot::Weapon => equip.weapon.replace(item_entity),
            EquipmentSlot::Armor => equip.armor.replace(item_entity),
            EquipmentSlot::Accessory => equip.accessory.replace(item_entity),
        }
    };

    if let Some(old) = old_equipped {
        let mut inv = ecs_world.get_mut::<Inventory>(player_entity).unwrap();
        inv.items.push(old);
    }

    {
        let tag_ids: Vec<game_tags::TagId> = item_tags.iter_present().collect();
        let mut player_tags = ecs_world.get_mut::<game_tags::Tags>(player_entity).unwrap();
        for tag_id in tag_ids {
            player_tags.add_tag(tag_id, game_tags::TagValue::None, &registry);
        }
    }

    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        bus.push(GameEvent::ItemEquipped { item_name });
    }
}

#[allow(dead_code)]
pub fn handle_unequip(ecs_world: &mut World, slot: EquipmentSlot) {
    use bevy_ecs::query::With;

    let player_entity = {
        let mut pq = ecs_world.query_filtered::<bevy_ecs::entity::Entity, With<Player>>();
        pq.single(ecs_world).ok()
    };
    let Some(player_entity) = player_entity else { return };

    let equipped_entity = {
        let equip = ecs_world.get::<Equipment>(player_entity);
        match equip {
            Some(e) => match slot {
                EquipmentSlot::Weapon => e.weapon,
                EquipmentSlot::Armor => e.armor,
                EquipmentSlot::Accessory => e.accessory,
            },
            None => None,
        }
    };

    let Some(entity) = equipped_entity else {
        if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("Nothing equipped in that slot.".to_string()));
        }
        return;
    };

    let registry = ecs_world.resource::<game_tags::TagRegistry>().clone();
    let item_name = get_item_display_name(ecs_world, entity, &registry);

    {
        let mut equip = ecs_world.get_mut::<Equipment>(player_entity).unwrap();
        match slot {
            EquipmentSlot::Weapon => equip.weapon = None,
            EquipmentSlot::Armor => equip.armor = None,
            EquipmentSlot::Accessory => equip.accessory = None,
        }
    }

    {
        let mut inv = ecs_world.get_mut::<Inventory>(player_entity).unwrap();
        inv.items.push(entity);
    }

    {
        let item_tags = ecs_world.get::<game_tags::Tags>(entity).cloned();
        if let Some(item_tags) = item_tags {
            let tag_ids: Vec<game_tags::TagId> = item_tags.iter_present().collect();
            let mut player_tags = ecs_world.get_mut::<game_tags::Tags>(player_entity).unwrap();
            for tag_id in tag_ids {
                player_tags.remove_tag(tag_id, &registry);
            }
        }
    }

    if let Some(mut bus) = ecs_world.get_resource_mut::<EventBus>() {
        bus.push(GameEvent::ItemUnequipped { item_name });
    }
}

pub fn calc_weapon_damage(equipment: &Equipment, world: &World, registry: &game_tags::TagRegistry) -> u32 {
    let weapon_entity = match equipment.weapon {
        Some(e) => e,
        None => return 0,
    };

    let tags = match world.get::<game_tags::Tags>(weapon_entity) {
        Some(t) => t.clone(),
        None => return 5,
    };

    let base: u32 = 5;
    let material_bonus: u32 = if registry.tag_id("METAL").is_some_and(|id| tags.has(id)) { 3 }
                              else if registry.tag_id("STONE").is_some_and(|id| tags.has(id)) { 1 }
                              else { 0 };

    let quality_mult = get_quality_multiplier(&tags, registry);

    (base + material_bonus).saturating_mul(quality_mult)
}

pub fn calc_armor_protection(equipment: &Equipment, world: &World, registry: &game_tags::TagRegistry) -> u32 {
    let armor_entity = match equipment.armor {
        Some(e) => e,
        None => return 0,
    };

    let tags = match world.get::<game_tags::Tags>(armor_entity) {
        Some(t) => t.clone(),
        None => return 0,
    };

    let material_bonus: u32 = if registry.tag_id("METAL").is_some_and(|id| tags.has(id)) { 5 }
                              else if registry.tag_id("LEATHER").is_some_and(|id| tags.has(id)) { 3 }
                              else if registry.tag_id("CLOTH").is_some_and(|id| tags.has(id)) { 1 }
                              else { 0 };

    let quality_mult = get_quality_multiplier(&tags, registry);

    material_bonus.saturating_mul(quality_mult)
}

pub fn get_quality_multiplier(tags: &game_tags::Tags, registry: &game_tags::TagRegistry) -> u32 {
    let quality_ids = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"];
    for qname in &quality_ids {
        if let Some(qid) = registry.tag_id(qname)
            && tags.has(qid)
                && let Some(mult) = registry.tag_by_id(qid).multiplier {
                    return mult as u32;
                }
    }
    1
}

pub fn get_quality_prefix(tags: &game_tags::Tags, registry: &game_tags::TagRegistry) -> String {
    let quality_ids: Vec<game_tags::TagId> = ["COMMON", "UNCOMMON", "RARE", "EPIC", "LEGENDARY"]
        .iter()
        .filter_map(|name| registry.tag_id(name))
        .collect();
    for qid in &quality_ids {
        if tags.has(*qid) {
            let name = &registry.tag_by_id(*qid).name;
            return match name.as_str() {
                "COMMON" => String::new(),
                other => format!("{} ", other.to_lowercase()),
            };
        }
    }
    String::new()
}

pub fn get_item_display_name(ecs_world: &World, item_entity: bevy_ecs::entity::Entity, registry: &game_tags::TagRegistry) -> String {
    let name_comp = ecs_world.get::<Name>(item_entity).map(|n| n.0.clone());
    if let Some(name) = name_comp {
        return name;
    }

    let tags = ecs_world.get::<game_tags::Tags>(item_entity).cloned();
    if let Some(tags) = tags {
        let quality_prefix = get_quality_prefix(&tags, registry);
        let material_name = tags.iter_present().filter_map(|id| {
            let n = &registry.tag_by_id(id).name;
            if n.starts_with("ORE_") || n.starts_with("HERB_") || n.starts_with("GEM_") {
                Some(n.replace("_", " ").to_lowercase())
            } else {
                None
            }
        }).next();
        if let Some(mat) = material_name {
            return format!("{}{}", quality_prefix, mat);
        }
    }

    let glyph = ecs_world.get::<Glyph>(item_entity).map(|g| g.char.to_string());
    glyph.unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAGS_TOML: &str = include_str!("../assets/config/tags.toml");
    const INTERACTIONS_TOML: &str = include_str!("../assets/config/interactions.toml");

    fn setup_world() -> World {
        let mut world = World::new();
        let registry = game_tags::load_tag_registry(TAGS_TOML).expect("tags");
        let rules = game_tags::load_interaction_rules(INTERACTIONS_TOML, &registry).expect("rules");
        world.insert_resource(registry);
        world.insert_resource(rules);
        world
    }

    use game_core::{Health, Position};

    fn registry(world: &World) -> game_tags::TagRegistry {
        world.resource::<game_tags::TagRegistry>().clone()
    }

    #[test]
    fn equip_item_moves_to_equipment_slot() {
        let mut world = setup_world();
        let reg = registry(&world);

        let metal_id = reg.tag_id("METAL").unwrap();
        let equip_weapon_id = reg.tag_id("EQUIP_WEAPON").unwrap();

        let sword = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Glyph { char: '/', color: (180, 180, 190) },
            Name("Metal Blade".to_string()),
            game_core::Item,
        )).id();

        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![sword], capacity: 20 },
            Equipment::default(),
        )).id();

        handle_equip(&mut world, 0);

        let equipment = world.get::<Equipment>(player).unwrap();
        assert_eq!(equipment.weapon, Some(sword), "weapon should be equipped");

        let inventory = world.get::<Inventory>(player).unwrap();
        assert!(!inventory.items.contains(&sword), "sword should be removed from inventory");
    }

    #[test]
    fn equip_replaces_existing_equipment() {
        let mut world = setup_world();
        let reg = registry(&world);

        let metal_id = reg.tag_id("METAL").unwrap();
        let equip_weapon_id = reg.tag_id("EQUIP_WEAPON").unwrap();

        let sword1 = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Glyph { char: '/', color: (180, 180, 190) },
            Name("Old Sword".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword1).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
        }

        let sword2 = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Glyph { char: '/', color: (200, 200, 210) },
            Name("New Sword".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword2).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![sword1, sword2], capacity: 20 },
            Equipment::default(),
        )).id();

        handle_equip(&mut world, 0);
        assert_eq!(world.get::<Equipment>(player).unwrap().weapon, Some(sword1));

        handle_equip(&mut world, 0);
        assert_eq!(world.get::<Equipment>(player).unwrap().weapon, Some(sword2));

        let inventory = world.get::<Inventory>(player).unwrap();
        assert!(inventory.items.contains(&sword1), "old weapon should be back in inventory");
        assert!(!inventory.items.contains(&sword2), "new weapon should be equipped");
    }

    #[test]
    fn unequip_returns_item_to_inventory() {
        let mut world = setup_world();
        let reg = registry(&world);

        let metal_id = reg.tag_id("METAL").unwrap();
        let equip_weapon_id = reg.tag_id("EQUIP_WEAPON").unwrap();

        let sword = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Glyph { char: '/', color: (180, 180, 190) },
            Name("Metal Blade".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![sword], capacity: 20 },
            Equipment::default(),
        )).id();

        handle_equip(&mut world, 0);
        assert_eq!(world.get::<Equipment>(player).unwrap().weapon, Some(sword));

        handle_unequip(&mut world, EquipmentSlot::Weapon);
        assert_eq!(world.get::<Equipment>(player).unwrap().weapon, None, "weapon slot should be empty");
        assert!(world.get::<Inventory>(player).unwrap().items.contains(&sword), "sword back in inventory");
    }

    #[test]
    fn weapon_damage_calculation() {
        let mut world = setup_world();
        let reg = registry(&world);

        let metal_id = reg.tag_id("METAL").unwrap();
        let equip_weapon_id = reg.tag_id("EQUIP_WEAPON").unwrap();
        let uncommon_id = reg.tag_id("UNCOMMON").unwrap();

        let sword = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Test Sword".to_string()),
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
            tags.add_tag(uncommon_id, game_tags::TagValue::None, &reg);
        }

        let equipment = Equipment { weapon: Some(sword), armor: None, accessory: None };
        let damage = calc_weapon_damage(&equipment, &world, &reg);

        assert!(damage > 0, "weapon damage should be positive");
        assert!(damage >= 8, "metal weapon should do at least base+metal bonus");
    }

    #[test]
    fn armor_damage_reduction() {
        let mut world = setup_world();
        let reg = registry(&world);

        let leather_id = reg.tag_id("LEATHER").unwrap();
        let equip_armor_id = reg.tag_id("EQUIP_ARMOR").unwrap();

        let armor = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Armor Vest".to_string()),
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(armor).unwrap();
            tags.add_tag(leather_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_armor_id, game_tags::TagValue::None, &reg);
        }

        let equipment = Equipment { weapon: None, armor: Some(armor), accessory: None };
        let protection = calc_armor_protection(&equipment, &world, &reg);

        assert!(protection >= 3, "leather armor should provide at least 3 protection");
    }

    #[test]
    fn equip_adds_tags_to_player() {
        let mut world = setup_world();
        let reg = registry(&world);

        let metal_id = reg.tag_id("METAL").unwrap();
        let equip_weapon_id = reg.tag_id("EQUIP_WEAPON").unwrap();

        let sword = world.spawn((
            game_tags::Tags::new(reg.tag_count()),
            Name("Metal Blade".to_string()),
            game_core::Item,
        )).id();
        {
            let mut tags = world.get_mut::<game_tags::Tags>(sword).unwrap();
            tags.add_tag(metal_id, game_tags::TagValue::None, &reg);
            tags.add_tag(equip_weapon_id, game_tags::TagValue::None, &reg);
        }

        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 100, max: 100 },
            game_tags::Tags::new(reg.tag_count()),
            Inventory { items: vec![sword], capacity: 20 },
            Equipment::default(),
        )).id();

        handle_equip(&mut world, 0);

        let player_tags = world.get::<game_tags::Tags>(player).unwrap();
        assert!(player_tags.has(metal_id), "player should have METAL tag from equipped weapon");
    }
}
