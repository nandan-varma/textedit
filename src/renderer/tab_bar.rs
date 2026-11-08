//! Tab bar geometry builder for GPU rendering.
//!
//! Builds vertex/index buffers for rendering tab bar backgrounds and text.

use super::glyph_cache::GlyphAtlas;
use super::layout::EditorLayout;
use super::text_geometry::TextVertex;
use super::ui_background::ColorVertex;
use crate::application::TabInfo;

/// Style configuration for tab bar rendering
#[derive(Debug, Clone)]
pub struct TabBarRenderStyle {
    pub background_color: [f32; 4],
    pub active_tab_bg: [f32; 4],
    pub inactive_tab_bg: [f32; 4],
    pub hovered_tab_bg: [f32; 4],
    pub text_color: [f32; 4],
    pub modified_indicator_color: [f32; 4],
    pub close_button_color: [f32; 4],
    pub separator_color: [f32; 4],
    pub active_indicator_color: [f32; 4],
    pub tab_padding: f32,
    pub tab_gap: f32,
    pub max_tab_width: f32,
    pub min_tab_width: f32,
    pub close_button_size: f32,
}

impl Default for TabBarRenderStyle {
    fn default() -> Self {
        Self {
            background_color: [0.1, 0.1, 0.1, 1.0],
            active_tab_bg: [0.18, 0.18, 0.18, 1.0],
            inactive_tab_bg: [0.12, 0.12, 0.12, 1.0],
            hovered_tab_bg: [0.15, 0.15, 0.15, 1.0],
            text_color: [0.9, 0.9, 0.9, 1.0],
            modified_indicator_color: [0.9, 0.6, 0.2, 1.0],
            close_button_color: [0.5, 0.5, 0.5, 1.0],
            separator_color: [0.25, 0.25, 0.25, 1.0],
            active_indicator_color: [0.0, 0.48, 0.8, 1.0],
            tab_padding: 12.0,
            tab_gap: 2.0,
            max_tab_width: 200.0,
            min_tab_width: 100.0,
            close_button_size: 14.0,
        }
    }
}

/// Geometry data for tab bar rendering
pub struct TabBarGeometry {
    pub bg_vertices: Vec<ColorVertex>,
    pub bg_indices: Vec<u32>,
    pub text_vertices: Vec<TextVertex>,
    pub text_indices: Vec<u32>,
}

impl TabBarGeometry {
    pub fn new() -> Self {
        Self {
            bg_vertices: Vec::new(),
            bg_indices: Vec::new(),
            text_vertices: Vec::new(),
            text_indices: Vec::new(),
        }
    }

    /// Build tab bar geometry from tab information
    pub fn build(
        tabs: &[TabInfo],
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        style: &TabBarRenderStyle,
        hovered_tab: Option<usize>,
    ) -> Result<Self, String> {
        let mut geometry = TabBarGeometry::new();

        // Only render if tab bar is visible
        if !layout.show_tab_bar || layout.tab_bar_height <= 0.0 {
            return Ok(geometry);
        }

        // Tab bar background
        geometry.add_rect(
            layout,
            layout.tab_bar.x,
            layout.tab_bar.y,
            layout.tab_bar.width,
            layout.tab_bar.height,
            style.background_color,
        );

        // Bottom separator line
        geometry.add_rect(
            layout,
            layout.tab_bar.x,
            layout.tab_bar.y + layout.tab_bar.height - 1.0,
            layout.tab_bar.width,
            1.0,
            style.separator_color,
        );

        if tabs.is_empty() {
            return Ok(geometry);
        }

        // Calculate tab dimensions
        let available_width = layout.tab_bar.width - style.tab_gap;
        let num_tabs = tabs.len();
        let ideal_width = available_width / num_tabs as f32 - style.tab_gap;
        let tab_width = ideal_width
            .min(style.max_tab_width)
            .max(style.min_tab_width);

        let mut x = style.tab_gap;
        let ascent = glyph_atlas.ascent();

        for (idx, tab) in tabs.iter().enumerate() {
            let tab_x = layout.tab_bar.x + x;
            let tab_y = layout.tab_bar.y;

            // Determine background color
            let bg_color = if tab.is_active {
                style.active_tab_bg
            } else if hovered_tab == Some(idx) {
                style.hovered_tab_bg
            } else {
                style.inactive_tab_bg
            };

            // Tab background
            geometry.add_rect(
                layout,
                tab_x,
                tab_y,
                tab_width,
                layout.tab_bar_height,
                bg_color,
            );

            // Active tab indicator (bottom highlight)
            if tab.is_active {
                geometry.add_rect(
                    layout,
                    tab_x,
                    tab_y + layout.tab_bar_height - 2.0,
                    tab_width,
                    2.0,
                    style.active_indicator_color,
                );
            }

            // Modified indicator dot
            let text_start_x = if tab.is_modified {
                let dot_size = 6.0;
                let dot_x = tab_x + style.tab_padding;
                let dot_y = tab_y + (layout.tab_bar_height - dot_size) / 2.0;
                geometry.add_rect(
                    layout,
                    dot_x,
                    dot_y,
                    dot_size,
                    dot_size,
                    style.modified_indicator_color,
                );
                tab_x + style.tab_padding + dot_size + 4.0
            } else {
                tab_x + style.tab_padding
            };

            // Calculate max text width (leaving room for close button)
            let close_area = style.close_button_size + style.tab_padding;
            let max_text_width = tab_width - (text_start_x - tab_x) - close_area;

            // Truncate title if needed
            let title = truncate_title(&tab.title, max_text_width, glyph_atlas);

            // Render title text
            let text_y = tab_y + (layout.tab_bar_height + ascent) * 0.5;
            geometry.add_text(
                layout,
                glyph_atlas,
                &title,
                text_start_x,
                text_y,
                style.text_color,
            )?;

            // Close button (X)
            let close_x = tab_x + tab_width - style.tab_padding - style.close_button_size;
            let close_y = tab_y + (layout.tab_bar_height - style.close_button_size) / 2.0;

            // Render X character for close button
            let x_char_y = close_y + (style.close_button_size + ascent) * 0.5;
            geometry.add_text(
                layout,
                glyph_atlas,
                "\u{00D7}", // multiplication sign (×)
                close_x,
                x_char_y,
                style.close_button_color,
            )?;

            x += tab_width + style.tab_gap;
        }

        Ok(geometry)
    }

