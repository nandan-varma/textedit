use crate::renderer::layout::Colors;
use crate::themes::EditorTheme;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::FontConfig;

    #[test]
    fn test_font_config_default() {
        let config = FontConfig::default();
        assert_eq!(config.size, 14.0);
        assert_eq!(config.family, "Courier New");
    }

    #[test]
    fn test_font_config_clone() {
        let config1 = FontConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.size, config2.size);
        assert_eq!(config1.family, config2.family);
    }
}
