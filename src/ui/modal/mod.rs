//! Modal UI components for the text editor
//!
//! This module provides modal dialogs like Find/Replace that overlay the editor.

pub mod find_modal;
pub mod input_field;

pub use find_modal::{FindField, FindModal};
pub use input_field::InputField;

/// Actions that can result from modal interactions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalAction {
    /// No action needed
    None,
    /// Close the modal
    Close,
    /// Find the next match
    FindNext,
    /// Find the previous match
    FindPrev,
    /// Replace the current match
    Replace,
    /// Replace all matches
    ReplaceAll,
    /// Update the search query (triggers re-search)
    UpdateQuery,
    /// Request redraw
    Redraw,
}

/// Represents the current modal state
#[derive(Debug, Clone)]
pub enum ModalState {
    /// No modal is open
    None,
    /// Find/Replace modal is open
    Find(FindModal),
}

impl Default for ModalState {
    fn default() -> Self {
        Self::None
    }
}

impl ModalState {
    /// Check if any modal is currently open
    pub fn is_open(&self) -> bool {
        !matches!(self, ModalState::None)
    }

    /// Open the find modal
    pub fn open_find(&mut self) {
        *self = ModalState::Find(FindModal::new(false));
    }

    /// Open the find modal with replace expanded
    pub fn open_replace(&mut self) {
        *self = ModalState::Find(FindModal::new(true));
    }

    /// Close any open modal
    pub fn close(&mut self) {
        *self = ModalState::None;
    }

    /// Get the find modal if it's open
    pub fn as_find(&self) -> Option<&FindModal> {
        match self {
            ModalState::Find(modal) => Some(modal),
            _ => None,
        }
    }

    /// Get mutable reference to find modal if it's open
    pub fn as_find_mut(&mut self) -> Option<&mut FindModal> {
        match self {
            ModalState::Find(modal) => Some(modal),
            _ => None,
        }
    }
}
