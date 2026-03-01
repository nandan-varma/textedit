use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
use super::text_geometry::WrappedText;
use crate::domain::{Buffer, Cursor};

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

    /// Build geometry for rendering cursor with selection highlighting and scrolling.
    /// Accepts pre-computed `WrappedText` to avoid redundant wrap_buffer calls.
    pub fn build_with_wrap(
        cursor: &Cursor,
        buffer: &Buffer,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        scroll_offset: usize,
        colors: &super::layout::Colors,
        wrapped_text: &WrappedText,
    ) -> Self {
        let mut geometry = CursorGeometry::new();

        // First, render selection background if there's a selection
        if let Some(sel) = cursor.selection() {
            if !sel.is_empty() {
                geometry = Self::render_selection(
                    cursor,
                    buffer,
                    layout,
                    glyph_atlas,
                    scroll_offset,
                    geometry,
                    colors,
                    wrapped_text,
                );
            }
        }

        // Then render cursor at current position
        let cursor_pos = cursor.position();
        let (logical_line, col) = buffer.char_to_line_col(cursor_pos);

        let (visual_line, visual_col) = wrapped_text.get_visual_position(logical_line, col, buffer);

        let base_x = layout.text_area.x + layout.text_area_padding_left;

        // Find the wrapped line that contains our cursor position
        let wrapped_opt = wrapped_text
            .wrapped_lines
            .iter()
            .find(|w| w.logical_line == logical_line && w.visual_line == visual_line);

        if let Some(wrapped) = wrapped_opt {
            if logical_line < buffer.len_lines() {
                let line_chars: Vec<char> = buffer
                    .line_slice(logical_line)
                    .map(|l| l.chars().collect())
                    .unwrap_or_default();

                // determine visible length (exclude trailing newline)
                let vis_len = if let Some(&last) = line_chars.last() {
                    if last == '\n' || last == '\r' {
                        line_chars.len().saturating_sub(1)
                    } else {
                        line_chars.len()
                    }
                } else {
                    0
                };

                // Calculate x position - only count characters from start of this visual line
                let vis_col = visual_col.min(vis_len);
                let mut x_pos = 0.0;
                let chars_in_visual = (wrapped.end_char - wrapped.start_char).min(vis_col);

                for i in 0..chars_in_visual {
                    let char_idx = wrapped.start_char + i;
                    if char_idx >= vis_len {
                        break;
                    }
                    let ch = line_chars[char_idx];
                    x_pos += glyph_atlas.char_advance_width(ch);
                }

                let screen_line = visual_line.saturating_sub(
                    scroll_offset.min(wrapped_text.total_visual_lines.saturating_sub(1)),
                );
                let x = base_x + x_pos;
                let y = layout.text_area_padding_top + (screen_line as f32 * layout.line_height);

                let cursor_width = 2.0;
                let cursor_height = layout.line_height;

                let [x1, y1] = layout.pixel_to_ndc(x, y);
                let [x2, y2] = layout.pixel_to_ndc(x + cursor_width, y + cursor_height);

                let color = colors.cursor_color;

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
        scroll_offset: usize,
        mut geometry: Self,
        colors: &super::layout::Colors,
        wrapped_text: &WrappedText,
    ) -> Self {
        let sel = cursor.selection().unwrap();
        let start = sel.start.min(sel.end);
        let end = sel.start.max(sel.end);

        // Get character positions
        let (start_line, start_col) = buffer.char_to_line_col(start);
        let (end_line, end_col) = buffer.char_to_line_col(end);

        let base_x = layout.text_area.x + layout.text_area_padding_left;
        let selection_color = colors.selection_color;

        // Find all visual lines that intersect with the selection
        for wrapped in &wrapped_text.wrapped_lines {
            let wrapped_logical = wrapped.logical_line;
            let wrapped_start = wrapped.start_char;
            let wrapped_end = wrapped.end_char;
            let visual_line = wrapped.visual_line;

            let first_visual = scroll_offset.min(wrapped_text.total_visual_lines.saturating_sub(1));
            if visual_line < first_visual {
                continue;
            }
            let screen_line = visual_line.saturating_sub(first_visual);

            // Get line content
            if wrapped_logical >= buffer.len_lines() {
                continue;
            }

            let line_chars: Vec<char> = buffer
                .line_slice(wrapped_logical)
                .map(|l| l.chars().collect())
                .unwrap_or_default();
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
            if sel_end == 0 {
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
            let y = layout.text_area_padding_top + (screen_line as f32 * layout.line_height);
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

    /// Build geometry for rendering match highlights
    /// This is used to highlight all search matches in the document.
    /// Accepts pre-computed `WrappedText` to avoid redundant wrap_buffer calls.
    pub fn build_match_highlights(
        buffer: &Buffer,
        matches: &[(usize, usize)],
        current_match: Option<usize>,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        scroll_offset: usize,
        colors: &super::layout::Colors,
        wrapped_text: &WrappedText,
    ) -> Self {
        let mut geometry = CursorGeometry::new();

        if matches.is_empty() {
            return geometry;
        }

        let base_x = layout.text_area.x + layout.text_area_padding_left;
        let first_visual = scroll_offset.min(wrapped_text.total_visual_lines.saturating_sub(1));

        for (match_idx, (start, end)) in matches.iter().enumerate() {
            let is_current = current_match == Some(match_idx);
            let highlight_color = if is_current {
                colors.current_match_highlight
            } else {
                colors.match_highlight
            };

            // Get line positions
            let (start_line, start_col) = buffer.char_to_line_col(*start);
            let (end_line, end_col) = buffer.char_to_line_col(*end);

            // For each wrapped line, check if it contains part of this match
            for wrapped in &wrapped_text.wrapped_lines {
                let wrapped_logical = wrapped.logical_line;
                let wrapped_start = wrapped.start_char;
                let wrapped_end = wrapped.end_char;
                let visual_line = wrapped.visual_line;

                // Skip if not visible
                if visual_line < first_visual {
                    continue;
                }
                let screen_line = visual_line.saturating_sub(first_visual);

                // Skip if line is not in the match range
                if wrapped_logical < start_line || wrapped_logical > end_line {
                    continue;
                }

                // Get line content
                if wrapped_logical >= buffer.len_lines() {
                    continue;
                }

                let line_chars: Vec<char> = buffer
                    .line_slice(wrapped_logical)
                    .map(|l| l.chars().collect())
                    .unwrap_or_default();
                let line_len = line_chars.len();

                // Calculate which characters of this visual line are in the match
                let (match_start_in_line, match_end_in_line) =
                    if wrapped_logical == start_line && wrapped_logical == end_line {
                        (start_col, end_col)
                    } else if wrapped_logical == start_line {
                        (start_col, line_len)
                    } else if wrapped_logical == end_line {
                        (0, end_col)
                    } else {
                        (0, line_len)
                    };

                // Check overlap with this wrapped segment
                let seg_start = wrapped_start;
                let seg_end = wrapped_end;

                let overlap_start = match_start_in_line.max(seg_start);
                let overlap_end = match_end_in_line.min(seg_end);

                if overlap_start >= overlap_end {
                    continue;
                }

                // Calculate x positions relative to wrapped line start
                let mut x_offset = 0.0;
                let mut sel_start_x = 0.0;
                let mut sel_end_x = 0.0;

                for i in seg_start..seg_end {
                    if i >= line_len {
                        break;
                    }

                    let ch = line_chars[i];
                    let advance = glyph_atlas.char_advance_width(ch);

                    if i == overlap_start {
                        sel_start_x = x_offset;
                    }
                    if i < overlap_end {
                        sel_end_x = x_offset + advance;
                    }

                    x_offset += advance;
                }

                let width = sel_end_x - sel_start_x;
                if width <= 0.0 {
                    continue;
                }

                let x = base_x + sel_start_x;
                let y = layout.text_area_padding_top + (screen_line as f32 * layout.line_height);
                let height = layout.line_height;

                let [x1, y1] = layout.pixel_to_ndc(x, y);
                let [x2, y2] = layout.pixel_to_ndc(x + width, y + height);

                let base_vertex = geometry.vertices.len() as u32;

                geometry.vertices.push(CursorVertex {
                    position: [x1, y1],
                    color: highlight_color,
                });
                geometry.vertices.push(CursorVertex {
                    position: [x2, y1],
                    color: highlight_color,
                });
                geometry.vertices.push(CursorVertex {
                    position: [x2, y2],
                    color: highlight_color,
                });
                geometry.vertices.push(CursorVertex {
                    position: [x1, y2],
                    color: highlight_color,
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
    #[allow(dead_code)]
    pub fn build(
        cursor: &Cursor,
        buffer: &Buffer,
        layout: &EditorLayout,
        colors: &super::layout::Colors,
    ) -> Self {
        let mut geometry = CursorGeometry::new();

        let (line, col) = buffer.char_to_line_col(cursor.position());

        let x =
            layout.text_area.x + layout.text_area_padding_left + (col as f32 * layout.char_width);
        let y = layout.text_area_padding_top + (line as f32 * layout.line_height);

        let cursor_width = 2.0;
        let cursor_height = layout.line_height;

        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + cursor_width, y + cursor_height);

        let color = colors.cursor_color;

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
