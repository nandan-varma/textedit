use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl Cursor {
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }

    pub fn move_to(&mut self, line: usize, col: usize, offset: usize) {
        self.line = line;
        self.col = col;
        self.offset = offset;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

impl Selection {
    pub fn new(start: Cursor, end: Cursor) -> Self {
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }

    pub fn normalize(&self) -> (usize, usize) {
        let start = self.start.offset.min(self.end.offset);
        let end = self.start.offset.max(self.end.offset);
        (start, end)
    }

    pub fn from_single_cursor(cursor: Cursor) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorState {
    pub primary: Cursor,
    pub selections: Vec<Selection>,
    pub anchor: Cursor,
}

impl CursorState {
    pub fn new() -> Self {
        let cursor = Cursor::new(0, 0, 0);
        Self {
            primary: cursor,
            selections: vec![Selection::from_single_cursor(cursor)],
            anchor: cursor,
        }
    }

    pub fn move_to(&mut self, line: usize, col: usize, offset: usize) {
        let cursor = Cursor::new(line, col, offset);
        self.primary = cursor;
        self.selections = vec![Selection::from_single_cursor(cursor)];
        self.anchor = cursor;
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.primary = if selection.start.offset <= selection.end.offset {
            selection.start
        } else {
            selection.end
        };
        self.selections = vec![selection];
    }

    pub fn get_cursor_offset(&self) -> usize {
        self.primary.offset
    }

    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        let sel = self.selections.first()?;
        let (start, end) = sel.normalize();
        if start == end {
            None
        } else {
            Some((start, end))
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selections.iter().any(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new(5, 10, 15);
        assert_eq!(cursor.line, 5);
        assert_eq!(cursor.col, 10);
        assert_eq!(cursor.offset, 15);
    }

    #[test]
    fn test_selection_empty() {
        let sel = Selection::from_single_cursor(Cursor::new(0, 0, 0));
        assert!(sel.is_empty());
    }

    #[test]
    fn test_selection_normalize() {
        let sel = Selection::new(Cursor::new(2, 5, 10), Cursor::new(1, 3, 5));
        let (start, end) = sel.normalize();
        assert_eq!(start, 5);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_cursor_state_selection_range() {
        let mut state = CursorState::new();

        state.set_selection(Selection::new(Cursor::new(0, 5, 5), Cursor::new(0, 10, 10)));

        assert_eq!(state.get_selection_range(), Some((5, 10)));

        state.move_to(0, 0, 0);
        assert!(state.get_selection_range().is_none());
    }

    #[test]
    fn test_has_selection() {
        let mut state = CursorState::new();
        assert!(!state.has_selection());

        state.set_selection(Selection::new(Cursor::new(0, 0, 0), Cursor::new(0, 5, 5)));
        assert!(state.has_selection());
    }
}
