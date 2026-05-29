use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use game_core::components::{Health, Name, Position};
use game_core::turn::TurnCounter;
use game_tags::serialization::{TagValueSnapshot, tags_to_snapshot};
use game_tags::{TagRegistry, Tags};

#[derive(Debug, Clone)]
pub struct PromptContext {
    pub system_instructions: String,
    pub scene: SceneInfo,
    pub player: EntityInfo,
    pub npc: EntityInfo,
    pub action: Option<InteractionAction>,
}

#[derive(Debug, Clone)]
pub struct SceneInfo {
    pub location: String,
    pub turn: u64,
    pub environment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub name: String,
    pub tag_names: Vec<String>,
    pub tag_details: Vec<TagDetail>,
    pub health: Option<EntityHealth>,
    pub position: Option<EntityPosition>,
    pub is_player: bool,
}

#[derive(Debug, Clone)]
pub struct TagDetail {
    pub name: String,
    pub kind: TagDetailKind,
}

#[derive(Debug, Clone)]
pub enum TagDetailKind {
    Present,
    Magnitude(f32),
    Ticks {
        remaining: u32,
        max: u32,
    },
    Both {
        magnitude: f32,
        remaining: u32,
        max: u32,
    },
}

#[derive(Debug, Clone)]
pub struct EntityHealth {
    pub current: u32,
    pub max: u32,
}

#[derive(Debug, Clone)]
pub struct EntityPosition {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Debug, Clone)]
pub enum InteractionOutcome {
    Success,
    Failure,
    CriticalSuccess,
    CriticalFailure,
}

