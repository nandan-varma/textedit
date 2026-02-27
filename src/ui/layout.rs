use crate::ui::components::LineNumbers;

pub struct UILayout {
    pub line_numbers_width: f32,
    pub status_bar_height: f32,
    pub editor_x: f32,
    pub editor_y: f32,
    pub editor_width: f32,
    pub editor_height: f32,
}

impl UILayout {
    pub fn new(window_width: f32, window_height: f32, line_count: usize) -> Self {
        let line_numbers_width = LineNumbers::width(line_count);
        let status_bar_height = 24.0;

        Self {
            line_numbers_width,
            status_bar_height,
            editor_x: line_numbers_width,
            editor_y: 0.0,
            editor_width: window_width - line_numbers_width,
            editor_height: window_height - status_bar_height,
        }
    }
}
