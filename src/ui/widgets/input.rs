//! Input widget - text input field with full editing support

use crate::ports::clipboard_port::Clipboard;
use crate::ui::primitives::{z_index, Color, Point, Primitive, Rect, RenderList};
use crate::ui::widget::{
    EventContext, MouseButton, RenderContext, Widget, WidgetAction, WidgetEvent,
};
use winit::keyboard::KeyCode;

/// A text input field widget
pub struct Input {
    text: String,
    cursor_pos: usize,
    selection: Option<(usize, usize)>,
    placeholder: String,
    focused: bool,
    bg_color: Option<Color>,
    text_color: Option<Color>,
    placeholder_color: Option<Color>,
    cursor_color: Option<Color>,
    selection_color: Option<Color>,
    border_color: Option<Color>,
    corner_radius: f32,
    padding: f32,
    /// Action to emit when content changes
    on_change_action: WidgetAction,
}

impl Input {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            selection: None,
            placeholder: placeholder.into(),
            focused: false,
            bg_color: None,
            text_color: None,
            placeholder_color: None,
            cursor_color: None,
            selection_color: None,
            border_color: None,
            corner_radius: 4.0,
            padding: 8.0,
            on_change_action: WidgetAction::Redraw,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor_pos = self.text.chars().count();
        self
    }

    pub fn with_on_change(mut self, action: WidgetAction) -> Self {
        self.on_change_action = action;
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor_pos = self.text.chars().count();
        self.selection = None;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.selection = None;
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    /// Get selected text if any
    pub fn selected_text(&self) -> Option<String> {
        self.selection.map(|(start, end)| {
            let (s, e) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            self.text.chars().skip(s).take(e - s).collect()
        })
    }

    /// Delete selected text and return to single cursor
    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection.take() {
            let (s, e) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            let before: String = self.text.chars().take(s).collect();
            let after: String = self.text.chars().skip(e).collect();
            self.text = before + &after;
            self.cursor_pos = s;
            return true;
        }
        false
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        self.delete_selection();

        let before: String = self.text.chars().take(self.cursor_pos).collect();
        let after: String = self.text.chars().skip(self.cursor_pos).collect();

        self.text = before + text + &after;
        self.cursor_pos += text.chars().count();
    }

    /// Insert a single character
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection();

        let before: String = self.text.chars().take(self.cursor_pos).collect();
        let after: String = self.text.chars().skip(self.cursor_pos).collect();

        self.text = before + &ch.to_string() + &after;
        self.cursor_pos += 1;
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }

        if self.cursor_pos > 0 {
            let before: String = self.text.chars().take(self.cursor_pos - 1).collect();
            let after: String = self.text.chars().skip(self.cursor_pos).collect();
            self.text = before + &after;
            self.cursor_pos -= 1;
            return true;
        }
        false
    }

    /// Delete character after cursor (delete)
    pub fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }

        let len = self.text.chars().count();
        if self.cursor_pos < len {
            let before: String = self.text.chars().take(self.cursor_pos).collect();
            let after: String = self.text.chars().skip(self.cursor_pos + 1).collect();
            self.text = before + &after;
            return true;
        }
        false
    }

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        if self.cursor_pos > 0 {
            if extend_selection {
                let anchor = self.selection.map(|(s, _)| s).unwrap_or(self.cursor_pos);
                self.cursor_pos -= 1;
                self.selection = Some((anchor, self.cursor_pos));
            } else {
                self.selection = None;
                self.cursor_pos -= 1;
            }
        } else if !extend_selection {
            self.selection = None;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        let len = self.text.chars().count();
        if self.cursor_pos < len {
            if extend_selection {
                let anchor = self.selection.map(|(s, _)| s).unwrap_or(self.cursor_pos);
                self.cursor_pos += 1;
                self.selection = Some((anchor, self.cursor_pos));
            } else {
                self.selection = None;
                self.cursor_pos += 1;
            }
        } else if !extend_selection {
            self.selection = None;
        }
    }

    /// Move cursor to start
    pub fn move_to_start(&mut self, extend_selection: bool) {
        if extend_selection {
            let anchor = self.selection.map(|(s, _)| s).unwrap_or(self.cursor_pos);
            self.cursor_pos = 0;
            self.selection = Some((anchor, 0));
        } else {
            self.selection = None;
            self.cursor_pos = 0;
        }
    }

    /// Move cursor to end
    pub fn move_to_end(&mut self, extend_selection: bool) {
        let len = self.text.chars().count();
        if extend_selection {
            let anchor = self.selection.map(|(s, _)| s).unwrap_or(self.cursor_pos);
            self.cursor_pos = len;
            self.selection = Some((anchor, len));
        } else {
            self.selection = None;
            self.cursor_pos = len;
        }
    }

    /// Select all text
    pub fn select_all(&mut self) {
        let len = self.text.chars().count();
        if len > 0 {
            self.selection = Some((0, len));
            self.cursor_pos = len;
        }
    }

    /// Copy selected text to clipboard
    pub fn copy(&self, clipboard: &mut dyn Clipboard) {
        if let Some(text) = self.selected_text() {
            let _ = clipboard.set_text(&text);
        }
    }

    /// Cut selected text to clipboard
    pub fn cut(&mut self, clipboard: &mut dyn Clipboard) -> bool {
        if let Some(text) = self.selected_text() {
            let _ = clipboard.set_text(&text);
            self.delete_selection();
            return true;
        }
        false
    }

    /// Paste from clipboard
    pub fn paste(&mut self, clipboard: &mut dyn Clipboard) -> bool {
        if let Ok(text) = clipboard.get_text() {
            // Filter out newlines for single-line input
            let text: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
            if !text.is_empty() {
                self.insert_text(&text);
                return true;
            }
        }
        false
    }

    /// Handle keyboard shortcuts (Ctrl+A, Ctrl+C, etc.)
    fn handle_shortcut(&mut self, key: KeyCode, ctx: &mut EventContext) -> Option<WidgetAction> {
        if ctx.ctrl_or_cmd() {
            match key {
                KeyCode::KeyA => {
                    self.select_all();
                    return Some(WidgetAction::Redraw);
                }
                KeyCode::KeyC => {
                    self.copy(ctx.clipboard);
                    return Some(WidgetAction::None);
                }
                KeyCode::KeyX => {
                    if self.cut(ctx.clipboard) {
                        return Some(self.on_change_action.clone());
                    }
                    return Some(WidgetAction::None);
                }
                KeyCode::KeyV => {
                    if self.paste(ctx.clipboard) {
                        return Some(self.on_change_action.clone());
                    }
                    return Some(WidgetAction::None);
                }
                _ => {}
            }
        }
        None
    }

    /// Get cursor X position within text for hit testing
    fn cursor_x_at(&self, char_pos: usize, ctx: &mut RenderContext) -> f32 {
        let prefix: String = self.text.chars().take(char_pos).collect();
        ctx.text_width(&prefix)
    }

    /// Find character position from X coordinate
    fn char_pos_at_x(&self, x: f32, bounds: Rect, ctx: &mut RenderContext) -> usize {
        let text_x = bounds.x + self.padding;
        let relative_x = (x - text_x).max(0.0);

        let mut accumulated = 0.0;
        for (i, ch) in self.text.chars().enumerate() {
            let char_width = ctx.glyph_atlas.char_advance_width(ch);
            if accumulated + char_width / 2.0 > relative_x {
                return i;
            }
            accumulated += char_width;
        }

        self.text.chars().count()
    }
}

