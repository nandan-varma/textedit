use crate::domain::{Buffer, Cursor, Operation, OperationHistory};
use crate::error::Result;
use crate::ports::{Clipboard, FileRepository};

pub struct EditorService {
    buffer: Buffer,
    cursor: Cursor,
    history: OperationHistory,
    file_path: Option<String>,
    is_modified: bool,
    show_line_numbers: bool,
    show_status_bar: bool,
    last_find_query: Option<String>,
}

impl EditorService {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            history: OperationHistory::new(),
            file_path: None,
            is_modified: false,
            show_line_numbers: true,
            show_status_bar: true,
            last_find_query: None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.is_modified = true;
        &mut self.buffer
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn history_mut(&mut self) -> &mut OperationHistory {
        &mut self.history
    }

    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
        self.is_modified = false;
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.is_modified = modified;
    }

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

    pub fn find_next(&mut self) -> bool {
        let query = self.last_find_query.clone().unwrap_or_default();
        if query.is_empty() {
            return false;
        }
        self.last_find_query = Some(query.clone());

        let from = self.cursor.position().min(self.buffer.len_chars());
        if let Some((s, e)) = find_next_range(&self.buffer, &query, from) {
            self.cursor.select_range(s, e);
            return true;
        }
        false
    }

    pub fn find_prev(&mut self) -> bool {
        let query = self.last_find_query.clone().unwrap_or_default();
        if query.is_empty() {
            return false;
        }
        self.last_find_query = Some(query.clone());

        let from = self.cursor.position().min(self.buffer.len_chars());
        if let Some((s, e)) = find_prev_range(&self.buffer, &query, from) {
            self.cursor.select_range(s, e);
            return true;
        }
        false
    }

    pub fn set_find_query(&mut self, query: Option<String>) {
        self.last_find_query = query;
    }

    pub fn find_query(&self) -> Option<&str> {
        self.last_find_query.as_deref()
    }

    pub fn new_file(&mut self) {
        self.buffer.clear();
        self.cursor.set_position(0);
        self.history.clear();
        self.file_path = None;
        self.is_modified = false;
    }

    pub fn load_content(&mut self, content: String) {
        self.buffer.set_content(&content);
        self.cursor.set_position(0);
        self.history.clear();
    }
}

impl Default for EditorService {
    fn default() -> Self {
        Self::new()
    }
}

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
        let start = char_off;
        return Some((start, start + qlen));
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
        let start = char_off;
        return Some((start, start + qlen));
    }

    let full = buffer.rope().to_string();
    if let Some(byte_idx) = full.rfind(query) {
        let char_off = full[..byte_idx].chars().count();
        let start = char_off;
        return Some((start, start + qlen));
    }

    None
}
