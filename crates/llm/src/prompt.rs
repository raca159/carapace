use std::collections::HashMap;

use crate::client::ChatMessage;
use crate::config::LlmConfig;
use crate::context::{EntityInfo, PromptContext};

pub struct PromptBuilder {
    pub system_prompt: String,
    pub examine_entity_template: String,
    pub examine_location_template: String,
    pub npc_interaction_template: String,
}

impl PromptBuilder {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            system_prompt: config.prompts.system.clone(),
            examine_entity_template: config.prompts.examine_entity.clone(),
            examine_location_template: config.prompts.examine_location.clone(),
            npc_interaction_template: config.prompts.npc_interaction.clone(),
        }
    }

    pub fn build_messages(&self, context: &PromptContext) -> Vec<ChatMessage> {
        let (template, npc_kind) = if context.action.is_some() {
            (&self.npc_interaction_template, "Creature")
        } else if !context.npc.name.is_empty() && context.npc.name != "Unknown" {
            (&self.examine_entity_template, "Creature")
        } else {
            (&self.examine_location_template, "Creature")
        };

        let vars = build_var_map(context, npc_kind, &[]);
        let rendered = substitute(template, &vars);

        let mut messages = Vec::new();

        if !self.system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: self.system_prompt.clone(),
            });
        } else if !context.system_instructions.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: context.system_instructions.clone(),
            });
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: rendered,
        });

        messages
    }

    pub fn build_messages_with_nearby(
        &self,
        context: &PromptContext,
        nearby: &[EntityInfo],
    ) -> Vec<ChatMessage> {
        let (template, npc_kind) = if context.action.is_some() {
            (&self.npc_interaction_template, "Creature")
        } else if !context.npc.name.is_empty() && context.npc.name != "Unknown" {
            (&self.examine_entity_template, "Creature")
        } else {
            (&self.examine_location_template, "Creature")
        };

        let vars = build_var_map(context, npc_kind, nearby);
        let rendered = substitute(template, &vars);

        let mut messages = Vec::new();

        if !self.system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: self.system_prompt.clone(),
            });
        } else if !context.system_instructions.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: context.system_instructions.clone(),
            });
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: rendered,
        });

        messages
    }
}

fn build_var_map(
    context: &PromptContext,
    npc_kind: &str,
    nearby: &[EntityInfo],
) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    vars.insert("player_name".into(), context.player.name.clone());
    vars.insert(
        "player_tags".into(),
        format_tag_list(&context.player.tag_names),
    );
    vars.insert(
        "player_health".into(),
        format_health(&context.player.health),
    );
    vars.insert(
        "player_health_context".into(),
        format_health_context(&context.player.health),
    );
    vars.insert(
        "player_position".into(),
        format_position(&context.player.position),
    );

    vars.insert("npc_name".into(), context.npc.name.clone());
    vars.insert("npc_kind".into(), npc_kind.into());
    vars.insert(
        "npc_tags".into(),
        format_tag_list(&context.npc.tag_names),
    );
    vars.insert(
        "npc_health".into(),
        format_health(&context.npc.health),
    );
    vars.insert(
        "npc_health_context".into(),
        format_health_context(&context.npc.health),
    );
    vars.insert(
        "npc_position".into(),
        format_position(&context.npc.position),
    );

    vars.insert("location".into(), context.scene.location.clone());
    vars.insert("turn".into(), context.scene.turn.to_string());

    let env = context
        .scene
        .environment
        .as_deref()
        .unwrap_or("an unknown environment");
    vars.insert("environment".into(), env.into());

    vars.insert("nearby_summary".into(), format_nearby_summary(nearby));

    if let Some(ref action) = context.action {
        vars.insert("action_intent".into(), action.intent.clone());
        vars.insert("action_outcome".into(), action.outcome.as_str().into());
    } else {
        vars.insert("action_intent".into(), String::new());
        vars.insert("action_outcome".into(), String::new());
    }

    vars
}

