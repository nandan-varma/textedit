//! Primitive builders for editor content
//!
//! This module provides helper functions to build primitives from editor state,
//! serving as a bridge between the editor domain and the primitive-based rendering system.

use crate::renderer::layout::{Colors, EditorLayout};
use crate::ui::layers::LayerId;
use crate::ui::primitives::{z_index, Point, Primitive, Rect, RenderList};

/// Build background primitives (editor background, gutter background)
pub fn build_background(layout: &EditorLayout, colors: &Colors) -> RenderList {
    let mut list = RenderList::with_capacity(3);

    // Main editor background
    list.push(Primitive::rect(
        Rect::new(0.0, 0.0, layout.viewport_width, layout.viewport_height),
        colors.background,
        z_index::BACKGROUND,
    ));

    // Gutter background
    if layout.gutter_width > 0.0 {
        list.push(Primitive::rect(
            Rect::new(0.0, 0.0, layout.gutter_width, layout.text_area.height),
            colors.gutter_background,
            z_index::GUTTER,
        ));

        // Gutter separator line
        list.push(Primitive::line(
            Point::new(layout.gutter_width, 0.0),
            Point::new(layout.gutter_width, layout.text_area.height),
            colors.gutter_separator,
            1.0,
            z_index::GUTTER,
        ));
    }

    list
}

/// Build status bar primitives
pub fn build_status_bar(
    layout: &EditorLayout,
    colors: &Colors,
    line: usize,
    col: usize,
    total_lines: usize,
    file_path: Option<&str>,
    override_text: Option<&str>,
) -> RenderList {
    let mut list = RenderList::with_capacity(2);

    if !layout.show_status_bar {
        return list;
    }

    let status_y = layout.viewport_height - layout.status_bar_height;

    // Status bar background
    list.push(Primitive::rect(
        Rect::new(
            0.0,
            status_y,
            layout.viewport_width,
            layout.status_bar_height,
        ),
        colors.status_bar_background,
        z_index::STATUS_BAR_BG,
    ));

    // Status bar text
    let text = if let Some(override_text) = override_text {
        override_text.to_string()
    } else {
        let file_display = file_path.unwrap_or("Untitled");
        format!(
            "{} | Ln {}, Col {} | {} lines",
            file_display, line, col, total_lines
        )
    };

    let text_x = layout.status_bar_padding;
    let text_y = status_y + (layout.status_bar_height - layout.line_height) / 2.0;

    list.push(Primitive::text(
        text,
        Point::new(text_x, text_y),
        colors.text_color,
        z_index::STATUS_BAR_TEXT,
    ));

    list
}

/// Build cursor primitive
pub fn build_cursor(
    cursor_x: f32,
    cursor_y: f32,
    cursor_width: f32,
    line_height: f32,
    colors: &Colors,
) -> RenderList {
    let mut list = RenderList::with_capacity(1);

    list.push(Primitive::rect(
        Rect::new(cursor_x, cursor_y, cursor_width, line_height),
        colors.cursor_color,
        z_index::CURSOR,
    ));

    list
}

/// Build selection highlight primitives
pub fn build_selection(
    selection_rects: &[(f32, f32, f32, f32)], // (x, y, width, height)
    colors: &Colors,
) -> RenderList {
    let mut list = RenderList::with_capacity(selection_rects.len());

    for (x, y, width, height) in selection_rects {
        list.push(Primitive::rect(
            Rect::new(*x, *y, *width, *height),
            colors.selection_color,
            z_index::SELECTION,
        ));
    }

    list
}

/// Build match highlight primitives for find results
pub fn build_match_highlights(
    match_rects: &[(f32, f32, f32, f32)], // (x, y, width, height)
    current_match_index: Option<usize>,
    colors: &Colors,
) -> RenderList {
    let mut list = RenderList::with_capacity(match_rects.len());

    for (i, (x, y, width, height)) in match_rects.iter().enumerate() {
        let color = if Some(i) == current_match_index {
            colors.current_match_highlight
        } else {
            colors.match_highlight
        };

        list.push(Primitive::rect(
            Rect::new(*x, *y, *width, *height),
            color,
            z_index::MATCH_HIGHLIGHT,
        ));
    }

    list
}

