use bevy_ecs::prelude::*;
use bevy_ecs::entity::Entity;
use rand::Rng;

use crate::events::{EventBus, GameEvent};
use crate::turn::TurnCounter;
use crate::{Glyph, Health, Name, Position};
use game_tags::{TagRegistry, TagValue, Tags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    PoisonDart,
    ExplosiveRune,
    SummonTrap,
    AlarmTrap,
    ChestTrap,
}

#[derive(Component, Debug, Clone)]
pub struct Trap {
    pub trap_type: TrapType,
    pub detected: bool,
    pub triggered: bool,
    pub damage: u32,
    pub summon_tags: Vec<String>,
    pub alarm_radius: u32,
}

#[derive(Component, Debug, Clone)]
pub struct TrappedStatus {
    pub trap_type: TrapType,
    pub ticks_remaining: u32,
    pub damage_per_tick: u32,
}

impl Trap {
    pub fn new(trap_type: TrapType) -> Self {
        match trap_type {
            TrapType::PoisonDart => Self { trap_type, detected: false, triggered: false, damage: 10, summon_tags: vec![], alarm_radius: 0 },
            TrapType::ExplosiveRune => Self { trap_type, detected: false, triggered: false, damage: 25, summon_tags: vec![], alarm_radius: 0 },
            TrapType::SummonTrap => Self { trap_type, detected: false, triggered: false, damage: 0, summon_tags: vec!["AGGRESSIVE".to_string(), "BEAST".to_string()], alarm_radius: 0 },
            TrapType::AlarmTrap => Self { trap_type, detected: false, triggered: false, damage: 0, summon_tags: vec![], alarm_radius: 8 },
            TrapType::ChestTrap => Self { trap_type, detected: false, triggered: false, damage: 15, summon_tags: vec![], alarm_radius: 0 },
        }
    }

    pub fn with_damage(mut self, dmg: u32) -> Self { self.damage = dmg; self }
    pub fn with_summon(mut self, tags: Vec<String>) -> Self { self.summon_tags = tags; self }
    pub fn with_alarm(mut self, radius: u32) -> Self { self.alarm_radius = radius; self }
}

pub fn try_disarm_trap(rng: &mut impl Rng) -> bool {
    rng.random::<f32>() < 0.5
}

pub fn try_detect_trap(world: &mut World, pos: &Position, _registry: &TagRegistry) -> bool {
    let mut tiles = world.query::<(Entity, &Trap, &Position)>();
    for (_entity, trap, trap_pos) in tiles.iter(world) {
        if trap_pos.x == pos.x && trap_pos.y == pos.y && !trap.detected {
            return true;
        }
    }
    false
}

