//! Tab bar widget for displaying and managing document tabs.

use crate::application::TabInfo;
use crate::domain::DocumentId;
use crate::ui::primitives::{Color, Point, Primitive, Rect, RenderList};
use crate::ui::widget::RenderContext;

/// Actions that the tab bar can emit
#[derive(Debug, Clone, PartialEq)]
pub enum TabBarAction {
    /// No action
    None,
    /// Switch to a different tab
    SelectTab(DocumentId),
    /// Close a tab
    CloseTab(DocumentId),
    /// Request to create a new tab
    NewTab,
    /// Tab bar needs redraw
    Redraw,
}

/// Visual configuration for the tab bar
#[derive(Debug, Clone)]
pub struct TabBarStyle {
    /// Height of the tab bar
    pub height: f32,
    /// Maximum tab width
    pub max_tab_width: f32,
    /// Minimum tab width
    pub min_tab_width: f32,
    /// Padding inside each tab
    pub tab_padding: f32,
    /// Gap between tabs
    pub tab_gap: f32,
    /// Corner radius for tabs
    pub tab_corner_radius: f32,
    /// Close button size
    pub close_button_size: f32,
    /// Background color
    pub background_color: Color,
    /// Active tab background
    pub active_tab_bg: Color,
    /// Inactive tab background
    pub inactive_tab_bg: Color,
    /// Hovered tab background
    pub hovered_tab_bg: Color,
    /// Tab text color
    pub text_color: Color,
    /// Modified indicator color
    pub modified_indicator_color: Color,
    /// Close button color
    pub close_button_color: Color,
    /// Close button hover color
    pub close_button_hover_color: Color,
}

impl Default for TabBarStyle {
    fn default() -> Self {
        Self {
            height: 32.0,
            max_tab_width: 200.0,
            min_tab_width: 100.0,
            tab_padding: 12.0,
            tab_gap: 2.0,
            tab_corner_radius: 4.0,
            close_button_size: 14.0,
            background_color: [0.1, 0.1, 0.1, 1.0],
            active_tab_bg: [0.18, 0.18, 0.18, 1.0],
            inactive_tab_bg: [0.12, 0.12, 0.12, 1.0],
            hovered_tab_bg: [0.15, 0.15, 0.15, 1.0],
            text_color: [0.9, 0.9, 0.9, 1.0],
            modified_indicator_color: [0.9, 0.6, 0.2, 1.0],
            close_button_color: [0.5, 0.5, 0.5, 1.0],
            close_button_hover_color: [0.9, 0.3, 0.3, 1.0],
        }
    }
}

/// Computed layout for a single tab
#[derive(Debug, Clone)]
struct TabLayout {
    id: DocumentId,
    bounds: Rect,
    close_button_bounds: Rect,
    title: String,
    is_modified: bool,
    is_active: bool,
}

/// Tab bar widget displaying open document tabs
pub struct TabBar {
    /// Tab information from DocumentManager
    tabs: Vec<TabInfo>,
    /// Visual style
    style: TabBarStyle,
    /// Currently hovered tab (by index)
    hovered_tab: Option<usize>,
    /// Is the close button being hovered
    hovered_close: Option<usize>,
    /// Computed tab layouts (updated on render)
    tab_layouts: Vec<TabLayout>,
    /// Width of the tab bar area
    width: f32,
}

