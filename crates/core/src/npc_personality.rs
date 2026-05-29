use bevy_ecs::prelude::*;
use serde::Deserialize;

fn deserialize_first_voice<'de, D>(deserializer: D) -> Result<Option<VoiceTemplate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let vec: Option<Vec<VoiceTemplate>> = Option::deserialize(deserializer)?;
    Ok(vec.and_then(|v| v.into_iter().next()))
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceTemplate {
    pub hostile: Option<String>,
    pub neutral: Option<String>,
    pub ally: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeightedValue {
    pub key: String,
    pub description: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeEntry {
    pub domain: String,
    pub description: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationTopic {
    pub trigger: String,
    pub template: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcPersonality {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub factions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub emotional_range: String,
    #[serde(default, deserialize_with = "deserialize_first_voice")]
    pub voice: Option<VoiceTemplate>,
    #[serde(default)]
    pub values: Option<Vec<WeightedValue>>,
    #[serde(default)]
    pub fears: Option<Vec<WeightedValue>>,
    #[serde(default)]
    pub knowledge: Option<Vec<KnowledgeEntry>>,
    #[serde(default)]
    pub conversation_topics: Option<Vec<ConversationTopic>>,
}

#[derive(Debug, Clone, Deserialize)]
struct NpcPersonalitiesToml {
    #[serde(rename = "personality")]
    personalities: Vec<NpcPersonality>,
}

pub fn load_npc_personalities(toml_str: &str) -> Result<Vec<NpcPersonality>, toml::de::Error> {
    let file: NpcPersonalitiesToml = toml::from_str(toml_str)?;
    Ok(file.personalities)
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NpcPersonalitiesResource {
    pub personalities: Vec<NpcPersonality>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_npc_personalities_from_inline_toml() {
        let toml = r#"
[[personality]]
id = "test_npc"
name = "Test NPC"
factions = ["test_faction"]
tags = ["HUMANOID"]
speech_style = "Casual"
emotional_range = "Calm to excited"

[[personality.voice]]
hostile = "Back off!"
neutral = "Hey there."
ally = "Good to see you, friend."

[[personality.values]]
key = "freedom"
description = "Values personal liberty"
weight = 0.8

[[personality.fears]]
key = "darkness"
description = "Afraid of the dark"
weight = 0.5

[[personality.knowledge]]
domain = "local_geography"
description = "Knows the area"
weight = 0.7

[[personality.conversation_topics]]
trigger = "greeting"
template = "Welcome, stranger."
"#;
        let personalities = load_npc_personalities(toml).unwrap();
        assert_eq!(personalities.len(), 1);
        assert_eq!(personalities[0].id, "test_npc");
        assert_eq!(personalities[0].name, "Test NPC");
        assert_eq!(personalities[0].factions, vec!["test_faction"]);
        assert_eq!(personalities[0].tags, vec!["HUMANOID"]);

        let voice = personalities[0].voice.as_ref().unwrap();
        assert_eq!(voice.hostile.as_deref(), Some("Back off!"));
        assert_eq!(voice.neutral.as_deref(), Some("Hey there."));
        assert_eq!(voice.ally.as_deref(), Some("Good to see you, friend."));

        let values = personalities[0].values.as_ref().unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].key, "freedom");
        assert!((values[0].weight - 0.8).abs() < f64::EPSILON);

        let fears = personalities[0].fears.as_ref().unwrap();
        assert_eq!(fears.len(), 1);
        assert_eq!(fears[0].key, "darkness");

        let knowledge = personalities[0].knowledge.as_ref().unwrap();
        assert_eq!(knowledge.len(), 1);
        assert_eq!(knowledge[0].domain, "local_geography");

        let topics = personalities[0].conversation_topics.as_ref().unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].trigger, "greeting");
    }

    #[test]
    fn load_actual_npc_personalities_toml() {
        let npc_toml = include_str!("../../../assets/config/npc_personalities.toml");
        let personalities = load_npc_personalities(npc_toml).unwrap();
        assert!(personalities.len() >= 11, "should have at least 11 personality archetypes");
        assert!(personalities.iter().any(|p| p.id == "sanguine_noble"));
        assert!(personalities.iter().any(|p| p.id == "sanguine_enforcer"));
        assert!(personalities.iter().any(|p| p.id == "familiar_zealot"));
    }

    #[test]
    fn npc_personality_resource_default() {
        let resource = NpcPersonalitiesResource::default();
        assert!(resource.personalities.is_empty());
    }

    #[test]
    fn load_minimal_personality() {
        let toml = r#"
[[personality]]
id = "minimal"
name = "Minimal NPC"
"#;
        let personalities = load_npc_personalities(toml).unwrap();
        assert_eq!(personalities.len(), 1);
        assert_eq!(personalities[0].id, "minimal");
        assert!(personalities[0].voice.is_none());
        assert!(personalities[0].values.is_none());
        assert!(personalities[0].fears.is_none());
        assert!(personalities[0].knowledge.is_none());
        assert!(personalities[0].conversation_topics.is_none());
    }
}
