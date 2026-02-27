use crate::editor::Cursor;

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

    /// Build geometry for rendering cursor
    pub fn build_from_cursor(
        cursor: &Cursor,
        font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
        char_width: f32,
        line_height: f32,
    ) -> Result<Self, String> {
        let mut geometry = CursorGeometry::new();

        let cursor_pos = cursor.position();

        // For now, just render a simple blinking cursor at (0, 0)
        // In the future, we'll track the actual cursor position based on buffer content
        let x_ndc = 0.0;
        let y_ndc = 0.95;

        let cursor_width = char_width * 0.1;
        let cursor_height = line_height * 0.8;

        let width_ndc = (cursor_width / viewport_width) * 2.0;
        let height_ndc = (cursor_height / viewport_height) * 2.0;

        let cursor_color = [0.95, 0.95, 0.95, 0.8]; // Light gray, semi-transparent

        let vertex_start = geometry.vertices.len() as u32;

        // Top-left
        geometry.vertices.push(CursorVertex {
            position: [x_ndc, y_ndc],
            color: cursor_color,
        });

        // Top-right
        geometry.vertices.push(CursorVertex {
            position: [x_ndc + width_ndc, y_ndc],
            color: cursor_color,
        });

        // Bottom-right
        geometry.vertices.push(CursorVertex {
            position: [x_ndc + width_ndc, y_ndc - height_ndc],
            color: cursor_color,
        });

        // Bottom-left
        geometry.vertices.push(CursorVertex {
            position: [x_ndc, y_ndc - height_ndc],
            color: cursor_color,
        });

        // Two triangles (CCW winding)
        geometry.indices.push(vertex_start);
        geometry.indices.push(vertex_start + 1);
        geometry.indices.push(vertex_start + 2);

        geometry.indices.push(vertex_start);
        geometry.indices.push(vertex_start + 2);
        geometry.indices.push(vertex_start + 3);

        Ok(geometry)
    }
}
