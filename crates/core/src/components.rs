use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    pub char: char,
    pub color: (u8, u8, u8),
}

#[derive(Component)]
pub struct Item;

#[derive(Component)]
pub struct Creature;

#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    pub items: Vec<bevy_ecs::entity::Entity>,
    pub capacity: usize,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Equipment {
    pub weapon: Option<bevy_ecs::entity::Entity>,
    pub armor: Option<bevy_ecs::entity::Entity>,
    pub accessory: Option<bevy_ecs::entity::Entity>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WeaponDamage(pub u32);

#[derive(Component, Debug, Clone, Copy)]
pub struct ArmorProtection(pub u32);

#[derive(Component, Debug, Clone)]
pub struct ItemEffects(pub Vec<String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageCategory {
    Combat,
    System,
    Item,
    Quest,
    Narrative,
}

#[derive(Resource, Debug, Clone)]
pub struct MessageLog {
    pub messages: Vec<String>,
    pub max: usize,
}

impl MessageLog {
    pub fn new(max: usize) -> Self {
        Self {
            messages: Vec::new(),
            max,
        }
    }

    pub fn push(&mut self, message: String) {
        if self.messages.len() >= self.max {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    pub fn recent(&self, count: usize) -> &[String] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }
}

/// Marker component for entities affected by weather (environmental tags).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WeatherSensitive;

/// Marker for entities that belong to the overworld (survive interior transitions).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct OverworldEntity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_components() {
        let pos = Position { x: 5, y: 10, z: 0 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);

        let health = Health { current: 100, max: 100 };
        assert_eq!(health.current, 100);

        let glyph = Glyph { char: '@', color: (255, 255, 0) };
        assert_eq!(glyph.char, '@');
    }

    #[test]
    fn message_log_push_and_recent() {
        let mut log = MessageLog::new(5);
        for i in 0..7 {
            log.push(format!("msg {}", i));
        }
        assert_eq!(log.messages.len(), 5);
        assert_eq!(log.messages[0], "msg 2");
        assert_eq!(log.recent(3), &["msg 4", "msg 5", "msg 6"]);
    }

    #[test]
    fn inventory_capacity() {
        let inv = Inventory { items: vec![], capacity: 20 };
        assert_eq!(inv.capacity, 20);
        assert!(inv.items.is_empty());
}

    #[test]
    fn equipment_slot_variants() {
        assert_eq!(EquipmentSlot::Weapon, EquipmentSlot::Weapon);
        assert_ne!(EquipmentSlot::Weapon, EquipmentSlot::Armor);
        assert_ne!(EquipmentSlot::Armor, EquipmentSlot::Accessory);
    }

    #[test]
    fn equipment_with_entities() {
        use bevy_ecs::world::World;
        let mut world = World::new();
        let sword = world.spawn(()).id();
        let shield = world.spawn(()).id();
        let ring = world.spawn(()).id();
        let eq = Equipment {
            weapon: Some(sword),
            armor: Some(shield),
            accessory: Some(ring),
        };
        assert_eq!(eq.weapon, Some(sword));
        assert_eq!(eq.armor, Some(shield));
        assert_eq!(eq.accessory, Some(ring));
    }

    #[test]
    fn name_component() {
        let name = Name("Hero".to_string());
        assert_eq!(name.0, "Hero");
    }

    #[test]
    fn creature_and_item_markers() {
        use bevy_ecs::world::World;
        let mut world = World::new();
        let e1 = world.spawn((Creature,)).id();
        let e2 = world.spawn((Item,)).id();
        assert!(world.get::<Creature>(e1).is_some());
        assert!(world.get::<Item>(e2).is_some());
        assert!(world.get::<Item>(e1).is_none());
    }

    #[test]
    fn glyph_equality() {
        let g1 = Glyph { char: '@', color: (255, 255, 0) };
        let g2 = Glyph { char: '@', color: (255, 255, 0) };
        let g3 = Glyph { char: '@', color: (255, 0, 0) };
        assert_eq!(g1, g2);
        assert_ne!(g1, g3);
    }
}
