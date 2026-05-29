use serde::{Deserialize, Serialize};

use crate::config::LlmConfig;
use crate::error::LlmError;
use crate::provider::LlmProvider;

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

fn build_messages(prompt: &str, system_prompt: Option<&str>) -> Vec<ChatMessage> {
    let mut messages = Vec::new();

    if let Some(sys) = system_prompt {
        messages.push(ChatMessage {
            role: "system".into(),
            content: sys.into(),
        });
    }

    messages.push(ChatMessage {
        role: "user".into(),
        content: prompt.into(),
    });

    messages
}

fn extract_response(response: ChatResponse) -> Result<String, LlmError> {
    response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or(LlmError::InvalidResponse)
}

fn send_chat_request(config: &LlmConfig, messages: Vec<ChatMessage>) -> Result<String, LlmError> {
    let body = ChatRequest {
        model: config.model.clone(),
        messages,
        max_tokens: config.max_tokens.unwrap_or(256),
        temperature: config.temperature.unwrap_or(0.7),
    };

    let client = reqwest::blocking::Client::new();
    let mut request = client
        .post(&config.endpoint)
        .header("Content-Type", "application/json");

    if let Some(ref key) = config.api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request.json(&body).send()?;
    let chat_response: ChatResponse = response.json()?;

    extract_response(chat_response)
}

pub fn send_messages(
    config: &LlmConfig,
    provider: &dyn LlmProvider,
    messages: Vec<ChatMessage>,
) -> Result<String, LlmError> {
    if provider.name() == "openrouter" {
        let body = ChatRequest {
            model: config.model.clone(),
            messages,
            max_tokens: config.max_tokens.unwrap_or(256),
            temperature: config.temperature.unwrap_or(0.7),
        };

        let client = reqwest::blocking::Client::new();
        let mut request = client
            .post(&config.endpoint)
            .header("Content-Type", "application/json");

        if let Some(ref key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        if let Some(ref site_url) = config.site_url {
            request = request.header("HTTP-Referer", site_url.as_str());
        }

        if let Some(ref app_name) = config.app_name {
            request = request.header("X-Title", app_name.as_str());
        }

        let response = request.json(&body).send()?;
        let chat_response: ChatResponse = response.json()?;

        extract_response(chat_response)
    } else {
        send_chat_request(config, messages)
    }
}

pub struct OpenAiProvider;

impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn send(
        &self,
        config: &LlmConfig,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String, LlmError> {
        let messages = build_messages(prompt, system_prompt);
        send_chat_request(config, messages)
    }
}

pub struct OpenRouterProvider;

impl LlmProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    fn send(
        &self,
        config: &LlmConfig,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String, LlmError> {
        let messages = build_messages(prompt, system_prompt);

        let body = ChatRequest {
            model: config.model.clone(),
            messages,
            max_tokens: config.max_tokens.unwrap_or(256),
            temperature: config.temperature.unwrap_or(0.7),
        };

        let client = reqwest::blocking::Client::new();
        let mut request = client
            .post(&config.endpoint)
            .header("Content-Type", "application/json");

        if let Some(ref key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        if let Some(ref site_url) = config.site_url {
            request = request.header("HTTP-Referer", site_url.as_str());
        }

        if let Some(ref app_name) = config.app_name {
            request = request.header("X-Title", app_name.as_str());
        }

        let response = request.json(&body).send()?;
        let chat_response: ChatResponse = response.json()?;

        extract_response(chat_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_provider_has_correct_name() {
        let provider = OpenAiProvider;
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn openrouter_provider_has_correct_name() {
        let provider = OpenRouterProvider;
        assert_eq!(provider.name(), "openrouter");
    }

    #[test]
    fn build_messages_without_system_prompt() {
        let messages = build_messages("Hello", None);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn build_messages_with_system_prompt() {
        let messages = build_messages("Hello", Some("Be helpful"));
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "Be helpful");
        assert_eq!(messages[1].role, "user");
        assert_eq!(messages[1].content, "Hello");
    }

    #[test]
    fn extract_response_returns_content() {
        let response = ChatResponse {
            choices: vec![ChatChoice {
                message: ChatMessage {
                    role: "assistant".into(),
                    content: "Hi there!".into(),
                },
            }],
        };
        let result = extract_response(response).unwrap();
        assert_eq!(result, "Hi there!");
    }

    #[test]
    fn extract_response_empty_choices_returns_error() {
        let response = ChatResponse { choices: vec![] };
        let result = extract_response(response);
        assert!(matches!(result, Err(LlmError::InvalidResponse)));
    }

    #[test]
    fn extract_response_multiple_choices_uses_first() {
        let response = ChatResponse {
            choices: vec![
                ChatChoice {
                    message: ChatMessage {
                        role: "assistant".into(),
                        content: "First choice.".into(),
                    },
                },
                ChatChoice {
                    message: ChatMessage {
                        role: "assistant".into(),
                        content: "Second choice.".into(),
                    },
                },
            ],
        };
        let result = extract_response(response).unwrap();
        assert_eq!(result, "First choice.");
    }

    #[test]
    fn build_messages_empty_string_prompt() {
        let messages = build_messages("", Some("System"));
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].content, "");
    }

    #[test]
    fn build_messages_empty_system_prompt_omitted() {
        let messages = build_messages("Hello", Some(""));
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "");
    }

    #[test]
    fn openai_provider_send_without_api_key_returns_error() {
        let provider = OpenAiProvider;
        let config = LlmConfig {
            api_key: None,
            ..LlmConfig::default()
        };
        let result = provider.send(&config, "Hello", None);
        assert!(result.is_err());
    }

    #[test]
    fn openrouter_provider_send_without_endpoint_returns_error() {
        let provider = OpenRouterProvider;
        let config = LlmConfig {
            api_key: Some("sk-test".into()),
            endpoint: "http://127.0.0.1:1/chat".into(),
            ..LlmConfig::openrouter("openai/gpt-4o")
        };
        let result = provider.send(&config, "Hello", None);
        assert!(result.is_err());
    }

    #[test]
    fn send_messages_routes_to_openrouter_path() {
        struct TestProvider;
        impl LlmProvider for TestProvider {
            fn name(&self) -> &str {
                "openrouter"
            }
            fn send(
                &self,
                _config: &LlmConfig,
                _prompt: &str,
                _system_prompt: Option<&str>,
            ) -> Result<String, LlmError> {
                unreachable!("send_messages should not call provider.send()")
            }
        }

        let config = LlmConfig {
            api_key: Some("sk-test".into()),
            endpoint: "http://127.0.0.1:1/chat".into(),
            ..LlmConfig::openrouter("openai/gpt-4o")
        };
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
        }];
        let result = send_messages(&config, &TestProvider, messages);
        assert!(result.is_err());
    }

    #[test]
    fn send_messages_routes_to_openai_path() {
        struct TestProvider;
        impl LlmProvider for TestProvider {
            fn name(&self) -> &str {
                "openai"
            }
            fn send(
                &self,
                _config: &LlmConfig,
                _prompt: &str,
                _system_prompt: Option<&str>,
            ) -> Result<String, LlmError> {
                unreachable!("send_messages should not call provider.send()")
            }
        }

        let config = LlmConfig {
            api_key: None,
            endpoint: "http://127.0.0.1:1/chat".into(),
            ..LlmConfig::default()
        };
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
        }];
        let result = send_messages(&config, &TestProvider, messages);
        assert!(result.is_err());
    }
}
