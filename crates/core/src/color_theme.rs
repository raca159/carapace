use bevy_ecs::prelude::Resource;

#[derive(Resource, Debug, Clone)]
pub struct ColorTheme {
    pub title: (u8, u8, u8),
    pub heading: (u8, u8, u8),
    pub label: (u8, u8, u8),
    pub body: (u8, u8, u8),
    pub muted: (u8, u8, u8),
    pub border: (u8, u8, u8),
    pub cursor: (u8, u8, u8),
    pub highlighted: (u8, u8, u8),
    pub success: (u8, u8, u8),
    pub warning: (u8, u8, u8),
    pub danger: (u8, u8, u8),
    pub accent: (u8, u8, u8),
    pub menu_border: (u8, u8, u8),
    pub quest_board: (u8, u8, u8),
    pub death: (u8, u8, u8),
    pub pause: (u8, u8, u8),
    pub debug: (u8, u8, u8),
    pub biome: (u8, u8, u8),
    pub equipped: (u8, u8, u8),
    pub empty_slot: (u8, u8, u8),
    pub loading: (u8, u8, u8),
    pub speaker: (u8, u8, u8),
    pub messages_title: (u8, u8, u8),
    pub selector: (u8, u8, u8),
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            title: (0, 255, 255),
            heading: (255, 255, 0),
            label: (0, 255, 255),
            body: (255, 255, 255),
            muted: (64, 64, 64),
            border: (255, 255, 255),
            cursor: (255, 255, 0),
            highlighted: (255, 255, 255),
            success: (0, 255, 0),
            warning: (255, 255, 0),
            danger: (255, 0, 0),
            accent: (255, 0, 255),
            menu_border: (0, 255, 255),
            quest_board: (160, 120, 60),
            death: (255, 0, 0),
            pause: (255, 255, 0),
            debug: (255, 255, 0),
            biome: (255, 0, 255),
            equipped: (0, 255, 0),
            empty_slot: (64, 64, 64),
            loading: (255, 255, 0),
            speaker: (0, 255, 255),
            messages_title: (64, 64, 64),
            selector: (255, 255, 0),
        }
    }
}

pub fn dim_color(color: (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    (
        (color.0 as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.1 as f32 * factor).clamp(0.0, 255.0) as u8,
        (color.2 as f32 * factor).clamp(0.0, 255.0) as u8,
    )
}

pub fn desaturate_color(color: (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    let r = color.0 as f32;
    let g = color.1 as f32;
    let b = color.2 as f32;
    let gray = r * 0.299 + g * 0.587 + b * 0.114;
    (
        (r + (gray - r) * (1.0 - factor)).clamp(0.0, 255.0) as u8,
        (g + (gray - g) * (1.0 - factor)).clamp(0.0, 255.0) as u8,
        (b + (gray - b) * (1.0 - factor)).clamp(0.0, 255.0) as u8,
    )
}
