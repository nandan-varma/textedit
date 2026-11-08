//! Editor service - orchestrates document editing operations.
//!
//! This service now uses DocumentManager internally to support multiple documents
//! while maintaining backward compatibility with the single-document API.

use std::path::PathBuf;

use crate::application::document_manager::{DocumentManager, TabInfo};
use crate::domain::{Buffer, Cursor, Document, DocumentId, Operation, OperationHistory};

/// Editor service with multi-document support via DocumentManager
pub struct EditorService {
    /// Document manager handling multiple documents
    documents: DocumentManager,
    /// UI preferences (global across documents)
    show_line_numbers: bool,
    show_status_bar: bool,
    show_tab_bar: bool,
    /// Find/replace state (persists across documents)
    last_find_query: Option<String>,
    last_replace_text: Option<String>,
}

impl EditorService {
    pub fn new() -> Self {
        let mut documents = DocumentManager::new();
        // Start with one empty document
        documents.new_document();

        Self {
            documents,
            show_line_numbers: true,
            show_status_bar: true,
            show_tab_bar: true,
            last_find_query: None,
            last_replace_text: None,
        }
    }

    // ========================================================================
    // Document Manager delegation
    // ========================================================================

    /// Get the document manager
    #[inline]
    pub fn documents(&self) -> &DocumentManager {
        &self.documents
    }

    /// Get the document manager mutably
    #[inline]
    pub fn documents_mut(&mut self) -> &mut DocumentManager {
        &mut self.documents
    }

    /// Get tab information for UI
    pub fn tabs(&self) -> Vec<TabInfo> {
        self.documents.tabs()
    }

    /// Get number of open documents
    #[inline]
    pub fn document_count(&self) -> usize {
        self.documents.document_count()
    }

    /// Get the active document ID
    #[inline]
    pub fn active_document_id(&self) -> Option<DocumentId> {
        self.documents.active_id()
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) -> bool {
        self.documents.next_tab()
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) -> bool {
        self.documents.prev_tab()
    }

    /// Switch to tab by index
    pub fn select_tab(&mut self, index: usize) -> bool {
        self.documents.set_active_by_index(index)
    }

    /// Switch to tab by document ID
    pub fn select_document(&mut self, id: DocumentId) -> bool {
        self.documents.set_active(id)
    }

    /// Create a new empty document
    pub fn new_document(&mut self) -> DocumentId {
        self.documents.new_document()
    }

    /// Open a file (or switch to it if already open)
    pub fn open_file(&mut self, path: PathBuf, content: String) -> DocumentId {
        self.documents.open_file(path, content)
    }

    /// Close the current document
    /// Returns the closed document if there was one
    pub fn close_current_document(&mut self) -> Option<Document> {
        self.documents.close_active()
    }

    /// Close a specific document by ID
    pub fn close_document(&mut self, id: DocumentId) -> Option<Document> {
        self.documents.close_document(id)
    }

