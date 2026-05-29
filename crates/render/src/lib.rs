pub mod camera;
// pub mod debug;        // References deleted GameState — restored in Phase 4
// pub mod headless;     // Depends on pipeline trait — restored in Phase 3
pub mod hud;
pub mod menu;
pub mod message_log;
pub mod overlays;
pub mod panels;
// pub mod pipeline;     // Depends on deleted WorldRenderData — restored in Phase 3
// pub mod terminal;     // Depends on ui::Renderer — removed permanently
// pub mod ui;           // References deleted GameState — restored in Phase 2
pub mod world;

// pub mod bevy_pipeline;  // Depends on snapshot::GameStateVariant — restored in Phase 3
pub mod spritesheet;
pub mod style;

// pub use bevy_pipeline::BevyPipeline;
pub use camera::Camera;
// pub use headless::HeadlessPipeline;
// pub use pipeline::{RenderPipeline, RenderResult, RenderState};
// pub use terminal::TerminalPipeline;
// pub use ui::{Renderer, WorldRenderContext};
