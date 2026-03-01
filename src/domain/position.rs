#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn from_char_index(buffer: &crate::domain::Buffer, char_idx: usize) -> Self {
        let (line, col) = buffer.char_to_line_col(char_idx);
        Self::new(line, col)
    }

    pub fn to_char_index(&self, buffer: &crate::domain::Buffer) -> Option<usize> {
        buffer.line_col_to_char(self.line, self.column)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
