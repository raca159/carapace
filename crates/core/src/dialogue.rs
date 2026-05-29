use bevy_ecs::prelude::Resource;
use serde::Deserialize;

use game_tags::{TagRegistry, Tags};

#[derive(Debug, Clone, Deserialize)]
pub struct DialogueLine {
    pub trigger_tags: Vec<String>,
    #[serde(default)]
    pub standing: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DialogueToml {
    #[serde(rename = "dialogue")]
    entries: Vec<DialogueLine>,
}

pub fn load_dialogue(toml_str: &str) -> Result<Vec<DialogueLine>, toml::de::Error> {
    let file: DialogueToml = toml::from_str(toml_str)?;
    Ok(file.entries)
}

pub fn select_dialogue(
    npc_tags: &Tags,
    faction_standing: &str,
    dialogue_lines: &[DialogueLine],
    registry: &TagRegistry,
) -> Option<String> {
    let mut best_match: Option<&DialogueLine> = None;
    let mut best_score: usize = 0;

    for entry in dialogue_lines {
        let mut matched_count: usize = 0;
        let mut all_matched = true;

        for tag_name in &entry.trigger_tags {
            if let Some(tag_id) = registry.tag_id(tag_name) {
                if npc_tags.has(tag_id) {
                    matched_count += 1;
                } else {
                    all_matched = false;
                }
            } else {
                all_matched = false;
            }
        }

        if !all_matched {
            continue;
        }

        let standing_match = match &entry.standing {
            Some(s) => s == faction_standing,
            None => true,
        };

        if !standing_match {
            continue;
        }

        let score = matched_count + if entry.standing.is_some() { 10 } else { 0 };
        if score > best_score {
            best_score = score;
            best_match = Some(entry);
        }
    }

    best_match.and_then(|entry| {
        if entry.lines.is_empty() {
            None
        } else {
            let idx = rand::random_range(0..entry.lines.len());
            Some(entry.lines[idx].clone())
        }
    })
}

#[derive(Resource, Debug, Clone, Default)]
pub struct DialogueLinesResource {
    pub lines: Vec<DialogueLine>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_tags::TagValue;

    const TAGS_TOML: &str = r#"
[[archetype]]
id = "personality"
name = "Personality"
exclusivity = "any"

[[archetype.tags]]
id = "AGGRESSIVE"
name = "AGGRESSIVE"

[[archetype.tags]]
id = "PEACEFUL"
name = "PEACEFUL"

[[archetype.tags]]
id = "TERRITORIAL"
name = "TERRITORIAL"

[[archetype.tags]]
id = "CURIOUS"
name = "CURIOUS"
"#;

    const DIALOGUE_TOML: &str = r#"
[[dialogue]]
trigger_tags = ["AGGRESSIVE"]
standing = "hostile"
lines = ["What do you want?", "Make it quick."]

[[dialogue]]
trigger_tags = ["PEACEFUL"]
standing = "ally"
lines = ["Good to see you, friend!", "How can I help?"]

[[dialogue]]
trigger_tags = ["TERRITORIAL"]
lines = ["You're in my territory.", "Watch yourself."]

[[dialogue]]
trigger_tags = ["CURIOUS"]
lines = ["What's that you're carrying?", "Interesting..."]
"#;

    fn setup_registry() -> TagRegistry {
        game_tags::load_tag_registry(TAGS_TOML).unwrap()
    }

    #[test]
    fn load_dialogue_parses_entries() {
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].trigger_tags, vec!["AGGRESSIVE"]);
        assert_eq!(entries[0].standing, Some("hostile".to_string()));
        assert_eq!(entries[0].lines.len(), 2);
    }

    #[test]
    fn select_dialogue_matches_aggressive_hostile() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);

        let result = select_dialogue(&tags, "hostile", &entries, &registry);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text == "What do you want?" || text == "Make it quick.");
    }

    #[test]
    fn select_dialogue_standing_filter_rejects_mismatch() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);

        let result = select_dialogue(&tags, "ally", &entries, &registry);
        assert!(result.is_none(), "AGGRESSIVE entry requires hostile standing, should not match ally");
    }

    #[test]
    fn select_dialogue_matches_peaceful_ally() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("PEACEFUL").unwrap(), TagValue::None, &registry);

        let result = select_dialogue(&tags, "ally", &entries, &registry);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text == "Good to see you, friend!" || text == "How can I help?");
    }

    #[test]
    fn select_dialogue_matches_no_standing_filter() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("TERRITORIAL").unwrap(), TagValue::None, &registry);

        let result = select_dialogue(&tags, "neutral", &entries, &registry);
        assert!(result.is_some());
    }

    #[test]
    fn select_dialogue_no_match_returns_none() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let tags = Tags::new(registry.tag_count());

        let result = select_dialogue(&tags, "neutral", &entries, &registry);
        assert!(result.is_none());
    }

    #[test]
    fn select_dialogue_prefers_standing_specific_match() {
        let registry = setup_registry();
        let entries = load_dialogue(DIALOGUE_TOML).unwrap();
        let mut tags = Tags::new(registry.tag_count());
        tags.add_tag(registry.tag_id("AGGRESSIVE").unwrap(), TagValue::None, &registry);
        tags.add_tag(registry.tag_id("TERRITORIAL").unwrap(), TagValue::None, &registry);

        let result = select_dialogue(&tags, "hostile", &entries, &registry);
        assert!(result.is_some());
    }
}
