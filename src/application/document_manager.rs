//! Document Manager - manages multiple open documents with tab support.
//!
//! Provides functionality for:
//! - Opening, closing, and switching between documents
//! - Tracking document order for tab display
//! - Managing the active document
//! - Tab navigation (next, previous, by index)

use std::collections::HashMap;
use std::path::PathBuf;

use crate::domain::{Document, DocumentId};

/// Manages multiple open documents
#[derive(Debug)]
pub struct DocumentManager {
    /// All open documents indexed by ID
    documents: HashMap<DocumentId, Document>,
    /// Tab order (document IDs in display order)
    tab_order: Vec<DocumentId>,
    /// Currently active document
    active_id: Option<DocumentId>,
    /// Maximum number of tabs (0 = unlimited)
    max_tabs: usize,
}

impl DocumentManager {
    /// Create a new document manager
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            tab_order: Vec::new(),
            active_id: None,
            max_tabs: 0, // unlimited by default
        }
    }

    /// Create a new document manager with an initial empty document
    pub fn with_initial_document() -> Self {
        let mut manager = Self::new();
        manager.new_document();
        manager
    }

    /// Set maximum number of tabs (0 = unlimited)
    pub fn set_max_tabs(&mut self, max: usize) {
        self.max_tabs = max;
    }

    /// Get the number of open documents
    #[inline]
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Check if there are any open documents
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get the active document ID
    #[inline]
    pub fn active_id(&self) -> Option<DocumentId> {
        self.active_id
    }

    /// Get the active document (immutable)
    pub fn active_document(&self) -> Option<&Document> {
        self.active_id.and_then(|id| self.documents.get(&id))
    }

    /// Get the active document (mutable)
    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.active_id.and_then(|id| self.documents.get_mut(&id))
    }

    /// Get a document by ID
    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    /// Get a document by ID (mutable)
    pub fn document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }

    /// Get the tab order (list of document IDs)
    #[inline]
    pub fn tab_order(&self) -> &[DocumentId] {
        &self.tab_order
    }

    /// Get tab info for UI display
    pub fn tabs(&self) -> Vec<TabInfo> {
        self.tab_order
            .iter()
            .filter_map(|id| {
                self.documents.get(id).map(|doc| TabInfo {
                    id: *id,
                    title: doc.title().to_string(),
                    is_modified: doc.is_modified(),
                    is_active: Some(*id) == self.active_id,
                    file_path: doc.file_path().cloned(),
                })
            })
            .collect()
    }

    /// Get the index of the active tab
    pub fn active_tab_index(&self) -> Option<usize> {
        self.active_id
            .and_then(|id| self.tab_order.iter().position(|&tid| tid == id))
    }

    /// Create a new empty document and make it active
    pub fn new_document(&mut self) -> DocumentId {
        let doc = Document::new();
        let id = doc.id();

        self.documents.insert(id, doc);
        self.tab_order.push(id);
        self.active_id = Some(id);

        // Enforce max tabs if set
        self.enforce_max_tabs();

        id
    }

    /// Open a file as a new document
    /// Returns the document ID, or existing ID if file is already open
    pub fn open_file(&mut self, path: PathBuf, content: String) -> DocumentId {
        // Check if file is already open
        if let Some(id) = self.find_document_by_path(&path) {
            self.set_active(id);
            return id;
        }

        let doc = Document::from_file(path, content);
        let id = doc.id();

        self.documents.insert(id, doc);
        self.tab_order.push(id);
        self.active_id = Some(id);

        // Enforce max tabs if set
        self.enforce_max_tabs();

        id
    }

    /// Find a document by its file path
    pub fn find_document_by_path(&self, path: &PathBuf) -> Option<DocumentId> {
        self.documents
            .iter()
            .find(|(_, doc)| doc.file_path() == Some(path))
            .map(|(id, _)| *id)
    }

    /// Set the active document by ID
    pub fn set_active(&mut self, id: DocumentId) -> bool {
        if self.documents.contains_key(&id) {
            self.active_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Set the active document by tab index
    pub fn set_active_by_index(&mut self, index: usize) -> bool {
        if let Some(&id) = self.tab_order.get(index) {
            self.active_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Switch to the next tab (wraps around)
    pub fn next_tab(&mut self) -> bool {
        if self.tab_order.len() <= 1 {
            return false;
        }

        if let Some(current_idx) = self.active_tab_index() {
            let next_idx = (current_idx + 1) % self.tab_order.len();
            self.active_id = Some(self.tab_order[next_idx]);
            true
        } else {
            false
        }
    }

    /// Switch to the previous tab (wraps around)
    pub fn prev_tab(&mut self) -> bool {
        if self.tab_order.len() <= 1 {
            return false;
        }

        if let Some(current_idx) = self.active_tab_index() {
            let prev_idx = if current_idx == 0 {
                self.tab_order.len() - 1
            } else {
                current_idx - 1
            };
            self.active_id = Some(self.tab_order[prev_idx]);
            true
        } else {
            false
        }
    }

    /// Close a document by ID
    /// Returns the closed document if it existed
    pub fn close_document(&mut self, id: DocumentId) -> Option<Document> {
        let doc = self.documents.remove(&id)?;

        // Remove from tab order
        if let Some(pos) = self.tab_order.iter().position(|&tid| tid == id) {
            self.tab_order.remove(pos);

            // Update active document if we closed the active one
            if self.active_id == Some(id) {
                self.active_id = if self.tab_order.is_empty() {
                    None
                } else {
                    // Try to stay at the same position, or go to previous
                    let new_idx = pos.min(self.tab_order.len().saturating_sub(1));
                    Some(self.tab_order[new_idx])
                };
            }
        }

        Some(doc)
    }

    /// Close the active document
    /// Returns the closed document if there was one
    pub fn close_active(&mut self) -> Option<Document> {
        self.active_id.and_then(|id| self.close_document(id))
    }

    /// Close all documents
    /// Returns documents that had unsaved changes
    pub fn close_all(&mut self) -> Vec<Document> {
        let modified: Vec<Document> = self
            .documents
            .drain()
            .filter_map(|(_, doc)| if doc.is_modified() { Some(doc) } else { None })
            .collect();

        self.tab_order.clear();
        self.active_id = None;

        modified
    }

    /// Move a tab from one position to another
    pub fn move_tab(&mut self, from: usize, to: usize) -> bool {
        if from >= self.tab_order.len() || to >= self.tab_order.len() {
            return false;
        }

        let id = self.tab_order.remove(from);
        self.tab_order.insert(to, id);
        true
    }

    /// Get all documents that have unsaved changes
    pub fn modified_documents(&self) -> Vec<&Document> {
        self.documents
            .values()
            .filter(|doc| doc.is_modified())
            .collect()
    }

    /// Check if any document has unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.documents.values().any(|doc| doc.is_modified())
    }

    /// Enforce the maximum tab limit by closing oldest unmodified tabs
    fn enforce_max_tabs(&mut self) {
        if self.max_tabs == 0 || self.tab_order.len() <= self.max_tabs {
            return;
        }

        // Find tabs to close (oldest first, skip modified)
        let mut to_close = Vec::new();
        for &id in &self.tab_order {
            if self.tab_order.len() - to_close.len() <= self.max_tabs {
                break;
            }
            if let Some(doc) = self.documents.get(&id) {
                if !doc.is_modified() && Some(id) != self.active_id {
                    to_close.push(id);
                }
            }
        }

        for id in to_close {
            self.close_document(id);
        }
    }

    /// Iterator over all documents
    pub fn iter(&self) -> impl Iterator<Item = (&DocumentId, &Document)> {
        self.documents.iter()
    }

    /// Iterator over documents in tab order
    pub fn iter_in_order(&self) -> impl Iterator<Item = &Document> {
        self.tab_order
            .iter()
            .filter_map(|id| self.documents.get(id))
    }
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tab information for UI display
#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id: DocumentId,
    pub title: String,
    pub is_modified: bool,
    pub is_active: bool,
    pub file_path: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_document_manager() {
        let manager = DocumentManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.document_count(), 0);
        assert!(manager.active_id().is_none());
    }

    #[test]
    fn test_with_initial_document() {
        let manager = DocumentManager::with_initial_document();
        assert!(!manager.is_empty());
        assert_eq!(manager.document_count(), 1);
        assert!(manager.active_id().is_some());
    }

    #[test]
    fn test_new_document() {
        let mut manager = DocumentManager::new();
        let id = manager.new_document();

        assert_eq!(manager.document_count(), 1);
        assert_eq!(manager.active_id(), Some(id));
        assert!(manager.document(id).is_some());
    }

    #[test]
    fn test_multiple_documents() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();
        let id3 = manager.new_document();

        assert_eq!(manager.document_count(), 3);
        assert_eq!(manager.active_id(), Some(id3)); // Last opened is active
        assert_eq!(manager.tab_order(), &[id1, id2, id3]);
    }

    #[test]
    fn test_set_active() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();

        assert_eq!(manager.active_id(), Some(id2));

        manager.set_active(id1);
        assert_eq!(manager.active_id(), Some(id1));
    }

    #[test]
    fn test_set_active_by_index() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let _id2 = manager.new_document();

        manager.set_active_by_index(0);
        assert_eq!(manager.active_id(), Some(id1));
    }

    #[test]
    fn test_next_tab() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();
        let id3 = manager.new_document();

        // Currently at id3 (last opened)
        manager.set_active(id1);

        manager.next_tab();
        assert_eq!(manager.active_id(), Some(id2));

        manager.next_tab();
        assert_eq!(manager.active_id(), Some(id3));

        manager.next_tab(); // Should wrap to id1
        assert_eq!(manager.active_id(), Some(id1));
    }

    #[test]
    fn test_prev_tab() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();
        let id3 = manager.new_document();

        manager.set_active(id1);

        manager.prev_tab(); // Should wrap to id3
        assert_eq!(manager.active_id(), Some(id3));

        manager.prev_tab();
        assert_eq!(manager.active_id(), Some(id2));
    }

    #[test]
    fn test_close_document() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();

        let closed = manager.close_document(id2);
        assert!(closed.is_some());
        assert_eq!(manager.document_count(), 1);
        assert_eq!(manager.active_id(), Some(id1));
    }

    #[test]
    fn test_close_active_switches_to_adjacent() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();
        let id3 = manager.new_document();

        manager.set_active(id2);
        manager.close_active();

        // Should switch to id3 (next in order) or id1 if at end
        assert_eq!(manager.active_id(), Some(id3));
        assert_eq!(manager.tab_order(), &[id1, id3]);
    }

    #[test]
    fn test_tabs_info() {
        let mut manager = DocumentManager::new();
        manager.new_document();
        manager.new_document();

        let tabs = manager.tabs();
        assert_eq!(tabs.len(), 2);
        assert!(!tabs[0].is_active);
        assert!(tabs[1].is_active);
    }

    #[test]
    fn test_open_file() {
        let mut manager = DocumentManager::new();
        let path = PathBuf::from("/test/file.rs");
        let content = "fn main() {}".to_string();

        let id = manager.open_file(path.clone(), content);

        let doc = manager.document(id).unwrap();
        assert_eq!(doc.title(), "file.rs");
        assert_eq!(doc.file_path(), Some(&path));
    }

    #[test]
    fn test_open_same_file_twice() {
        let mut manager = DocumentManager::new();
        let path = PathBuf::from("/test/file.rs");

        let id1 = manager.open_file(path.clone(), "content1".to_string());
        let id2 = manager.open_file(path.clone(), "content2".to_string());

        // Should return same ID, not open a new document
        assert_eq!(id1, id2);
        assert_eq!(manager.document_count(), 1);
    }

    #[test]
    fn test_move_tab() {
        let mut manager = DocumentManager::new();
        let id1 = manager.new_document();
        let id2 = manager.new_document();
        let id3 = manager.new_document();

        assert!(manager.move_tab(0, 2));
        assert_eq!(manager.tab_order(), &[id2, id3, id1]);
    }

    #[test]
    fn test_modified_documents() {
        let mut manager = DocumentManager::new();
        manager.new_document();
        let id2 = manager.new_document();

        // Mark one as modified
        manager.document_mut(id2).unwrap().set_modified(true);

        let modified = manager.modified_documents();
        assert_eq!(modified.len(), 1);
        assert!(manager.has_unsaved_changes());
    }
}
