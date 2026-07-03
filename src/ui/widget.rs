//! Widget trait and supporting types for UI components.
//!
//! Widgets are interactive UI components that can render themselves as primitives
//! and handle input events.

use crate::ports::clipboard_port::Clipboard;
use crate::renderer::glyph_cache::GlyphAtlas;
use crate::renderer::layout::Colors;
use crate::ui::primitives::{Point, Rect, RenderList};
use winit::keyboard::{KeyCode, ModifiersState};

/// Context provided during rendering
pub struct RenderContext<'a> {
    pub colors: &'a Colors,
    pub char_width: f32,
    pub line_height: f32,
    pub scale_factor: f32,
    pub glyph_atlas: &'a mut GlyphAtlas,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        colors: &'a Colors,
        char_width: f32,
        line_height: f32,
        scale_factor: f32,
        glyph_atlas: &'a mut GlyphAtlas,
    ) -> Self {
        Self {
            colors,
            char_width,
            line_height,
            scale_factor,
            glyph_atlas,
        }
    }

    /// Calculate text width for a string
    pub fn text_width(&mut self, text: &str) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            width += self.glyph_atlas.char_advance_width(ch);
        }
        width
    }
}

/// Context provided during event handling
pub struct EventContext<'a> {
    pub clipboard: &'a mut dyn Clipboard,
    pub modifiers: ModifiersState,
}

impl<'a> EventContext<'a> {
    pub fn new(clipboard: &'a mut dyn Clipboard, modifiers: ModifiersState) -> Self {
        Self {
            clipboard,
            modifiers,
        }
    }

    pub fn ctrl_or_cmd(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.modifiers.super_key()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.modifiers.control_key()
        }
    }

    pub fn shift(&self) -> bool {
        self.modifiers.shift_key()
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Events that widgets can handle
#[derive(Debug, Clone)]
pub enum WidgetEvent {
    /// A key was pressed
    KeyPress {
        key: KeyCode,
        modifiers: ModifiersState,
    },
    /// A character was typed
    CharInput { char: char },
    /// Mouse button pressed
    MousePress {
        position: Point,
        button: MouseButton,
    },
    /// Mouse button released
    MouseRelease {
        position: Point,
        button: MouseButton,
    },
    /// Mouse moved
    MouseMove { position: Point },
    /// Widget gained focus
    Focus,
    /// Widget lost focus
    Blur,
}

/// Actions that widgets can emit
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetAction {
    /// No action
    None,
    /// Widget needs redraw
    Redraw,
    /// Close the widget/modal
    Close,
    /// Custom action with data
    Custom(String),
    /// Find next match
    FindNext,
    /// Find previous match
    FindPrev,
    /// Replace current match
    Replace,
    /// Replace all matches
    ReplaceAll,
    /// Update search query
    UpdateQuery,
}

/// Trait for interactive UI widgets
pub trait Widget {
    /// Render the widget as primitives
    fn render(&self, bounds: Rect, ctx: &mut RenderContext) -> RenderList;

    /// Handle an input event
    /// Returns true if the event was consumed
    fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        ctx: &mut EventContext,
    ) -> (bool, WidgetAction);

    /// Get the preferred size of the widget
    fn preferred_size(&self, ctx: &mut RenderContext) -> (f32, f32);

    /// Check if the widget is focusable
    fn is_focusable(&self) -> bool {
        true
    }

    /// Check if the widget is currently focused
    fn is_focused(&self) -> bool {
        false
    }

    /// Set the focus state
    fn set_focused(&mut self, _focused: bool) {}
}

/// A simple container that can hold multiple widgets
pub struct WidgetContainer {
    widgets: Vec<(Rect, Box<dyn Widget>)>,
    focused_index: Option<usize>,
}

impl WidgetContainer {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            focused_index: None,
        }
    }

    pub fn add(&mut self, bounds: Rect, widget: Box<dyn Widget>) {
        self.widgets.push((bounds, widget));
    }

    pub fn render(&self, ctx: &mut RenderContext) -> RenderList {
        let mut list = RenderList::new();
        for (bounds, widget) in &self.widgets {
            list.merge(widget.render(*bounds, ctx));
        }
        list
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        ctx: &mut EventContext,
    ) -> (bool, WidgetAction) {
        // First, try the focused widget
        if let Some(idx) = self.focused_index {
            if let Some((bounds, widget)) = self.widgets.get_mut(idx) {
                let (consumed, action) = widget.handle_event(event, *bounds, ctx);
                if consumed {
                    return (true, action);
                }
            }
        }

        // Then try all widgets for mouse events
        if let WidgetEvent::MousePress { position, .. } = event {
            // First pass: find which widget was clicked
            let mut clicked_widget_idx: Option<usize> = None;
            for (i, (bounds, widget)) in self.widgets.iter().enumerate() {
                if bounds.contains(*position) && widget.is_focusable() {
                    clicked_widget_idx = Some(i);
                    break;
                }
            }

            // Second pass: update focus and handle event
            if let Some(new_idx) = clicked_widget_idx {
                // Unfocus old widget if different
                if let Some(old_idx) = self.focused_index {
                    if old_idx != new_idx {
                        if let Some((_, old_widget)) = self.widgets.get_mut(old_idx) {
                            old_widget.set_focused(false);
                        }
                    }
                }

                // Focus and handle event on new widget
                if let Some((bounds, widget)) = self.widgets.get_mut(new_idx) {
                    widget.set_focused(true);
                    self.focused_index = Some(new_idx);
                    let (consumed, action) = widget.handle_event(event, *bounds, ctx);
                    return (consumed, action);
                }
            }
        }

        (false, WidgetAction::None)
    }

    pub fn focus_next(&mut self) {
        if self.widgets.is_empty() {
            return;
        }

        let focusable: Vec<usize> = self
            .widgets
            .iter()
            .enumerate()
            .filter(|(_, (_, w))| w.is_focusable())
            .map(|(i, _)| i)
            .collect();

        if focusable.is_empty() {
            return;
        }

        let current = self.focused_index.unwrap_or(0);
        let next = focusable
            .iter()
            .find(|&&i| i > current)
            .or(focusable.first())
            .copied();

        if let Some(new_idx) = next {
            if let Some(old_idx) = self.focused_index {
                if let Some((_, widget)) = self.widgets.get_mut(old_idx) {
                    widget.set_focused(false);
                }
            }
            if let Some((_, widget)) = self.widgets.get_mut(new_idx) {
                widget.set_focused(true);
            }
            self.focused_index = Some(new_idx);
        }
    }
}

impl Default for WidgetContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_event_variants() {
        let event = WidgetEvent::CharInput { char: 'a' };
        match event {
            WidgetEvent::CharInput { char } => assert_eq!(char, 'a'),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_widget_action_variants() {
        let action = WidgetAction::FindNext;
        assert_eq!(action, WidgetAction::FindNext);
    }

    #[test]
    fn test_widget_container_new() {
        let container = WidgetContainer::new();
        assert!(container.widgets.is_empty());
        assert!(container.focused_index.is_none());
    }
}
