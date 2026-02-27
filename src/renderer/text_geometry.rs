use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
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
    pub fn build_from_buffer(
        buffer: &Buffer,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
    ) -> Result<Self, String> {
        let mut geometry = TextGeometry::new();

        let lines = buffer.lines();
        let visible_lines = layout.visible_lines().min(lines.len());

        // Get font metrics for baseline positioning
        let ascent = glyph_atlas.ascent();

        for line_idx in 0..visible_lines {
            let line = &lines[line_idx];

            // Get baseline position for this line
            // The baseline is ascent pixels down from the top of the line
            let base_x = layout.text_area.x + layout.text_area_padding_left;
            let baseline_y =
                layout.text_area_padding_top + (line_idx as f32 * layout.line_height) + ascent;

            let mut x_offset = 0.0;

            for ch in line.chars() {
                if ch == '\n' || ch == '\r' {
                    break;
                }

                // Get glyph from atlas
                let entry = match glyph_atlas.get_or_rasterize(ch) {
                    Ok(e) => e.clone(),
                    Err(_) => {
                        x_offset += layout.char_width;
                        continue;
                    }
                };

                // Skip invisible glyphs but advance position
                if entry.width == 0 || entry.height == 0 {
                    x_offset += entry.metrics.advance_width;
                    continue;
                }

                // Calculate pixel position using proper font metrics
                // xmin is the horizontal bearing (offset from pen position)
                // ymin is the vertical offset from baseline (positive = glyph extends above baseline)
                let glyph_x = base_x + x_offset + entry.metrics.xmin as f32;
                // For top-left origin: baseline_y - ymin - height positions the glyph correctly
                // ymin tells us how far the top of the glyph is from the baseline
                let glyph_y = baseline_y - entry.metrics.ymin as f32 - entry.height as f32;

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
