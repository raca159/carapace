pub mod client;
pub mod config;
pub mod context;
pub mod dialogue;
pub mod error;
pub mod prompt;
pub mod provider;

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use game_tags::TagRegistry;

pub use client::{ChatMessage, OpenAiProvider, OpenRouterProvider, send_messages};
pub use config::{LlmConfig, PromptConfig};
pub use context::{
    EntityHealth, EntityInfo, EntityPosition, InteractionAction, InteractionOutcome, PromptContext,
    SceneInfo, TagDetail, TagDetailKind, build_prompt_context, render_prompt,
};
pub use error::LlmError;
pub use prompt::{
    PromptBuilder, format_faction_standings, format_nearby_summary, format_tag_list,
};
pub use provider::LlmProvider;

/// Assembles ECS context, builds the prompt via the builder, and calls the LLM provider.
/// Returns the narrative text or an error.
#[allow(clippy::too_many_arguments)]
pub fn narrate_scene(
    world: &World,
    client: &dyn LlmProvider,
    builder: &PromptBuilder,
    config: &LlmConfig,
    tag_registry: &TagRegistry,
    location: &str,
    environment: Option<&str>,
    player: Entity,
    target: Entity,
) -> Result<String, LlmError> {
    let context =
        build_prompt_context(world, player, target, tag_registry, location, environment, None);

    let messages = builder.build_messages(&context);

    send_messages(config, client, messages)
}

/// Convenience wrapper that loads config from environment variables.
#[allow(clippy::too_many_arguments)]
pub fn narrate_scene_from_env(
    world: &World,
    client: &dyn LlmProvider,
    builder: &PromptBuilder,
    tag_registry: &TagRegistry,
    location: &str,
    environment: Option<&str>,
    player: Entity,
    target: Entity,
) -> Result<String, LlmError> {
    let config = LlmConfig::from_env()?;
    narrate_scene(
        world, client, builder, &config, tag_registry, location, environment, player, target,
    )
}
