//! Reusable text input field component
//!
//! Provides a text input with cursor, selection, and clipboard support.

use winit::keyboard::{KeyCode, ModifiersState};

/// A text input field with cursor and selection support
#[derive(Debug, Clone)]
pub struct InputField {
    /// The text content of the input
    pub text: String,
    /// Cursor position (character index)
    pub cursor_pos: usize,
    /// Selection range (start, end) - None if no selection
    pub selection: Option<(usize, usize)>,
    /// Placeholder text when empty
    pub placeholder: String,
    /// Whether this input is focused
    pub focused: bool,
}

impl InputField {
    /// Create a new empty input field
    pub fn new(placeholder: &str) -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            selection: None,
            placeholder: placeholder.to_string(),
            focused: false,
        }
    }

    /// Create an input field with initial text
    #[allow(dead_code)]
    pub fn with_text(text: &str, placeholder: &str) -> Self {
        let len = text.chars().count();
        Self {
            text: text.to_string(),
            cursor_pos: len,
            selection: None,
            placeholder: placeholder.to_string(),
            focused: false,
        }
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text content and move cursor to end
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor_pos = self.text.chars().count();
        self.selection = None;
    }

    /// Clear the input
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.selection = None;
    }

    /// Get the length in characters
    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Get the selected text if any
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

    /// Get ordered selection range (min, max)
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.map(|(start, end)| {
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }

    /// Delete the current selection
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            if start == end {
                self.selection = None;
                return false;
            }

            let before: String = self.text.chars().take(start).collect();
            let after: String = self.text.chars().skip(end).collect();
            self.text = format!("{}{}", before, after);
            self.cursor_pos = start;
            self.selection = None;
            return true;
        }
        false
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, insert: &str) {
        // Delete selection first if any
        self.delete_selection();

        let before: String = self.text.chars().take(self.cursor_pos).collect();
        let after: String = self.text.chars().skip(self.cursor_pos).collect();
        self.text = format!("{}{}{}", before, insert, after);
        self.cursor_pos += insert.chars().count();
    }

    /// Insert a single character
    pub fn insert_char(&mut self, c: char) {
        self.insert_text(&c.to_string());
    }

    /// Handle a key event
    /// Returns true if the key was handled
    pub fn handle_key(&mut self, code: KeyCode, modifiers: ModifiersState) -> bool {
        let ctrl_or_cmd = modifiers.control_key() || modifiers.super_key();
        let shift = modifiers.shift_key();

        match code {
            KeyCode::ArrowLeft => {
                if ctrl_or_cmd {
                    // Move to start of input
                    if shift {
                        self.extend_selection(0);
                    } else {
                        self.cursor_pos = 0;
                        self.selection = None;
                    }
                } else if shift {
                    // Extend selection left
                    if self.cursor_pos > 0 {
                        self.extend_selection(self.cursor_pos - 1);
                    }
                } else {
                    // Move cursor left
                    if let Some((start, end)) = self.selection_range() {
                        self.cursor_pos = start.min(end);
                        self.selection = None;
                    } else if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                true
            }
            KeyCode::ArrowRight => {
                let len = self.len();
                if ctrl_or_cmd {
                    // Move to end of input
                    if shift {
                        self.extend_selection(len);
                    } else {
                        self.cursor_pos = len;
                        self.selection = None;
                    }
                } else if shift {
                    // Extend selection right
                    if self.cursor_pos < len {
                        self.extend_selection(self.cursor_pos + 1);
                    }
                } else {
                    // Move cursor right
                    if let Some((start, end)) = self.selection_range() {
                        self.cursor_pos = start.max(end);
                        self.selection = None;
                    } else if self.cursor_pos < len {
                        self.cursor_pos += 1;
                    }
                }
                true
            }
            KeyCode::Home => {
                if shift {
                    self.extend_selection(0);
                } else {
                    self.cursor_pos = 0;
                    self.selection = None;
                }
                true
            }
            KeyCode::End => {
                let len = self.len();
                if shift {
                    self.extend_selection(len);
                } else {
                    self.cursor_pos = len;
                    self.selection = None;
                }
                true
            }
            KeyCode::Backspace => {
                if self.delete_selection() {
                    // Deleted selection
                } else if self.cursor_pos > 0 {
                    let before: String = self.text.chars().take(self.cursor_pos - 1).collect();
                    let after: String = self.text.chars().skip(self.cursor_pos).collect();
                    self.text = format!("{}{}", before, after);
                    self.cursor_pos -= 1;
                }
                true
            }
            KeyCode::Delete => {
                if self.delete_selection() {
                    // Deleted selection
                } else if self.cursor_pos < self.len() {
                    let before: String = self.text.chars().take(self.cursor_pos).collect();
                    let after: String = self.text.chars().skip(self.cursor_pos + 1).collect();
                    self.text = format!("{}{}", before, after);
                }
                true
            }
            KeyCode::KeyA if ctrl_or_cmd => {
                // Select all
                self.select_all();
                true
            }
            _ => false,
        }
    }

    /// Handle a character input
    /// Returns true if the character was inserted
    #[allow(dead_code)]
    pub fn handle_char(&mut self, c: char) -> bool {
        if c.is_control() {
            return false;
        }
        self.insert_char(c);
        true
    }

    /// Select all text
    pub fn select_all(&mut self) {
        if !self.text.is_empty() {
            self.selection = Some((0, self.len()));
            self.cursor_pos = self.len();
        }
    }

    /// Extend selection to a position
    fn extend_selection(&mut self, to: usize) {
        let anchor = match self.selection {
            Some((start, _)) => start,
            None => self.cursor_pos,
        };
        self.selection = Some((anchor, to));
        self.cursor_pos = to;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_new() {
        let field = InputField::new("placeholder");
        assert!(field.is_empty());
        assert_eq!(field.cursor_pos, 0);
        assert_eq!(field.placeholder, "placeholder");
    }

    #[test]
    fn test_input_field_insert_text() {
        let mut field = InputField::new("");
        field.insert_text("hello");
        assert_eq!(field.text(), "hello");
        assert_eq!(field.cursor_pos, 5);
    }

    #[test]
    fn test_input_field_insert_char() {
        let mut field = InputField::new("");
        field.insert_char('a');
        field.insert_char('b');
        assert_eq!(field.text(), "ab");
        assert_eq!(field.cursor_pos, 2);
    }

    #[test]
    fn test_input_field_backspace() {
        let mut field = InputField::with_text("hello", "");
        field.handle_key(KeyCode::Backspace, ModifiersState::empty());
        assert_eq!(field.text(), "hell");
        assert_eq!(field.cursor_pos, 4);
    }

    #[test]
    fn test_input_field_delete() {
        let mut field = InputField::with_text("hello", "");
        field.cursor_pos = 0;
        field.handle_key(KeyCode::Delete, ModifiersState::empty());
        assert_eq!(field.text(), "ello");
        assert_eq!(field.cursor_pos, 0);
    }

    #[test]
    fn test_input_field_select_all() {
        let mut field = InputField::with_text("hello", "");
        field.select_all();
        assert_eq!(field.selection, Some((0, 5)));
    }

    #[test]
    fn test_input_field_delete_selection() {
        let mut field = InputField::with_text("hello world", "");
        field.selection = Some((0, 6));
        field.delete_selection();
        assert_eq!(field.text(), "world");
        assert_eq!(field.cursor_pos, 0);
    }
}
