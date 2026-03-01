//! Find and Replace modal dialog
//!
//! Provides VS Code-style find and replace functionality.

use super::input_field::InputField;
use super::ModalAction;
use winit::keyboard::{KeyCode, ModifiersState};

/// Which field is currently focused in the Find modal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindField {
    Find,
    Replace,
}

/// Button identifiers for hit testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindButton {
    Close,
    FindNext,
    FindPrev,
    Replace,
    ReplaceAll,
    ToggleReplace,
}

/// Find and Replace modal state
#[derive(Debug, Clone)]
pub struct FindModal {
    /// The find input field
    pub find_input: InputField,
    /// The replace input field
    pub replace_input: InputField,
    /// Which field is focused
    pub focused_field: FindField,
    /// Whether the replace section is expanded
    pub show_replace: bool,
    /// Cached match positions (start, end) character indices
    pub matches: Vec<(usize, usize)>,
    /// Index of the current match (0-based)
    pub current_match: Option<usize>,
    /// Total match count for display
    pub match_count: usize,
}

impl FindModal {
    /// Create a new find modal
    pub fn new(show_replace: bool) -> Self {
        let mut find_input = InputField::new("Find");
        find_input.focused = true;

        Self {
            find_input,
            replace_input: InputField::new("Replace"),
            focused_field: FindField::Find,
            show_replace,
            matches: Vec::new(),
            current_match: None,
            match_count: 0,
        }
    }

    /// Get the current search query
    pub fn query(&self) -> &str {
        self.find_input.text()
    }

    /// Get the replacement text
    pub fn replacement(&self) -> &str {
        self.replace_input.text()
    }

    /// Toggle between showing Find only and Find+Replace
    pub fn toggle_replace(&mut self) {
        self.show_replace = !self.show_replace;
        if !self.show_replace && self.focused_field == FindField::Replace {
            self.focus_find();
        }
    }

    /// Focus the find input
    pub fn focus_find(&mut self) {
        self.find_input.focused = true;
        self.replace_input.focused = false;
        self.focused_field = FindField::Find;
    }

    /// Focus the replace input
    pub fn focus_replace(&mut self) {
        if self.show_replace {
            self.find_input.focused = false;
            self.replace_input.focused = true;
            self.focused_field = FindField::Replace;
        }
    }

    /// Toggle focus between find and replace
    pub fn toggle_focus(&mut self) {
        match self.focused_field {
            FindField::Find if self.show_replace => self.focus_replace(),
            _ => self.focus_find(),
        }
    }

    /// Update match information from editor
    pub fn update_matches(&mut self, matches: Vec<(usize, usize)>, current_index: Option<usize>) {
        self.match_count = matches.len();
        self.matches = matches;
        self.current_match = current_index.map(|i| i.saturating_sub(1)); // Convert to 0-based
    }

    /// Get the display string for match count (e.g., "1 of 5" or "No results")
    pub fn match_status(&self) -> String {
        if self.find_input.is_empty() {
            String::new()
        } else if self.match_count == 0 {
            "No results".to_string()
        } else if let Some(current) = self.current_match {
            format!("{} of {}", current + 1, self.match_count)
        } else {
            format!("{} results", self.match_count)
        }
    }

    /// Get the currently focused input field
    pub fn focused_input(&self) -> &InputField {
        match self.focused_field {
            FindField::Find => &self.find_input,
            FindField::Replace => &self.replace_input,
        }
    }

    /// Get mutable reference to the currently focused input field
    pub fn focused_input_mut(&mut self) -> &mut InputField {
        match self.focused_field {
            FindField::Find => &mut self.find_input,
            FindField::Replace => &mut self.replace_input,
        }
    }

