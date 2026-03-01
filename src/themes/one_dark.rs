/// One Dark syntax highlight palette
pub fn one_dark_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.80, 0.36, 0.98, 1.0], "keyword"),
        ([0.98, 0.80, 0.36, 1.0], "string"),
        ([0.56, 0.67, 1.00, 1.0], "function"),
        ([0.36, 0.98, 0.80, 1.0], "type"),
        ([0.98, 0.60, 0.01, 1.0], "number"),
        ([0.36, 0.44, 0.53, 1.0], "comment"),
    ]
}
// One Dark theme for the editor UI
use crate::renderer::layout::Colors;

pub fn one_dark() -> Colors {
    Colors {
        background: [0.16, 0.18, 0.23, 1.0],
        gutter_background: [0.18, 0.20, 0.25, 1.0],
        status_bar_background: [0.20, 0.22, 0.27, 1.0],
        text_color: [0.73, 0.80, 0.87, 1.0],
        line_number_color: [0.36, 0.42, 0.49, 1.0],
        cursor_color: [0.98, 0.38, 0.34, 1.0],
        selection_color: [0.25, 0.29, 0.38, 1.0],
        gutter_separator: [0.22, 0.24, 0.29, 1.0],
        scrollbar_track: [0.18, 0.20, 0.25, 1.0],
        scrollbar_thumb: [0.28, 0.30, 0.35, 1.0],
        // Modal colors
        modal_background: [0.20, 0.22, 0.27, 1.0],
        modal_border: [0.28, 0.30, 0.35, 1.0],
        input_background: [0.16, 0.18, 0.23, 1.0],
        input_border: [0.28, 0.30, 0.35, 1.0],
        input_border_focused: [0.38, 0.54, 0.78, 1.0],
        match_highlight: [0.55, 0.40, 0.00, 0.4],
        current_match_highlight: [0.70, 0.50, 0.00, 0.7],
        button_background: [0.22, 0.24, 0.29, 1.0],
        button_hover: [0.28, 0.30, 0.35, 1.0],
    }
}
