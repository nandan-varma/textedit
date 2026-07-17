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

    /// Line content *without* the trailing line terminator (`\n`, `\r\n`, or `\r`).
    /// Ropey's own `Rope::line` includes the terminator for every line except
    /// the last, which would otherwise make col/length math (and cursor
    /// End-of-line placement) land one line too far for every non-last line.
    pub fn line(&self, line_idx: usize) -> Option<String> {
        self.line_slice(line_idx).map(|s| s.to_string())
    }

    #[inline]
    pub fn rope(&self) -> &Rope {
        &self.content
    }

    #[inline]
    pub fn line_slice(&self, line_idx: usize) -> Option<RopeSlice<'_>> {
        if line_idx < self.content.len_lines() {
            Some(strip_line_ending(self.content.line(line_idx)))
        } else {
            None
        }
    }

    pub fn line_len_chars(&self, line_idx: usize) -> usize {
        self.line_slice(line_idx)
            .map(|s| s.len_chars())
            .unwrap_or(0)
    }

    /// Clamps out-of-bounds indices to the end of the buffer rather than
    /// panicking, since callers (e.g. cursor navigation after an edit) may
    /// transiently pass a char index past the current buffer length.
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.content.len_chars());
        let line = self.content.char_to_line(char_idx);
        let line_start = self.content.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    pub fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line < self.content.len_lines() {
            let line_start = self.content.line_to_char(line);
            let line_len = self.line_len_chars(line);
            if col <= line_len {
                return Some(line_start + col);
            }
        }
        None
    }

    /// Clamps an out-of-bounds line index to the end of the buffer rather
    /// than panicking.
    pub fn line_to_char(&self, line: usize) -> usize {
        let line = line.min(self.content.len_lines());
        self.content.line_to_char(line)
    }

    pub fn as_str(&self) -> String {
        self.content.to_string()
    }

    /// Extract a substring by character range without copying the whole buffer.
    /// Clamps to buffer bounds; returns an empty string for an empty or invalid range.
    pub fn slice_string(&self, start: usize, end: usize) -> String {
        let len = self.content.len_chars();
        let start = start.min(len);
        let end = end.min(len);
        if start >= end {
            return String::new();
        }
        self.content.slice(start..end).to_string()
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

/// Trims a trailing `\r\n`, `\n`, or `\r` off a ropey line slice.
fn strip_line_ending(line: RopeSlice<'_>) -> RopeSlice<'_> {
    let len = line.len_chars();
    if len == 0 {
        return line;
    }
    let last = line.char(len - 1);
    if last == '\n' {
        if len >= 2 && line.char(len - 2) == '\r' {
            return line.slice(0..len - 2);
        }
        return line.slice(0..len - 1);
    }
    if last == '\r' {
        return line.slice(0..len - 1);
    }
    line
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

    #[test]
    fn test_buffer_slice_string() {
        let buffer = Buffer::from_str("hello world");
        assert_eq!(buffer.slice_string(0, 5), "hello");
        assert_eq!(buffer.slice_string(6, 11), "world");
        assert_eq!(buffer.slice_string(0, 0), "");
    }

    #[test]
    fn test_buffer_slice_string_multibyte() {
        // "héllo" has 5 chars but 6 bytes (é is 2 bytes in UTF-8)
        let buffer = Buffer::from_str("héllo world");
        assert_eq!(buffer.slice_string(0, 5), "héllo");
        assert_eq!(buffer.slice_string(1, 2), "é");
        assert_eq!(buffer.slice_string(6, 11), "world");
    }

    #[test]
    fn test_buffer_line_excludes_trailing_newline_for_non_last_line() {
        let buffer = Buffer::from_str("line1\nline2\nline3");
        assert_eq!(buffer.line(0), Some("line1".to_string()));
        assert_eq!(buffer.line(1), Some("line2".to_string()));
        assert_eq!(buffer.line(2), Some("line3".to_string()));
    }

    #[test]
    fn test_buffer_line_slice_excludes_trailing_newline() {
        let buffer = Buffer::from_str("hello\nworld");
        let slice = buffer.line_slice(0).unwrap();
        assert_eq!(slice.to_string(), "hello");
    }

    #[test]
    fn test_buffer_line_len_chars_excludes_trailing_newline() {
        let buffer = Buffer::from_str("hello\nworld");
        assert_eq!(buffer.line_len_chars(0), 5);
        assert_eq!(buffer.line_len_chars(1), 5);
    }

    #[test]
    fn test_buffer_line_col_to_char_rejects_col_past_visible_text() {
        // Line 0 is "hello" (5 visible chars); the newline itself is not
        // an addressable column.
        let buffer = Buffer::from_str("hello\nworld");
        assert_eq!(buffer.line_col_to_char(0, 5), Some(5));
        assert_eq!(buffer.line_col_to_char(0, 6), None);
    }

    #[test]
    fn test_buffer_char_to_line_col_clamps_instead_of_panicking() {
        let buffer = Buffer::from_str("hello");
        assert_eq!(buffer.char_to_line_col(100), (0, 5));
    }

    #[test]
    fn test_buffer_line_to_char_clamps_instead_of_panicking() {
        let buffer = Buffer::from_str("hello");
        assert_eq!(buffer.line_to_char(1), 5);
        assert_eq!(buffer.line_to_char(100), 5);
    }

    #[test]
    fn test_buffer_remove_middle() {
        let mut buffer = Buffer::from_str("hello world");
        buffer.remove(5, 6);
        assert_eq!(buffer.as_str(), "hello");
    }
}
