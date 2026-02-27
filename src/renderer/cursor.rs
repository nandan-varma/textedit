use super::layout::{Colors, EditorLayout, TEXT_AREA_PADDING_LEFT, TEXT_AREA_PADDING_TOP};
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

    /// Build geometry for rendering cursor at correct position
    pub fn build(cursor: &Cursor, buffer: &Buffer, layout: &EditorLayout) -> Self {
        let mut geometry = CursorGeometry::new();

        // Get line and column from cursor position
        let (line, col) = buffer.char_to_line_col(cursor.position());

        // Calculate pixel position
        let x = layout.text_area.x + TEXT_AREA_PADDING_LEFT + (col as f32 * layout.char_width);
        let y = TEXT_AREA_PADDING_TOP + (line as f32 * layout.line_height);

        // Cursor is a thin vertical bar
        let cursor_width = 2.0;
        let cursor_height = layout.line_height;

        // Convert to NDC
        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + cursor_width, y + cursor_height);

        let color = Colors::CURSOR_COLOR;

        // Top-left
        geometry.vertices.push(CursorVertex {
            position: [x1, y1],
            color,
        });

        // Top-right
        geometry.vertices.push(CursorVertex {
            position: [x2, y1],
            color,
        });

        // Bottom-right
        geometry.vertices.push(CursorVertex {
            position: [x2, y2],
            color,
        });

        // Bottom-left
        geometry.vertices.push(CursorVertex {
            position: [x1, y2],
            color,
        });

        // Two triangles
        geometry.indices.push(0);
        geometry.indices.push(1);
        geometry.indices.push(2);

        geometry.indices.push(0);
        geometry.indices.push(2);
        geometry.indices.push(3);

        geometry
    }
}