/// Build scrollbar primitives
pub fn build_scrollbar(
    layout: &EditorLayout,
    colors: &Colors,
    total_lines: usize,
    visible_lines: usize,
    scroll_offset: usize,
) -> RenderList {
    let mut list = RenderList::with_capacity(2);

    if total_lines <= visible_lines {
        return list;
    }

    let scrollbar_width = layout.scrollbar_width;
    let scrollbar_x = layout.viewport_width - scrollbar_width;
    let track_height = layout.text_area.height;

    // Scrollbar track
    list.push(Primitive::rect(
        Rect::new(scrollbar_x, 0.0, scrollbar_width, track_height),
        colors.scrollbar_track,
        z_index::SCROLLBAR,
    ));

    // Scrollbar thumb
    let thumb_height = (visible_lines as f32 / total_lines as f32 * track_height).max(20.0);
    let max_scroll = total_lines.saturating_sub(visible_lines);
    let scroll_ratio = if max_scroll > 0 {
        scroll_offset as f32 / max_scroll as f32
    } else {
        0.0
    };
    let thumb_y = scroll_ratio * (track_height - thumb_height);

    list.push(Primitive::rounded_rect(
        Rect::new(
            scrollbar_x + 2.0,
            thumb_y,
            scrollbar_width - 4.0,
            thumb_height,
        ),
        colors.scrollbar_thumb,
        4.0,
        z_index::SCROLLBAR + 1,
    ));

    list
}

/// Build modal overlay (semi-transparent background)
pub fn build_modal_overlay(layout: &EditorLayout) -> RenderList {
    let mut list = RenderList::with_capacity(1);

    list.push(Primitive::rect(
        Rect::new(0.0, 0.0, layout.viewport_width, layout.viewport_height),
        [0.0, 0.0, 0.0, 0.3], // Semi-transparent black
        z_index::MODAL_OVERLAY,
    ));

    list
}

/// Build modal background
pub fn build_modal_background(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    colors: &Colors,
) -> RenderList {
    let mut list = RenderList::with_capacity(2);

    // Modal background with rounded corners
    list.push(Primitive::rounded_rect(
        Rect::new(x, y, width, height),
        colors.modal_background,
        8.0,
        z_index::MODAL_BG,
    ));

    // Modal border
    list.push(Primitive::border(
        Rect::new(x, y, width, height),
        colors.modal_border,
        1.0,
        8.0,
        z_index::MODAL_BG + 1,
    ));

    list
}

/// Convert a layer ID to its corresponding z_index base
pub fn layer_to_z_index(layer: LayerId) -> i32 {
    layer.z_index()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_colors() -> Colors {
        Colors::default()
    }

    fn test_layout() -> EditorLayout {
        EditorLayout::new(800.0, 600.0, 14.0, 1.0, true, true)
    }

    #[test]
    fn test_build_background() {
        let layout = test_layout();
        let colors = test_colors();

        let primitives = build_background(&layout, &colors);
        assert!(primitives.len() >= 1); // At least the main background
    }

    #[test]
    fn test_build_cursor() {
        let colors = test_colors();

        let primitives = build_cursor(100.0, 50.0, 2.0, 20.0, &colors);
        assert_eq!(primitives.len(), 1);
    }

    #[test]
    fn test_build_selection() {
        let colors = test_colors();
        let rects = vec![(10.0, 20.0, 100.0, 20.0), (10.0, 40.0, 80.0, 20.0)];

        let primitives = build_selection(&rects, &colors);
        assert_eq!(primitives.len(), 2);
    }

    #[test]
    fn test_build_match_highlights() {
        let colors = test_colors();
        let rects = vec![(10.0, 20.0, 50.0, 20.0), (10.0, 60.0, 50.0, 20.0)];

        let primitives = build_match_highlights(&rects, Some(0), &colors);
        assert_eq!(primitives.len(), 2);
    }

    #[test]
    fn test_build_scrollbar_hidden_when_all_visible() {
        let layout = test_layout();
        let colors = test_colors();

        // When all content is visible, no scrollbar
        let primitives = build_scrollbar(&layout, &colors, 10, 100, 0);
        assert!(primitives.is_empty());
    }

    #[test]
    fn test_build_scrollbar_shown_when_needed() {
        let layout = test_layout();
        let colors = test_colors();

        // When content exceeds visible area
        let primitives = build_scrollbar(&layout, &colors, 100, 10, 0);
        assert_eq!(primitives.len(), 2); // Track and thumb
    }

    #[test]
    fn test_build_modal_overlay() {
        let layout = test_layout();

        let primitives = build_modal_overlay(&layout);
        assert_eq!(primitives.len(), 1);
    }

    #[test]
    fn test_build_modal_background() {
        let colors = test_colors();

        let primitives = build_modal_background(100.0, 50.0, 400.0, 200.0, &colors);
        assert_eq!(primitives.len(), 2); // Background and border
    }
}
