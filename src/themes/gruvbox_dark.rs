/// Gruvbox syntax highlight palette
pub fn gruvbox_syntax_palette() -> &'static [([f32; 4], &'static str)] {
    &[
        ([0.80, 0.36, 0.13, 1.0], "keyword"),
        ([0.98, 0.80, 0.36, 1.0], "string"),
        ([0.56, 0.67, 1.00, 1.0], "function"),
        ([0.98, 0.36, 0.36, 1.0], "type"),
        ([0.98, 0.60, 0.01, 1.0], "number"),
        ([0.45, 0.40, 0.32, 1.0], "comment"),
    ]
}
// Gruvbox Dark theme for the editor UI
use crate::renderer::layout::Colors;

pub fn gruvbox_dark() -> Colors {
    Colors {
        background: [0.15, 0.13, 0.11, 1.0],
        gutter_background: [0.18, 0.16, 0.13, 1.0],
        status_bar_background: [0.20, 0.18, 0.15, 1.0],
        text_color: [0.87, 0.80, 0.68, 1.0],
        line_number_color: [0.45, 0.40, 0.32, 1.0],
        cursor_color: [0.98, 0.60, 0.01, 1.0],
        selection_color: [0.25, 0.21, 0.17, 1.0],
        gutter_separator: [0.22, 0.20, 0.17, 1.0],
        scrollbar_track: [0.18, 0.16, 0.13, 1.0],
        scrollbar_thumb: [0.28, 0.25, 0.20, 1.0],
    }
}
