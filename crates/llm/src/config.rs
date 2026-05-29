use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptConfig {
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub examine_entity: String,
    #[serde(default)]
    pub examine_location: String,
    #[serde(default)]
    pub npc_interaction: String,
    #[serde(default)]
    pub greeting: String,
    #[serde(default)]
    pub conversation: String,
    #[serde(default)]
    pub farewell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub site_url: Option<String>,
    pub app_name: Option<String>,
    #[serde(default)]
    pub prompts: PromptConfig,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            model: "gpt-4o-mini".into(),
            endpoint: "https://api.openai.com/v1/chat/completions".into(),
            api_key: None,
            max_tokens: Some(256),
            temperature: Some(0.7),
            site_url: None,
            app_name: None,
            prompts: PromptConfig::default(),
        }
    }
}

impl LlmConfig {
    pub fn openrouter(model: impl Into<String>) -> Self {
        Self {
            provider: "openrouter".into(),
            model: model.into(),
            endpoint: "https://openrouter.ai/api/v1/chat/completions".into(),
            api_key: None,
            max_tokens: Some(256),
            temperature: Some(0.7),
            site_url: None,
            app_name: None,
            prompts: PromptConfig::default(),
        }
    }

    pub fn from_toml(content: &str) -> Result<Self, crate::error::LlmError> {
        let config: Self = toml::from_str(content)
            .map_err(|e| crate::error::LlmError::ConfigParse(e.to_string()))?;
        Ok(config)
    }

    pub fn load_defaults() -> Result<Self, crate::error::LlmError> {
        Self::from_toml(include_str!("../assets/config/llm.toml"))
    }

    /// Build a config from environment variables.
    /// Reads `OPENROUTER_API_KEY` (required) and optionally
    /// `OPENROUTER_MODEL`, `LLM_SITE_URL`, `LLM_APP_NAME`.
    pub fn from_env() -> Result<Self, crate::error::LlmError> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .ok()
            .or_else(|| std::env::var("LLM_API_KEY").ok())
            .ok_or(crate::error::LlmError::MissingConfig(
                "OPENROUTER_API_KEY or LLM_API_KEY environment variable not set".into(),
            ))?;

        let model = std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openai/gpt-4o-mini".into());

        let site_url = std::env::var("LLM_SITE_URL").ok();
        let app_name = std::env::var("LLM_APP_NAME").ok();

        let mut config = Self {
            provider: "openrouter".into(),
            model,
            endpoint: "https://openrouter.ai/api/v1/chat/completions".into(),
            api_key: Some(api_key),
            max_tokens: Some(256),
            temperature: Some(0.7),
            site_url,
            app_name,
            prompts: PromptConfig::default(),
        };

