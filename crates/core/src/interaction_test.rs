use bevy_ecs::prelude::World;
use rand::Rng;
use rand::SeedableRng;

use crate::barter::{BarterOffer, BarterResult, resolve_barter};
use crate::calc::{calc_armor_protection, calc_weapon_damage};
use crate::components::{Equipment, Health, Inventory, Name, Player};
use crate::dialogue::{DialogueLine, select_dialogue};
use game_tags::{TagRegistry, Tags};

#[derive(Debug, Clone)]
pub struct EntitySpec {
    pub name: String,
    pub tag_names: Vec<String>,
    pub hp: u32,
    pub equipment: Vec<EquipmentSpec>,
    pub inventory: Vec<InventorySpec>,
    pub is_player: bool,
}

#[derive(Debug, Clone)]
pub struct EquipmentSpec {
    pub slot: String,
    pub name: String,
    pub tag_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InventorySpec {
    pub name: String,
    pub tag_names: Vec<String>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct CombatScenario {
    pub attacker: EntitySpec,
    pub defender: EntitySpec,
    pub num_turns: u32,
}

#[derive(Debug, Clone)]
pub struct BarterScenario {
    pub trader: EntitySpec,
    pub customer: EntitySpec,
    pub offer: BarterOffer,
}

#[derive(Debug, Clone)]
pub struct ConversationScenario {
    pub speaker: EntitySpec,
    pub listener: EntitySpec,
    pub dialogue_lines: Vec<DialogueLine>,
    pub faction_standing: String,
}

#[derive(Debug, Clone)]
pub enum InteractionKind {
    Combat(CombatScenario),
    Barter(BarterScenario),
    Conversation(ConversationScenario),
}

#[derive(Debug, Clone)]
pub struct InteractionSnapshot {
    pub seed: u64,
    pub kind: &'static str,
    pub entities: Vec<EntitySnapshot>,
    pub combat_rounds: Vec<CombatRoundSnapshot>,
    pub barter_result: Option<BarterResult>,
    pub dialogue_result: Option<String>,
    pub deterministic_fingerprint: String,
}

#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    pub name: String,
    pub hp: u32,
    pub alive: bool,
    pub tags: Vec<String>,
    pub position: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct CombatRoundSnapshot {
    pub round: u32,
    pub attacker: String,
    pub defender: String,
    pub raw_damage: u32,
    pub blocked: u32,
    pub actual_damage: u32,
    pub defender_hp_after: u32,
    pub killed: bool,
}

pub fn spawn_entity_spec(
    world: &mut World,
    spec: &EntitySpec,
    x: u32,
    y: u32,
    registry: &TagRegistry,
) -> bevy_ecs::entity::Entity {
    let tag_count = registry.tag_count();
    let mut tags = Tags::new(tag_count);
    for name in &spec.tag_names {
        if let Some(id) = registry.tag_id(name) {
            tags.add_tag(id, game_tags::TagValue::None, registry);
        }
    }

    let mut builder = world.spawn((
        tags,
        crate::components::Position { x, y, z: 0 },
        Health {
            current: spec.hp,
            max: spec.hp,
        },
        Name(spec.name.clone()),
        crate::components::Glyph {
            char: '@',
            color: (255, 255, 255),
        },
    ));

    if spec.is_player {
        builder.insert(Player);
    } else {
        builder.insert(crate::components::Creature);
    }

    let entity = builder.id();

    let mut equip = Equipment::default();
    for eq_spec in &spec.equipment {
        let mut eq_tags = Tags::new(tag_count);
        for name in &eq_spec.tag_names {
            if let Some(id) = registry.tag_id(name) {
                eq_tags.add_tag(id, game_tags::TagValue::None, registry);
            }
        }
        let item = world
            .spawn((
                crate::components::Position { x, y, z: 0 },
                crate::components::Glyph {
                    char: '/',
                    color: (200, 200, 200),
                },
                eq_tags,
                Name(eq_spec.name.clone()),
                crate::components::Item,
            ))
            .id();

        match eq_spec.slot.as_str() {
            "weapon" => equip.weapon = Some(item),
            "armor" => equip.armor = Some(item),
            "accessory" => equip.accessory = Some(item),
            _ => {}
        }
    }
    world.entity_mut(entity).insert(equip);

    let inv = Inventory {
        items: vec![],
        capacity: 20,
    };
    world.entity_mut(entity).insert(inv);

    entity
}

pub fn run_combat_scenario(
    scenario: &CombatScenario,
    seed: u64,
    registry: &TagRegistry,
) -> InteractionSnapshot {
    let mut world = World::new();
    world.insert_resource(registry.clone());

    let attacker = spawn_entity_spec(&mut world, &scenario.attacker, 5, 5, registry);
    let defender = spawn_entity_spec(&mut world, &scenario.defender, 6, 5, registry);

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut rounds = Vec::new();

    for round in 1..=scenario.num_turns {
        let defender_hp = world
            .get::<Health>(defender)
            .map(|h| h.current)
            .unwrap_or(0);
        if defender_hp == 0 {
            break;
        }

        let raw_damage = {
            let equip = world.get::<Equipment>(attacker).cloned();
            if let Some(ref eq) = equip {
                calc_weapon_damage(eq, &world, registry)
            } else {
                
                std::cmp::max(1u32, rng.random_range(1..=6))
            }
        };

        let blocked = {
            let equip = world.get::<Equipment>(defender).cloned();
            if let Some(ref eq) = equip {
                calc_armor_protection(eq, &world, registry)
            } else {
                0
            }
        };

        let actual_damage = std::cmp::max(1, raw_damage.saturating_sub(blocked));

        let mut hp = world.get_mut::<Health>(defender).unwrap();
        hp.current = hp.current.saturating_sub(actual_damage);
        let hp_after = hp.current;
        let killed = hp_after == 0;

        rounds.push(CombatRoundSnapshot {
            round,
            attacker: scenario.attacker.name.clone(),
            defender: scenario.defender.name.clone(),
            raw_damage,
            blocked,
            actual_damage,
            defender_hp_after: hp_after,
            killed,
        });
    }

    let (entities, fingerprint) = build_snapshot_fingerprint(&mut world, registry);
    InteractionSnapshot {
        seed,
        kind: "combat",
        entities,
        combat_rounds: rounds,
        barter_result: None,
        dialogue_result: None,
        deterministic_fingerprint: fingerprint,
    }
}

pub fn run_barter_scenario(
    scenario: &BarterScenario,
    seed: u64,
    registry: &TagRegistry,
) -> InteractionSnapshot {
    let mut world = World::new();
    world.insert_resource(registry.clone());

    let trader = spawn_entity_spec(&mut world, &scenario.trader, 5, 5, registry);
    let customer = spawn_entity_spec(&mut world, &scenario.customer, 6, 5, registry);

    let trader_tags = world.get::<Tags>(trader).cloned();
    let customer_tags = world.get::<Tags>(customer).cloned();

    let result = resolve_barter(
        &scenario.offer,
        trader_tags.as_ref(),
        customer_tags.as_ref(),
        Some(registry),
    );

    let (entities, fingerprint) = build_snapshot_fingerprint(&mut world, registry);
    InteractionSnapshot {
        seed,
        kind: "barter",
        entities,
        combat_rounds: vec![],
        barter_result: Some(result),
        dialogue_result: None,
        deterministic_fingerprint: fingerprint,
    }
}

pub fn run_conversation_scenario(
    scenario: &ConversationScenario,
    seed: u64,
    registry: &TagRegistry,
) -> InteractionSnapshot {
    let mut world = World::new();
    world.insert_resource(registry.clone());

    let speaker = spawn_entity_spec(&mut world, &scenario.speaker, 5, 5, registry);
    let _listener = spawn_entity_spec(&mut world, &scenario.listener, 6, 5, registry);

    let speaker_tags = world
        .get::<Tags>(speaker)
        .cloned()
        .unwrap_or_else(|| Tags::new(registry.tag_count()));

    let _rng = rand::rngs::StdRng::seed_from_u64(seed);

    let selected = select_dialogue(
        &speaker_tags,
        &scenario.faction_standing,
        &scenario.dialogue_lines,
        registry,
    );

    let (entities, fingerprint) = build_snapshot_fingerprint(&mut world, registry);
    InteractionSnapshot {
        seed,
        kind: "conversation",
        entities,
        combat_rounds: vec![],
        barter_result: None,
        dialogue_result: selected,
        deterministic_fingerprint: fingerprint,
    }
}

pub fn run_interaction(
    kind: &InteractionKind,
    seed: u64,
    registry: &TagRegistry,
) -> InteractionSnapshot {
    match kind {
        InteractionKind::Combat(scenario) => run_combat_scenario(scenario, seed, registry),
        InteractionKind::Barter(scenario) => run_barter_scenario(scenario, seed, registry),
        InteractionKind::Conversation(scenario) => {
            run_conversation_scenario(scenario, seed, registry)
        }
    }
}

fn build_tag_list(tags: &Tags, registry: &TagRegistry) -> Vec<String> {
    tags.iter_present()
        .map(|id| registry.tag_by_id(id).name.clone())
        .collect()
}

fn build_snapshot_fingerprint(
    world: &mut World,
    registry: &TagRegistry,
) -> (Vec<EntitySnapshot>, String) {
    let mut entities = Vec::new();
    let mut fp_parts = Vec::new();

    for (entity, name, hp, tags, pos) in world
        .query::<(
            bevy_ecs::entity::Entity,
            &Name,
            &Health,
            &Tags,
            &crate::components::Position,
        )>()
        .iter(world)
    {
        let tag_names = build_tag_list(tags, registry);
        let alive = hp.current > 0;
        fp_parts.push(format!(
            "{}:hp={}/{}:tags={}:pos=({},{})",
            name.0,
            hp.current,
            hp.max,
            tag_names.join(","),
            pos.x,
            pos.y
        ));

        let _ = entity;
        entities.push(EntitySnapshot {
            name: name.0.clone(),
            hp: hp.current,
            alive,
            tags: tag_names,
            position: (pos.x, pos.y),
        });
    }

    (entities, fp_parts.join("|"))
}

pub fn assert_deterministic(kind: &InteractionKind, seed: u64, registry: &TagRegistry) {
    let run1 = run_interaction(kind, seed, registry);
    let run2 = run_interaction(kind, seed, registry);
    assert_eq!(
        run1.deterministic_fingerprint, run2.deterministic_fingerprint,
        "same seed {} must produce identical interaction result for {:?}",
        seed, kind
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::barter::BarterItem;
    use crate::dialogue::DialogueLine;

    const TAGS_TOML: &str = include_str!("../../tags/assets/config/tags.toml");

    fn registry() -> TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).expect("tags")
    }

    #[test]
    fn harness_combat_scenario_deterministic() {
        let reg = registry();
        let scenario = CombatScenario {
            attacker: EntitySpec {
                name: "Creature".into(),
                tag_names: vec!["AGGRESSIVE".into(), "HUMANOID".into(), "SMALL".into()],
                hp: 25,
                equipment: vec![EquipmentSpec {
                    slot: "weapon".into(),
                    name: "Knife".into(),
                    tag_names: vec![
                        "METAL".into(),
                        "EQUIP_WEAPON".into(),
                        "MELEE".into(),
                        "COMMON".into(),
                    ],
                }],
                inventory: vec![],
                is_player: false,
            },
            defender: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into()],
                hp: 50,
                equipment: vec![
                    EquipmentSpec {
                        slot: "weapon".into(),
                        name: "Blade".into(),
                        tag_names: vec![
                            "METAL".into(),
                            "EQUIP_WEAPON".into(),
                            "MELEE".into(),
                            "UNCOMMON".into(),
                        ],
                    },
                    EquipmentSpec {
                        slot: "armor".into(),
                        name: "Armor Vest".into(),
                        tag_names: vec!["LEATHER".into(), "EQUIP_ARMOR".into(), "COMMON".into()],
                    },
                ],
                inventory: vec![],
                is_player: true,
            },
            num_turns: 10,
        };

