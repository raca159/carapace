use bevy_ecs::prelude::*;
use bevy_ecs::entity::Entity;
use serde::{Deserialize, Serialize};

use crate::events::EventBus;
use crate::{Equipment, GameEvent, Name, Player};
use game_tags::{TagRegistry, Tags};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Durability {
    pub current: u32,
    pub max: u32,
    pub broken: bool,
}

impl Durability {
    pub fn new(max: u32) -> Self {
        Self { current: max, max, broken: false }
    }

    pub fn pct(&self) -> f32 {
        if self.broken || self.max == 0 { 0.0 } else { self.current as f32 / self.max as f32 }
    }

    pub fn degrade(&mut self, amount: u32) -> bool {
        self.current = self.current.saturating_sub(amount);
        if self.current == 0 {
            self.broken = true;
            true
        } else {
            false
        }
    }

    pub fn is_broken(&self) -> bool {
        self.broken || self.current == 0
    }

    pub fn is_worn(&self) -> bool {
        let p = self.pct();
        p > 0.33 && p <= 0.67
    }

    pub fn is_damaged(&self) -> bool {
        let p = self.pct();
        p > 0.0 && p <= 0.33
    }

    pub fn repair(&mut self, amount: u32) -> u32 {
        let repaired = amount.min(self.max.saturating_sub(self.current));
        self.current += repaired;
        if self.current > 0 {
            self.broken = false;
        }
        repaired
    }
}

pub fn degrade_weapon(world: &mut World, entity: Entity, amount: u32) {
    let broke = {
        if let Some(mut dur) = world.get_mut::<Durability>(entity) {
            dur.degrade(amount)
        } else {
            false
        }
    };
    if broke {
        let name = world.get::<Name>(entity).map(|n| n.0.clone()).unwrap_or_else(|| "weapon".to_string());
        let mut eq_query = world.query_filtered::<&mut Equipment, With<Player>>();
        if let Ok(mut eq) = eq_query.single_mut(world)
            && eq.weapon == Some(entity)
        {
            eq.weapon = None;
        }
        if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message(format!("Your {} breaks!", name)));
        }
    }
}

pub fn degrade_armor(world: &mut World, entity: Entity, amount: u32) {
    let broke = {
        if let Some(mut dur) = world.get_mut::<Durability>(entity) {
            dur.degrade(amount)
        } else {
            false
        }
    };
    if broke {
        let name = world.get::<Name>(entity).map(|n| n.0.clone()).unwrap_or_else(|| "armor".to_string());
        let mut eq_query = world.query_filtered::<&mut Equipment, With<Player>>();
        if let Ok(mut eq) = eq_query.single_mut(world)
            && eq.armor == Some(entity)
        {
            eq.armor = None;
        }
        if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message(format!("Your {} breaks!", name)));
        }
    }
}

pub fn repair_item(world: &mut World, item_entity: Entity, repair_amount: u32) -> u32 {
    let mut actual_repair = 0;
    if let Some(mut dur) = world.get_mut::<Durability>(item_entity) {
        actual_repair = dur.repair(repair_amount);
    }
    actual_repair
}

pub fn set_durability_on_equip(world: &mut World, entity: Entity) {
    if world.get::<Durability>(entity).is_none() {
        let max = default_durability_for_item(world, entity);
        world.entity_mut(entity).insert(Durability::new(max));
    }
}

fn default_durability_for_item(world: &World, entity: Entity) -> u32 {
    if let Some(tags) = world.get::<Tags>(entity)
        && let Some(registry) = world.get_resource::<TagRegistry>()
    {
        let tiers = [
            ("COMMON", 20u32),
            ("UNCOMMON", 40u32),
            ("RARE", 80u32),
            ("EPIC", 160u32),
            ("LEGENDARY", 320u32),
        ];
        for (name, max_dur) in &tiers {
            if let Some(id) = registry.tag_id(name)
                && tags.has(id)
            {
                return *max_dur;
            }
        }
    }
    20
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durability_new() {
        let d = Durability::new(50);
        assert_eq!(d.current, 50);
        assert_eq!(d.max, 50);
    }

    #[test]
    fn durability_degrade() {
        let mut d = Durability::new(30);
        assert!(!d.degrade(5));
        assert_eq!(d.current, 25);
    }

    #[test]
    fn durability_breaks_at_zero() {
        let mut d = Durability::new(10);
        assert!(d.degrade(10));
        assert_eq!(d.current, 0);
        assert!(d.is_broken());
    }

    #[test]
    fn durability_does_not_underflow() {
        let mut d = Durability::new(5);
        d.degrade(100);
        assert_eq!(d.current, 0);
        assert!(d.is_broken());
    }

    #[test]
    fn durability_pct() {
        let d = Durability::new(100);
        assert!((d.pct() - 1.0).abs() < 0.001);
        let mut d = Durability::new(100);
        d.current = 50;
        assert!((d.pct() - 0.5).abs() < 0.001);
    }

    #[test]
    fn repair_restores_durability() {
        let mut d = Durability::new(100);
        d.current = 50;
        let repaired = d.repair(30);
        assert_eq!(repaired, 30);
        assert_eq!(d.current, 80);
    }

    #[test]
    fn repair_cannot_exceed_max() {
        let mut d = Durability::new(100);
        d.current = 90;
        let repaired = d.repair(20);
        assert_eq!(repaired, 10);
        assert_eq!(d.current, 100);
    }

    #[test]
    fn repair_clears_broken_flag() {
        let mut d = Durability::new(100);
        d.degrade(100);
        assert!(d.is_broken());
        let repaired = d.repair(50);
        assert_eq!(repaired, 50);
        assert_eq!(d.current, 50);
        assert!(!d.is_broken());
    }

    #[test]
    fn durability_is_worn_and_damaged() {
        let mut d = Durability::new(100);
        assert!(!d.is_worn());
        assert!(!d.is_damaged());
        d.current = 50;
        assert!(d.is_worn());
        assert!(!d.is_damaged());
        d.current = 20;
        assert!(!d.is_worn());
        assert!(d.is_damaged());
        d.current = 0;
        assert!(!d.is_worn());
        assert!(!d.is_damaged());
        assert!(d.is_broken());
    }

    #[test]
    fn set_durability_on_equip_no_durability_yet() {
        let mut world = World::new();
        world.insert_resource(game_tags::TagRegistryBuilder::default().build().unwrap());
        let entity = world.spawn(Name("Sword".to_string())).id();
        set_durability_on_equip(&mut world, entity);
        assert!(world.get::<Durability>(entity).is_some());
        assert_eq!(world.get::<Durability>(entity).unwrap().max, 20);
    }

    #[test]
    fn set_durability_does_not_overwrite_existing() {
        let mut world = World::new();
        let entity = world.spawn((Name("Sword".to_string()), Durability::new(99))).id();
        set_durability_on_equip(&mut world, entity);
        assert_eq!(world.get::<Durability>(entity).unwrap().current, 99);
    }
}
