/// Solarized Dark syntax highlight palette
pub fn solarized_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.42, 0.44, 0.57, 1.0], "keyword"),
        ([0.52, 0.60, 0.36, 1.0], "string"),
        ([0.38, 0.54, 0.68, 1.0], "function"),
        ([0.86, 0.20, 0.18, 1.0], "type"),
        ([0.99, 0.52, 0.00, 1.0], "number"),
        ([0.44, 0.50, 0.56, 1.0], "comment"),
    ]
}
// Solarized Dark theme for the editor UI
use crate::renderer::layout::Colors;

pub fn solarized_dark() -> Colors {
    Colors {
        background: [0.00, 0.17, 0.21, 1.0],
        gutter_background: [0.02, 0.20, 0.24, 1.0],
        status_bar_background: [0.03, 0.22, 0.26, 1.0],
        text_color: [0.51, 0.58, 0.59, 1.0],
        line_number_color: [0.24, 0.36, 0.37, 1.0],
        cursor_color: [0.99, 0.52, 0.00, 1.0],
        selection_color: [0.07, 0.21, 0.25, 1.0],
        gutter_separator: [0.10, 0.22, 0.25, 1.0],
        scrollbar_track: [0.03, 0.22, 0.26, 1.0],
        scrollbar_thumb: [0.13, 0.28, 0.32, 1.0],
        // Modal colors
        modal_background: [0.03, 0.22, 0.26, 1.0],
        modal_border: [0.13, 0.28, 0.32, 1.0],
        input_background: [0.00, 0.17, 0.21, 1.0],
        input_border: [0.13, 0.28, 0.32, 1.0],
        input_border_focused: [0.38, 0.54, 0.68, 1.0],
        match_highlight: [0.50, 0.35, 0.00, 0.4],
        current_match_highlight: [0.65, 0.45, 0.00, 0.7],
        button_background: [0.05, 0.25, 0.30, 1.0],
        button_hover: [0.10, 0.30, 0.35, 1.0],
    }
}