    /// Handle a key event
    /// Returns the action to take
    pub fn handle_key(&mut self, code: KeyCode, modifiers: ModifiersState) -> ModalAction {
        let ctrl_or_cmd = modifiers.control_key() || modifiers.super_key();
        let shift = modifiers.shift_key();

        match code {
            KeyCode::Escape => ModalAction::Close,

            KeyCode::Enter => {
                if ctrl_or_cmd && self.focused_field == FindField::Replace {
                    ModalAction::ReplaceAll
                } else if shift {
                    ModalAction::FindPrev
                } else if self.focused_field == FindField::Replace {
                    ModalAction::Replace
                } else {
                    ModalAction::FindNext
                }
            }

            KeyCode::Tab => {
                self.toggle_focus();
                ModalAction::Redraw
            }

            KeyCode::KeyG if ctrl_or_cmd => {
                if shift {
                    ModalAction::FindPrev
                } else {
                    ModalAction::FindNext
                }
            }

            KeyCode::KeyH if ctrl_or_cmd => {
                self.toggle_replace();
                ModalAction::Redraw
            }

            // Let the input field handle navigation keys
            KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::Backspace
            | KeyCode::Delete
            | KeyCode::KeyA
                if ctrl_or_cmd =>
            {
                let handled = self.focused_input_mut().handle_key(code, modifiers);
                if handled && matches!(code, KeyCode::Backspace | KeyCode::Delete) {
                    // Query changed, update search
                    if self.focused_field == FindField::Find {
                        ModalAction::UpdateQuery
                    } else {
                        ModalAction::Redraw
                    }
                } else if handled {
                    ModalAction::Redraw
                } else {
                    ModalAction::None
                }
            }

            _ => {
                // Try handling as navigation in input field
                if self.focused_input_mut().handle_key(code, modifiers) {
                    ModalAction::Redraw
                } else {
                    ModalAction::None
                }
            }
        }
    }

    /// Handle character input
    /// Returns the action to take
    pub fn handle_char(&mut self, c: char) -> ModalAction {
        if c.is_control() {
            return ModalAction::None;
        }

        self.focused_input_mut().insert_char(c);

        // If find field changed, update query
        if self.focused_field == FindField::Find {
            ModalAction::UpdateQuery
        } else {
            ModalAction::Redraw
        }
    }

    /// Handle button click
    #[allow(dead_code)]
    pub fn handle_button(&mut self, button: FindButton) -> ModalAction {
        match button {
            FindButton::Close => ModalAction::Close,
            FindButton::FindNext => ModalAction::FindNext,
            FindButton::FindPrev => ModalAction::FindPrev,
            FindButton::Replace => ModalAction::Replace,
            FindButton::ReplaceAll => ModalAction::ReplaceAll,
            FindButton::ToggleReplace => {
                self.toggle_replace();
                ModalAction::Redraw
            }
        }
    }

    /// Calculate modal height based on whether replace is shown
    #[allow(dead_code)]
    pub fn height(&self, line_height: f32) -> f32 {
        let padding = 8.0;
        let input_height = line_height + padding * 2.0;
        let spacing = 4.0;

        if self.show_replace {
            padding * 2.0 + input_height * 2.0 + spacing
        } else {
            padding * 2.0 + input_height
        }
    }

    /// Calculate modal width
    pub fn width(&self) -> f32 {
        420.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_modal_new() {
        let modal = FindModal::new(false);
        assert!(!modal.show_replace);
        assert_eq!(modal.focused_field, FindField::Find);
        assert!(modal.find_input.focused);
    }

    #[test]
    fn test_find_modal_toggle_replace() {
        let mut modal = FindModal::new(false);
        assert!(!modal.show_replace);

        modal.toggle_replace();
        assert!(modal.show_replace);

        modal.toggle_replace();
        assert!(!modal.show_replace);
    }

    #[test]
    fn test_find_modal_toggle_focus() {
        let mut modal = FindModal::new(true);
        assert_eq!(modal.focused_field, FindField::Find);

        modal.toggle_focus();
        assert_eq!(modal.focused_field, FindField::Replace);

        modal.toggle_focus();
        assert_eq!(modal.focused_field, FindField::Find);
    }

    #[test]
    fn test_find_modal_match_status() {
        let mut modal = FindModal::new(false);
        assert_eq!(modal.match_status(), "");

        modal.find_input.insert_text("test");
        assert_eq!(modal.match_status(), "No results");

        modal.update_matches(vec![(0, 4), (10, 14), (20, 24)], Some(2));
        assert_eq!(modal.match_status(), "2 of 3");
    }

    #[test]
    fn test_find_modal_escape_closes() {
        let mut modal = FindModal::new(false);
        let action = modal.handle_key(KeyCode::Escape, ModifiersState::empty());
        assert_eq!(action, ModalAction::Close);
    }

    #[test]
    fn test_find_modal_enter_finds_next() {
        let mut modal = FindModal::new(false);
        let action = modal.handle_key(KeyCode::Enter, ModifiersState::empty());
        assert_eq!(action, ModalAction::FindNext);
    }
}
