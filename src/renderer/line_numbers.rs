use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
use super::text_geometry::{TextVertex, WrappedText};

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

    /// Build geometry for rendering line numbers with word wrapping support and scrolling.
    pub fn build_with_wrap(
        total_lines: usize,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
        wrapped_text: &WrappedText,
        scroll_offset: usize,
        colors: &super::layout::Colors,
    ) -> Result<Self, String> {
        let mut geometry = LineNumbersGeometry::new();

        if wrapped_text.wrapped_lines.is_empty() {
            return Ok(geometry);
        }

        let visible_lines = layout.visible_lines().max(1);
        let total_visual = wrapped_text.total_visual_lines.max(1);
        let first_visual = scroll_offset.min(total_visual.saturating_sub(1));
        let last_visual = (first_visual + visible_lines).min(total_visual);
        let max_digits = total_lines.max(1).to_string().len();

        let ascent = glyph_atlas.ascent();

        // Track which logical lines we've already rendered
        let mut rendered_logical_lines = Vec::new();

        for wrapped in &wrapped_text.wrapped_lines {
            let visual_line = wrapped.visual_line;
            if visual_line < first_visual || visual_line >= last_visual {
                continue;
            }

            // Only show line number for the first visual line of each logical line
            if rendered_logical_lines.contains(&wrapped.logical_line) {
                continue;
            }
            rendered_logical_lines.push(wrapped.logical_line);

            let line_num = wrapped.logical_line + 1; // 1-indexed
            let line_str = format!("{:>width$}", line_num, width = max_digits);

            let text_width: f32 = line_str.len() as f32 * layout.char_width;
            let base_x = layout.gutter.width - layout.line_number_padding_right - text_width;
            let screen_line = visual_line.saturating_sub(first_visual);
            let baseline_y =
                layout.text_area_padding_top + (screen_line as f32 * layout.line_height) + ascent;

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

                let glyph_x = base_x + x_offset + entry.metrics.xmin as f32;
                let glyph_y = baseline_y - entry.metrics.ymin as f32 - entry.height as f32;

                let [x1, y1] = layout.pixel_to_ndc(glyph_x, glyph_y);
                let [x2, y2] = layout
                    .pixel_to_ndc(glyph_x + entry.width as f32, glyph_y + entry.height as f32);

                let vertex_start = geometry.vertices.len() as u32;

                geometry.vertices.push(TextVertex {
                    position: [x1, y1],
                    uv: [entry.uv_min_x, entry.uv_min_y],
                    color: colors.line_number_color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x2, y1],
                    uv: [entry.uv_max_x, entry.uv_min_y],
                    color: colors.line_number_color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x2, y2],
                    uv: [entry.uv_max_x, entry.uv_max_y],
                    color: colors.line_number_color,
                });

                geometry.vertices.push(TextVertex {
                    position: [x1, y2],
                    uv: [entry.uv_min_x, entry.uv_max_y],
                    color: colors.line_number_color,
                });

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

    /// Build geometry for rendering line numbers (legacy, without wrapping)
    #[allow(dead_code)]
    pub fn build(
        total_lines: usize,
        glyph_atlas: &mut GlyphAtlas,
        layout: &EditorLayout,
        colors: &super::layout::Colors,
    ) -> Result<Self, String> {
        let mut geometry = LineNumbersGeometry::new();

        let visible_lines = layout.visible_lines().min(total_lines.max(1));
        let max_digits = total_lines.max(1).to_string().len();

        // Get font metrics for baseline positioning
        let ascent = glyph_atlas.ascent();

        for line_num in 1..=visible_lines {
            let line_str = format!("{:>width$}", line_num, width = max_digits);

            // Calculate position - right-aligned in gutter
            let text_width: f32 = line_str.len() as f32 * layout.char_width;
            let base_x = layout.gutter.width - layout.line_number_padding_right - text_width;
            let baseline_y = layout.text_area_padding_top
                + ((line_num - 1) as f32 * layout.line_height)
                + ascent;

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

                // Calculate pixel position using proper font metrics
                let glyph_x = base_x + x_offset + entry.metrics.xmin as f32;
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
                    color: colors.line_number_color,
                });

                // Top-right
                geometry.vertices.push(TextVertex {
                    position: [x2, y1],
                    uv: [entry.uv_max_x, entry.uv_min_y],
                    color: colors.line_number_color,
                });

                // Bottom-right
                geometry.vertices.push(TextVertex {
                    position: [x2, y2],
                    uv: [entry.uv_max_x, entry.uv_max_y],
                    color: colors.line_number_color,
                });

                // Bottom-left
                geometry.vertices.push(TextVertex {
                    position: [x1, y2],
                    uv: [entry.uv_min_x, entry.uv_max_y],
                    color: colors.line_number_color,
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
