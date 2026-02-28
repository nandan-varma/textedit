// ...existing code...
// Theme registry and selector for the editor UI
use crate::renderer::layout::Colors;

pub mod dracula;
pub mod solarized_dark;
pub mod one_dark;
pub mod gruvbox_dark;
pub mod light;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum EditorTheme {
    Dracula,
    SolarizedDark,
    OneDark,
    GruvboxDark,
    Light,
}

impl EditorTheme {
    pub fn colors(&self) -> Colors {
        match self {
            EditorTheme::Dracula => dracula::dracula(),
            EditorTheme::SolarizedDark => solarized_dark::solarized_dark(),
            EditorTheme::OneDark => one_dark::one_dark(),
            EditorTheme::GruvboxDark => gruvbox_dark::gruvbox_dark(),
            EditorTheme::Light => light::light(),
        }
    }
    pub fn syntax_palette(&self) -> &'static [([f32; 4], &'static str)] {
        match self {
            EditorTheme::Dracula => crate::themes::dracula::dracula_syntax_palette(),
            EditorTheme::SolarizedDark => crate::themes::solarized_dark::solarized_syntax_palette(),
            EditorTheme::OneDark => crate::themes::one_dark::one_dark_syntax_palette(),
            EditorTheme::GruvboxDark => crate::themes::gruvbox_dark::gruvbox_syntax_palette(),
            EditorTheme::Light => crate::themes::light::light_syntax_palette(),
        }
    }
}
