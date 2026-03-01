use ropey::{Rope, RopeSlice};

pub struct Buffer {
    content: Rope,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
        }
    }

    pub fn from_str(text: &str) -> Self {
        Self {
            content: Rope::from(text),
        }
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if char_idx <= self.content.len_chars() {
            self.content.insert(char_idx, text);
        }
    }

    pub fn remove(&mut self, char_idx: usize, len: usize) {
        if char_idx + len <= self.content.len_chars() {
            self.content.remove(char_idx..char_idx + len);
        }
    }

    pub fn get_char(&self, char_idx: usize) -> Option<char> {
        self.content.get_char(char_idx)
    }

    pub fn len_chars(&self) -> usize {
        self.content.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.content.get_line(line_idx).map(|l| l.to_string())
    }

    pub fn rope(&self) -> &Rope {
        &self.content
    }

    pub fn line_slice(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.content.len_lines() {
            Some(self.content.line(line_idx))
        } else {
            None
        }
    }

    pub fn line_len_chars(&self, line_idx: usize) -> usize {
        self.content
            .get_line(line_idx)
            .map(|l| l.len_chars())
            .unwrap_or(0)
    }

    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let line = self.content.char_to_line(char_idx);
        let line_start = self.content.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    pub fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line < self.content.len_lines() {
            let line_start = self.content.line_to_char(line);
            let line_len = self.content.line(line).len_chars();
            if col <= line_len {
                return Some(line_start + col);
            }
        }
        None
    }

    pub fn line_to_char(&self, line: usize) -> usize {
        self.content.line_to_char(line)
    }

    pub fn as_str(&self) -> String {
        self.content.to_string()
    }

    pub fn clear(&mut self) {
        self.content = Rope::new();
    }

    pub fn set_content(&mut self, text: &str) {
        self.content = Rope::from(text);
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
