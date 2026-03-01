use crate::domain::buffer::Buffer;

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

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
    preferred_col: Option<usize>,
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

    pub fn extend_selection(&mut self, pos: usize) {
        if let Some(ref mut sel) = self.selection {
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

    pub fn extend_selection_up(&mut self, buffer: &Buffer) {
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

    pub fn extend_selection_down(&mut self, buffer: &Buffer) {
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

    pub fn move_to_word_start(&mut self, buffer: &Buffer) {
        let text = buffer.as_str();
        let pos = self.position;
        let mut new_pos = pos;
        let chars: Vec<char> = text.chars().collect();

        if new_pos > 0 {
            let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

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

        self.position = new_pos;
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn move_to_word_end(&mut self, buffer: &Buffer) {
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

        self.position = new_pos.min(len);
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn extend_selection_to_word_start(&mut self, buffer: &Buffer) {
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

    pub fn extend_selection_to_word_end(&mut self, buffer: &Buffer) {
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

    pub fn select_line(&mut self, buffer: &Buffer) {
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

    pub fn select_word_at_cursor(&mut self, buffer: &Buffer) {
        let text = buffer.as_str();
        let pos = self.position;
        let chars: Vec<char> = text.chars().collect();

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        let mut start = pos;
        let mut end = pos;

        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

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

    pub fn move_to_line_start(&mut self, buffer: &Buffer) {
        let (line, _) = buffer.char_to_line_col(self.position);
        if let Some(idx) = buffer.line_col_to_char(line, 0) {
            self.position = idx;
        }
        self.selection = None;
        self.preferred_col = Some(0);
    }

    pub fn move_to_line_end(&mut self, buffer: &Buffer) {
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

    pub fn move_up(&mut self, buffer: &Buffer) {
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

    pub fn move_down(&mut self, buffer: &Buffer) {
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

    pub fn select_range(&mut self, start: usize, end: usize) {
        self.position = start;
        self.selection = Some(Selection::new(start, start));
        self.extend_selection(end);
        self.preferred_col = None;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
