use crate::editor::Cursor;
use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::text_geometry::TextVertex;

pub struct StatusBarGeometry {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl StatusBarGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for rendering status bar showing line and column info
    pub fn build(
        cursor: &Cursor,
        glyph_atlas: &mut GlyphAtlas,
        font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<Self, String> {
        let mut geometry = StatusBarGeometry::new();

        let char_width = font_size * 0.6;
        let line_height = font_size * 1.2;

        // Status bar at the bottom
        let status_text = format!("Line {}, Col {}", cursor.position(), 0); // TODO: Calculate actual line/column

        // Position status bar at bottom-left with padding
        let padding = char_width * 0.5;
        let y_pos = viewport_height - line_height * 1.5; // Above bottom

        let x_ndc = -1.0 + (padding / viewport_width) * 2.0;
        let y_ndc = 1.0 - (y_pos / viewport_height) * 2.0;

        let mut x_pos = x_ndc;
        for ch in status_text.chars() {
            let entry = match glyph_atlas.get_or_rasterize(ch) {
                Ok(e) => e,
                Err(_) => {
                    x_pos += char_width / viewport_width * 2.0;
                    continue;
                }
            };

            let metrics = &entry.metrics;

            if entry.width == 0 || entry.height == 0 {
                x_pos += metrics.advance_width / viewport_width * 2.0;
                continue;
            }

            let width_ndc = (entry.width as f32 / viewport_width) * 2.0;
            let height_ndc = (entry.height as f32 / viewport_height) * 2.0;

            let vertex_start = geometry.vertices.len() as u32;

            // Top-left
            geometry.vertices.push(TextVertex {
                position: [x_pos, y_ndc],
                uv: [entry.uv_min_x, entry.uv_min_y],
            });

            // Top-right
            geometry.vertices.push(TextVertex {
                position: [x_pos + width_ndc, y_ndc],
                uv: [entry.uv_max_x, entry.uv_min_y],
            });

            // Bottom-right
            geometry.vertices.push(TextVertex {
                position: [x_pos + width_ndc, y_ndc - height_ndc],
                uv: [entry.uv_max_x, entry.uv_max_y],
            });

            // Bottom-left
            geometry.vertices.push(TextVertex {
                position: [x_pos, y_ndc - height_ndc],
                uv: [entry.uv_min_x, entry.uv_max_y],
            });

            // Two triangles
            geometry.indices.push(vertex_start);
            geometry.indices.push(vertex_start + 1);
            geometry.indices.push(vertex_start + 2);

            geometry.indices.push(vertex_start);
            geometry.indices.push(vertex_start + 2);
            geometry.indices.push(vertex_start + 3);

            x_pos += metrics.advance_width / viewport_width * 2.0;
        }

        Ok(geometry)
    }
}