pub fn substitute(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            let start = i;
            i += 1;
            let key_start = i;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            if i < chars.len() {
                let key: String = chars[key_start..i].iter().collect();
                if let Some(value) = vars.get(&key) {
                    result.push_str(value);
                }
                i += 1;
            } else {
                result.push_str(&chars[start..].iter().collect::<String>());
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

pub fn format_tag_list(tags: &[String]) -> String {
    if tags.is_empty() {
        "none".into()
    } else {
        tags.join(", ")
    }
}

pub fn format_faction_standings(standings: &[(String, i32)]) -> String {
    if standings.is_empty() {
        "no known affiliations".into()
    } else {
        standings
            .iter()
            .map(|(faction, rep)| {
                let stance = if *rep >= 50 {
                    "friendly"
                } else if *rep >= 0 {
                    "neutral"
                } else if *rep >= -50 {
                    "hostile"
                } else {
                    "sworn enemy"
                };
                format!("{} ({}, {})", faction, stance, rep)
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub fn format_nearby_summary(entities: &[EntityInfo]) -> String {
    if entities.is_empty() {
        "none notable".into()
    } else {
        entities
            .iter()
            .map(|e| {
                let tags = format_tag_list(&e.tag_names);
                format!("{} [{}]", e.name, tags)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_health(health: &Option<crate::context::EntityHealth>) -> String {
    health
        .as_ref()
        .map_or_else(|| "unknown".into(), |h| format!("{}/{}", h.current, h.max))
}

fn format_health_context(health: &Option<crate::context::EntityHealth>) -> String {
    health
        .as_ref()
        .map_or_else(String::new, |h| format!("Health: {}/{}. ", h.current, h.max))
}

fn format_position(pos: &Option<crate::context::EntityPosition>) -> String {
    pos.as_ref()
        .map_or_else(|| "unknown".into(), |p| format!("({}, {})", p.x, p.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_context() -> PromptContext {
        use crate::context::{
            EntityHealth, EntityInfo, EntityPosition, InteractionAction, InteractionOutcome,
            SceneInfo,
        };
        PromptContext {
            system_instructions: "Dark fantasy RPG narrator.".into(),
            scene: SceneInfo {
                location: "Ruined Cathedral".into(),
                turn: 42,
                environment: Some("Moonlit night with drifting ash".into()),
            },
            player: EntityInfo {
                name: "Remnant-7".into(),
                tag_names: vec!["HasChitin".into(), "CanTalk".into()],
                tag_details: vec![],
                health: Some(EntityHealth {
                    current: 75,
                    max: 100,
                }),
                position: Some(EntityPosition { x: 10, y: 15, z: 0 }),
                is_player: true,
            },
            npc: EntityInfo {
                name: "Gate Guard".into(),
                tag_names: vec!["Predatory".into(), "Deceptive".into()],
                tag_details: vec![],
                health: Some(EntityHealth {
                    current: 30,
                    max: 50,
                }),
                position: Some(EntityPosition { x: 12, y: 14, z: 0 }),
                is_player: false,
            },
            action: Some(InteractionAction {
                intent: "DECEIVE via chromatophores".into(),
                outcome: InteractionOutcome::Success,
            }),
        }
    }

    fn make_test_builder() -> PromptBuilder {
        PromptBuilder {
            system_prompt: "You are a dark fantasy narrator.".into(),
            examine_entity_template: "Describe {npc_name} ({npc_kind}) at {npc_position} with tags: {npc_tags}.".into(),
            examine_location_template: "Describe {location} at turn {turn}. Environment: {environment}. Player {player_name} at {player_position}. Nearby: {nearby_summary}.".into(),
            npc_interaction_template: "{player_name} attempts {action_intent} on {npc_name}. Outcome: {action_outcome}. {npc_tags}.".into(),
        }
    }

    #[test]
    fn builder_extracts_templates_from_config() {
        let config = LlmConfig {
            prompts: crate::config::PromptConfig {
                system: "System prompt".into(),
                examine_entity: "Entity template".into(),
                examine_location: "Location template".into(),
                npc_interaction: "Interaction template".into(),
                greeting: String::new(),
                conversation: String::new(),
                farewell: String::new(),
            },
            ..LlmConfig::default()
        };
        let builder = PromptBuilder::new(&config);
        assert_eq!(builder.system_prompt, "System prompt");
        assert_eq!(builder.examine_entity_template, "Entity template");
        assert_eq!(builder.examine_location_template, "Location template");
        assert_eq!(builder.npc_interaction_template, "Interaction template");
    }

    #[test]
    fn build_messages_with_action_uses_interaction_template() {
        let builder = make_test_builder();
        let ctx = make_test_context();
        let messages = builder.build_messages(&ctx);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert!(messages[1].content.contains("Remnant-7"));
        assert!(messages[1].content.contains("DECEIVE via chromatophores"));
        assert!(messages[1].content.contains("SUCCESS"));
        assert!(messages[1].content.contains("Gate Guard"));
    }

    #[test]
    fn build_messages_no_action_uses_entity_template() {
        let builder = make_test_builder();
        let mut ctx = make_test_context();
        ctx.action = None;
        let messages = builder.build_messages(&ctx);
        assert_eq!(messages.len(), 2);
        assert!(messages[1].content.contains("Describe"));
        assert!(messages[1].content.contains("Gate Guard"));
        assert!(messages[1].content.contains("Creature"));
    }

    #[test]
    fn build_messages_falls_back_to_location_when_npc_unknown() {
        let builder = make_test_builder();
        let mut ctx = make_test_context();
        ctx.action = None;
        ctx.npc.name = "Unknown".into();
        let messages = builder.build_messages(&ctx);
        assert!(messages[1].content.contains("Ruined Cathedral"));
        assert!(messages[1].content.contains("turn 42"));
    }

    #[test]
    fn missing_context_keys_do_not_panic() {
        let builder = make_test_builder();
        let ctx = make_test_context();
        let messages = builder.build_messages(&ctx);
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn empty_system_prompt_falls_back_to_context_instructions() {
        let builder = PromptBuilder {
            system_prompt: "".into(),
            examine_entity_template: "Describe {npc_name}.".into(),
            examine_location_template: "Describe {location}.".into(),
            npc_interaction_template: "{player_name} interacts with {npc_name}.".into(),
        };
        let ctx = make_test_context();
        let messages = builder.build_messages(&ctx);
        assert_eq!(messages[0].role, "system");
        assert_eq!(
            messages[0].content,
            "Dark fantasy RPG narrator."
        );
    }

    #[test]
    fn build_messages_with_nearby_includes_summary() {
        use crate::context::{EntityHealth, EntityPosition};
        let builder = make_test_builder();
        let mut ctx = make_test_context();
        ctx.action = None;
        ctx.npc.name = "Unknown".into();
        let nearby = vec![
            EntityInfo {
                name: "Scavenger".into(),
                tag_names: vec!["HOSTILE".into()],
                tag_details: vec![],
                health: Some(EntityHealth {
                    current: 40,
                    max: 40,
                }),
                position: Some(EntityPosition { x: 11, y: 14, z: 0 }),
                is_player: false,
            },
        ];
        let messages = builder.build_messages_with_nearby(&ctx, &nearby);
        assert_eq!(messages.len(), 2);
        assert!(messages[1].content.contains("Scavenger"));
        assert!(messages[1].content.contains("Ruined Cathedral"));
    }

    #[test]
    fn build_messages_with_nearby_uses_interaction_template_when_action_present() {
        let builder = make_test_builder();
        let ctx = make_test_context();
        let nearby = vec![];
        let messages = builder.build_messages_with_nearby(&ctx, &nearby);
        assert!(messages[1].content.contains("DECEIVE via chromatophores"));
        assert!(messages[1].content.contains("SUCCESS"));
    }

    #[test]
    fn format_tag_list_empty() {
        assert_eq!(format_tag_list(&[]), "none");
    }

    #[test]
    fn format_tag_list_populated() {
        let tags = vec!["FIRE".into(), "BURNING".into(), "HOSTILE".into()];
        assert_eq!(format_tag_list(&tags), "FIRE, BURNING, HOSTILE");
    }

    #[test]
    fn format_faction_standings_empty() {
        assert_eq!(format_faction_standings(&[]), "no known affiliations");
    }

    #[test]
    fn format_faction_standings_various_rep() {
        let standings = vec![
            ("Iron Covenant".into(), 75),
            ("Ash Walkers".into(), 0),
            ("Scavenger Guild".into(), -30),
            ("Bone Court".into(), -80),
        ];
        let result = format_faction_standings(&standings);
        assert!(result.contains("Iron Covenant (friendly, 75)"));
        assert!(result.contains("Ash Walkers (neutral, 0)"));
        assert!(result.contains("Scavenger Guild (hostile, -30)"));
        assert!(result.contains("Bone Court (sworn enemy, -80)"));
    }

    #[test]
    fn format_nearby_summary_empty() {
        assert_eq!(format_nearby_summary(&[]), "none notable");
    }

    #[test]
    fn format_nearby_summary_with_entities() {
        use crate::context::{EntityHealth, EntityPosition};
        let entities = vec![
            EntityInfo {
                name: "Scavenger".into(),
                tag_names: vec!["HOSTILE".into(), "ARMED".into()],
                tag_details: vec![],
                health: Some(EntityHealth {
                    current: 40,
                    max: 40,
                }),
                position: Some(EntityPosition { x: 11, y: 14, z: 0 }),
                is_player: false,
            },
            EntityInfo {
                name: "Wreckage".into(),
                tag_names: vec!["LOOTABLE".into()],
                tag_details: vec![],
                health: None,
                position: Some(EntityPosition { x: 9, y: 16, z: 0 }),
                is_player: false,
            },
        ];
        let result = format_nearby_summary(&entities);
        assert!(result.contains("Scavenger [HOSTILE, ARMED]"));
        assert!(result.contains("Wreckage [LOOTABLE]"));
    }

    #[test]
    fn substitution_replaces_all_keys() {
        let template = "{player_name} sees {npc_name} in {location}.";
        let mut vars = HashMap::new();
        vars.insert("player_name".into(), "Remnant-7".into());
        vars.insert("npc_name".into(), "Gate Guard".into());
        vars.insert("location".into(), "Ruined Cathedral".into());
        let result = substitute(template, &vars);
        assert_eq!(result, "Remnant-7 sees Gate Guard in Ruined Cathedral.");
    }

    #[test]
    fn substitution_missing_key_omitted_gracefully() {
        let template = "{player_name} sees {npc_name} at {missing_key}.";
        let mut vars = HashMap::new();
        vars.insert("player_name".into(), "Remnant-7".into());
        vars.insert("npc_name".into(), "Gate Guard".into());
        let result = substitute(template, &vars);
        assert_eq!(result, "Remnant-7 sees Gate Guard at .");
    }

    #[test]
    fn substitution_no_keys_returns_template_as_is() {
        let template = "Plain text with no substitutions.";
        let vars = HashMap::new();
        let result = substitute(template, &vars);
        assert_eq!(result, "Plain text with no substitutions.");
    }
}
