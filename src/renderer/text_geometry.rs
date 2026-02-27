use super::glyph_cache::GlyphAtlas;
use crate::editor::Buffer;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub struct TextGeometry {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl TextGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for rendering text buffer
    /// Returns (vertices, indices, glyph_atlas_width, glyph_atlas_height)
    pub fn build_from_buffer(
        buffer: &Buffer,
        glyph_atlas: &mut GlyphAtlas,
        font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<Self, String> {
        let mut geometry = TextGeometry::new();

        let char_width = font_size * 0.6; // Monospace approximation
        let line_height = font_size * 1.2;

        // Iterate through buffer and build quads for each character
        let lines = buffer.lines();
        for (line_idx, line) in lines.iter().enumerate() {
            let y_pos = (line_idx as f32) * line_height;
            if y_pos > viewport_height {
                break;
            }

            let mut x_pos = 0.0;
            for ch in line.chars() {
                if ch == '\n' {
                    break;
                }

                // Get glyph from atlas
                let entry = match glyph_atlas.get_or_rasterize(ch) {
                    Ok(e) => e,
                    Err(_) => {
                        // Skip unrenderable characters
                        x_pos += char_width;
                        continue;
                    }
                };

                let metrics = &entry.metrics;

                // Skip invisible glyphs (space, etc.)
                if entry.width == 0 || entry.height == 0 {
                    x_pos += metrics.advance_width;
                    continue;
                }

                // Convert screen space to NDC (-1 to 1)
                let x_ndc = (x_pos / viewport_width) * 2.0 - 1.0;
                let y_ndc = 1.0 - (y_pos / viewport_height) * 2.0; // Flip Y for top-left origin
                let width_ndc = (entry.width as f32 / viewport_width) * 2.0;
                let height_ndc = (entry.height as f32 / viewport_height) * 2.0;

                // Add quad (2 triangles = 6 vertices, or using indices = 4 vertices + 6 indices)
                let vertex_start = geometry.vertices.len() as u32;

                // Top-left
                geometry.vertices.push(TextVertex {
                    position: [x_ndc, y_ndc],
                    uv: [entry.uv_min_x, entry.uv_min_y],
                });

                // Top-right
                geometry.vertices.push(TextVertex {
                    position: [x_ndc + width_ndc, y_ndc],
                    uv: [entry.uv_max_x, entry.uv_min_y],
                });

                // Bottom-right
                geometry.vertices.push(TextVertex {
                    position: [x_ndc + width_ndc, y_ndc - height_ndc],
                    uv: [entry.uv_max_x, entry.uv_max_y],
                });

                // Bottom-left
                geometry.vertices.push(TextVertex {
                    position: [x_ndc, y_ndc - height_ndc],
                    uv: [entry.uv_min_x, entry.uv_max_y],
                });

                // Two triangles (CCW winding)
                geometry.indices.push(vertex_start);
                geometry.indices.push(vertex_start + 1);
                geometry.indices.push(vertex_start + 2);

                geometry.indices.push(vertex_start);
                geometry.indices.push(vertex_start + 2);
                geometry.indices.push(vertex_start + 3);

                x_pos += metrics.advance_width;
            }
        }

        Ok(geometry)
    }
}
