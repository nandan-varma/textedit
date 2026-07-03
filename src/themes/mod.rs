use std::fmt;
use std::str::FromStr;

use crate::error::{EditorError, Result};
use crate::renderer::layout::Colors;

pub mod dark;
pub mod light;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default, PartialEq)]
pub enum EditorTheme {
    #[default]
    Dark,
    Light,
}

impl EditorTheme {
    pub fn all() -> &'static [(Self, &'static str)] {
        &[(Self::Dark, "Dark"), (Self::Light, "Light")]
    }

    pub fn colors(&self) -> Colors {
        match self {
            EditorTheme::Dark => dark::dark(),
            EditorTheme::Light => light::light(),
        }
    }

    pub fn syntax_palette(&self) -> &'static [([f32; 4], &'static str)] {
        match self {
            EditorTheme::Dark => dark::dark_syntax_palette(),
            EditorTheme::Light => light::light_syntax_palette(),
        }
    }
}

impl fmt::Display for EditorTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorTheme::Dark => write!(f, "Dark"),
            EditorTheme::Light => write!(f, "Light"),
        }
    }
}

impl FromStr for EditorTheme {
    type Err = EditorError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Dark" | "dark" => Ok(Self::Dark),
            "Light" | "light" => Ok(Self::Light),
            _ => Err(EditorError::InvalidTheme(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_display() {
        assert_eq!(EditorTheme::Dark.to_string(), "Dark");
        assert_eq!(EditorTheme::Light.to_string(), "Light");
    }

    #[test]
    fn test_theme_from_str() {
        assert_eq!("Dark".parse::<EditorTheme>().unwrap(), EditorTheme::Dark);
        assert_eq!("dark".parse::<EditorTheme>().unwrap(), EditorTheme::Dark);
        assert_eq!("Light".parse::<EditorTheme>().unwrap(), EditorTheme::Light);
        assert_eq!("light".parse::<EditorTheme>().unwrap(), EditorTheme::Light);
    }

    #[test]
    fn test_theme_from_str_invalid() {
        assert!("Dracula".parse::<EditorTheme>().is_err());
        assert!("".parse::<EditorTheme>().is_err());
    }

    #[test]
    fn test_all_variants_round_trip() {
        for (theme, label) in EditorTheme::all() {
            assert_eq!(theme.to_string(), *label);
            assert_eq!(label.parse::<EditorTheme>().unwrap(), *theme);
        }
    }

    #[test]
    fn test_dark_returns_colors() {
        let colors = EditorTheme::Dark.colors();
        // Spot-check that all fields are sensible (non-zero alpha)
        assert!((colors.background[3] - 1.0).abs() < f32::EPSILON);
        assert!((colors.text_color[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_light_returns_colors() {
        let colors = EditorTheme::Light.colors();
        assert!((colors.background[3] - 1.0).abs() < f32::EPSILON);
        assert!((colors.text_color[3] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_syntax_palette_nonempty() {
        assert!(!EditorTheme::Dark.syntax_palette().is_empty());
        assert!(!EditorTheme::Light.syntax_palette().is_empty());
    }
}
