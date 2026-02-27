use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBuffer {
    id: Uuid,
    #[serde(skip)]
    text: String,
    #[serde(skip)]
    is_modified: bool,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            text: String::new(),
            is_modified: false,
        }
    }

    pub fn from_string(content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: content.to_string(),
            is_modified: false,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.is_modified = modified;
    }

    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn num_lines(&self) -> usize {
        self.text.lines().count()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.text.lines().nth(line_idx).map(|s| s.to_string())
    }

    pub fn line_len(&self, line_idx: usize) -> Option<usize> {
        self.text.lines().nth(line_idx).map(|s| s.chars().count())
    }

    pub fn char_at(&self, idx: usize) -> Option<char> {
        self.text.chars().nth(idx)
    }

    pub fn insert(&mut self, idx: usize, text: &str) {
        let idx = idx.min(self.text.len());
        self.text.insert_str(idx, text);
        self.is_modified = true;
    }

    pub fn remove(&mut self, start: usize, end: usize) {
        let start = start.min(self.text.len());
        let end = end.min(self.text.len());
        if start < end {
            self.text.drain(start..end);
            self.is_modified = true;
        }
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn get_text_range(&self, start: usize, end: usize) -> String {
        if start >= end || start >= self.text.len() {
            return String::new();
        }
        let end = end.min(self.text.len());
        self.text[start..end].to_string()
    }

    pub fn line_to_byte(&self, line_idx: usize) -> Option<usize> {
        let mut byte_offset = 0;
        for (i, line) in self.text.lines().enumerate() {
            if i == line_idx {
                return Some(byte_offset);
            }
            byte_offset += line.len() + 1; // +1 for newline
        }
        None
    }

    pub fn byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        if byte_idx >= self.text.len() {
            return None;
        }
        Some(self.text.chars().take(byte_idx).count())
    }

    pub fn char_to_line(&self, char_idx: usize) -> usize {
        let mut line = 0;
        let mut count = 0;
        for c in self.text.chars() {
            if count >= char_idx {
                break;
            }
            if c == '\n' {
                line += 1;
            }
            count += 1;
        }
        line
    }

    pub fn line_beginning(&self, char_idx: usize) -> usize {
        let mut start = char_idx;
        while start > 0 {
            if self.text.chars().nth(start - 1) == Some('\n') {
                break;
            }
            start -= 1;
        }
        start
    }

    pub fn line_ending(&self, char_idx: usize) -> usize {
        let line_idx = self.char_to_line(char_idx);
        let line_start = self.line_beginning(char_idx);
        let line_len = self.line_len(line_idx).unwrap_or(0);
        line_start + line_len
    }

    pub fn next_line(&self, char_idx: usize) -> usize {
        let line_idx = self.char_to_line(char_idx);
        let num_lines = self.num_lines();
        if line_idx + 1 < num_lines {
            self.line_to_byte(line_idx + 1).unwrap_or(char_idx)
        } else {
            self.len()
        }
    }

    pub fn prev_line(&self, char_idx: usize) -> usize {
        let line_idx = self.char_to_line(char_idx);
        if line_idx > 0 {
            self.line_to_byte(line_idx - 1).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn words(&self) -> Vec<(usize, usize)> {
        let mut words = Vec::new();
        let mut start = 0;
        let mut in_word = false;

        for (i, c) in self.text.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                if !in_word {
                    start = i;
                    in_word = true;
                }
            } else {
                if in_word {
                    words.push((start, i));
                    in_word = false;
                }
            }
        }

        if in_word {
            words.push((start, self.text.len()));
        }

        words
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buf = TextBuffer::new();
        assert!(buf.len() == 0);
        assert!(!buf.is_modified());
    }

    #[test]
    fn test_insert() {
        let mut buf = TextBuffer::new();
        buf.insert(0, "Hello");
        assert_eq!(buf.get_text(), "Hello");
        assert!(buf.is_modified());
    }

    #[test]
    fn test_lines() {
        let buf = TextBuffer::from_string("Line 1\nLine 2\nLine 3");
        assert_eq!(buf.num_lines(), 3);
        assert_eq!(buf.line(0), Some("Line 1".to_string()));
        assert_eq!(buf.line(1), Some("Line 2".to_string()));
        assert_eq!(buf.line(2), Some("Line 3".to_string()));
    }

    #[test]
    fn test_remove() {
        let mut buf = TextBuffer::from_string("Hello World");
        buf.remove(5, 11);
        assert_eq!(buf.get_text(), "Hello");
    }

    #[test]
    fn test_modified_flag() {
        let mut buf = TextBuffer::new();
        assert!(!buf.is_modified());
        buf.insert(0, "test");
        assert!(buf.is_modified());
        buf.set_modified(false);
        assert!(!buf.is_modified());
    }
}