        let snapshot = run_combat_scenario(&scenario, 42, &reg);
        assert_eq!(snapshot.kind, "combat");
        assert!(
            !snapshot.combat_rounds.is_empty(),
            "combat should produce rounds"
        );
        assert_eq!(snapshot.seed, 42);

        assert_deterministic(&InteractionKind::Combat(scenario), 42, &reg);
    }

    #[test]
    fn harness_combat_damage_dealt() {
        let reg = registry();
        let scenario = CombatScenario {
            attacker: EntitySpec {
                name: "Creature".into(),
                tag_names: vec!["AGGRESSIVE".into(), "HUMANOID".into(), "SMALL".into()],
                hp: 25,
                equipment: vec![EquipmentSpec {
                    slot: "weapon".into(),
                    name: "Knife".into(),
                    tag_names: vec!["METAL".into(), "EQUIP_WEAPON".into(), "COMMON".into()],
                }],
                inventory: vec![],
                is_player: false,
            },
            defender: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into()],
                hp: 50,
                equipment: vec![EquipmentSpec {
                    slot: "armor".into(),
                    name: "Armor Vest".into(),
                    tag_names: vec!["LEATHER".into(), "EQUIP_ARMOR".into(), "COMMON".into()],
                }],
                inventory: vec![],
                is_player: true,
            },
            num_turns: 5,
        };

        let snapshot = run_combat_scenario(&scenario, 42, &reg);
        for round in &snapshot.combat_rounds {
            assert!(round.raw_damage > 0, "each round should deal damage");
            assert!(
                round.actual_damage <= round.raw_damage,
                "armor should not increase damage"
            );
        }
    }

    #[test]
    fn harness_barter_scenario_deterministic() {
        let reg = registry();
        let scenario = BarterScenario {
            trader: EntitySpec {
                name: "Merchant".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into(), "PEACEFUL".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: false,
            },
            customer: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: true,
            },
            offer: BarterOffer {
                offered: vec![BarterItem {
                    name: "Chip".into(),
                    quantity: 5,
                    base_value: 10,
                }],
                requested: vec![BarterItem {
                    name: "Metal Blade".into(),
                    quantity: 1,
                    base_value: 50,
                }],
            },
        };

        let snapshot = run_barter_scenario(&scenario, 42, &reg);
        assert_eq!(snapshot.kind, "barter");
        let result = snapshot.barter_result.unwrap();
        assert!(
            result.accepted,
            "5 chips (50 value) for 1 metal blade (50 value) should be accepted"
        );

        assert_deterministic(&InteractionKind::Barter(scenario), 42, &reg);
    }

    #[test]
    fn harness_barter_unfair_rejected() {
        let reg = registry();
        let scenario = BarterScenario {
            trader: EntitySpec {
                name: "Merchant".into(),
                tag_names: vec!["HUMANOID".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: false,
            },
            customer: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: true,
            },
            offer: BarterOffer {
                offered: vec![BarterItem {
                    name: "Chip".into(),
                    quantity: 1,
                    base_value: 10,
                }],
                requested: vec![BarterItem {
                    name: "Metal Blade".into(),
                    quantity: 1,
                    base_value: 50,
                }],
            },
        };

        let snapshot = run_barter_scenario(&scenario, 42, &reg);
        let result = snapshot.barter_result.unwrap();
        assert!(
            !result.accepted,
            "1 chip (10) for 1 metal blade (50) should be rejected"
        );
    }

    #[test]
    fn harness_conversation_scenario_deterministic() {
        let reg = registry();
        let scenario = ConversationScenario {
            speaker: EntitySpec {
                name: "Guard".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into(), "AGGRESSIVE".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: false,
            },
            listener: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into(), "MEDIUM".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: true,
            },
            dialogue_lines: vec![
                DialogueLine {
                    trigger_tags: vec!["AGGRESSIVE".into()],
                    standing: Some("hostile".into()),
                    lines: vec![
                        "What do you want?".into(),
                        "Make it quick.".into(),
                        "You're testing my patience.".into(),
                    ],
                },
                DialogueLine {
                    trigger_tags: vec!["PEACEFUL".into()],
                    standing: Some("ally".into()),
                    lines: vec!["Good to see you, friend!".into(), "How can I help?".into()],
                },
            ],
            faction_standing: "hostile".into(),
        };

        let snapshot = run_conversation_scenario(&scenario, 42, &reg);
        assert_eq!(snapshot.kind, "conversation");
        assert!(
            snapshot.dialogue_result.is_some(),
            "aggressive NPC with hostile standing should produce dialogue"
        );

        assert_deterministic(&InteractionKind::Conversation(scenario), 42, &reg);
    }

    #[test]
    fn harness_conversation_no_match_returns_none() {
        let reg = registry();
        let scenario = ConversationScenario {
            speaker: EntitySpec {
                name: "Peaceful NPC".into(),
                tag_names: vec!["PEACEFUL".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: false,
            },
            listener: EntitySpec {
                name: "Player".into(),
                tag_names: vec!["HUMANOID".into()],
                hp: 50,
                equipment: vec![],
                inventory: vec![],
                is_player: true,
            },
            dialogue_lines: vec![DialogueLine {
                trigger_tags: vec!["AGGRESSIVE".into()],
                standing: Some("hostile".into()),
                lines: vec!["Go away!".into()],
            }],
            faction_standing: "neutral".into(),
        };

        let snapshot = run_conversation_scenario(&scenario, 42, &reg);
        assert!(
            snapshot.dialogue_result.is_none(),
            "PEACEFUL NPC with no matching dialogue line should return None"
        );
    }

    #[test]
    fn harness_all_types_use_actual_tags_config() {
        let reg = registry();
        assert!(
            reg.tag_id("HUMANOID").is_some(),
            "tags.toml should define HUMANOID"
        );
        assert!(
            reg.tag_id("AGGRESSIVE").is_some(),
            "tags.toml should define AGGRESSIVE"
        );
        assert!(
            reg.tag_id("PEACEFUL").is_some(),
            "tags.toml should define PEACEFUL"
        );
        assert!(
            reg.tag_id("METAL").is_some(),
            "tags.toml should define METAL"
        );
        assert!(
            reg.tag_id("LEATHER").is_some(),
            "tags.toml should define LEATHER"
        );
        assert!(
            reg.tag_id("EQUIP_WEAPON").is_some(),
            "tags.toml should define EQUIP_WEAPON"
        );
        assert!(
            reg.tag_id("EQUIP_ARMOR").is_some(),
            "tags.toml should define EQUIP_ARMOR"
        );
        assert!(
            reg.tag_id("COMMON").is_some(),
            "tags.toml should define COMMON"
        );
        assert!(
            reg.tag_id("UNCOMMON").is_some(),
            "tags.toml should define UNCOMMON"
        );
    }
}