impl InteractionOutcome {
    pub fn as_str(&self) -> &str {
        match self {
            InteractionOutcome::Success => "SUCCESS",
            InteractionOutcome::Failure => "FAILURE",
            InteractionOutcome::CriticalSuccess => "CRITICAL SUCCESS",
            InteractionOutcome::CriticalFailure => "CRITICAL FAILURE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InteractionAction {
    pub intent: String,
    pub outcome: InteractionOutcome,
}

fn collect_entity_info(
    world: &World,
    entity: Entity,
    registry: &TagRegistry,
    is_player: bool,
) -> EntityInfo {
    let name = world
        .get::<Name>(entity)
        .map_or_else(|| "Unknown".into(), |n| n.0.clone());

    let tag_names: Vec<String> = if let Some(tags) = world.get::<Tags>(entity) {
        let snapshot = tags_to_snapshot(tags, registry);
        snapshot.tags.iter().map(|(n, _)| n.clone()).collect()
    } else {
        Vec::new()
    };

    let tag_details: Vec<TagDetail> = if let Some(tags) = world.get::<Tags>(entity) {
        let snapshot = tags_to_snapshot(tags, registry);
        snapshot
            .tags
            .iter()
            .map(|(n, v)| TagDetail {
                name: n.clone(),
                kind: match v {
                    TagValueSnapshot::None => TagDetailKind::Present,
                    TagValueSnapshot::Magnitude(m) => TagDetailKind::Magnitude(*m),
                    TagValueSnapshot::Ticks { remaining, max } => TagDetailKind::Ticks {
                        remaining: *remaining,
                        max: *max,
                    },
                    TagValueSnapshot::MagnitudeAndTicks {
                        magnitude,
                        remaining,
                        max,
                    } => TagDetailKind::Both {
                        magnitude: *magnitude,
                        remaining: *remaining,
                        max: *max,
                    },
                },
            })
            .collect()
    } else {
        Vec::new()
    };

    let health = world.get::<Health>(entity).map(|h| EntityHealth {
        current: h.current,
        max: h.max,
    });

    let position = world
        .get::<Position>(entity)
        .map(|p| EntityPosition { x: p.x, y: p.y, z: 0 });

    EntityInfo {
        name,
        tag_names,
        tag_details,
        health,
        position,
        is_player,
    }
}

pub fn build_prompt_context(
    world: &World,
    player_entity: Entity,
    npc_entity: Entity,
    registry: &TagRegistry,
    location: &str,
    environment: Option<&str>,
    action: Option<InteractionAction>,
) -> PromptContext {
    let turn = world
        .get_resource::<TurnCounter>()
        .map_or(0, |tc| tc.current());

    let system_instructions = "You are the narrative overlay for a dark fantasy roguelike RPG set \
        a century after the collapse of a technologically advanced civilization. \
        Translate the following game engine states into raw, atmospheric, concise narrative. \
        Do not change the calculated mathematical outcomes. Keep responses to 1-3 sentences. \
        Never break character.";

    PromptContext {
        system_instructions: system_instructions.into(),
        scene: SceneInfo {
            location: location.into(),
            turn,
            environment: environment.map(String::from),
        },
        player: collect_entity_info(world, player_entity, registry, true),
        npc: collect_entity_info(world, npc_entity, registry, false),
        action,
    }
}

pub fn render_prompt(context: &PromptContext) -> String {
    let mut out = String::new();

    out.push_str("### SYSTEM INSTRUCTIONS\n");
    out.push_str(&context.system_instructions);
    out.push('\n');

    out.push_str("\n### SCENE METADATA\n");
    out.push_str(&format!("- Location: {}\n", context.scene.location));
    out.push_str(&format!("- Turn: {}\n", context.scene.turn));
    if let Some(ref env) = context.scene.environment {
        out.push_str(&format!("- Environment: {}\n", env));
    }

    if let Some(ref action) = context.action {
        out.push_str(&format!("- Action Intent: {}\n", action.intent));
        out.push_str(&format!(
            "- Engine Roll Outcome: {}\n",
            action.outcome.as_str()
        ));
    }

    out.push_str("\n### ENTITY A: THE PLAYER\n");
    render_entity(&mut out, &context.player, "Remnant");

    out.push_str("\n### ENTITY B: THE NPC\n");
    render_entity(&mut out, &context.npc, "Creature");

    out.push_str("\n### OUTPUT FORMAT\n");
    out.push_str(
        "[Player Line]: Dialogue and physical action description.\n\
         [NPC Line]: Dialogue and reactive body language.\n\
         [World Context]: Text describing the environmental change.\n",
    );

    out
}

fn render_entity(out: &mut String, info: &EntityInfo, kind: &str) {
    out.push_str(&format!("- Name: {}\n", info.name));
    out.push_str(&format!("- Kind: {}\n", kind));

    if let Some(ref health) = info.health {
        out.push_str(&format!("- Health: {}/{}\n", health.current, health.max));
    }

    if let Some(ref pos) = info.position {
        out.push_str(&format!("- Position: ({}, {})\n", pos.x, pos.y));
    }

    if !info.tag_names.is_empty() {
        out.push_str(&format!("- Tags: [{}]\n", info.tag_names.join(", ")));
    }

    for detail in &info.tag_details {
        let line = match detail.kind {
            TagDetailKind::Present => format!("  - {}", detail.name),
            TagDetailKind::Magnitude(m) => {
                format!("  - {} (magnitude: {:.1})", detail.name, m)
            }
            TagDetailKind::Ticks { remaining, max } => {
                format!("  - {} (duration: {}/{})", detail.name, remaining, max)
            }
            TagDetailKind::Both {
                magnitude,
                remaining,
                max,
            } => {
                format!(
                    "  - {} (magnitude: {:.1}, duration: {}/{})",
                    detail.name, magnitude, remaining, max
                )
            }
        };
        out.push_str(&line);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::components::{Creature, Glyph, Player as PlayerComp};

    fn setup_test_world() -> (World, Entity, Entity, TagRegistry) {
        let registry = {
            let mut builder = game_tags::TagRegistryBuilder::new();
            let phys = builder.add_archetype("physical", "Physical", game_tags::Exclusivity::Any);
            let ment = builder.add_archetype("mental", "Mental", game_tags::Exclusivity::Any);
            let el = builder.add_archetype("element", "Element", game_tags::Exclusivity::Mutual);
            let stat = builder.add_archetype("status", "Status", game_tags::Exclusivity::Any);

            builder
                .add_tag(
                    phys,
                    "HasChitin",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            builder
                .add_tag(
                    phys,
                    "CanTalk",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            builder
                .add_tag(
                    ment,
                    "Predatory",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            builder
                .add_tag(
                    ment,
                    "Deceptive",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            builder
                .add_tag(
                    el,
                    "FIRE",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            builder
                .add_tag(
                    stat,
                    "BURNING",
                    vec![],
                    vec![],
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();

            builder.build().unwrap()
        };

        let mut world = World::new();

        let player = world
            .spawn((
                PlayerComp,
                Name("Remnant-7".into()),
                Position { x: 10, y: 15, z: 0 },
                Health {
                    current: 80,
                    max: 100,
                },
                Glyph {
                    char: '@',
                    color: (255, 255, 0),
                },
                Tags::new(registry.tag_count()),
            ))
            .id();

        let npc = world
            .spawn((
                Creature,
                Name("Gate Guard".into()),
                Position { x: 12, y: 14, z: 0 },
                Health {
                    current: 50,
                    max: 50,
                },
                Glyph {
                    char: 'G',
                    color: (200, 200, 200),
                },
                Tags::new(registry.tag_count()),
            ))
            .id();

        world.insert_resource(TurnCounter::new());

        (world, player, npc, registry)
    }

    #[test]
    fn build_context_includes_entity_names() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);

        assert_eq!(ctx.player.name, "Remnant-7");
        assert_eq!(ctx.npc.name, "Gate Guard");
    }

    #[test]
    fn build_context_includes_health() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);

        let p_health = ctx.player.health.unwrap();
        assert_eq!(p_health.current, 80);
        assert_eq!(p_health.max, 100);

        let n_health = ctx.npc.health.unwrap();
        assert_eq!(n_health.current, 50);
        assert_eq!(n_health.max, 50);
    }

    #[test]
    fn build_context_includes_position() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);

        assert_eq!(ctx.player.position.as_ref().unwrap().x, 10);
        assert_eq!(ctx.player.position.as_ref().unwrap().y, 15);
        assert_eq!(ctx.npc.position.as_ref().unwrap().x, 12);
        assert_eq!(ctx.npc.position.as_ref().unwrap().y, 14);
    }

    #[test]
    fn build_context_with_tags_populates_tag_names() {
        let (mut world, player, npc, registry) = setup_test_world();
        let chitin_id = registry.tag_id("HasChitin").unwrap();
        let pred_id = registry.tag_id("Predatory").unwrap();

        {
            let mut tags = world.get_mut::<Tags>(player).unwrap();
            tags.add_tag(chitin_id, game_tags::TagValue::None, &registry);
        }
        {
            let mut tags = world.get_mut::<Tags>(npc).unwrap();
            tags.add_tag(pred_id, game_tags::TagValue::None, &registry);
        }

        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);

        assert!(ctx.player.tag_names.contains(&"HasChitin".to_string()));
        assert!(ctx.npc.tag_names.contains(&"Predatory".to_string()));
    }

    #[test]
    fn build_context_includes_scene_metadata() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(
            &world,
            player,
            npc,
            &registry,
            "Deep Trench",
            Some("Stormy Night"),
            None,
        );

        assert_eq!(ctx.scene.location, "Deep Trench");
        assert_eq!(ctx.scene.turn, 0);
        assert_eq!(ctx.scene.environment.unwrap(), "Stormy Night");
    }

    #[test]
    fn build_context_with_action_includes_action_info() {
        let (world, player, npc, registry) = setup_test_world();
        let action = InteractionAction {
            intent: "DECEIVE via chromatophores".into(),
            outcome: InteractionOutcome::Success,
        };

        let ctx = build_prompt_context(
            &world,
            player,
            npc,
            &registry,
            "Village Gates",
            Some("Pitch Black"),
            Some(action),
        );

        let a = ctx.action.unwrap();
        assert_eq!(a.intent, "DECEIVE via chromatophores");
        assert!(matches!(a.outcome, InteractionOutcome::Success));
    }

    #[test]
    fn render_prompt_outputs_expected_sections() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);
        let rendered = render_prompt(&ctx);