pub fn trigger_trap(world: &mut World, trap_entity: Entity, player_entity: Entity) {
    let trap_data = if let Some(t) = world.get::<Trap>(trap_entity).cloned() { t } else { return };
    if trap_data.triggered { return; }

    world.entity_mut(trap_entity).insert(Trap { triggered: true, ..trap_data.clone() });

    let turn = world.get_resource::<TurnCounter>().map(|tc| tc.0).unwrap_or(0);

    match trap_data.trap_type {
        TrapType::PoisonDart => {
            world.entity_mut(player_entity).insert(TrappedStatus {
                trap_type: TrapType::PoisonDart,
                ticks_remaining: 8,
                damage_per_tick: 3,
            });
            let msg = "A poison dart trap injects venom! You feel it spreading...".to_string();
            push_event(world, GameEvent::Message(msg), turn);
        }
        TrapType::ExplosiveRune => {
            if let Some(mut hp) = world.get_mut::<Health>(player_entity) {
                hp.current = hp.current.saturating_sub(trap_data.damage);
            }
            let msg = format!("An explosive rune explodes for {} damage!", trap_data.damage);
            push_event(world, GameEvent::Message(msg), turn);
        }
        TrapType::SummonTrap => {
            push_event(world, GameEvent::Message("A summoning trap activates!".to_string()), turn);
            if let Some(registry) = world.get_resource::<TagRegistry>().cloned()
                && let Some(pos) = world.get::<Position>(trap_entity).copied() {
                    let creature_tags = &trap_data.summon_tags;
                    for i in 0..3 {
                        let spawn_pos = Position { x: pos.x + i, y: pos.y + (i % 3), z: 0 };
                        let mut tags = Tags::new(registry.tag_count());
                        for tname in creature_tags {
                            if let Some(tid) = registry.tag_id(tname) {
                                tags.add_tag(tid, TagValue::None, &registry);
                            }
                        }
                        world.spawn((
                            crate::Creature,
                            spawn_pos,
                            Health { current: 15, max: 15 },
                            Glyph { char: 'm', color: (200, 50, 50) },
                            Name(format!("Summoned {}", tname(&trap_data.summon_tags))),
                            tags,
                        ));
                    }
                }
        }
        TrapType::AlarmTrap => {
            push_event(world, GameEvent::Message("An alarm trap blasts loudly!".to_string()), turn);
        }
        TrapType::ChestTrap => {
            if let Some(mut hp) = world.get_mut::<Health>(player_entity) {
                hp.current = hp.current.saturating_sub(trap_data.damage);
            }
            let msg = format!("The chest was trapped! You take {} damage.", trap_data.damage);
            push_event(world, GameEvent::Message(msg), turn);
        }
    }
}

fn tname(tags: &[String]) -> String {
    tags.first().map(|t| t.replace("_", " ").to_lowercase()).unwrap_or_else(|| "monster".to_string())
}

pub fn process_trapped_status(world: &mut World) {
    let mut to_remove = Vec::new();
    let trapped_entities: Vec<(Entity, TrappedStatus)> = {
        let mut q = world.query::<(Entity, &TrappedStatus)>();
        q.iter(world).map(|(e, s)| (e, s.clone())).collect()
    };

    for (entity, mut status) in trapped_entities {
        status.ticks_remaining = status.ticks_remaining.saturating_sub(1);

        if status.trap_type == TrapType::PoisonDart
            && let Some(mut hp) = world.get_mut::<Health>(entity)
        {
            let dmg = status.damage_per_tick;
            hp.current = hp.current.saturating_sub(dmg);
            let msg = format!("Poison deals {} damage. ({}/8 ticks)", dmg, status.ticks_remaining);
            if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
                bus.push(GameEvent::Message(msg));
            }
        }

        if status.ticks_remaining == 0 {
            to_remove.push(entity);
        } else {
            if let Some(mut s) = world.get_mut::<TrappedStatus>(entity) {
                *s = status;
            }
        }
    }

    for entity in to_remove {
        world.entity_mut(entity).remove::<TrappedStatus>();
        if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
            bus.push(GameEvent::Message("The poison runs its course.".to_string()));
        }
    }
}

