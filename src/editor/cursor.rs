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
    /// Create a new selection with given endpoints. The values are stored
    /// as provided; use `range()` or `len()` to query the normalized interval.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Return the ordered endpoints as a (start, end) pair.
    pub fn range(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn is_empty(&self) -> bool {
        let (s, e) = self.range();
        s == e
    }

    pub fn len(&self) -> usize {
        let (s, e) = self.range();
        e - s
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
            // maintain anchor at sel.start, update end only
            sel.end = pos;
        } else {
            self.selection = Some(Selection::new(self.position, pos));
        }
        self.position = pos;
    }

    pub fn set_selection_start(&mut self, pos: usize) {
        self.selection = Some(Selection::new(pos, self.position));
    }

    pub fn set_selection_end(&mut self, pos: usize) {
        if let Some(ref mut sel) = self.selection {
            sel.end = pos;
        } else {
            self.selection = Some(Selection::new(self.position, pos));
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn extend_selection_backward(&mut self) {
        if self.position > 0 {
            self.extend_selection(self.position - 1);
        }
    }

    pub fn extend_selection_forward(&mut self, buffer_len: usize) {
        if self.position < buffer_len {
            self.extend_selection(self.position + 1);
        }
    }

    pub fn extend_selection_up(&mut self, buffer: &crate::editor::Buffer) {
        let (line, col) = buffer.char_to_line_col(self.position);
        if line > 0 {
            let target_col = self.preferred_col.unwrap_or(col);
            let new_pos = if let Some(idx) = buffer.line_col_to_char(line - 1, target_col) {
                idx
            } else if let Some(line_str) = buffer.line(line - 1) {
                buffer
                    .line_col_to_char(line - 1, line_str.len())
                    .unwrap_or(target_col)
            } else {
                return;
            };
            self.extend_selection(new_pos);
            self.preferred_col = Some(target_col);
        }
    }

    pub fn extend_selection_down(&mut self, buffer: &crate::editor::Buffer) {
        let (line, col) = buffer.char_to_line_col(self.position);
        if line + 1 < buffer.len_lines() {
            let target_col = self.preferred_col.unwrap_or(col);
            let new_pos = if let Some(idx) = buffer.line_col_to_char(line + 1, target_col) {
                idx
            } else if let Some(line_str) = buffer.line(line + 1) {
                buffer
                    .line_col_to_char(line + 1, line_str.len())
                    .unwrap_or(target_col)
            } else {
                return;
            };
            self.extend_selection(new_pos);
            self.preferred_col = Some(target_col);
        }
    }

    /// Move cursor to the start of the current word
    pub fn move_to_word_start(&mut self, buffer: &crate::editor::Buffer) {
        let text = buffer.as_str();
        let pos = self.position;

        // Find start of word
        let mut new_pos = pos;
        let chars: Vec<char> = text.chars().collect();

        if new_pos > 0 {
            // If we're in the middle of a word, move to its start
            // If we're at whitespace, move to next word
            let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

            if new_pos < chars.len() && is_word_char(chars[new_pos]) {
                // Move backward to find word start
                while new_pos > 0 && is_word_char(chars[new_pos - 1]) {
                    new_pos -= 1;
                }
            } else {
                // Skip whitespace
                while new_pos < chars.len() && !is_word_char(chars[new_pos]) {
                    new_pos += 1;
                }
                // Find word start
                while new_pos > 0 && new_pos < chars.len() && is_word_char(chars[new_pos - 1]) {
                    new_pos -= 1;
                }
            }
        }

        self.position = new_pos;
        self.selection = None;
        self.preferred_col = None;
    }

    /// Move cursor to the end of the current word
    pub fn move_to_word_end(&mut self, buffer: &crate::editor::Buffer) {
        let text = buffer.as_str();
        let pos = self.position;

        let mut new_pos = pos;
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // Skip current word
        while new_pos < len && is_word_char(chars[new_pos]) {
            new_pos += 1;
        }
        // Skip whitespace
        while new_pos < len && !is_word_char(chars[new_pos]) {
            new_pos += 1;
        }

        self.position = new_pos.min(len);
        self.selection = None;
        self.preferred_col = None;
    }

    /// Extend selection to word start
    pub fn extend_selection_to_word_start(&mut self, buffer: &crate::editor::Buffer) {
        let text = buffer.as_str();
        let pos = self.position;

        let mut new_pos = pos;
        let chars: Vec<char> = text.chars().collect();

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        if new_pos > 0 {
            if new_pos < chars.len() && is_word_char(chars[new_pos]) {
                while new_pos > 0 && is_word_char(chars[new_pos - 1]) {
                    new_pos -= 1;
                }
            } else {
                while new_pos < chars.len() && !is_word_char(chars[new_pos]) {
                    new_pos += 1;
                }
                while new_pos > 0 && new_pos < chars.len() && is_word_char(chars[new_pos - 1]) {
                    new_pos -= 1;
                }
            }
        }

        self.extend_selection(new_pos);
    }

    /// Extend selection to word end
    pub fn extend_selection_to_word_end(&mut self, buffer: &crate::editor::Buffer) {
        let text = buffer.as_str();
        let pos = self.position;

        let mut new_pos = pos;
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        while new_pos < len && is_word_char(chars[new_pos]) {
            new_pos += 1;
        }
        while new_pos < len && !is_word_char(chars[new_pos]) {
            new_pos += 1;
        }

        self.extend_selection(new_pos.min(len));
    }

    /// Select the entire line at cursor position
    pub fn select_line(&mut self, buffer: &crate::editor::Buffer) {
        let (line, _) = buffer.char_to_line_col(self.position);

        let start = buffer.line_to_char(line);
        let end = if let Some(line_str) = buffer.line(line) {
            start + line_str.len()
        } else {
            start
        };

        self.selection = Some(Selection { start, end });
        self.position = end;
        self.preferred_col = Some(end - start);
    }

    /// Select the word at cursor position
    pub fn select_word_at_cursor(&mut self, buffer: &crate::editor::Buffer) {
        let text = buffer.as_str();
        let pos = self.position;
        let chars: Vec<char> = text.chars().collect();

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // Find word boundaries
        let mut start = pos;
        let mut end = pos;

        // Find start
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Find end
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        self.selection = Some(Selection { start, end });
        self.position = end;
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