impl Widget for Input {
    fn render(&self, bounds: Rect, ctx: &mut RenderContext) -> RenderList {
        let mut list = RenderList::new();

        // Background
        let bg_color = self.bg_color.unwrap_or(ctx.colors.input_background);
        if self.corner_radius > 0.0 {
            list.push(Primitive::rounded_rect(
                bounds,
                bg_color,
                self.corner_radius,
                z_index::MODAL_INPUT_BG,
            ));
        } else {
            list.push(Primitive::rect(bounds, bg_color, z_index::MODAL_INPUT_BG));
        }

        // Border when focused
        if self.focused {
            let border_color = self.border_color.unwrap_or(ctx.colors.input_border_focused);
            list.push(Primitive::border(
                bounds,
                border_color,
                1.0,
                self.corner_radius,
                z_index::MODAL_INPUT_BG + 1,
            ));
        }

        let text_x = bounds.x + self.padding;
        let text_y = bounds.y + (bounds.height - ctx.line_height) / 2.0;

        // Selection highlight
        if let Some((start, end)) = self.selection {
            let (s, e) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            let sel_start_x = text_x + self.cursor_x_at(s, ctx);
            let sel_end_x = text_x + self.cursor_x_at(e, ctx);

            let selection_color = self.selection_color.unwrap_or(ctx.colors.selection_color);
            list.push(Primitive::rect(
                Rect::new(
                    sel_start_x,
                    text_y,
                    sel_end_x - sel_start_x,
                    ctx.line_height,
                ),
                selection_color,
                z_index::MODAL_INPUT_SELECTION,
            ));
        }

        // Text or placeholder
        if self.text.is_empty() {
            let placeholder_color = self.placeholder_color.unwrap_or([0.5, 0.5, 0.5, 1.0]);
            list.push(Primitive::text(
                &self.placeholder,
                Point::new(text_x, text_y),
                placeholder_color,
                z_index::MODAL_INPUT_TEXT,
            ));
        } else {
            let text_color = self.text_color.unwrap_or(ctx.colors.text_color);
            list.push(Primitive::text(
                &self.text,
                Point::new(text_x, text_y),
                text_color,
                z_index::MODAL_INPUT_TEXT,
            ));
        }

        // Cursor (only when focused)
        if self.focused {
            let cursor_x = text_x + self.cursor_x_at(self.cursor_pos, ctx);
            let cursor_color = self.cursor_color.unwrap_or(ctx.colors.cursor_color);
            list.push(Primitive::rect(
                Rect::new(cursor_x, text_y, 2.0, ctx.line_height),
                cursor_color,
                z_index::MODAL_INPUT_CURSOR,
            ));
        }

        list
    }

    fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        ctx: &mut EventContext,
    ) -> (bool, WidgetAction) {
        match event {
            WidgetEvent::Focus => {
                self.focused = true;
                return (true, WidgetAction::Redraw);
            }
            WidgetEvent::Blur => {
                self.focused = false;
                return (true, WidgetAction::Redraw);
            }
            WidgetEvent::CharInput { char } => {
                if self.focused && !char.is_control() {
                    self.insert_char(*char);
                    return (true, self.on_change_action.clone());
                }
            }
            WidgetEvent::KeyPress { key, modifiers } => {
                if !self.focused {
                    return (false, WidgetAction::None);
                }

                // Create a temporary context for shortcut handling
                let mut temp_ctx = EventContext {
                    clipboard: ctx.clipboard,
                    modifiers: *modifiers,
                };

                // Handle shortcuts first
                if let Some(action) = self.handle_shortcut(*key, &mut temp_ctx) {
                    return (true, action);
                }

                let shift = modifiers.shift_key();

                match key {
                    KeyCode::Backspace => {
                        if self.backspace() {
                            return (true, self.on_change_action.clone());
                        }
                    }
                    KeyCode::Delete => {
                        if self.delete() {
                            return (true, self.on_change_action.clone());
                        }
                    }
                    KeyCode::ArrowLeft => {
                        self.move_left(shift);
                        return (true, WidgetAction::Redraw);
                    }
                    KeyCode::ArrowRight => {
                        self.move_right(shift);
                        return (true, WidgetAction::Redraw);
                    }
                    KeyCode::Home => {
                        self.move_to_start(shift);
                        return (true, WidgetAction::Redraw);
                    }
                    KeyCode::End => {
                        self.move_to_end(shift);
                        return (true, WidgetAction::Redraw);
                    }
                    _ => {}
                }
            }
            WidgetEvent::MousePress {
                position,
                button: MouseButton::Left,
            } if bounds.contains(*position) => {
                self.focused = true;
                // TODO: Implement click-to-position cursor using RenderContext
                // For now, move cursor to end
                self.cursor_pos = self.text.chars().count();
                self.selection = None;
                return (true, WidgetAction::Redraw);
            }
            _ => {}
        }

        (false, WidgetAction::None)
    }

    fn preferred_size(&self, ctx: &mut RenderContext) -> (f32, f32) {
        let text_width = if self.text.is_empty() {
            ctx.text_width(&self.placeholder)
        } else {
            ctx.text_width(&self.text)
        };
        (
            text_width + self.padding * 2.0,
            ctx.line_height + self.padding * 2.0,
        )
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_new() {
        let input = Input::new("Enter text...");
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor_pos(), 0);
    }

    #[test]
    fn test_input_with_text() {
        let input = Input::new("").with_text("hello");
        assert_eq!(input.text(), "hello");
        assert_eq!(input.cursor_pos(), 5);
    }

    #[test]
    fn test_input_insert_char() {
        let mut input = Input::new("");
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        assert_eq!(input.text(), "abc");
        assert_eq!(input.cursor_pos(), 3);
    }

    #[test]
    fn test_input_backspace() {
        let mut input = Input::new("").with_text("hello");
        input.backspace();
        assert_eq!(input.text(), "hell");
        assert_eq!(input.cursor_pos(), 4);
    }

    #[test]
    fn test_input_select_all() {
        let mut input = Input::new("").with_text("hello");
        input.select_all();
        assert_eq!(input.selection(), Some((0, 5)));
    }

    #[test]
    fn test_input_delete_selection() {
        let mut input = Input::new("").with_text("hello");
        input.selection = Some((1, 4)); // Select "ell"
        input.delete_selection();
        assert_eq!(input.text(), "ho");
        assert_eq!(input.cursor_pos(), 1);
    }

    #[test]
    fn test_input_insert_text() {
        let mut input = Input::new("").with_text("hello");
        input.cursor_pos = 2;
        input.insert_text("XX");
        assert_eq!(input.text(), "heXXllo");
    }

    #[test]
    fn test_input_focusable() {
        let input = Input::new("");
        assert!(input.is_focusable());
    }
}