fn push_event(world: &mut World, event: GameEvent, _turn: u64) {
    if let Some(mut bus) = world.get_resource_mut::<EventBus>() {
        bus.push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Player;
    use rand::SeedableRng;

    #[test]
    fn poison_trap_applies_status() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        world.insert_resource(EventBus::new());
        let player = world.spawn((
            Player,
            Position { x: 5, y: 5, z: 0 },
            Health { current: 50, max: 100 },
        )).id();
        let trap = world.spawn((
            Trap::new(TrapType::PoisonDart).with_damage(10),
            Position { x: 5, y: 5, z: 0 },
        )).id();

        trigger_trap(&mut world, trap, player);
        assert!(world.get::<TrappedStatus>(player).is_some(), "should have TrappedStatus");
        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 50, "poison does not deal immediate damage");
    }

    #[test]
    fn explosive_rune_damages_area() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        world.insert_resource(EventBus::new());
        let player = world.spawn((
            Player,
            Position { x: 10, y: 10, z: 0 },
            Health { current: 100, max: 100 },
        )).id();
        let trap = world.spawn((
            Trap::new(TrapType::ExplosiveRune).with_damage(25),
            Position { x: 10, y: 10, z: 0 },
        )).id();

        trigger_trap(&mut world, trap, player);
        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 75);
    }

    #[test]
    fn trap_cannot_be_triggered_twice() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        world.insert_resource(EventBus::new());
        let player = world.spawn((Player, Position { x: 0, y: 0, z: 0 }, Health { current: 100, max: 100 })).id();
        let trap = world.spawn((Trap::new(TrapType::ExplosiveRune).with_damage(25), Position { x: 0, y: 0, z: 0 })).id();

        trigger_trap(&mut world, trap, player);
        trigger_trap(&mut world, trap, player);
        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 75);
    }

    #[test]
    fn chest_trap_damages_on_open() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        world.insert_resource(EventBus::new());
        let player = world.spawn((Player, Position { x: 3, y: 3, z: 0 }, Health { current: 40, max: 50 })).id();
        let trap = world.spawn((Trap::new(TrapType::ChestTrap).with_damage(15), Position { x: 3, y: 3, z: 0 })).id();

        trigger_trap(&mut world, trap, player);
        let hp = world.get::<Health>(player).unwrap();
        assert_eq!(hp.current, 25);
    }

    #[test]
    fn detect_trap_finds_undetected() {
        let mut world = World::new();
        let registry = game_tags::TagRegistryBuilder::default().build().unwrap();
        world.spawn((Trap::new(TrapType::ExplosiveRune), Position { x: 5, y: 5, z: 0 }));
        assert!(try_detect_trap(&mut world, &Position { x: 5, y: 5, z: 0 }, &registry));
        assert!(!try_detect_trap(&mut world, &Position { x: 1, y: 1, z: 0 }, &registry));
    }

    #[test]
    fn summon_trap_spawns_creatures() {
        let mut world = World::new();
        world.insert_resource(TurnCounter(0));
        world.insert_resource(EventBus::new());
        let registry = game_tags::TagRegistryBuilder::default().build().unwrap();
        world.insert_resource(registry);
        let player = world.spawn((Player, Position { x: 0, y: 0, z: 0 }, Health { current: 100, max: 100 })).id();
        let trap = world.spawn((
            Trap::new(TrapType::SummonTrap).with_summon(vec!["AGGRESSIVE".to_string()]),
            Position { x: 10, y: 10, z: 0 },
        )).id();

        let before = world.query::<&crate::Creature>().iter(&world).count();
        trigger_trap(&mut world, trap, player);
        let after = world.query::<&crate::Creature>().iter(&world).count();
        assert!(after > before);
    }

    #[test]
    fn alarm_trap_has_radius() {
        let trap = Trap::new(TrapType::AlarmTrap).with_alarm(8);
        assert_eq!(trap.alarm_radius, 8);
        assert_eq!(trap.trap_type, TrapType::AlarmTrap);
    }

    #[test]
    fn disarm_trap_succeeds_within_range() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut success_count = 0u32;
        let trials = 1000;
        for _ in 0..trials {
            if try_disarm_trap(&mut rng) {
                success_count += 1;
            }
        }
        let rate = success_count as f32 / trials as f32;
        assert!(rate > 0.35, "disarm success rate {rate} should be near 0.5");
        assert!(rate < 0.65, "disarm success rate {rate} should be near 0.5");
    }

    #[test]
    fn disarm_trap_is_deterministic() {
        let run = || -> bool {
            let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
            try_disarm_trap(&mut rng)
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn disarm_trap_exhaustive_check() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(99999);
        let mut saw_true = false;
        let mut saw_false = false;
        for _ in 0..100 {
            if try_disarm_trap(&mut rng) {
                saw_true = true;
            } else {
                saw_false = true;
            }
            if saw_true && saw_false {
                break;
            }
        }
        assert!(saw_true, "disarm should sometimes succeed");
        assert!(saw_false, "disarm should sometimes fail");
    }
}
