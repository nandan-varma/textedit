use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
use crate::domain::Buffer;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub struct TextGeometry {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct WrappedLine {
    pub logical_line: usize,
    pub visual_line: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub x_offset: f32,
}

pub struct WrappedText {
    pub wrapped_lines: Vec<WrappedLine>,
    pub total_visual_lines: usize,
}

impl WrappedText {
    pub fn new() -> Self {
        Self {
            wrapped_lines: Vec::new(),
            total_visual_lines: 0,
        }
    }

    pub fn wrap_buffer(
        buffer: &Buffer,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
    ) -> Self {
        let mut wrapped_text = WrappedText::new();

        let text_area_width =
            layout.text_area.width - layout.text_area_padding_left - layout.text_area_padding_right;

        for (logical_line_idx, line) in buffer.rope().lines().enumerate() {
            if line.len_chars() == 0 {
                wrapped_text.wrapped_lines.push(WrappedLine {
                    logical_line: logical_line_idx,
                    visual_line: wrapped_text.total_visual_lines,
                    start_char: 0,
                    end_char: 0,
                    x_offset: 0.0,
                });
                wrapped_text.total_visual_lines += 1;
                continue;
            }

            let mut x_offset: f32;
            let _char_count = 0;
            let line_chars: Vec<char> = line.chars().collect();
            let total_chars = line_chars.len();
            let mut start_char = 0;

            while start_char < total_chars {
                let mut end_char = start_char;
                x_offset = 0.0;

                // Find how many characters fit on this visual line
                while end_char < total_chars {
                    let ch = line_chars[end_char];
                    let advance = glyph_atlas.char_advance_width(ch);

                    if x_offset + advance > text_area_width && end_char > start_char {
                        break;
                    }
                    x_offset += advance;
                    end_char += 1;
                }

                // Ensure at least one character per line
                if end_char == start_char {
                    end_char = (start_char + 1).min(total_chars);
                }

                wrapped_text.wrapped_lines.push(WrappedLine {
                    logical_line: logical_line_idx,
                    visual_line: wrapped_text.total_visual_lines,
                    start_char,
                    end_char,
                    x_offset: 0.0,
                });
                wrapped_text.total_visual_lines += 1;
                start_char = end_char;
            }
        }

        wrapped_text
    }

    /// Get visual line info from logical line and column
    pub fn get_visual_position(
        &self,
        logical_line: usize,
        col: usize,
        buffer: &Buffer,
    ) -> (usize, usize) {
        if logical_line >= buffer.len_lines() {
            return (self.total_visual_lines.saturating_sub(1), 0);
        }

        let mut current_char = 0;

        for wrapped in &self.wrapped_lines {
            if wrapped.logical_line == logical_line {
                let line_len = wrapped.end_char - wrapped.start_char;

                if current_char + line_len > col {
                    // The column falls within this visual line
                    let col_in_visual = col - current_char;
                    return (wrapped.visual_line, col_in_visual);
                }
                current_char += line_len;
            }
        }

        // Column is past end of line, return last visual line for this logical line
        if let Some(last) = self
            .wrapped_lines
            .iter()
            .rev()
            .find(|w| w.logical_line == logical_line)
        {
            let col_in_visual =
                (col.saturating_sub(last.start_char)).min(last.end_char - last.start_char);
            (last.visual_line, col_in_visual)
        } else {
            (0, 0)
        }
    }

    /// Get logical position from visual line and column
    #[allow(dead_code)]
    pub fn get_logical_position(
        &self,
        visual_line: usize,
        col: usize,
        buffer: &Buffer,
    ) -> (usize, usize) {
        if visual_line >= self.total_visual_lines {
            return (buffer.len_lines().saturating_sub(1), 0);
        }

        let wrapped = &self.wrapped_lines[visual_line];
        let logical_col = wrapped.start_char + col.min(wrapped.end_char - wrapped.start_char);

        (wrapped.logical_line, logical_col)
    }
}

impl Default for WrappedText {
    fn default() -> Self {
        Self::new()
    }
}

impl TextGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for rendering text buffer with word wrapping and vertical scrolling.
    ///
    /// `wrapped_text` contains layout information for all visual lines.
    /// `scroll_offset` is the index of the first visual line visible at the top of the viewport.
    pub fn build_from_buffer(
        buffer: &Buffer,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
        wrapped_text: &WrappedText,
        scroll_offset: usize,
        line_colors: Option<&std::collections::HashMap<usize, Vec<[f32; 4]>>>,
        colors: &super::layout::Colors,
    ) -> Result<Self, String> {
        let mut geometry = TextGeometry::new();

        let ascent = glyph_atlas.ascent();

        if wrapped_text.wrapped_lines.is_empty() {
            return Ok(geometry);
        }

        let visible_lines = layout.visible_lines().max(1);
        let first_visual = scroll_offset.min(wrapped_text.total_visual_lines.saturating_sub(1));
        let last_visual = (first_visual + visible_lines).min(wrapped_text.total_visual_lines);

        let mut cached_line_idx: Option<usize> = None;
        let mut cached_line_chars: Vec<char> = Vec::new();

        for wrapped in &wrapped_text.wrapped_lines {
            if wrapped.visual_line < first_visual || wrapped.visual_line >= last_visual {
                continue;
            }

            if wrapped.logical_line >= buffer.len_lines() {
                continue;
            }

            if cached_line_idx != Some(wrapped.logical_line) {
                if let Some(line) = buffer.line_slice(wrapped.logical_line) {
                    cached_line_chars = line.chars().collect();
                    cached_line_idx = Some(wrapped.logical_line);
                } else {
                    continue;
                }
            }
            let line_chars = &cached_line_chars;

            let base_x = layout.text_area.x + layout.text_area_padding_left;
            let screen_line = wrapped.visual_line.saturating_sub(first_visual);
            let baseline_y =
                layout.text_area_padding_top + (screen_line as f32 * layout.line_height) + ascent;

            let mut x_offset = 0.0;

            let colors_for_line = line_colors.and_then(|m| m.get(&wrapped.logical_line));

            for (i_in_line, ch) in line_chars
                .iter()
                .skip(wrapped.start_char)
                .take(wrapped.end_char - wrapped.start_char)
                .enumerate()
            {
                if *ch == '\n' || *ch == '\r' {
                    break;
                }

                let entry = match glyph_atlas.get_or_rasterize(*ch) {
                    Ok(e) => e.clone(),
                    Err(_) => {
                        x_offset += layout.char_width;
                        continue;
                    }
                };

                if entry.width == 0 || entry.height == 0 {
                    x_offset += entry.metrics.advance_width;
                    continue;
                }

                let glyph_x = base_x + x_offset + entry.metrics.xmin as f32;
                let glyph_y = baseline_y - entry.metrics.ymin as f32 - entry.height as f32;

                let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
                let [x2, y2] = layout
                    .pixel_to_ndc(glyph_x + entry.width as f32, glyph_y + entry.height as f32);

                let vertex_start = geometry.vertices.len() as u32;
                let char_idx_in_line = wrapped.start_char + i_in_line;
                let color = colors_for_line
                    .and_then(|v| v.get(char_idx_in_line).copied())
                    .unwrap_or(colors.text_color);

                geometry.vertices.push(TextVertex {
                    position: [x1, y1],
                    uv: [entry.uv_min_x, entry.uv_min_y],
                    color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x2, y1],
                    uv: [entry.uv_max_x, entry.uv_min_y],
                    color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x2, y2],
                    uv: [entry.uv_max_x, entry.uv_max_y],
                    color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x1, y2],
                    uv: [entry.uv_min_x, entry.uv_max_y],
                    color,
                });

                geometry.indices.push(vertex_start);
                geometry.indices.push(vertex_start + 1);
                geometry.indices.push(vertex_start + 2);

                geometry.indices.push(vertex_start);
                geometry.indices.push(vertex_start + 2);
                geometry.indices.push(vertex_start + 3);

                x_offset += entry.metrics.advance_width;
            }
        }

        Ok(geometry)
    }
}
