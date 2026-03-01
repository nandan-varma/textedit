/// Light theme syntax highlight palette
pub fn light_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.00, 0.33, 0.67, 1.0], "keyword"),
        ([0.80, 0.36, 0.13, 1.0], "string"),
        ([0.36, 0.44, 0.53, 1.0], "function"),
        ([0.20, 0.22, 0.25, 1.0], "type"),
        ([0.98, 0.60, 0.01, 1.0], "number"),
        ([0.60, 0.62, 0.65, 1.0], "comment"),
    ]
}
// Light theme for the editor UI
use crate::renderer::layout::Colors;

pub fn light() -> Colors {
    Colors {
        background: [0.98, 0.98, 0.96, 1.0],
        gutter_background: [0.95, 0.95, 0.93, 1.0],
        status_bar_background: [0.92, 0.92, 0.90, 1.0],
        text_color: [0.20, 0.22, 0.25, 1.0],
        line_number_color: [0.60, 0.62, 0.65, 1.0],
        cursor_color: [0.20, 0.22, 0.25, 1.0],
        selection_color: [0.85, 0.87, 0.90, 1.0],
        gutter_separator: [0.80, 0.80, 0.78, 1.0],
        scrollbar_track: [0.92, 0.92, 0.90, 1.0],
        scrollbar_thumb: [0.80, 0.80, 0.78, 1.0],
        // Modal colors
        modal_background: [0.95, 0.95, 0.93, 1.0],
        modal_border: [0.80, 0.80, 0.78, 1.0],
        input_background: [0.98, 0.98, 0.96, 1.0],
        input_border: [0.80, 0.80, 0.78, 1.0],
        input_border_focused: [0.00, 0.45, 0.70, 1.0],
        match_highlight: [1.0, 0.85, 0.30, 0.5],
        current_match_highlight: [1.0, 0.70, 0.20, 0.8],
        button_background: [0.92, 0.92, 0.90, 1.0],
        button_hover: [0.85, 0.85, 0.83, 1.0],
    }
}
