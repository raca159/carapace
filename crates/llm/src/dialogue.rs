use crate::client::ChatMessage;
use crate::config::LlmConfig;
use crate::context::PromptContext;
use crate::prompt::substitute;
use std::collections::HashMap;

pub struct DialoguePromptBuilder {
    pub system_prompt: String,
    pub greeting_template: String,
    pub conversation_template: String,
    pub farewell_template: String,
}

impl DialoguePromptBuilder {
    pub fn new(config: &LlmConfig) -> Self {
        Self {
            system_prompt: config.prompts.system.clone(),
            greeting_template: config.prompts.greeting.clone(),
            conversation_template: config.prompts.conversation.clone(),
            farewell_template: config.prompts.farewell.clone(),
        }
    }

    pub fn build_greeting(
        &self,
        context: &PromptContext,
        personality_prompt: &str,
    ) -> Vec<ChatMessage> {
        let mut vars = HashMap::new();
        vars.insert("player_name".into(), context.player.name.clone());
        vars.insert("npc_name".into(), context.npc.name.clone());
        vars.insert("location".into(), context.scene.location.clone());

        let rendered = substitute(&self.greeting_template, &vars);

        self.build_messages(personality_prompt, &rendered)
    }

    pub fn build_conversation(
        &self,
        context: &PromptContext,
        personality_prompt: &str,
        history: &[(String, String)],
        player_input: &str,
    ) -> Vec<ChatMessage> {
        let mut history_str = String::new();
        for (speaker, text) in history {
            history_str.push_str(&format!("{}: {}\n", speaker, text));
        }

        let mut vars = HashMap::new();
        vars.insert("player_name".into(), context.player.name.clone());
        vars.insert("npc_name".into(), context.npc.name.clone());
        vars.insert("location".into(), context.scene.location.clone());
        vars.insert("history".into(), history_str);
        vars.insert("player_input".into(), player_input.to_string());

        let rendered = substitute(&self.conversation_template, &vars);

        self.build_messages(personality_prompt, &rendered)
    }

    pub fn build_farewell(
        &self,
        context: &PromptContext,
        personality_prompt: &str,
        history: &[(String, String)],
    ) -> Vec<ChatMessage> {
        let mut history_str = String::new();
        for (speaker, text) in history {
            history_str.push_str(&format!("{}: {}\n", speaker, text));
        }

        let mut vars = HashMap::new();
        vars.insert("player_name".into(), context.player.name.clone());
        vars.insert("npc_name".into(), context.npc.name.clone());
        vars.insert("location".into(), context.scene.location.clone());
        vars.insert("history".into(), history_str);

        let rendered = substitute(&self.farewell_template, &vars);

        self.build_messages(personality_prompt, &rendered)
    }

    fn build_messages(
        &self,
        personality_prompt: &str,
        content: &str,
    ) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        if !personality_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: personality_prompt.into(),
            });
        } else if !self.system_prompt.is_empty() {
            messages.push(ChatMessage {
                role: "system".into(),
                content: self.system_prompt.clone(),
            });
        }

        messages.push(ChatMessage {
            role: "user".into(),
            content: content.into(),
        });

        messages
    }
}
