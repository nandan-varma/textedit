use ropey::{Rope, RopeSlice};

pub struct Buffer {
    content: Rope,
}

impl Buffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
        }
    }

    #[allow(dead_code)]
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

    #[inline]
    pub fn get_char(&self, char_idx: usize) -> Option<char> {
        self.content.get_char(char_idx)
    }

    #[inline]
    pub fn len_chars(&self) -> usize {
        self.content.len_chars()
    }

    #[inline]
    pub fn len_lines(&self) -> usize {
        self.content.len_lines()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.content.get_line(line_idx).map(|l| l.to_string())
    }

    #[inline]
    pub fn rope(&self) -> &Rope {
        &self.content
    }

    #[inline]
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

    /// Get a character at an index, returning None if out of bounds.
    /// More efficient than as_str() for sparse access patterns.
    #[inline]
    #[allow(dead_code)]
    pub fn char_at(&self, char_idx: usize) -> Option<char> {
        self.content.get_char(char_idx)
    }

    /// Check if a character at an index satisfies a predicate.
    /// Returns false for out-of-bounds indices.
    #[inline]
    pub fn char_matches(&self, char_idx: usize, predicate: impl Fn(char) -> bool) -> bool {
        self.content.get_char(char_idx).is_some_and(predicate)
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

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn test_buffer_new_is_empty() {
        let buffer = Buffer::new();
        assert_eq!(buffer.len_chars(), 0);
    }

    #[test]
    fn test_buffer_from_str() {
        let buffer = Buffer::from_str("hello");
        assert_eq!(buffer.len_chars(), 5);
        assert_eq!(buffer.as_str(), "hello");
    }

    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::from_str("hello");
        buffer.insert(2, "X");
        assert_eq!(buffer.as_str(), "heXllo");
    }

    #[test]
    fn test_buffer_remove() {
        let mut buffer = Buffer::from_str("hello");
        buffer.remove(0, 2);
        assert_eq!(buffer.as_str(), "llo");
    }

    #[test]
    fn test_buffer_get_char() {
        let buffer = Buffer::from_str("hello");
        assert_eq!(buffer.get_char(0), Some('h'));
        assert_eq!(buffer.get_char(4), Some('o'));
    }

    #[test]
    fn test_buffer_len() {
        let buffer = Buffer::from_str("hello");
        assert_eq!(buffer.len_chars(), 5);
    }

    #[test]
    fn test_buffer_len_lines() {
        let buffer = Buffer::from_str("line1\nline2");
        assert_eq!(buffer.len_lines(), 2);
    }

    #[test]
    fn test_buffer_as_str() {
        let buffer = Buffer::from_str("hello world");
        assert_eq!(buffer.as_str(), "hello world");
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = Buffer::from_str("hello");
        buffer.clear();
        assert_eq!(buffer.len_chars(), 0);
    }

    #[test]
    fn test_buffer_set_content() {
        let mut buffer = Buffer::new();
        buffer.set_content("test");
        assert_eq!(buffer.as_str(), "test");
    }

    #[test]
    fn test_buffer_clone() {
        let buffer1 = Buffer::from_str("hello");
        let buffer2 = buffer1.clone();
        assert_eq!(buffer1.as_str(), buffer2.as_str());
    }

    #[test]
    fn test_buffer_default() {
        let buffer = Buffer::default();
        assert_eq!(buffer.len_chars(), 0);
    }
}
