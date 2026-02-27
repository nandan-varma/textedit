use crate::config::EditorConfig;
use crate::editor::Buffer;

pub struct TextRenderer {
    config: EditorConfig,
}

impl TextRenderer {
    pub fn new(config: EditorConfig) -> Self {
        Self { config }
    }

    pub fn render_text(&self, buffer: &Buffer, line_idx: usize) -> Option<String> {
        buffer.line(line_idx)
    }

    pub fn line_height(&self) -> f32 {
        self.config.font.size * 1.2 // 20% line spacing
    }

    pub fn char_width(&self) -> f32 {
        self.config.font.size * 0.6 // Monospace approximation
    }

    pub fn column_width(&self, col: usize) -> f32 {
        let mut width = 0.0;
        for _ in 0..col {
            width += self.char_width();
        }
        width
    }
}
