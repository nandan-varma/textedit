use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
use super::text_geometry::TextVertex;
use crate::editor::{Buffer, Cursor};

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

    /// Build geometry for rendering status bar text
    pub fn build(
        cursor: &Cursor,
        buffer: &Buffer,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
    ) -> Result<Self, String> {
        let mut geometry = StatusBarGeometry::new();

        // Get cursor line and column
        let (line, col) = buffer.char_to_line_col(cursor.position());
        let total_lines = buffer.len_lines();

        // Status bar text: "Ln X, Col Y | UTF-8 | Lines: N"
        let status_text = format!(
            "Ln {}, Col {}  |  UTF-8  |  {} lines",
            line + 1,
            col + 1,
            total_lines
        );

        // Get font metrics for baseline positioning
        let ascent = glyph_atlas.ascent();

        // Position in status bar - baseline is centered vertically
        let base_x = layout.status_bar.x + layout.status_bar_padding;
        let baseline_y = layout.status_bar.y + (layout.status_bar.height + ascent) * 0.5;

        let mut x_offset = 0.0;

        for ch in status_text.chars() {
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

            // Calculate pixel position using proper font metrics
            let glyph_x = base_x + x_offset + entry.metrics.xmin as f32;
            let glyph_y = baseline_y - entry.metrics.ymin as f32 - entry.height as f32;

            // Convert to NDC
            let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
            let [x2, y2] =
                layout.pixel_to_ndc(glyph_x + entry.width as f32, glyph_y + entry.height as f32);

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

        Ok(geometry)
    }
}