        if let Ok(defaults) = Self::load_defaults() {
            config.prompts = defaults.prompts;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_uses_openai() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(
            config.endpoint,
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn default_has_default_parameters() {
        let config = LlmConfig::default();
        assert_eq!(config.max_tokens, Some(256));
        assert_eq!(config.temperature, Some(0.7));
        assert!(config.api_key.is_none());
        assert!(config.site_url.is_none());
        assert!(config.app_name.is_none());
    }

    #[test]
    fn openrouter_constructor_sets_correct_defaults() {
        let config = LlmConfig::openrouter("openai/gpt-4o");
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.model, "openai/gpt-4o");
        assert_eq!(
            config.endpoint,
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(config.max_tokens, Some(256));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn config_can_be_customized() {
        let config = LlmConfig {
            api_key: Some("sk-test-key".into()),
            site_url: Some("https://carapace.game".into()),
            app_name: Some("Carapace".into()),
            ..LlmConfig::openrouter("anthropic/claude-3-haiku")
        };
        assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
        assert_eq!(config.site_url.as_deref(), Some("https://carapace.game"));
        assert_eq!(config.app_name.as_deref(), Some("Carapace"));
        assert_eq!(config.model, "anthropic/claude-3-haiku");
    }

    #[test]
    fn from_toml_valid_config() {
        let toml = r#"
provider = "openrouter"
model = "openai/gpt-4o"
endpoint = "https://openrouter.ai/api/v1/chat/completions"
max_tokens = 512
temperature = 0.5

[prompts]
system = "You are a test narrator."
examine_entity = "Test entity template"
"#;
        let config = LlmConfig::from_toml(toml).unwrap();
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.model, "openai/gpt-4o");
        assert_eq!(
            config.endpoint,
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(config.max_tokens, Some(512));
        assert_eq!(config.temperature, Some(0.5));
        assert_eq!(config.prompts.system, "You are a test narrator.");
        assert_eq!(config.prompts.examine_entity, "Test entity template");
    }

    #[test]
    fn from_toml_minimal_config_uses_defaults() {
        let toml = r#"
provider = "openai"
model = "gpt-3.5-turbo"
endpoint = "https://api.openai.com/v1/chat/completions"
"#;
        let config = LlmConfig::from_toml(toml).unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.max_tokens, None);
        assert_eq!(config.temperature, None);
        assert_eq!(config.prompts.system, "");
    }

    #[test]
    fn from_toml_invalid_content_returns_config_parse_error() {
        let toml = "this is not [[[valid toml";
        let result = LlmConfig::from_toml(toml);
        assert!(matches!(result, Err(crate::error::LlmError::ConfigParse(_))));
    }

    #[test]
    fn from_toml_missing_required_fields_returns_error() {
        let toml = r#"
max_tokens = 100
"#;
        let result = LlmConfig::from_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn from_toml_with_missing_sections_uses_default_prompts() {
        let toml = r#"
provider = "openai"
model = "gpt-4"
endpoint = "https://api.openai.com/v1/chat/completions"
"#;
        let config = LlmConfig::from_toml(toml).unwrap();
        assert_eq!(config.prompts.system, "");
        assert_eq!(config.prompts.examine_entity, "");
        assert_eq!(config.prompts.examine_location, "");
        assert_eq!(config.prompts.npc_interaction, "");
    }

    #[test]
    fn from_env_without_key_returns_missing_config() {
        let openrouter_key = std::env::var("OPENROUTER_API_KEY").ok();
        let llm_key = std::env::var("LLM_API_KEY").ok();

        unsafe {
            std::env::remove_var("OPENROUTER_API_KEY");
            std::env::remove_var("LLM_API_KEY");
        }

        let result = LlmConfig::from_env();

        if let Some(val) = openrouter_key {
            unsafe { std::env::set_var("OPENROUTER_API_KEY", val) };
        }
        if let Some(val) = llm_key {
            unsafe { std::env::set_var("LLM_API_KEY", val) };
        }

        assert!(matches!(
            result,
            Err(crate::error::LlmError::MissingConfig(_))
        ));
    }

    #[test]
    fn from_env_with_openrouter_key_reads_correctly() {
        let openrouter_key = std::env::var("OPENROUTER_API_KEY").ok();
        let model = std::env::var("OPENROUTER_MODEL").ok();

        unsafe {
            std::env::set_var("OPENROUTER_API_KEY", "sk-test-key-12345");
            std::env::remove_var("OPENROUTER_MODEL");
        }

        let result = LlmConfig::from_env();

        if let Some(val) = openrouter_key {
            unsafe { std::env::set_var("OPENROUTER_API_KEY", val) };
        } else {
            unsafe { std::env::remove_var("OPENROUTER_API_KEY") };
        }
        if let Some(val) = model {
            unsafe { std::env::set_var("OPENROUTER_MODEL", val) };
        }

        let config = result.unwrap();
        assert_eq!(config.api_key.as_deref(), Some("sk-test-key-12345"));
        assert_eq!(config.provider, "openrouter");
        assert_eq!(
            config.endpoint,
            "https://openrouter.ai/api/v1/chat/completions"
        );
    }

    #[test]
    fn prompt_config_defaults_are_empty() {
        let prompts = PromptConfig::default();
        assert_eq!(prompts.system, "");
        assert_eq!(prompts.examine_entity, "");
        assert_eq!(prompts.examine_location, "");
        assert_eq!(prompts.npc_interaction, "");
    }

    #[test]
    fn prompt_config_serialize_deserialize_roundtrip() {
        let original = PromptConfig {
            system: "System".into(),
            examine_entity: "Entity".into(),
            examine_location: "Location".into(),
            npc_interaction: "Interaction".into(),
            greeting: "Greeting".into(),
            conversation: "Conversation".into(),
            farewell: "Farewell".into(),
        };
        let toml_str = toml::to_string(&original).unwrap();
        let restored: PromptConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(original.system, restored.system);
        assert_eq!(original.examine_entity, restored.examine_entity);
        assert_eq!(original.examine_location, restored.examine_location);
        assert_eq!(original.npc_interaction, restored.npc_interaction);
        assert_eq!(original.greeting, restored.greeting);
        assert_eq!(original.conversation, restored.conversation);
        assert_eq!(original.farewell, restored.farewell);
    }
}
