use ratatui::style::{Modifier, Style};

use super::palette::NierPalette;

pub struct NierTheme;

impl NierTheme {
    pub fn base() -> Style {
        Style::default()
            .bg(NierPalette::BG_DEEP)
            .fg(NierPalette::FG_PRIMARY)
    }

    pub fn panel() -> Style {
        Style::default()
            .bg(NierPalette::BG_PANEL)
            .fg(NierPalette::FG_PRIMARY)
    }

    pub fn highlight() -> Style {
        Style::default()
            .bg(NierPalette::BG_SURFACE)
            .fg(NierPalette::HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn accent() -> Style {
        Style::default().fg(NierPalette::ACCENT)
    }

    pub fn success() -> Style {
        Style::default().fg(NierPalette::ACCENT_RARE)
    }

    pub fn border() -> Style {
        Style::default().fg(NierPalette::BORDER)
    }

    pub fn secondary() -> Style {
        Style::default().fg(NierPalette::FG_SECONDARY)
    }
}
