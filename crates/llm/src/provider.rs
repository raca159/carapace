use crate::config::LlmConfig;
use crate::error::LlmError;

pub trait LlmProvider {
    fn name(&self) -> &str;
    fn send(
        &self,
        config: &LlmConfig,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Result<String, LlmError>;
}
