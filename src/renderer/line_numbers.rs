use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::text_geometry::TextVertex;

pub struct LineNumbersGeometry {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
}

impl LineNumbersGeometry {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Build geometry for rendering line numbers
    pub fn build(
        total_lines: usize,
        glyph_atlas: &mut GlyphAtlas,
        font_size: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Result<Self, String> {
        let mut geometry = LineNumbersGeometry::new();

        let char_width = font_size * 0.6;
        let line_height = font_size * 1.2;

        // Line numbers area starts at x = -1.0 (left edge) and takes up space for up to 4 digits + padding
        let line_nums_width = char_width * 5.0; // 4 digits + 1 padding
        let line_nums_width_ndc = (line_nums_width / viewport_width) * 2.0;

        for line_num in 1..=total_lines {
            let y_pos = ((line_num - 1) as f32) * line_height;
            if y_pos > viewport_height {
                break;
            }

            // Position line numbers right-aligned in the line numbers area
            let line_str = line_num.to_string();
            let num_chars = line_str.len();
            let x_offset = line_nums_width - (num_chars as f32 * char_width + char_width * 0.5);

            let x_ndc = -1.0 + (x_offset / viewport_width) * 2.0;
            let y_ndc = 1.0 - (y_pos / viewport_height) * 2.0;

            let mut x_pos = x_ndc;
            for ch in line_str.chars() {
                let entry = match glyph_atlas.get_or_rasterize(ch) {
                    Ok(e) => e,
                    Err(_) => continue,
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
        }

        Ok(geometry)
    }
}
