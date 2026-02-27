use egui::{Color32, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: Color32,
    pub foreground: Color32,
    pub selection: Color32,
    pub current_line: Color32,
    pub line_number: Color32,
    pub line_number_background: Color32,
    pub cursor: Color32,
    pub keyword: Color32,
    pub string: Color32,
    pub number: Color32,
    pub comment: Color32,
    pub function: Color32,
    pub variable: Color32,
    pub type_name: Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: Color32::from_rgb(30, 30, 30),
            foreground: Color32::from_rgb(220, 220, 220),
            selection: Color32::from_rgb(100, 100, 120),
            current_line: Color32::from_rgb(40, 40, 40),
            line_number: Color32::from_rgb(120, 120, 120),
            line_number_background: Color32::from_rgb(25, 25, 25),
            cursor: Color32::WHITE,
            keyword: Color32::from_rgb(200, 100, 150),
            string: Color32::from_rgb(150, 200, 150),
            number: Color32::from_rgb(200, 180, 100),
            comment: Color32::from_rgb(100, 150, 100),
            function: Color32::from_rgb(100, 180, 250),
            variable: Color32::from_rgb(200, 200, 200),
            type_name: Color32::from_rgb(250, 180, 120),
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: Color32::from_rgb(250, 250, 250),
            foreground: Color32::from_rgb(30, 30, 30),
            selection: Color32::from_rgb(180, 180, 200),
            current_line: Color32::from_rgb(245, 245, 245),
            line_number: Color32::from_rgb(150, 150, 150),
            line_number_background: Color32::from_rgb(240, 240, 240),
            cursor: Color32::BLACK,
            keyword: Color32::from_rgb(180, 50, 100),
            string: Color32::from_rgb(80, 140, 80),
            number: Color32::from_rgb(140, 120, 50),
            comment: Color32::from_rgb(80, 120, 80),
            function: Color32::from_rgb(50, 100, 180),
            variable: Color32::from_rgb(30, 30, 30),
            type_name: Color32::from_rgb(180, 120, 60),
        }
    }

    pub fn apply_to_egui(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.visuals = match self.name.as_str() {
            "Light" => Visuals::light(),
            _ => Visuals::dark(),
        };

        style.visuals.widgets.inactive.bg_fill = self.background;
        style.visuals.widgets.active.bg_fill = self.selection;
        style.visuals.widgets.hovered.bg_fill = self.current_line;

        ctx.set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default_is_dark() {
        let theme = Theme::default();
        assert_eq!(theme.name, "Dark");
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.name, "Light");
    }
}
