#[derive(Clone, Copy, Debug)]
pub struct CursorPos {
    pub char_idx: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

pub struct Cursor {
    position: usize,
    selection: Option<Selection>,
    preferred_col: Option<usize>, // For maintaining column during vertical movement
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            position: 0,
            selection: None,
            preferred_col: None,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }

    pub fn start_selection(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(Selection::new(self.position, self.position));
        }
    }

    pub fn end_selection(&mut self) {
        self.selection = None;
    }

    pub fn extend_selection(&mut self, pos: usize) {
        if let Some(ref mut sel) = self.selection {
            sel.end = pos;
        } else {
            self.selection = Some(Selection::new(self.position, pos));
        }
        self.position = pos;
    }

    pub fn move_forward(&mut self, buffer_len: usize) {
        if self.position < buffer_len {
            self.position = self.position.saturating_add(1).min(buffer_len);
        }
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn move_backward(&mut self) {
        self.position = self.position.saturating_sub(1);
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn move_to_line_start(&mut self, buffer: &crate::editor::Buffer) {
        let (line, _) = buffer.char_to_line_col(self.position);
        if let Some(idx) = buffer.line_col_to_char(line, 0) {
            self.position = idx;
        }
        self.selection = None;
        self.preferred_col = Some(0);
    }

    pub fn move_to_line_end(&mut self, buffer: &crate::editor::Buffer) {
        let (line, _) = buffer.char_to_line_col(self.position);
        if let Some(line_str) = buffer.line(line) {
            let col = line_str.len();
            if let Some(idx) = buffer.line_col_to_char(line, col) {
                self.position = idx;
                self.preferred_col = Some(col);
            }
        }
        self.selection = None;
    }

    pub fn move_up(&mut self, buffer: &crate::editor::Buffer) {
        let (line, col) = buffer.char_to_line_col(self.position);
        if line > 0 {
            let target_col = self.preferred_col.unwrap_or(col);
            if let Some(idx) = buffer.line_col_to_char(line - 1, target_col) {
                self.position = idx;
            } else if let Some(line_str) = buffer.line(line - 1) {
                if let Some(idx) = buffer.line_col_to_char(line - 1, line_str.len()) {
                    self.position = idx;
                }
            }
            self.preferred_col = Some(target_col);
        }
        self.selection = None;
    }

    pub fn move_down(&mut self, buffer: &crate::editor::Buffer) {
        let (line, col) = buffer.char_to_line_col(self.position);
        if line + 1 < buffer.len_lines() {
            let target_col = self.preferred_col.unwrap_or(col);
            if let Some(idx) = buffer.line_col_to_char(line + 1, target_col) {
                self.position = idx;
            } else if let Some(line_str) = buffer.line(line + 1) {
                if let Some(idx) = buffer.line_col_to_char(line + 1, line_str.len()) {
                    self.position = idx;
                }
            }
            self.preferred_col = Some(target_col);
        }
        self.selection = None;
    }
}
