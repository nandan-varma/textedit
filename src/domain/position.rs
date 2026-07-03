#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[allow(dead_code)]
impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn from_char_index(buffer: &crate::domain::Buffer, char_idx: usize) -> Self {
        let (line, col) = buffer.char_to_line_col(char_idx);
        Self::new(line, col)
    }

    pub fn to_char_index(self, buffer: &crate::domain::Buffer) -> Option<usize> {
        buffer.line_col_to_char(self.line, self.column)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::Position;
    use crate::domain::Buffer;

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_position_from_char_index() {
        let buffer = Buffer::from_str("hello\nworld");
        let pos = Position::from_char_index(&buffer, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        let pos = Position::from_char_index(&buffer, 5);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 5);

        let pos = Position::from_char_index(&buffer, 6);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_position_from_char_index_end_of_buffer() {
        let buffer = Buffer::from_str("hi");
        let pos = Position::from_char_index(&buffer, 1);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn test_position_to_char_index() {
        let buffer = Buffer::from_str("hello\nworld");
        let pos = Position::new(0, 0);
        assert_eq!(pos.to_char_index(&buffer), Some(0));

        let pos = Position::new(0, 5);
        assert_eq!(pos.to_char_index(&buffer), Some(5));

        let pos = Position::new(1, 0);
        assert_eq!(pos.to_char_index(&buffer), Some(6));

        let pos = Position::new(1, 5);
        assert_eq!(pos.to_char_index(&buffer), Some(11));
    }

    #[test]
    fn test_position_to_char_index_out_of_bounds() {
        let buffer = Buffer::from_str("hi");
        let pos = Position::new(10, 0);
        assert_eq!(pos.to_char_index(&buffer), None);

        let pos = Position::new(0, 100);
        assert_eq!(pos.to_char_index(&buffer), None);
    }

    #[test]
    fn test_position_partial_eq() {
        let pos1 = Position::new(1, 2);
        let pos2 = Position::new(1, 2);
        let pos3 = Position::new(1, 3);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_clone_copy() {
        let pos1 = Position::new(1, 2);
        let pos2 = pos1;
        assert_eq!(pos1, pos2);

        let pos3 = pos1.clone();
        assert_eq!(pos1, pos3);
    }
}
