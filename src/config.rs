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
    pub theme: Theme,
    pub font: FontConfig,
    pub tab_width: usize,
    pub use_spaces: bool,
    pub line_numbers: bool,
    pub syntax_theme: String, // syntect theme name
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            font: FontConfig::default(),
            tab_width: 4,
            use_spaces: true,
            line_numbers: true,
            syntax_theme: "base16-ocean.dark".to_string(),
        }
    }
}
