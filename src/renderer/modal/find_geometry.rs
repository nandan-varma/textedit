//! Find modal geometry builder
//!
//! Renders the Find/Replace modal dialog with input fields, buttons, and text.

use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::layout::{Colors, EditorLayout, Rect};
use crate::renderer::text_geometry::TextVertex;
use crate::renderer::ui_background::ColorVertex;
use crate::ui::modal::find_modal::{FindButton, FindField, FindModal};
use crate::ui::modal::InputField;

/// Modal padding and sizing constants
const MODAL_PADDING: f32 = 8.0;
const MODAL_RIGHT_MARGIN: f32 = 20.0;
const MODAL_TOP_MARGIN: f32 = 10.0;
const INPUT_PADDING: f32 = 6.0;
const INPUT_HEIGHT: f32 = 24.0;
const BUTTON_SIZE: f32 = 22.0;
const BUTTON_SPACING: f32 = 2.0;
const INPUT_SPACING: f32 = 4.0;
const BORDER_WIDTH: f32 = 1.0;

/// Geometry for rendering the find modal
pub struct FindModalGeometry {
    /// Background vertices (color pipeline)
    pub bg_vertices: Vec<ColorVertex>,
    /// Background indices
    pub bg_indices: Vec<u32>,
    /// Text vertices (text pipeline)
    pub text_vertices: Vec<TextVertex>,
    /// Text indices
    pub text_indices: Vec<u32>,
    /// Hit test regions for buttons
    pub button_regions: Vec<(FindButton, Rect)>,
    /// Hit test regions for inputs
    pub input_regions: Vec<(FindField, Rect)>,
    /// The modal bounding box
    pub modal_rect: Rect,
}