    /// Check if any document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.documents.has_unsaved_changes()
    }

    // ========================================================================
    // Active document accessors (backward compatible API)
    // ========================================================================

    /// Get the active document's buffer (immutable)
    pub fn buffer(&self) -> &Buffer {
        self.documents
            .active_document()
            .map(|d| d.buffer())
            .expect("No active document")
    }

    /// Get the active document's buffer (mutable)
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.documents
            .active_document_mut()
            .map(|d| d.buffer_mut())
            .expect("No active document")
    }

    /// Get the active document's cursor (immutable)
    pub fn cursor(&self) -> &Cursor {
        self.documents
            .active_document()
            .map(|d| d.cursor())
            .expect("No active document")
    }

    /// Get the active document's cursor (mutable)
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        self.documents
            .active_document_mut()
            .map(|d| d.cursor_mut())
            .expect("No active document")
    }

    /// Get the active document's history (mutable)
    pub fn history_mut(&mut self) -> &mut OperationHistory {
        self.documents
            .active_document_mut()
            .map(|d| d.history_mut())
            .expect("No active document")
    }

    /// Set the active document's file path
    pub fn set_file_path(&mut self, path: String) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.set_file_path(PathBuf::from(path));
        }
    }

    /// Get the active document's file path
    pub fn file_path(&self) -> Option<&str> {
        self.documents
            .active_document()
            .and_then(|d| d.file_path())
            .and_then(|p| p.to_str())
    }

    /// Check if the active document is modified
    pub fn is_modified(&self) -> bool {
        self.documents
            .active_document()
            .map(|d| d.is_modified())
            .unwrap_or(false)
    }

    /// Set the active document's modified state
    pub fn set_modified(&mut self, modified: bool) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.set_modified(modified);
        }
    }

    /// Get the active document's scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.documents
            .active_document()
            .map(|d| d.scroll_offset())
            .unwrap_or(0)
    }

    /// Set the active document's scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.set_scroll_offset(offset);
        }
    }

    /// Get the active document's title
    pub fn title(&self) -> &str {
        self.documents
            .active_document()
            .map(|d| d.title())
            .unwrap_or("Untitled")
    }

    /// Get the active document's language
    pub fn language(&self) -> Option<&str> {
        self.documents.active_document().and_then(|d| d.language())
    }

    // ========================================================================
    // UI preferences
    // ========================================================================

    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    pub fn show_status_bar(&self) -> bool {
        self.show_status_bar
    }

    pub fn toggle_status_bar(&mut self) {
        self.show_status_bar = !self.show_status_bar;
    }

    pub fn show_tab_bar(&self) -> bool {
        self.show_tab_bar
    }

    pub fn toggle_tab_bar(&mut self) {
        self.show_tab_bar = !self.show_tab_bar;
    }

    // ========================================================================
    // Find/Replace functionality
    // ========================================================================

    pub fn find_next(&mut self) -> bool {
        let query = self.last_find_query.clone().unwrap_or_default();
        if query.is_empty() {
            return false;
        }

        if let Some(doc) = self.documents.active_document_mut() {
            let from = doc.cursor().position().min(doc.buffer().len_chars());
            if let Some((s, e)) = find_next_range(doc.buffer(), &query, from) {
                doc.cursor_mut().select_range(s, e);
                return true;
            }
        }
        false
    }

    pub fn find_prev(&mut self) -> bool {
        let query = self.last_find_query.clone().unwrap_or_default();
        if query.is_empty() {
            return false;
        }

        if let Some(doc) = self.documents.active_document_mut() {
            let from = doc.cursor().position().min(doc.buffer().len_chars());
            if let Some((s, e)) = find_prev_range(doc.buffer(), &query, from) {
                doc.cursor_mut().select_range(s, e);
                return true;
            }
        }
        false
    }

    pub fn set_find_query(&mut self, query: Option<String>) {
        self.last_find_query = query;
    }

    pub fn find_query(&self) -> Option<&str> {
        self.last_find_query.as_deref()
    }

    pub fn set_replace_text(&mut self, text: Option<String>) {
        self.last_replace_text = text;
    }

    pub fn replace_text(&self) -> Option<&str> {
        self.last_replace_text.as_deref()
    }

    /// Find all matches of the current query in the active buffer
    pub fn find_all_matches(&self) -> Vec<(usize, usize)> {
        let query = match &self.last_find_query {
            Some(q) if !q.is_empty() => q,
            _ => return Vec::new(),
        };

        let buffer = match self.documents.active_document() {
            Some(doc) => doc.buffer(),
            None => return Vec::new(),
        };

        let mut matches = Vec::new();
        let content = buffer.as_str();
        let qlen = query.chars().count();

        let mut byte_pos = 0;
        let mut char_pos = 0;

        while let Some(rel_byte_idx) = content[byte_pos..].find(query) {
            let match_byte_start = byte_pos + rel_byte_idx;
            let chars_before = content[byte_pos..match_byte_start].chars().count();
            let start_char = char_pos + chars_before;

            matches.push((start_char, start_char + qlen));

            let query_bytes = query.len();
            byte_pos = match_byte_start + query_bytes;
            char_pos = start_char + qlen;
        }

        matches
    }

    /// Get the total number of matches for the current query
    #[allow(dead_code)]
    pub fn match_count(&self) -> usize {
        self.find_all_matches().len()
    }

    /// Get the index (1-based) of the current match based on cursor position
    pub fn current_match_index(&self) -> Option<usize> {
        let matches = self.find_all_matches();
        if matches.is_empty() {
            return None;
        }

        let cursor_pos = self.cursor().position();

        for (i, (start, end)) in matches.iter().enumerate() {
            if cursor_pos >= *start && cursor_pos <= *end {
                return Some(i + 1);
            }
            if cursor_pos < *start {
                return Some(i + 1);
            }
        }

        Some(1)
    }

    /// Replace the current selection if it matches the find query
    pub fn replace_current(&mut self) -> bool {
        let query = match &self.last_find_query {
            Some(q) if !q.is_empty() => q.clone(),
            _ => return false,
        };

        let replacement = self.last_replace_text.clone().unwrap_or_default();

        let doc = match self.documents.active_document_mut() {
            Some(d) => d,
            None => return false,
        };

        let selection = match doc.cursor().selection() {
            Some(sel) if !sel.is_empty() => sel,
            _ => return false,
        };

        let (start, end) = selection.range();
        let selected_text: String = doc
            .buffer()
            .as_str()
            .chars()
            .skip(start)
            .take(end - start)
            .collect();

        if selected_text != query {
            return false;
        }

        // Delete the selection
        doc.buffer_mut().remove(start, end - start);
        doc.history_mut().push(Operation::Delete {
            position: start,
            text: selected_text,
        });

        // Insert the replacement
        if !replacement.is_empty() {
            doc.buffer_mut().insert(start, &replacement);
            doc.history_mut().push(Operation::Insert {
                position: start,
                text: replacement.clone(),
            });
        }

        // Position cursor after the replacement
        doc.cursor_mut()
            .set_position(start + replacement.chars().count());

        true
    }

    /// Replace the current match and find the next one
    pub fn replace_and_find_next(&mut self) -> bool {
        let replaced = self.replace_current();
        if replaced {
            self.find_next();
        }
        replaced
    }

    /// Replace all occurrences of the query with the replacement text
    pub fn replace_all(&mut self) -> usize {
        let query = match &self.last_find_query {
            Some(q) if !q.is_empty() => q.clone(),
            _ => return 0,
        };

        let replacement = self.last_replace_text.clone().unwrap_or_default();
        let qlen = query.chars().count();
        let rlen = replacement.chars().count();

        let mut matches = self.find_all_matches();
        if matches.is_empty() {
            return 0;
        }

        matches.reverse();

        let doc = match self.documents.active_document_mut() {
            Some(d) => d,
            None => return 0,
        };

        for (start, end) in &matches {
            let deleted_text: String = doc
                .buffer()
                .as_str()
                .chars()
                .skip(*start)
                .take(*end - *start)
                .collect();

            doc.buffer_mut().remove(*start, qlen);
            doc.history_mut().push(Operation::Delete {
                position: *start,
                text: deleted_text,
            });

            if !replacement.is_empty() {
                doc.buffer_mut().insert(*start, &replacement);
                doc.history_mut().push(Operation::Insert {
                    position: *start,
                    text: replacement.clone(),
                });
            }
        }

        let count = matches.len();

        if let Some((start, _)) = matches.last() {
            doc.cursor_mut().set_position(*start + rlen);
        }

        count
    }

    // ========================================================================
    // Document operations (backward compatible)
    // ========================================================================

    pub fn new_file(&mut self) {
        // Create a new document and switch to it
        self.documents.new_document();
    }

    pub fn load_content(&mut self, content: String) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.load_content(&content);
        }
    }

    // ========================================================================
    // Cursor movement helpers
    // ========================================================================

    #[inline]
    pub fn cursor_move_up(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            // Use raw pointer to avoid borrow conflict
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_up(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_down(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_down(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_to_line_start(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_to_line_start(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_to_line_end(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_to_line_end(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_to_word_start(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_to_word_start(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_to_word_end(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().move_to_word_end(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_extend_selection_up(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().extend_selection_up(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_extend_selection_down(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().extend_selection_down(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_extend_selection_to_word_start(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut()
                    .extend_selection_to_word_start(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_extend_selection_to_word_end(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().extend_selection_to_word_end(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_select_word(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().select_word_at_cursor(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_select_line(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().select_line(&*buffer_ptr);
            }
        }
    }

    // ========================================================================
    // Text editing operations (for decoupled KeyAction handling)
    // ========================================================================

    /// Insert a character at cursor position, deleting any selection first
    pub fn insert_char(&mut self, ch: char) {
        self.delete_selection_if_any();
        if let Some(doc) = self.documents.active_document_mut() {
            let pos = doc.cursor().position();
            let text = ch.to_string();
            doc.buffer_mut().insert(pos, &text);
            doc.cursor_mut().set_position(pos + 1);
            doc.history_mut().push(Operation::Insert {
                position: pos,
                text,
            });
            doc.set_modified(true);
        }
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) {
        if self.delete_selection_if_any() {
            return;
        }
        if let Some(doc) = self.documents.active_document_mut() {
            let pos = doc.cursor().position();
            if pos > 0 {
                if let Some(ch) = doc.buffer().get_char(pos - 1) {
                    doc.buffer_mut().remove(pos - 1, 1);
                    doc.cursor_mut().set_position(pos - 1);
                    doc.history_mut().push(Operation::Delete {
                        position: pos - 1,
                        text: ch.to_string(),
                    });
                    doc.set_modified(true);
                }
            }
        }
    }

    /// Delete character after cursor (delete key)
    pub fn delete_forward(&mut self) {
        if self.delete_selection_if_any() {
            return;
        }
        if let Some(doc) = self.documents.active_document_mut() {
            let pos = doc.cursor().position();
            let len = doc.buffer().len_chars();
            if pos < len {
                if let Some(ch) = doc.buffer().get_char(pos) {
                    doc.buffer_mut().remove(pos, 1);
                    doc.history_mut().push(Operation::Delete {
                        position: pos,
                        text: ch.to_string(),
                    });
                    doc.set_modified(true);
                }
            }
        }
    }

    /// Helper: Delete selection if there is one, returns true if deleted
    fn delete_selection_if_any(&mut self) -> bool {
        if let Some(doc) = self.documents.active_document_mut() {
            if let Some(sel) = doc.cursor().selection() {
                if !sel.is_empty() {
                    let (start, end) = sel.range();
                    let text: String = doc
                        .buffer()
                        .as_str()
                        .chars()
                        .skip(start)
                        .take(end - start)
                        .collect();
                    doc.buffer_mut().remove(start, end - start);
                    doc.cursor_mut().set_position(start);
                    doc.history_mut().push(Operation::Delete {
                        position: start,
                        text,
                    });
                    doc.set_modified(true);
                    return true;
                }
            }
        }
        false
    }

    // ========================================================================
    // Simple cursor movement (no buffer reference needed)
    // ========================================================================

    #[inline]
    pub fn cursor_move_left(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.cursor_mut().move_backward();
        }
    }

    #[inline]
    pub fn cursor_move_right(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let len = doc.buffer().len_chars();
            doc.cursor_mut().move_forward(len);
        }
    }

    #[inline]
    pub fn cursor_move_left_select(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.cursor_mut().extend_selection_backward();
        }
    }

    #[inline]
    pub fn cursor_move_right_select(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let len = doc.buffer().len_chars();
            doc.cursor_mut().extend_selection_forward(len);
        }
    }

    #[inline]
    pub fn cursor_move_up_select(&mut self) {
        self.cursor_extend_selection_up();
    }

    #[inline]
    pub fn cursor_move_down_select(&mut self) {
        self.cursor_extend_selection_down();
    }

    #[inline]
    pub fn cursor_move_home(&mut self) {
        self.cursor_move_to_line_start();
        if let Some(doc) = self.documents.active_document_mut() {
            doc.cursor_mut().clear_selection();
        }
    }

    #[inline]
    pub fn cursor_move_end(&mut self) {
        self.cursor_move_to_line_end();
        if let Some(doc) = self.documents.active_document_mut() {
            doc.cursor_mut().clear_selection();
        }
    }

    #[inline]
    pub fn cursor_move_home_select(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut()
                    .extend_selection_to_line_start(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_end_select(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let buffer = doc.buffer();
            let buffer_ptr = buffer as *const Buffer;
            unsafe {
                doc.cursor_mut().extend_selection_to_line_end(&*buffer_ptr);
            }
        }
    }

    #[inline]
    pub fn cursor_move_word_left(&mut self) {
        self.cursor_move_to_word_start();
    }

    #[inline]
    pub fn cursor_move_word_right(&mut self) {
        self.cursor_move_to_word_end();
    }

    #[inline]
    pub fn cursor_move_word_left_select(&mut self) {
        self.cursor_extend_selection_to_word_start();
    }

    #[inline]
    pub fn cursor_move_word_right_select(&mut self) {
        self.cursor_extend_selection_to_word_end();
    }

    #[inline]
    pub fn cursor_move_to_document_start(&mut self, extend_selection: bool) {
        if let Some(doc) = self.documents.active_document_mut() {
            if extend_selection {
                doc.cursor_mut().set_selection_start(0);
            } else {
                doc.cursor_mut().set_position(0);
            }
        }
    }

    #[inline]
    pub fn cursor_move_to_document_end(&mut self, extend_selection: bool) {
        if let Some(doc) = self.documents.active_document_mut() {
            let len = doc.buffer().len_chars();
            if extend_selection {
                doc.cursor_mut().set_selection_end(len);
            } else {
                doc.cursor_mut().set_position(len);
            }
        }
    }

    #[inline]
    pub fn cursor_page_up(&mut self) {
        for _ in 0..10 {
            self.cursor_move_up();
        }
    }

    #[inline]
    pub fn cursor_page_down(&mut self) {
        for _ in 0..10 {
            self.cursor_move_down();
        }
    }

    // ========================================================================
    // Selection operations
    // ========================================================================

    /// Select all text in the buffer
    pub fn select_all(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let len = doc.buffer().len_chars();
            if len > 0 {
                doc.cursor_mut().set_selection_start(0);
                doc.cursor_mut().set_selection_end(len);
            }
        }
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            doc.cursor_mut().clear_selection();
        }
    }

    // ========================================================================
    // Clipboard operations
    // ========================================================================

    /// Copy selection to clipboard
    pub fn copy_selection(&mut self) {
        use crate::infrastructure::clipboard::ArboardClipboard;
        use crate::ports::clipboard_port::Clipboard;

        if let Some(doc) = self.documents.active_document() {
            if let Some(sel) = doc.cursor().selection() {
                if !sel.is_empty() {
                    let (start, end) = sel.range();
                    let text: String = doc
                        .buffer()
                        .as_str()
                        .chars()
                        .skip(start)
                        .take(end - start)
                        .collect();
                    let clipboard = ArboardClipboard::new();
                    let _ = clipboard.set_text(&text);
                }
            }
        }
    }

    /// Cut selection to clipboard
    pub fn cut_selection(&mut self) {
        use crate::infrastructure::clipboard::ArboardClipboard;
        use crate::ports::clipboard_port::Clipboard;

        if let Some(doc) = self.documents.active_document() {
            if let Some(sel) = doc.cursor().selection() {
                if !sel.is_empty() {
                    let (start, end) = sel.range();
                    let text: String = doc
                        .buffer()
                        .as_str()
                        .chars()
                        .skip(start)
                        .take(end - start)
                        .collect();
                    let clipboard = ArboardClipboard::new();
                    let _ = clipboard.set_text(&text);
                }
            }
        }
        // Now delete the selection
        self.delete_selection_if_any();
    }

    /// Paste from clipboard
    pub fn paste(&mut self) {
        use crate::infrastructure::clipboard::ArboardClipboard;
        use crate::ports::clipboard_port::Clipboard;

        let clipboard = ArboardClipboard::new();
        if let Ok(text) = clipboard.get_text() {
            // Delete selection first
            self.delete_selection_if_any();

            if let Some(doc) = self.documents.active_document_mut() {
                let pos = doc.cursor().position();
                doc.buffer_mut().insert(pos, &text);
                let new_pos = pos + text.chars().count();
                doc.cursor_mut().set_position(new_pos);
                doc.history_mut().push(Operation::Insert {
                    position: pos,
                    text,
                });
                doc.set_modified(true);
            }
        }
    }

    // ========================================================================
    // Undo/Redo operations
    // ========================================================================

    /// Undo the last operation
    pub fn undo(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            if let Some(op) = doc.history_mut().undo() {
                match op {
                    Operation::Insert { position, text } => {
                        doc.buffer_mut().remove(position, text.len());
                        doc.cursor_mut().set_position(position);
                    }
                    Operation::Delete { position, text } => {
                        doc.buffer_mut().insert(position, &text);
                        doc.cursor_mut().set_position(position + text.len());
                    }
                }
                doc.set_modified(true);
            }
        }
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            if let Some(op) = doc.history_mut().redo() {
                match op {
                    Operation::Insert { position, text } => {
                        doc.buffer_mut().insert(position, &text);
                        doc.cursor_mut().set_position(position + text.len());
                    }
                    Operation::Delete { position, text } => {
                        doc.buffer_mut().remove(position, text.len());
                        doc.cursor_mut().set_position(position);
                    }
                }
                doc.set_modified(true);
            }
        }
    }

    // ========================================================================
    // Indentation operations
    // ========================================================================

    /// Remove indentation from current line
    pub fn unindent(&mut self) {
        if let Some(doc) = self.documents.active_document_mut() {
            let cursor_pos = doc.cursor().position();
            let buffer = doc.buffer();
            let (line, _) = buffer.char_to_line_col(cursor_pos);

            if let Some(line_str) = buffer.line(line) {
                let indent = line_str
                    .chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .count();
                if indent > 0 {
                    let start = buffer.line_to_char(line);
                    let remove = if indent >= 4 { 4 } else { indent };
                    let removed_text: String = line_str.chars().take(remove).collect();

                    doc.buffer_mut().remove(start, remove);
                    doc.history_mut().push(Operation::Delete {
                        position: start,
                        text: removed_text,
                    });

                    let cursor_pos = doc.cursor().position();
                    if cursor_pos > start {
                        doc.cursor_mut()
                            .set_position(cursor_pos.saturating_sub(remove));
                    }
                    doc.set_modified(true);
                }
            }
        }
    }
}

impl Default for EditorService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper functions for find/replace
// ============================================================================

fn find_next_range(buffer: &Buffer, query: &str, from: usize) -> Option<(usize, usize)> {
    let total = buffer.len_chars();
    if query.is_empty() || total == 0 {
        return None;
    }
    let qlen = query.chars().count();
    let from = from.min(total);

    let tail = buffer.rope().slice(from..total).to_string();
    if let Some(byte_idx) = tail.find(query) {
        let char_off = tail[..byte_idx].chars().count();
        let start = from + char_off;
        return Some((start, start + qlen));
    }

    let head = buffer.rope().slice(0..from).to_string();
    if let Some(byte_idx) = head.find(query) {
        let char_off = head[..byte_idx].chars().count();
        return Some((char_off, char_off + qlen));
    }

    None
}

fn find_prev_range(buffer: &Buffer, query: &str, from: usize) -> Option<(usize, usize)> {
    let total = buffer.len_chars();
    if query.is_empty() || total == 0 {
        return None;
    }
    let qlen = query.chars().count();
    let from = from.min(total);

    let head = buffer.rope().slice(0..from).to_string();
    if let Some(byte_idx) = head.rfind(query) {
        let char_off = head[..byte_idx].chars().count();
        return Some((char_off, char_off + qlen));
    }

    let full = buffer.rope().to_string();
    if let Some(byte_idx) = full.rfind(query) {
        let char_off = full[..byte_idx].chars().count();
        return Some((char_off, char_off + qlen));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::EditorService;

    fn create_editor(content: &str) -> EditorService {
        let mut editor = EditorService::new();
        editor.load_content(content.to_string());
        editor
    }

    #[test]
    fn test_editor_service_new() {
        let editor = EditorService::new();
        assert_eq!(editor.buffer().len_chars(), 0);
        assert_eq!(editor.cursor().position(), 0);
        assert!(editor.file_path().is_none());
        assert!(!editor.is_modified());
        assert!(editor.show_line_numbers());
        assert!(editor.show_status_bar());
        assert!(editor.show_tab_bar());
        assert_eq!(editor.document_count(), 1);
    }

    #[test]
    fn test_editor_service_file_path() {
        let mut editor = EditorService::new();
        assert!(editor.file_path().is_none());

        editor.set_file_path("/test/path.txt".to_string());
        assert_eq!(editor.file_path(), Some("/test/path.txt"));
    }

    #[test]
    fn test_editor_service_set_file_path_clears_modified() {
        let mut editor = EditorService::new();
        editor.set_modified(true);
        assert!(editor.is_modified());

        editor.set_file_path("/test/path.txt".to_string());
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_service_is_modified() {
        let mut editor = EditorService::new();
        assert!(!editor.is_modified());

        editor.set_modified(true);
        assert!(editor.is_modified());
    }

    #[test]
    fn test_editor_service_buffer_mut_sets_modified() {
        let mut editor = EditorService::new();
        assert!(!editor.is_modified());

        editor.buffer_mut().insert(0, "test");
        assert!(editor.is_modified());
    }

    #[test]
    fn test_editor_service_toggle_line_numbers() {
        let mut editor = EditorService::new();
        assert!(editor.show_line_numbers());

        editor.toggle_line_numbers();
        assert!(!editor.show_line_numbers());

        editor.toggle_line_numbers();
        assert!(editor.show_line_numbers());
    }

    #[test]
    fn test_editor_service_toggle_status_bar() {
        let mut editor = EditorService::new();
        assert!(editor.show_status_bar());

        editor.toggle_status_bar();
        assert!(!editor.show_status_bar());

        editor.toggle_status_bar();
        assert!(editor.show_status_bar());
    }

    #[test]
    fn test_editor_service_toggle_tab_bar() {
        let mut editor = EditorService::new();
        assert!(editor.show_tab_bar());

        editor.toggle_tab_bar();
        assert!(!editor.show_tab_bar());

        editor.toggle_tab_bar();
        assert!(editor.show_tab_bar());
    }

    #[test]
    fn test_editor_service_set_find_query() {
        let mut editor = EditorService::new();
        assert!(editor.find_query().is_none());

        editor.set_find_query(Some("test".to_string()));
        assert_eq!(editor.find_query(), Some("test"));
    }

    #[test]
    fn test_editor_service_new_file() {
        let mut editor = create_editor("existing content");
        let initial_count = editor.document_count();

        editor.new_file();

        // New file creates a new document
        assert_eq!(editor.document_count(), initial_count + 1);
        // New document should be active and empty
        assert_eq!(editor.buffer().len_chars(), 0);
    }

    #[test]
    fn test_editor_service_load_content() {
        let mut editor = EditorService::new();
        editor.load_content("hello world".to_string());

        assert_eq!(editor.buffer().as_str(), "hello world");
        assert_eq!(editor.cursor().position(), 0);
    }

    #[test]
    fn test_editor_service_find_next_no_query() {
        let mut editor = create_editor("hello world");
        let result = editor.find_next();
        assert!(!result);
    }

    #[test]
    fn test_editor_service_find_next_empty_query() {
        let mut editor = create_editor("hello world");
        editor.set_find_query(Some("".to_string()));
        let result = editor.find_next();
        assert!(!result);
    }

    #[test]
    fn test_editor_service_find_next_finds_match() {
        let mut editor = create_editor("hello world");
        editor.set_find_query(Some("world".to_string()));

        let result = editor.find_next();
        assert!(result);

        let sel = editor.cursor().selection().unwrap();
        let (start, end) = sel.range();
        assert_eq!(start, 6);
        assert_eq!(end, 11);
    }

    #[test]
    fn test_editor_service_find_next_not_found() {
        let mut editor = create_editor("hello world");
        editor.set_find_query(Some("xyz".to_string()));

        let result = editor.find_next();
        assert!(!result);
    }

    #[test]
    fn test_editor_service_find_prev_finds_match() {
        let mut editor = create_editor("hello world hello");
        editor.set_find_query(Some("hello".to_string()));

        let result = editor.find_prev();
        assert!(result);

        let sel = editor.cursor().selection().unwrap();
        let (start, end) = sel.range();
        assert_eq!(start, 12);
        assert_eq!(end, 17);
    }

    #[test]
    fn test_editor_service_find_prev_no_query() {
        let mut editor = create_editor("hello world");
        let result = editor.find_prev();
        assert!(!result);
    }

    #[test]
    fn test_multi_document_support() {
        let mut editor = EditorService::new();
        assert_eq!(editor.document_count(), 1);

        // Create second document
        let id2 = editor.new_document();
        assert_eq!(editor.document_count(), 2);
        assert_eq!(editor.active_document_id(), Some(id2));

        // Create third document
        let _id3 = editor.new_document();
        assert_eq!(editor.document_count(), 3);

        // Navigate tabs
        editor.prev_tab();
        assert_eq!(editor.active_document_id(), Some(id2));

        editor.select_tab(0);
        assert_ne!(editor.active_document_id(), Some(id2));
    }

    #[test]
    fn test_tabs_info() {
        let mut editor = EditorService::new();
        editor.new_document();

        let tabs = editor.tabs();
        assert_eq!(tabs.len(), 2);
    }
}
