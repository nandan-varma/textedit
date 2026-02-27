use crate::editor::Document;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    #[serde(skip)]
    pub document: Arc<RwLock<Document>>,
    pub title: String,
    pub is_modified: bool,
}

impl Tab {
    pub fn new(id: usize, document: Document) -> Self {
        let title = document.file_name();
        let is_modified = document.is_modified();
        Self {
            id,
            document: Arc::new(RwLock::new(document)),
            title,
            is_modified,
        }
    }

    pub fn update_state(&mut self) {
        let doc = self.document.read();
        self.title = doc.file_name();
        self.is_modified = doc.is_modified();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TabBar {
    #[serde(skip)]
    tabs: Vec<Tab>,
    pub active_tab: Option<usize>,
    #[serde(skip)]
    next_id: usize,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            next_id: 0,
        }
    }

    pub fn add_tab(&mut self, document: Document) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let tab = Tab::new(id, document);
        self.tabs.push(tab);
        self.active_tab = Some(self.tabs.len() - 1);
        id
    }

    pub fn close_tab(&mut self, index: usize) -> Option<Document> {
        if index < self.tabs.len() {
            let tab = self.tabs.remove(index);
            if let Some(active) = self.active_tab {
                if index < active {
                    self.active_tab = Some(active - 1);
                } else if index == active {
                    self.active_tab = if self.tabs.is_empty() {
                        None
                    } else {
                        Some(index.saturating_sub(1).min(self.tabs.len() - 1))
                    };
                }
            }
            Some(
                Arc::try_unwrap(tab.document)
                    .map(|d| d.into_inner())
                    .unwrap_or_default(),
            )
        } else {
            None
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = Some(index);
        }
    }

    pub fn get_active_document(&self) -> Option<Arc<RwLock<Document>>> {
        self.active_tab
            .and_then(|i| self.tabs.get(i).map(|t| t.document.clone()))
    }

    pub fn get_tab(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    pub fn get_tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    pub fn get_active_index(&self) -> Option<usize> {
        self.active_tab
    }

    pub fn update_tab_state(&mut self) {
        for tab in &mut self.tabs {
            tab.update_state();
        }
    }

    pub fn has_modified_tabs(&self) -> bool {
        self.tabs.iter().any(|t| t.is_modified)
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tabs.len()
    }
}
