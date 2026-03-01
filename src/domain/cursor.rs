use crate::domain::buffer::Buffer;

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn range(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let (s, e) = self.range();
        s == e
    }

    #[inline]
    #[allow(dead_code)]
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
    #[inline]
    pub fn new() -> Self {
        Self {
            position: 0,
            selection: None,
            preferred_col: None,
        }
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, pos: usize) {
        self.position = pos;
        self.selection = None;
        self.preferred_col = None;
    }

    #[inline]
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
        let len = buffer.len_chars();
        let mut new_pos = self.position;
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        if new_pos > 0 {
            if buffer.char_matches(new_pos, is_word_char) {
                while new_pos > 0 && buffer.char_matches(new_pos - 1, is_word_char) {
                    new_pos -= 1;
                }
            } else {
                while new_pos < len && !buffer.char_matches(new_pos, is_word_char) {
                    new_pos += 1;
                }
                while new_pos > 0 && new_pos < len && buffer.char_matches(new_pos - 1, is_word_char)
                {
                    new_pos -= 1;
                }
            }
        }

        self.position = new_pos;
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn move_to_word_end(&mut self, buffer: &Buffer) {
        let len = buffer.len_chars();
        let mut new_pos = self.position;
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        while new_pos < len && buffer.char_matches(new_pos, is_word_char) {
            new_pos += 1;
        }
        while new_pos < len && !buffer.char_matches(new_pos, is_word_char) {
            new_pos += 1;
        }

        self.position = new_pos.min(len);
        self.selection = None;
        self.preferred_col = None;
    }

    pub fn extend_selection_to_word_start(&mut self, buffer: &Buffer) {
        let len = buffer.len_chars();
        let mut new_pos = self.position;
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        if new_pos > 0 {
            if buffer.char_matches(new_pos, is_word_char) {
                while new_pos > 0 && buffer.char_matches(new_pos - 1, is_word_char) {
                    new_pos -= 1;
                }
            } else {
                while new_pos < len && !buffer.char_matches(new_pos, is_word_char) {
                    new_pos += 1;
                }
                while new_pos > 0 && new_pos < len && buffer.char_matches(new_pos - 1, is_word_char)
                {
                    new_pos -= 1;
                }
            }
        }

        self.extend_selection(new_pos);
    }

    pub fn extend_selection_to_word_end(&mut self, buffer: &Buffer) {
        let len = buffer.len_chars();
        let mut new_pos = self.position;
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        while new_pos < len && buffer.char_matches(new_pos, is_word_char) {
            new_pos += 1;
        }
        while new_pos < len && !buffer.char_matches(new_pos, is_word_char) {
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
        let len = buffer.len_chars();
        let pos = self.position;
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        let mut start = pos;
        let mut end = pos;

        while start > 0 && buffer.char_matches(start - 1, is_word_char) {
            start -= 1;
        }

        while end < len && buffer.char_matches(end, is_word_char) {
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

#[cfg(test)]
mod tests {
    use super::{Cursor, Selection};
    use crate::domain::buffer::Buffer;

    #[test]
    fn test_selection_new() {
        let sel = Selection::new(5, 10);
        assert_eq!(sel.start, 5);
        assert_eq!(sel.end, 10);
    }

    #[test]
    fn test_selection_range() {
        let sel = Selection::new(5, 10);
        let (start, end) = sel.range();
        assert_eq!(start, 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_selection_range_reverse() {
        let sel = Selection::new(10, 5);
        let (start, end) = sel.range();
        assert_eq!(start, 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_selection_is_empty() {
        let sel = Selection::new(5, 5);
        assert!(sel.is_empty());
        let sel2 = Selection::new(5, 10);
        assert!(!sel2.is_empty());
    }

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.position(), 0);
        assert!(cursor.selection().is_none());
    }

    #[test]
    fn test_cursor_set_position() {
        let mut cursor = Cursor::new();
        cursor.set_position(10);
        assert_eq!(cursor.position(), 10);
    }

    #[test]
    fn test_cursor_extend_selection() {
        let mut cursor = Cursor::new();
        cursor.set_position(5);
        cursor.extend_selection(10);
        assert_eq!(cursor.position(), 10);
        assert!(cursor.selection().is_some());
    }

    #[test]
    fn test_cursor_clear_selection() {
        let mut cursor = Cursor::new();
        cursor.set_selection_start(5);
        cursor.set_selection_end(10);
        assert!(cursor.selection().is_some());
        cursor.clear_selection();
        assert!(cursor.selection().is_none());
    }

    #[test]
    fn test_cursor_move_forward() {
        let mut cursor = Cursor::new();
        cursor.set_position(5);
        cursor.move_forward(20);
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn test_cursor_move_backward() {
        let mut cursor = Cursor::new();
        cursor.set_position(5);
        cursor.move_backward();
        assert_eq!(cursor.position(), 4);
    }

    #[test]
    fn test_cursor_move_to_line_start() {
        let buffer = Buffer::from_str("hello\nworld");
        let mut cursor = Cursor::new();
        cursor.set_position(8);
        cursor.move_to_line_start(&buffer);
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn test_cursor_move_to_line_end() {
        let buffer = Buffer::from_str("hello\nworld");
        let mut cursor = Cursor::new();
        cursor.set_position(6);
        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.position(), 11);
    }

    #[test]
    fn test_cursor_move_up() {
        let buffer = Buffer::from_str("hello\nworld");
        let mut cursor = Cursor::new();
        cursor.set_position(8);
        cursor.move_up(&buffer);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_cursor_move_down() {
        let buffer = Buffer::from_str("hello\nworld");
        let mut cursor = Cursor::new();
        cursor.set_position(2);
        cursor.move_down(&buffer);
        assert_eq!(cursor.position(), 8);
    }

    #[test]
    fn test_cursor_select_line() {
        let buffer = Buffer::from_str("hello\nworld");
        let mut cursor = Cursor::new();
        cursor.set_position(7);
        cursor.select_line(&buffer);
        assert!(cursor.selection().is_some());
    }

    #[test]
    fn test_cursor_select_word_at_cursor() {
        let buffer = Buffer::from_str("hello world");
        let mut cursor = Cursor::new();
        cursor.set_position(7);
        cursor.select_word_at_cursor(&buffer);
        assert!(cursor.selection().is_some());
    }

    #[test]
    fn test_cursor_select_range() {
        let mut cursor = Cursor::new();
        cursor.select_range(5, 10);
        // After select_range, position is at end
        assert!(cursor.selection().is_some());
        let sel = cursor.selection().unwrap();
        assert_eq!(sel.start, 5);
        assert_eq!(sel.end, 10);
    }
}
