use ratatui::style::Color;

pub struct NierPalette;

impl NierPalette {
    pub const BG_DEEP: Color = Color::Rgb(46, 42, 39);
    pub const BG_PANEL: Color = Color::Rgb(75, 65, 61);
    pub const BG_SURFACE: Color = Color::Rgb(89, 78, 74);
    pub const FG_PRIMARY: Color = Color::Rgb(195, 189, 168);
    pub const FG_SECONDARY: Color = Color::Rgb(176, 171, 152);
    pub const ACCENT: Color = Color::Rgb(200, 121, 65);
    pub const HIGHLIGHT: Color = Color::Rgb(212, 201, 176);
    pub const ACCENT_RARE: Color = Color::Rgb(168, 181, 160);
    pub const GRID: Color = Color::Rgb(58, 54, 50);
    pub const BORDER: Color = Color::Rgb(139, 128, 112);
}