        assert!(rendered.contains("### SYSTEM INSTRUCTIONS"));
        assert!(rendered.contains("### SCENE METADATA"));
        assert!(rendered.contains("### ENTITY A: THE PLAYER"));
        assert!(rendered.contains("### ENTITY B: THE NPC"));
        assert!(rendered.contains("### OUTPUT FORMAT"));
    }

    #[test]
    fn render_prompt_includes_entity_details() {
        let (world, player, npc, registry) = setup_test_world();
        let ctx = build_prompt_context(&world, player, npc, &registry, "Village Gates", None, None);
        let rendered = render_prompt(&ctx);

        assert!(rendered.contains("Remnant-7"));
        assert!(rendered.contains("Gate Guard"));
        assert!(rendered.contains("Health: 80/100"));
        assert!(rendered.contains("Health: 50/50"));
    }

    #[test]
    fn interaction_outcome_display() {
        assert_eq!(InteractionOutcome::Success.as_str(), "SUCCESS");
        assert_eq!(InteractionOutcome::Failure.as_str(), "FAILURE");
        assert_eq!(
            InteractionOutcome::CriticalSuccess.as_str(),
            "CRITICAL SUCCESS"
        );
        assert_eq!(
            InteractionOutcome::CriticalFailure.as_str(),
            "CRITICAL FAILURE"
        );
    }

    #[test]
    fn tag_detail_kind_roundtrip() {
        let present = TagDetail {
            name: "FIRE".into(),
            kind: TagDetailKind::Present,
        };
        assert_eq!(present.name, "FIRE");
        assert!(matches!(present.kind, TagDetailKind::Present));

        let mag = TagDetail {
            name: "POISON".into(),
            kind: TagDetailKind::Magnitude(3.5),
        };
        assert!(matches!(mag.kind, TagDetailKind::Magnitude(3.5)));
    }

    #[test]
    fn context_with_unknown_entity_uses_fallback_name() {
        let registry = {
            let builder = game_tags::TagRegistryBuilder::new();
            builder.build().unwrap()
        };
        let mut world = World::new();
        let entity = world.spawn(()).id();
        let player = world.spawn((PlayerComp, Name("Hero".into()))).id();

        world.insert_resource(TurnCounter::new());

        let ctx = build_prompt_context(&world, player, entity, &registry, "Nowhere", None, None);

        assert_eq!(ctx.npc.name, "Unknown");
    }
}
