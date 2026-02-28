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

    /// Render a line with per-character colors (for syntax highlighting)
    pub fn render_text_colored(&self, buffer: &Buffer, line_idx: usize, colors: Option<&[[f32; 4]]>) -> Option<Vec<(char, [f32; 4])>> {
        let line = buffer.line(line_idx)?;
        let chars: Vec<char> = line.chars().collect();
        let mut out = Vec::with_capacity(chars.len());
        if let Some(colors) = colors {
            for (i, ch) in chars.iter().enumerate() {
                let color = colors.get(i).copied().unwrap_or(self.config.theme.foreground);
                out.push((*ch, color));
            }
        } else {
            for ch in chars {
                out.push((ch, self.config.theme.foreground));
            }
        }
        Some(out)
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
