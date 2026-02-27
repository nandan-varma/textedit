use crate::editor::{CursorState, Document};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EditorPanel {
    pub document: Arc<parking_lot::RwLock<Document>>,
    pub cursor_state: CursorState,
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub word_wrap: bool,
    pub tab_size: usize,
    scroll_x: f32,
    scroll_y: f32,
}

impl EditorPanel {
    pub fn new(document: Document) -> Self {
        Self {
            document: Arc::new(parking_lot::RwLock::new(document)),
            cursor_state: CursorState::new(),
            show_line_numbers: true,
            highlight_current_line: true,
            word_wrap: false,
            tab_size: 4,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize, usize) {
        (
            self.cursor_state.primary.line,
            self.cursor_state.primary.col,
            self.cursor_state.primary.offset,
        )
    }

    pub fn set_cursor_position(&mut self, line: usize, col: usize, offset: usize) {
        self.cursor_state.move_to(line, col, offset);
    }

    pub fn get_status_info(&self) -> (String, String, String) {
        let doc = self.document.read();
        let (line, col, _) = self.get_cursor_position();
        let line_str = format!("Ln {}", line + 1);
        let col_str = format!("Col {}", col + 1);
        let encoding = doc.encoding.name().to_string();
        let modified = if doc.is_modified() {
            "Modified"
        } else {
            "Saved"
        };
        (line_str, col_str, format!("{} | {}", encoding, modified))
    }

    pub fn get_text(&self) -> String {
        self.document.read().buffer.get_text()
    }

    pub fn set_text(&mut self, text: &str) {
        let mut doc = self.document.write();
        doc.buffer = crate::editor::TextBuffer::from_string(text);
    }

    pub fn has_selection(&self) -> bool {
        self.cursor_state.has_selection()
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let doc = self.document.read();
        let range = self.cursor_state.get_selection_range()?;
        Some(doc.buffer.get_text_range(range.0, range.1))
    }
}
