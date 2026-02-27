use super::glyph_cache::GlyphAtlas;
use super::layout::{EditorLayout, LINE_NUMBER_PADDING_RIGHT, TEXT_AREA_PADDING_TOP};
use super::text_geometry::TextVertex;

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
        layout: &EditorLayout,
    ) -> Result<Self, String> {
        let mut geometry = LineNumbersGeometry::new();

        let visible_lines = layout.visible_lines().min(total_lines.max(1));
        let max_digits = total_lines.max(1).to_string().len();

        for line_num in 1..=visible_lines {
            let line_str = format!("{:>width$}", line_num, width = max_digits);

            // Calculate position - right-aligned in gutter
            let text_width: f32 = line_str.len() as f32 * layout.char_width;
            let base_x = layout.gutter.width - LINE_NUMBER_PADDING_RIGHT - text_width;
            let base_y = TEXT_AREA_PADDING_TOP + ((line_num - 1) as f32 * layout.line_height);

            let mut x_offset = 0.0;

            for ch in line_str.chars() {
                let entry = match glyph_atlas.get_or_rasterize(ch) {
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

                // Calculate pixel position
                let glyph_x = base_x + x_offset;
                let glyph_y = base_y + (layout.line_height - entry.height as f32) * 0.5;

                // Convert to NDC
                let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
                let [x2, y2] = layout
                    .pixel_to_ndc(glyph_x + entry.width as f32, glyph_y + entry.height as f32);

                let vertex_start = geometry.vertices.len() as u32;

                // Top-left
                geometry.vertices.push(TextVertex {
                    position: [x1, y1],
                    uv: [entry.uv_min_x, entry.uv_min_y],
                });

                // Top-right
                geometry.vertices.push(TextVertex {
                    position: [x2, y1],
                    uv: [entry.uv_max_x, entry.uv_min_y],
                });

                // Bottom-right
                geometry.vertices.push(TextVertex {
                    position: [x2, y2],
                    uv: [entry.uv_max_x, entry.uv_max_y],
                });

                // Bottom-left
                geometry.vertices.push(TextVertex {
                    position: [x1, y2],
                    uv: [entry.uv_min_x, entry.uv_max_y],
                });

                // Two triangles
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
