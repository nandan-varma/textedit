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

        // Find the wrapped line that contains our cursor position
        let wrapped_opt = wrapped_text
            .wrapped_lines
            .iter()
            .find(|w| w.logical_line == logical_line && w.visual_line == visual_line);

        if let Some(wrapped) = wrapped_opt {
            if logical_line < lines.len() {
                let line = &lines[logical_line];
                let line_chars: Vec<char> = line.chars().collect();

                // Calculate x position - only count characters from start of this visual line
                let mut x_pos = 0.0;
                let chars_in_visual = (wrapped.end_char - wrapped.start_char).min(visual_col);

                for i in 0..chars_in_visual {
                    let char_idx = wrapped.start_char + i;
                    if char_idx >= line_chars.len() {
                        break;
                    }
                    let ch = line_chars[char_idx];
                    x_pos += glyph_atlas.char_advance_width(ch);
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

        // Get character positions
        let (start_line, start_col) = buffer.char_to_line_col(start);
        let (end_line, end_col) = buffer.char_to_line_col(end);

        let wrapped_text =
            super::text_geometry::WrappedText::wrap_buffer(buffer, glyph_atlas, layout);

        let base_x = layout.text_area.x + layout.text_area_padding_left;
        let selection_color = Colors::SELECTION_COLOR;

        // Find all visual lines that intersect with the selection
        for wrapped in &wrapped_text.wrapped_lines {
            let wrapped_logical = wrapped.logical_line;
            let wrapped_start = wrapped.start_char;
            let wrapped_end = wrapped.end_char;
            let visual_line = wrapped.visual_line;

            // Get line content
            let lines = buffer.lines();
            if wrapped_logical >= lines.len() {
                continue;
            }

            let line = &lines[wrapped_logical];
            let line_chars: Vec<char> = line.chars().collect();
            let line_len = line_chars.len();

            // Calculate which characters of this visual line are in selection
            let (sel_start_in_line, sel_end_in_line) =
                if wrapped_logical == start_line && wrapped_logical == end_line {
                    // Selection within single line
                    (start_col, end_col)
                } else if wrapped_logical == start_line {
                    // First line of multi-line selection: from start_col to end of line
                    (start_col, line_len)
                } else if wrapped_logical == end_line {
                    // Last line of multi-line selection: from beginning to end_col
                    (0, end_col)
                } else if wrapped_logical > start_line && wrapped_logical < end_line {
                    // Middle lines: entire line
                    (0, line_len)
                } else {
                    // This visual line is not in selection
                    continue;
                };

            // Adjust for wrapped line start
            let sel_start = sel_start_in_line.saturating_sub(wrapped_start);
            let sel_end = sel_end_in_line.saturating_sub(wrapped_start);

            // Skip if no selection in this visual line
            if sel_start >= sel_end && sel_end > 0 {
                continue;
            }
            if sel_end <= 0 {
                continue;
            }

            // Calculate x positions
            let mut x_offset = 0.0;
            let mut sel_start_x = 0.0;
            let mut sel_end_x = 0.0;

            // Only iterate through characters in this visual line
            let chars_in_visual = wrapped_end.saturating_sub(wrapped_start);
            for i in 0..chars_in_visual {
                let char_global_idx = wrapped_start + i;
                if char_global_idx >= line_len {
                    break;
                }

                let ch = line_chars[char_global_idx];
                let advance = glyph_atlas.char_advance_width(ch);

                if i == sel_start {
                    sel_start_x = x_offset;
                }
                if i < sel_end {
                    sel_end_x = x_offset + advance;
                }

                x_offset += advance;
            }

            let width = sel_end_x - sel_start_x;
            if width <= 0.0 {
                continue;
            }

            let x = base_x + sel_start_x;
            let y = layout.text_area_padding_top + (visual_line as f32 * layout.line_height);
            let height = layout.line_height;

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
