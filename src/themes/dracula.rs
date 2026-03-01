/// Dracula syntax highlight palette (for keywords, strings, etc.)
pub fn dracula_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.80, 0.25, 0.33, 1.0], "keyword"),
        ([0.98, 0.80, 0.36, 1.0], "string"),
        ([0.56, 0.67, 1.00, 1.0], "function"),
        ([0.80, 0.36, 0.98, 1.0], "type"),
        ([0.36, 0.98, 0.80, 1.0], "number"),
        ([0.98, 0.36, 0.36, 1.0], "comment"),
    ]
}
// Dracula theme for the editor UI
use crate::renderer::layout::Colors;

pub fn dracula() -> Colors {
    Colors {
        background: [0.07, 0.08, 0.13, 1.0],
        gutter_background: [0.10, 0.11, 0.16, 1.0],
        status_bar_background: [0.12, 0.13, 0.18, 1.0],
        text_color: [0.83, 0.85, 0.89, 1.0],
        line_number_color: [0.40, 0.44, 0.53, 1.0],
        cursor_color: [0.98, 0.36, 0.36, 1.0],
        selection_color: [0.25, 0.29, 0.38, 1.0],
        gutter_separator: [0.20, 0.22, 0.28, 1.0],
        scrollbar_track: [0.13, 0.14, 0.19, 1.0],
        scrollbar_thumb: [0.25, 0.27, 0.33, 1.0],
        // Modal colors
        modal_background: [0.12, 0.13, 0.18, 1.0],
        modal_border: [0.25, 0.27, 0.33, 1.0],
        input_background: [0.07, 0.08, 0.13, 1.0],
        input_border: [0.25, 0.27, 0.33, 1.0],
        input_border_focused: [0.55, 0.35, 0.90, 1.0],
        match_highlight: [0.55, 0.35, 0.00, 0.4],
        current_match_highlight: [0.70, 0.45, 0.00, 0.7],
        button_background: [0.18, 0.19, 0.24, 1.0],
        button_hover: [0.25, 0.27, 0.33, 1.0],
    }
}
