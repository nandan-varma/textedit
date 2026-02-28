pub mod buffer;
pub mod command_bar;
pub mod cursor;
pub mod keyboard;
pub mod operations;

pub use buffer::Buffer;
pub use command_bar::{CommandBarMode, CommandBarState};
pub use cursor::Cursor;
pub use keyboard::KeyboardController;
pub use operations::OperationHistory;

pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    history: OperationHistory,
    file_path: Option<String>,
    is_modified: bool,
    show_line_numbers: bool,
    show_status_bar: bool,
    command_bar: Option<CommandBarState>,
    last_find_query: Option<String>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            history: OperationHistory::new(),
            file_path: None,
            is_modified: false,
            show_line_numbers: true,
            show_status_bar: true,
            command_bar: None,
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

    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    pub fn show_status_bar(&self) -> bool {
        self.show_status_bar
    }

    pub fn set_show_status_bar(&mut self, show: bool) {
        self.show_status_bar = show;
    }

    pub fn toggle_status_bar(&mut self) {
        self.show_status_bar = !self.show_status_bar;
    }

    pub fn command_bar(&self) -> Option<&CommandBarState> {
        self.command_bar.as_ref()
    }

    pub fn command_bar_mut(&mut self) -> Option<&mut CommandBarState> {
        self.command_bar.as_mut()
    }

    pub fn command_bar_status_text(&self) -> Option<&str> {
        self.command_bar.as_ref().map(|c| c.status_text())
    }

    pub fn is_command_bar_active(&self) -> bool {
        self.command_bar.is_some()
    }

    pub fn begin_find(&mut self) {
        let initial = self
            .cursor
            .selection()
            .and_then(|s| {
                let (a, b) = s.range();
                if a < b {
                    Some(self.buffer.rope().slice(a..b).to_string())
                } else {
                    None
                }
            })
            .or_else(|| self.last_find_query.clone())
            .unwrap_or_default();
        self.command_bar = Some(CommandBarState::new_find(Some(&initial)));
        self.last_find_query = Some(initial);
    }

    pub fn begin_replace(&mut self) {
        let initial = self
            .cursor
            .selection()
            .and_then(|s| {
                let (a, b) = s.range();
                if a < b {
                    Some(self.buffer.rope().slice(a..b).to_string())
                } else {
                    None
                }
            })
            .or_else(|| self.last_find_query.clone())
            .unwrap_or_default();
        self.command_bar = Some(CommandBarState::new_replace(Some(&initial)));
        self.last_find_query = Some(initial);
    }

    pub fn cancel_command_bar(&mut self) {
        self.command_bar = None;
    }

    pub fn find_next(&mut self) -> bool {
        let query = self
            .command_bar
            .as_ref()
            .map(|c| c.find_query().to_string())
            .or_else(|| self.last_find_query.clone())
            .unwrap_or_default();
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
        let query = self
            .command_bar
            .as_ref()
            .map(|c| c.find_query().to_string())
            .or_else(|| self.last_find_query.clone())
            .unwrap_or_default();
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

    pub fn replace_next(&mut self) -> bool {
        let Some(cb) = self.command_bar.as_ref() else {
            return false;
        };
        if cb.mode != CommandBarMode::Replace {
            return false;
        }
        let query = cb.find_query();
        if query.is_empty() {
            return false;
        }
        let replacement = cb.replace_text().to_string();
        self.last_find_query = Some(query.to_string());

        // If current selection matches the query, replace it.
        if let Some(sel) = self.cursor.selection() {
            let (s, e) = sel.range();
            if s < e {
                let selected = self.buffer.rope().slice(s..e).to_string();
                if selected == query {
                    self.buffer_mut().remove(s, e - s);
                    self.history_mut().push(crate::editor::operations::Operation::Delete {
                        position: s,
                        text: selected,
                    });
                    if !replacement.is_empty() {
                        self.buffer_mut().insert(s, &replacement);
                        self.history_mut().push(crate::editor::operations::Operation::Insert {
                            position: s,
                            text: replacement.clone(),
                        });
                    }
                    let new_end = s + replacement.chars().count();
                    self.cursor.select_range(s, new_end);
                }
            }
        }

        // Then find next occurrence after current cursor.
        self.find_next()
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

    // Wrap search to start.
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

    // Wrap search from end.
    let full = buffer.rope().to_string();
    if let Some(byte_idx) = full.rfind(query) {
        let char_off = full[..byte_idx].chars().count();
        let start = char_off;
        return Some((start, start + qlen));
    }

    None
}
