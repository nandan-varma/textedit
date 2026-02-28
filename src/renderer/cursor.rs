use super::glyph_cache::GlyphAtlas;
use super::layout::{Colors, EditorLayout};
use crate::editor::{Buffer, Cursor};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CursorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

pub struct CursorGeometry {
    pub vertices: Vec<CursorVertex>,
    pub indices: Vec<u32>,
}

impl CursorGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for rendering cursor with selection highlighting
    pub fn build_with_wrap(
        cursor: &Cursor,
        buffer: &Buffer,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
    ) -> Self {
        let mut geometry = CursorGeometry::new();

        // First, render selection background if there's a selection
        if let Some(sel) = cursor.selection() {
            if !sel.is_empty() {
                geometry = Self::render_selection(cursor, buffer, layout, glyph_atlas, geometry);
            }
        }

        // Then render cursor at current position
        let cursor_pos = cursor.position();
        let (logical_line, col) = buffer.char_to_line_col(cursor_pos);

        let wrapped_text =
            super::text_geometry::WrappedText::wrap_buffer(buffer, glyph_atlas, layout);
        let (visual_line, visual_col) = wrapped_text.get_visual_position(logical_line, col, buffer);

        let base_x = layout.text_area.x + layout.text_area_padding_left;

        let lines = buffer.lines();
        if logical_line < lines.len() {
            let line = &lines[logical_line];
            let line_chars: Vec<char> = line.chars().collect();

            let mut x_pos = 0.0;
            let start_char = wrapped_text
                .wrapped_lines
                .iter()
                .find(|w| w.logical_line == logical_line && w.visual_line == visual_line)
                .map(|w| w.start_char)
                .unwrap_or(0);

            for ch in line_chars.iter().skip(start_char).take(visual_col) {
                x_pos += glyph_atlas.char_advance_width(*ch);
            }

            let x = base_x + x_pos;
            let y = layout.text_area_padding_top + (visual_line as f32 * layout.line_height);

            let cursor_width = 2.0;
            let cursor_height = layout.line_height;

            let [x1, y1] = layout.pixel_to_ndc(x, y);
            let [x2, y2] = layout.pixel_to_ndc(x + cursor_width, y + cursor_height);

            let color = Colors::CURSOR_COLOR;

            let base_vertex = geometry.vertices.len() as u32;

            geometry.vertices.push(CursorVertex {
                position: [x1, y1],
                color,
            });

            geometry.vertices.push(CursorVertex {
                position: [x2, y1],
                color,
            });

            geometry.vertices.push(CursorVertex {
                position: [x2, y2],
                color,
            });

            geometry.vertices.push(CursorVertex {
                position: [x1, y2],
                color,
            });

            geometry.indices.push(base_vertex);
            geometry.indices.push(base_vertex + 1);
            geometry.indices.push(base_vertex + 2);

            geometry.indices.push(base_vertex);
            geometry.indices.push(base_vertex + 2);
            geometry.indices.push(base_vertex + 3);
        }

        geometry
    }

    fn render_selection(
        cursor: &Cursor,
        buffer: &Buffer,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        mut geometry: Self,
    ) -> Self {
        let sel = cursor.selection().unwrap();
        let start = sel.start.min(sel.end);
        let end = sel.start.max(sel.end);

        let (start_line, start_col) = buffer.char_to_line_col(start);
        let (end_line, end_col) = buffer.char_to_line_col(end);

        let wrapped_text =
            super::text_geometry::WrappedText::wrap_buffer(buffer, glyph_atlas, layout);

        let base_x = layout.text_area.x + layout.text_area_padding_left;
        let selection_color = Colors::SELECTION_COLOR;

        // Find all visual lines that intersect with the selection
        for wrapped in &wrapped_text.wrapped_lines {
            // Check if this visual line intersects with selection
            let line_start_char = wrapped.start_char;
            let line_end_char = wrapped.end_char;

            // Get logical line for this visual line
            let lines = buffer.lines();
            if wrapped.logical_line >= lines.len() {
                continue;
            }

            let line = &lines[wrapped.logical_line];
            let line_chars: Vec<char> = line.chars().collect();

            // Determine selection start and end within this visual line
            let sel_start_in_line = if wrapped.logical_line == start_line {
                start_col.saturating_sub(wrapped.start_char)
            } else {
                0
            };

            let sel_end_in_line = if wrapped.logical_line == end_line {
                (end_col).min(wrapped.end_char - wrapped.start_char)
            } else {
                wrapped.end_char - wrapped.start_char
            };

            if sel_start_in_line >= sel_end_in_line {
                continue;
            }

            // Calculate x positions
            let mut start_x = 0.0;
            let mut end_x = 0.0;
            let mut char_idx = 0;

            for ch in line_chars.iter().skip(wrapped.start_char) {
                let advance = glyph_atlas.char_advance_width(*ch);
                if char_idx == sel_start_in_line {
                    start_x = end_x;
                }
                if char_idx < sel_end_in_line {
                    end_x += advance;
                }
                char_idx += 1;
            }

            let x = base_x + start_x;
            let y =
                layout.text_area_padding_top + (wrapped.visual_line as f32 * layout.line_height);
            let width = end_x - start_x;
            let height = layout.line_height;

            if width > 0.0 {
                let [x1, y1] = layout.pixel_to_ndc(x, y);
                let [x2, y2] = layout.pixel_to_ndc(x + width, y + height);

                let base_vertex = geometry.vertices.len() as u32;

                geometry.vertices.push(CursorVertex {
                    position: [x1, y1],
                    color: selection_color,
                });

                geometry.vertices.push(CursorVertex {
                    position: [x2, y1],
                    color: selection_color,
                });

                geometry.vertices.push(CursorVertex {
                    position: [x2, y2],
                    color: selection_color,
                });

                geometry.vertices.push(CursorVertex {
                    position: [x1, y2],
                    color: selection_color,
                });

                geometry.indices.push(base_vertex);
                geometry.indices.push(base_vertex + 1);
                geometry.indices.push(base_vertex + 2);

                geometry.indices.push(base_vertex);
                geometry.indices.push(base_vertex + 2);
                geometry.indices.push(base_vertex + 3);
            }
        }

        geometry
    }

    /// Build geometry for rendering cursor (legacy, without wrapping)
    pub fn build(cursor: &Cursor, buffer: &Buffer, layout: &EditorLayout) -> Self {
        let mut geometry = CursorGeometry::new();

        let (line, col) = buffer.char_to_line_col(cursor.position());

        let x =
            layout.text_area.x + layout.text_area_padding_left + (col as f32 * layout.char_width);
        let y = layout.text_area_padding_top + (line as f32 * layout.line_height);

        let cursor_width = 2.0;
        let cursor_height = layout.line_height;

        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + cursor_width, y + cursor_height);

        let color = Colors::CURSOR_COLOR;

        geometry.vertices.push(CursorVertex {
            position: [x1, y1],
            color,
        });

        geometry.vertices.push(CursorVertex {
            position: [x2, y1],
            color,
        });

        geometry.vertices.push(CursorVertex {
            position: [x2, y2],
            color,
        });

        geometry.vertices.push(CursorVertex {
            position: [x1, y2],
            color,
        });

        geometry.indices.push(0);
        geometry.indices.push(1);
        geometry.indices.push(2);

        geometry.indices.push(0);
        geometry.indices.push(2);
        geometry.indices.push(3);

        geometry
    }
}
