use crate::renderer::layout::Colors;
use crate::themes::EditorTheme;
impl Theme {
    pub fn from_syntect(theme: &syntect::highlighting::Theme) -> Colors {
        // Try to extract background, foreground, etc. from syntect theme settings
        let bg = theme.settings.background.unwrap_or(syntect::highlighting::Color { r: 30, g: 30, b: 30, a: 255 });
        let fg = theme.settings.foreground.unwrap_or(syntect::highlighting::Color { r: 220, g: 220, b: 220, a: 255 });
        let sel = theme.settings.selection.unwrap_or(syntect::highlighting::Color { r: 60, g: 120, b: 200, a: 180 });
        let cursor = theme.settings.caret.unwrap_or(syntect::highlighting::Color { r: 255, g: 255, b: 255, a: 255 });
        let line_num = fg;
        let to_rgba = |c: syntect::highlighting::Color| {
            [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0, c.a as f32 / 255.0]
        };
        let bg_rgba = to_rgba(bg);
        Colors {
            background: bg_rgba,
            gutter_background: bg_rgba,
            status_bar_background: bg_rgba,
            text_color: to_rgba(fg),
            line_number_color: to_rgba(line_num),
            cursor_color: to_rgba(cursor),
            selection_color: to_rgba(sel),
            gutter_separator: bg_rgba,
            scrollbar_track: bg_rgba,
            scrollbar_thumb: bg_rgba,
        }
    }
}
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub line_number: [f32; 4],
    pub cursor: [f32; 4],
    pub selection: [f32; 4],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: [0.03, 0.03, 0.03, 1.0], // Dark gray
            foreground: [0.95, 0.95, 0.95, 1.0], // Almost white
            line_number: [0.4, 0.4, 0.4, 1.0],   // Medium gray
            cursor: [1.0, 1.0, 1.0, 1.0],        // White
            selection: [0.2, 0.4, 0.8, 0.3],     // Blue with alpha
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontConfig {
    pub size: f32,
    pub family: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            size: 14.0,
            family: "Courier New".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorConfig {
    pub theme: EditorTheme,
    pub font: FontConfig,
    pub tab_width: usize,
    pub use_spaces: bool,
    pub line_numbers: bool,
    pub syntax_theme: String, // syntect theme name
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            theme: EditorTheme::Dracula,
            font: FontConfig::default(),
            tab_width: 4,
            use_spaces: true,
            line_numbers: true,
            syntax_theme: "base16-ocean.dark".to_string(),
        }
    }
}
impl EditorConfig {
    pub fn colors(&self) -> Colors {
        self.theme.colors()
    }
}