    fn add_rect(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) {
        let [x1, y1] = layout.pixel_to_ndc(x, y);
        let [x2, y2] = layout.pixel_to_ndc(x + width, y + height);

        let vertex_start = self.bg_vertices.len() as u32;

        self.bg_vertices.push(ColorVertex {
            position: [x1, y1],
            color,
        });
        self.bg_vertices.push(ColorVertex {
            position: [x2, y1],
            color,
        });
        self.bg_vertices.push(ColorVertex {
            position: [x2, y2],
            color,
        });
        self.bg_vertices.push(ColorVertex {
            position: [x1, y2],
            color,
        });

        self.bg_indices.push(vertex_start);
        self.bg_indices.push(vertex_start + 1);
        self.bg_indices.push(vertex_start + 2);
        self.bg_indices.push(vertex_start);
        self.bg_indices.push(vertex_start + 2);
        self.bg_indices.push(vertex_start + 3);
    }

    fn add_text(
        &mut self,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        text: &str,
        base_x: f32,
        baseline_y: f32,
        color: [f32; 4],
    ) -> Result<(), String> {
        let mut x_offset = 0.0;

        for ch in text.chars() {
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
            let [x2, y2] =
                layout.pixel_to_ndc(glyph_x + entry.width as f32, glyph_y + entry.height as f32);

            let vertex_start = self.text_vertices.len() as u32;

            self.text_vertices.push(TextVertex {
                position: [x1, y1],
                uv: [entry.uv_min_x, entry.uv_min_y],
                color,
            });
            self.text_vertices.push(TextVertex {
                position: [x2, y1],
                uv: [entry.uv_max_x, entry.uv_min_y],
                color,
            });
            self.text_vertices.push(TextVertex {
                position: [x2, y2],
                uv: [entry.uv_max_x, entry.uv_max_y],
                color,
            });
            self.text_vertices.push(TextVertex {
                position: [x1, y2],
                uv: [entry.uv_min_x, entry.uv_max_y],
                color,
            });

            self.text_indices.push(vertex_start);
            self.text_indices.push(vertex_start + 1);
            self.text_indices.push(vertex_start + 2);
            self.text_indices.push(vertex_start);
            self.text_indices.push(vertex_start + 2);
            self.text_indices.push(vertex_start + 3);

            x_offset += entry.metrics.advance_width;
        }

        Ok(())
    }
}

/// Truncate a title to fit within max_width, adding ellipsis if needed
fn truncate_title(title: &str, max_width: f32, glyph_atlas: &GlyphAtlas) -> String {
    let mut width = 0.0;
    for ch in title.chars() {
        width += glyph_atlas.char_advance_width_cached(ch);
    }

    if width <= max_width {
        return title.to_string();
    }

    let ellipsis = "\u{2026}"; // horizontal ellipsis
    let ellipsis_width = glyph_atlas.char_advance_width_cached('\u{2026}');
    let target_width = max_width - ellipsis_width;

    let mut result = String::new();
    let mut current_width = 0.0;

    for ch in title.chars() {
        let char_width = glyph_atlas.char_advance_width_cached(ch);
        if current_width + char_width > target_width {
            break;
        }
        result.push(ch);
        current_width += char_width;
    }

    result.push_str(ellipsis);
    result
}

impl Default for TabBarGeometry {
    fn default() -> Self {
        Self::new()
    }
}