impl TabBar {
    /// Create a new tab bar
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            style: TabBarStyle::default(),
            hovered_tab: None,
            hovered_close: None,
            tab_layouts: Vec::new(),
            width: 0.0,
        }
    }

    /// Create a tab bar with custom style
    pub fn with_style(style: TabBarStyle) -> Self {
        Self {
            tabs: Vec::new(),
            style,
            hovered_tab: None,
            hovered_close: None,
            tab_layouts: Vec::new(),
            width: 0.0,
        }
    }

    /// Update tab information from DocumentManager
    pub fn set_tabs(&mut self, tabs: Vec<TabInfo>) {
        self.tabs = tabs;
        // Reset hover state when tabs change
        self.hovered_tab = None;
        self.hovered_close = None;
    }

    /// Get the height of the tab bar
    #[inline]
    pub fn height(&self) -> f32 {
        self.style.height
    }

    /// Get style reference
    #[inline]
    pub fn style(&self) -> &TabBarStyle {
        &self.style
    }

    /// Set width of the tab bar
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    /// Compute tab layouts based on current tabs and width
    fn compute_layouts(&mut self, ctx: &mut RenderContext) {
        self.tab_layouts.clear();

        if self.tabs.is_empty() {
            return;
        }

        let available_width = self.width - self.style.tab_gap;
        let num_tabs = self.tabs.len();

        // Calculate tab width (fit all tabs, respecting min/max)
        let ideal_width = available_width / num_tabs as f32 - self.style.tab_gap;
        let tab_width = ideal_width
            .min(self.style.max_tab_width)
            .max(self.style.min_tab_width);

        let mut x = self.style.tab_gap;

        for tab in &self.tabs {
            let bounds = Rect::new(x, 0.0, tab_width, self.style.height);

            // Close button in top-right corner of tab
            let close_size = self.style.close_button_size;
            let close_x = bounds.x + bounds.width - self.style.tab_padding - close_size;
            let close_y = (bounds.height - close_size) / 2.0;
            let close_button_bounds = Rect::new(close_x, close_y, close_size, close_size);

            // Truncate title if needed
            let max_text_width =
                tab_width - self.style.tab_padding * 2.0 - close_size - self.style.tab_gap;
            let title = self.truncate_title(&tab.title, max_text_width, ctx);

            self.tab_layouts.push(TabLayout {
                id: tab.id,
                bounds,
                close_button_bounds,
                title,
                is_modified: tab.is_modified,
                is_active: tab.is_active,
            });

            x += tab_width + self.style.tab_gap;
        }
    }

    /// Truncate title to fit within max width, adding ellipsis if needed
    fn truncate_title(&self, title: &str, max_width: f32, ctx: &mut RenderContext) -> String {
        let full_width = ctx.text_width(title);
        if full_width <= max_width {
            return title.to_string();
        }

        let ellipsis = "…";
        let ellipsis_width = ctx.text_width(ellipsis);
        let target_width = max_width - ellipsis_width;

        let mut result = String::new();
        let mut width = 0.0;

        for ch in title.chars() {
            let char_width = ctx.text_width(&ch.to_string());
            if width + char_width > target_width {
                break;
            }
            result.push(ch);
            width += char_width;
        }

        result.push_str(ellipsis);
        result
    }

    /// Render the tab bar
    pub fn render(&mut self, bounds: Rect, ctx: &mut RenderContext) -> RenderList {
        self.width = bounds.width;
        self.compute_layouts(ctx);

        let mut list = RenderList::with_capacity(self.tabs.len() * 4);

        // Tab bar background
        list.push(Primitive::rect(
            bounds,
            self.style.background_color,
            crate::ui::primitives::z_index::BACKGROUND,
        ));

        // Bottom border/separator
        list.push(Primitive::rect(
            Rect::new(bounds.x, bounds.bottom() - 1.0, bounds.width, 1.0),
            [0.25, 0.25, 0.25, 1.0],
            crate::ui::primitives::z_index::GUTTER,
        ));

        // Render each tab
        for (idx, layout) in self.tab_layouts.iter().enumerate() {
            let tab_bounds = Rect::new(
                bounds.x + layout.bounds.x,
                bounds.y + layout.bounds.y,
                layout.bounds.width,
                layout.bounds.height,
            );

            // Determine background color
            let bg_color = if layout.is_active {
                self.style.active_tab_bg
            } else if self.hovered_tab == Some(idx) {
                self.style.hovered_tab_bg
            } else {
                self.style.inactive_tab_bg
            };

            // Tab background
            list.push(Primitive::rounded_rect(
                tab_bounds,
                bg_color,
                self.style.tab_corner_radius,
                crate::ui::primitives::z_index::GUTTER,
            ));

            // Active tab indicator (bottom highlight)
            if layout.is_active {
                list.push(Primitive::rect(
                    Rect::new(
                        tab_bounds.x,
                        tab_bounds.bottom() - 2.0,
                        tab_bounds.width,
                        2.0,
                    ),
                    [0.0, 0.48, 0.8, 1.0], // Accent color
                    crate::ui::primitives::z_index::SELECTION,
                ));
            }

            // Modified indicator (dot before title)
            let text_x = if layout.is_modified {
                let dot_x = tab_bounds.x + self.style.tab_padding;
                let dot_y = tab_bounds.y + (tab_bounds.height - 6.0) / 2.0;
                list.push(Primitive::rounded_rect(
                    Rect::new(dot_x, dot_y, 6.0, 6.0),
                    self.style.modified_indicator_color,
                    3.0,
                    crate::ui::primitives::z_index::TEXT,
                ));
                dot_x + 10.0
            } else {
                tab_bounds.x + self.style.tab_padding
            };

            // Tab title
            let text_y = tab_bounds.y + (tab_bounds.height - ctx.line_height) / 2.0;
            list.push(Primitive::text(
                &layout.title,
                Point::new(text_x, text_y),
                self.style.text_color,
                crate::ui::primitives::z_index::TEXT,
            ));

            // Close button
            let close_bounds = Rect::new(
                bounds.x + layout.close_button_bounds.x,
                bounds.y + layout.close_button_bounds.y,
                layout.close_button_bounds.width,
                layout.close_button_bounds.height,
            );

            let close_color = if self.hovered_close == Some(idx) {
                self.style.close_button_hover_color
            } else {
                self.style.close_button_color
            };

            // Draw X for close button
            let cx = close_bounds.x + close_bounds.width / 2.0;
            let cy = close_bounds.y + close_bounds.height / 2.0;
            let half = close_bounds.width / 4.0;

            // Background circle on hover
            if self.hovered_close == Some(idx) {
                list.push(Primitive::rounded_rect(
                    close_bounds,
                    [0.3, 0.3, 0.3, 1.0],
                    close_bounds.width / 2.0,
                    crate::ui::primitives::z_index::CURSOR,
                ));
            }

            // X symbol (two lines)
            list.push(Primitive::text(
                "×",
                Point::new(cx - half, cy - ctx.line_height / 2.0),
                close_color,
                crate::ui::primitives::z_index::LINE_NUMBERS,
            ));
        }

        list
    }

    /// Handle mouse move event
    pub fn handle_mouse_move(&mut self, position: Point, bounds: Rect) -> TabBarAction {
        // Adjust position to tab bar local coordinates
        let local_x = position.x - bounds.x;
        let local_y = position.y - bounds.y;
        let local_pos = Point::new(local_x, local_y);

        // Check if within tab bar bounds
        if local_y < 0.0 || local_y > self.style.height {
            let changed = self.hovered_tab.is_some() || self.hovered_close.is_some();
            self.hovered_tab = None;
            self.hovered_close = None;
            return if changed {
                TabBarAction::Redraw
            } else {
                TabBarAction::None
            };
        }

        let old_hovered = self.hovered_tab;
        let old_close = self.hovered_close;

        self.hovered_tab = None;
        self.hovered_close = None;

        for (idx, layout) in self.tab_layouts.iter().enumerate() {
            if layout.bounds.contains(local_pos) {
                self.hovered_tab = Some(idx);

                // Check close button
                if layout.close_button_bounds.contains(local_pos) {
                    self.hovered_close = Some(idx);
                }
                break;
            }
        }

        if self.hovered_tab != old_hovered || self.hovered_close != old_close {
            TabBarAction::Redraw
        } else {
            TabBarAction::None
        }
    }

    /// Handle mouse click event
    pub fn handle_mouse_click(&mut self, position: Point, bounds: Rect) -> TabBarAction {
        let local_x = position.x - bounds.x;
        let local_y = position.y - bounds.y;
        let local_pos = Point::new(local_x, local_y);

        // Check if within tab bar
        if local_y < 0.0 || local_y > self.style.height {
            return TabBarAction::None;
        }

        for layout in &self.tab_layouts {
            if layout.bounds.contains(local_pos) {
                // Check close button first
                if layout.close_button_bounds.contains(local_pos) {
                    return TabBarAction::CloseTab(layout.id);
                }

                // Click on tab - select it
                if !layout.is_active {
                    return TabBarAction::SelectTab(layout.id);
                }

                return TabBarAction::None;
            }
        }

        TabBarAction::None
    }

    /// Check if a point is within the tab bar bounds
    pub fn contains(&self, position: Point, bounds: Rect) -> bool {
        bounds.contains(position)
    }

    /// Get tab at position (for context menu etc.)
    pub fn tab_at_position(&self, position: Point, bounds: Rect) -> Option<DocumentId> {
        let local_x = position.x - bounds.x;
        let local_y = position.y - bounds.y;
        let local_pos = Point::new(local_x, local_y);

        for layout in &self.tab_layouts {
            if layout.bounds.contains(local_pos) {
                return Some(layout.id);
            }
        }
        None
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_new() {
        let tab_bar = TabBar::new();
        assert!(tab_bar.tabs.is_empty());
        assert_eq!(tab_bar.height(), 32.0);
    }

    #[test]
    fn test_tab_bar_style_default() {
        let style = TabBarStyle::default();
        assert_eq!(style.height, 32.0);
        assert_eq!(style.max_tab_width, 200.0);
        assert_eq!(style.min_tab_width, 100.0);
    }

    #[test]
    fn test_tab_bar_set_tabs() {
        let mut tab_bar = TabBar::new();
        let tabs = vec![TabInfo {
            id: DocumentId::new(),
            title: "test.rs".to_string(),
            is_modified: false,
            is_active: true,
            file_path: None,
        }];

        tab_bar.set_tabs(tabs);
        assert_eq!(tab_bar.tabs.len(), 1);
    }

    #[test]
    fn test_tab_bar_action_variants() {
        let action = TabBarAction::NewTab;
        assert_eq!(action, TabBarAction::NewTab);
    }
}
