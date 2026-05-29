use ratatui::style::{Color, Modifier, Style};

use game_core::ColorTheme;

fn tc(c: (u8, u8, u8)) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

pub struct Typography;

impl Typography {
    #[inline]
    pub fn title(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.title)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn heading(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.heading)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn label(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.label))
    }
    #[inline]
    pub fn body(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.body))
    }
    #[inline]
    pub fn muted(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.muted))
    }
    #[inline]
    pub fn border(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.border))
    }
    #[inline]
    pub fn cursor(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.cursor))
    }
    #[inline]
    pub fn highlighted(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.highlighted)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn success(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.success)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn success_body(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.success))
    }
    #[inline]
    pub fn warning(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.warning))
    }
    #[inline]
    pub fn danger(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.danger)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn danger_body(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.danger))
    }
    #[inline]
    pub fn accent(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.accent))
    }
    #[inline]
    pub fn menu_border(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.menu_border))
    }
    #[inline]
    pub fn quest_board_title(theme: &ColorTheme) -> Style {
        Style::default().fg(tc(theme.quest_board)).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn hp_style(color: Color) -> Style {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    }
    #[inline]
    pub fn slot_style(theme: &ColorTheme, has_item: bool) -> Style {
        Style::default().fg(if has_item { tc(theme.equipped) } else { tc(theme.empty_slot) })
    }
}