impl FindModalGeometry {
    pub fn new() -> Self {
        Self {
            bg_vertices: Vec::new(),
            bg_indices: Vec::new(),
            text_vertices: Vec::new(),
            text_indices: Vec::new(),
            button_regions: Vec::new(),
            input_regions: Vec::new(),
            modal_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Build geometry for the find modal
    pub fn build(
        modal: &FindModal,
        layout: &EditorLayout,
        glyph_atlas: &mut GlyphAtlas,
        colors: &Colors,
        cursor_visible: bool, // For blinking cursor
    ) -> Self {
        let mut geometry = FindModalGeometry::new();

        // Calculate modal dimensions
        let modal_width = modal.width();
        let modal_height = geometry.calculate_height(modal, layout);

        // Position in top-right corner
        let modal_x = layout.viewport_width - modal_width - MODAL_RIGHT_MARGIN;
        let modal_y = MODAL_TOP_MARGIN;

        geometry.modal_rect = Rect::new(modal_x, modal_y, modal_width, modal_height);

        // Draw modal background
        geometry.add_rect_bg(
            layout,
            modal_x,
            modal_y,
            modal_width,
            modal_height,
            colors.modal_background,
        );

        // Draw modal border
        geometry.add_border(
            layout,
            modal_x,
            modal_y,
            modal_width,
            modal_height,
            colors.modal_border,
        );

        // Current Y position for layout
        let mut y = modal_y + MODAL_PADDING;

        // Toggle expand/collapse button (chevron)
        let toggle_x = modal_x + MODAL_PADDING;
        let toggle_size = 16.0;
        geometry.add_toggle_button(
            layout,
            toggle_x,
            y + (INPUT_HEIGHT - toggle_size) / 2.0,
            toggle_size,
            modal.show_replace,
            colors,
            glyph_atlas,
        );

        // Find input field
        let input_x = toggle_x + toggle_size + 4.0;
        let input_width = modal_width
            - MODAL_PADDING * 2.0
            - toggle_size
            - 4.0
            - (BUTTON_SIZE * 4.0 + BUTTON_SPACING * 3.0)
            - 8.0;

        geometry.add_input_field(
            layout,
            input_x,
            y,
            input_width,
            INPUT_HEIGHT,
            &modal.find_input,
            modal.focused_field == FindField::Find,
            colors,
            glyph_atlas,
            cursor_visible,
        );
        geometry.input_regions.push((
            FindField::Find,
            Rect::new(input_x, y, input_width, INPUT_HEIGHT),
        ));

        // Match status text (right of input, before buttons)
        let status_text = modal.match_status();
        let status_width = geometry.text_width(&status_text, glyph_atlas);
        let status_x = input_x + input_width + 4.0;
        geometry.add_text(
            layout,
            status_x,
            y + INPUT_HEIGHT / 2.0,
            &status_text,
            colors.line_number_color,
            glyph_atlas,
        );

        // Navigation buttons (Prev, Next, Select All, Close)
        let buttons_x =
            modal_x + modal_width - MODAL_PADDING - (BUTTON_SIZE * 4.0 + BUTTON_SPACING * 3.0);

        // Previous button (up arrow)
        geometry.add_icon_button(
            layout,
            buttons_x,
            y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
            BUTTON_SIZE,
            '\u{2191}', // ↑
            FindButton::FindPrev,
            colors,
            glyph_atlas,
        );

        // Next button (down arrow)
        geometry.add_icon_button(
            layout,
            buttons_x + BUTTON_SIZE + BUTTON_SPACING,
            y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
            BUTTON_SIZE,
            '\u{2193}', // ↓
            FindButton::FindNext,
            colors,
            glyph_atlas,
        );

        // Select all button (lines icon - using ≡)
        geometry.add_icon_button(
            layout,
            buttons_x + (BUTTON_SIZE + BUTTON_SPACING) * 2.0,
            y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
            BUTTON_SIZE,
            '\u{2261}',             // ≡
            FindButton::ReplaceAll, // TODO: This should be SelectAll when implemented
            colors,
            glyph_atlas,
        );

        // Close button
        geometry.add_icon_button(
            layout,
            buttons_x + (BUTTON_SIZE + BUTTON_SPACING) * 3.0,
            y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
            BUTTON_SIZE,
            '\u{00D7}', // ×
            FindButton::Close,
            colors,
            glyph_atlas,
        );

        y += INPUT_HEIGHT + INPUT_SPACING;

        // Replace input field (if expanded)
        if modal.show_replace {
            let replace_input_x = input_x;
            let replace_input_width = input_width + status_width + 4.0;

            geometry.add_input_field(
                layout,
                replace_input_x,
                y,
                replace_input_width,
                INPUT_HEIGHT,
                &modal.replace_input,
                modal.focused_field == FindField::Replace,
                colors,
                glyph_atlas,
                cursor_visible,
            );
            geometry.input_regions.push((
                FindField::Replace,
                Rect::new(replace_input_x, y, replace_input_width, INPUT_HEIGHT),
            ));

            // Replace buttons
            let replace_buttons_x = replace_input_x + replace_input_width + 8.0;

            // Replace one button
            geometry.add_icon_button(
                layout,
                replace_buttons_x,
                y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
                BUTTON_SIZE,
                '\u{21B7}', // ↷ or use a simple replacement char
                FindButton::Replace,
                colors,
                glyph_atlas,
            );

            // Replace all button
            geometry.add_icon_button(
                layout,
                replace_buttons_x + BUTTON_SIZE + BUTTON_SPACING,
                y + (INPUT_HEIGHT - BUTTON_SIZE) / 2.0,
                BUTTON_SIZE,
                '\u{21C4}', // ⇄
                FindButton::ReplaceAll,
                colors,
                glyph_atlas,
            );
        }

        geometry
    }

    fn calculate_height(&self, modal: &FindModal, _layout: &EditorLayout) -> f32 {
        let base_height = MODAL_PADDING * 2.0 + INPUT_HEIGHT;
        if modal.show_replace {
            base_height + INPUT_SPACING + INPUT_HEIGHT
        } else {
            base_height
        }
    }

    fn add_rect_bg(
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

    fn add_border(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) {
        // Top border
        self.add_rect_bg(layout, x, y, width, BORDER_WIDTH, color);
        // Bottom border
        self.add_rect_bg(
            layout,
            x,
            y + height - BORDER_WIDTH,
            width,
            BORDER_WIDTH,
            color,
        );
        // Left border
        self.add_rect_bg(layout, x, y, BORDER_WIDTH, height, color);
        // Right border
        self.add_rect_bg(
            layout,
            x + width - BORDER_WIDTH,
            y,
            BORDER_WIDTH,
            height,
            color,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn add_input_field(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        input: &InputField,
        focused: bool,
        colors: &Colors,
        glyph_atlas: &mut GlyphAtlas,
        cursor_visible: bool,
    ) {
        // Input background
        self.add_rect_bg(layout, x, y, width, height, colors.input_background);

        // Input border
        let border_color = if focused {
            colors.input_border_focused
        } else {
            colors.input_border
        };
        self.add_border(layout, x, y, width, height, border_color);

        // Text content
        let text_x = x + INPUT_PADDING;
        let text_y = y + height / 2.0;
        let text_color = if input.is_empty() {
            colors.line_number_color // Placeholder color
        } else {
            colors.text_color
        };

        let display_text = if input.is_empty() {
            &input.placeholder
        } else {
            input.text()
        };

        // Calculate visible text area
        let text_area_width = width - INPUT_PADDING * 2.0;

        // Draw selection background if any
        if let Some((sel_start, sel_end)) = input.selection_range() {
            let text = input.text();
            let before_sel: String = text.chars().take(sel_start).collect();
            let sel_text: String = text
                .chars()
                .skip(sel_start)
                .take(sel_end - sel_start)
                .collect();

            let sel_x = text_x + self.text_width(&before_sel, glyph_atlas);
            let sel_width = self.text_width(&sel_text, glyph_atlas);

            self.add_rect_bg(
                layout,
                sel_x,
                y + 2.0,
                sel_width.min(text_area_width - (sel_x - text_x)),
                height - 4.0,
                colors.selection_color,
            );
        }

        // Draw text
        self.add_text(
            layout,
            text_x,
            text_y,
            display_text,
            text_color,
            glyph_atlas,
        );

        // Draw cursor if focused and visible
        if focused && cursor_visible {
            let cursor_x = if input.is_empty() {
                text_x
            } else {
                let before_cursor: String = input.text().chars().take(input.cursor_pos).collect();
                text_x + self.text_width(&before_cursor, glyph_atlas)
            };

            self.add_rect_bg(
                layout,
                cursor_x,
                y + 4.0,
                2.0,
                height - 8.0,
                colors.cursor_color,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_icon_button(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        size: f32,
        icon: char,
        button: FindButton,
        colors: &Colors,
        glyph_atlas: &mut GlyphAtlas,
    ) {
        // Button background
        self.add_rect_bg(layout, x, y, size, size, colors.button_background);

        // Center the icon
        let icon_str = icon.to_string();
        let icon_width = self.text_width(&icon_str, glyph_atlas);
        let icon_x = x + (size - icon_width) / 2.0;
        let icon_y = y + size / 2.0;

        self.add_text(
            layout,
            icon_x,
            icon_y,
            &icon_str,
            colors.text_color,
            glyph_atlas,
        );

        // Register hit region
        self.button_regions
            .push((button, Rect::new(x, y, size, size)));
    }

    #[allow(clippy::too_many_arguments)]
    fn add_toggle_button(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        y: f32,
        size: f32,
        expanded: bool,
        colors: &Colors,
        glyph_atlas: &mut GlyphAtlas,
    ) {
        // Chevron icon
        let icon = if expanded { '\u{25BC}' } else { '\u{25B6}' }; // ▼ or ▶
        let icon_str = icon.to_string();
        let icon_width = self.text_width(&icon_str, glyph_atlas);
        let icon_x = x + (size - icon_width) / 2.0;
        let icon_y = y + size / 2.0;

        self.add_text(
            layout,
            icon_x,
            icon_y,
            &icon_str,
            colors.line_number_color,
            glyph_atlas,
        );

        // Register hit region
        self.button_regions
            .push((FindButton::ToggleReplace, Rect::new(x, y, size, size)));
    }

    fn add_text(
        &mut self,
        layout: &EditorLayout,
        x: f32,
        baseline_y: f32,
        text: &str,
        color: [f32; 4],
        glyph_atlas: &mut GlyphAtlas,
    ) {
        let ascent = glyph_atlas.ascent();
        let actual_baseline = baseline_y + ascent * 0.3; // Adjust for visual centering

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

            let glyph_x = x + x_offset + entry.metrics.xmin as f32;
            let glyph_y = actual_baseline - entry.metrics.ymin as f32 - entry.height as f32;

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
    }

    fn text_width(&self, text: &str, glyph_atlas: &mut GlyphAtlas) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            if let Ok(entry) = glyph_atlas.get_or_rasterize(ch) {
                width += entry.metrics.advance_width;
            }
        }
        width
    }

    /// Hit test a point against the modal
    /// Returns the button or input field that was hit
    #[allow(dead_code)]
    pub fn hit_test(&self, x: f32, y: f32) -> Option<HitResult> {
        // Check if inside modal at all
        if !self.modal_rect.contains(x, y) {
            return None;
        }

        // Check buttons first
        for (button, rect) in &self.button_regions {
            if rect.contains(x, y) {
                return Some(HitResult::Button(*button));
            }
        }

        // Check inputs
        for (field, rect) in &self.input_regions {
            if rect.contains(x, y) {
                return Some(HitResult::Input(*field));
            }
        }

        // Inside modal but no specific element
        Some(HitResult::ModalBackground)
    }
}

impl Default for FindModalGeometry {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of hit testing the find modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HitResult {
    Button(FindButton),
    Input(FindField),
    ModalBackground,
}
