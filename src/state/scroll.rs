use super::init::State;
use crate::domain::{Buffer, Cursor};

impl State {
    pub fn scroll_by_lines(
        &mut self,
        delta_lines: i32,
        buffer: &Buffer,
        show_line_numbers: bool,
        show_status_bar: bool,
    ) {
        let size = self.window.inner_size();
        let layout = crate::renderer::layout::EditorLayout::new(
            size.width as f32,
            size.height as f32,
            self.scaled_font_size,
            self.scale_factor,
            show_line_numbers,
            show_status_bar,
        );

        if let Some(glyph_atlas) = &mut self.glyph_atlas {
            let wrapped_text = crate::renderer::text_geometry::WrappedText::wrap_buffer(
                buffer,
                glyph_atlas,
                &layout,
            );
            self.total_visual_lines = wrapped_text.total_visual_lines;

            let visible = layout.visible_lines().max(1);
            if self.total_visual_lines <= visible {
                self.scroll_visual_offset = 0;
                return;
            }

            let max_offset = self.total_visual_lines.saturating_sub(visible) as i32;
            let current = self.scroll_visual_offset as i32;
            let mut next = current + delta_lines;
            if next < 0 {
                next = 0;
            } else if next > max_offset {
                next = max_offset;
            }
            self.scroll_visual_offset = next as usize;
        }
    }
    pub fn ensure_cursor_visible(
        &mut self,
        cursor: &Cursor,
        buffer: &Buffer,
        show_line_numbers: bool,
        show_status_bar: bool,
    ) {
        let size = self.window.inner_size();
        let layout = crate::renderer::layout::EditorLayout::new(
            size.width as f32,
            size.height as f32,
            self.scaled_font_size,
            self.scale_factor,
            show_line_numbers,
            show_status_bar,
        );

        if let Some(glyph_atlas) = &mut self.glyph_atlas {
            let wrapped_text = crate::renderer::text_geometry::WrappedText::wrap_buffer(
                buffer,
                glyph_atlas,
                &layout,
            );
            self.total_visual_lines = wrapped_text.total_visual_lines;

            let visible = layout.visible_lines().max(1);
            if self.total_visual_lines == 0 || visible == 0 {
                self.scroll_visual_offset = 0;
                return;
            }

            let (logical_line, col) = buffer.char_to_line_col(cursor.position());
            let (visual_line, _) = wrapped_text.get_visual_position(logical_line, col, buffer);

            let max_offset = self.total_visual_lines.saturating_sub(visible);
            let mut offset = self.scroll_visual_offset.min(max_offset);

            if visual_line < offset {
                offset = visual_line;
            } else if visual_line >= offset + visible {
                offset = visual_line.saturating_sub(visible.saturating_sub(1));
            }

            self.scroll_visual_offset = offset.min(max_offset);
        }
    }
}
