use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum Operation {
    Insert { position: usize, text: String },
    Delete { position: usize, text: String },
}

pub struct OperationHistory {
    undo_stack: VecDeque<Operation>,
    redo_stack: VecDeque<Operation>,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(1000),
            redo_stack: VecDeque::with_capacity(1000),
        }
    }

    pub fn push(&mut self, operation: Operation) {
        self.undo_stack.push_back(operation);
        self.redo_stack.clear();
    }

    /// Returns the original operation to undo. The caller is expected to
    /// apply its *inverse* (Insert -> remove, Delete -> insert).
    pub fn undo(&mut self) -> Option<Operation> {
        if let Some(op) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(op.clone());
            Some(op)
        } else {
            None
        }
    }

    /// Returns the original operation to redo. The caller is expected to
    /// apply it literally (Insert -> insert, Delete -> remove), replaying
    /// exactly what was undone.
    pub fn redo(&mut self) -> Option<Operation> {
        if let Some(op) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(op.clone());
            Some(op)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    #[allow(dead_code)]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for OperationHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Operation, OperationHistory};

    #[test]
    fn test_history_new_is_empty() {
        let history = OperationHistory::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_push_adds_to_undo() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "test".to_string(),
        });
        assert!(history.can_undo());
    }

    #[test]
    fn test_history_undo_returns_op() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "hello".to_string(),
        });
        let op = history.undo();
        assert!(op.is_some());
    }

    #[test]
    fn test_history_redo_after_undo() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "hello".to_string(),
        });
        history.undo();
        assert!(history.can_redo());
    }

    /// redo() must hand back the ORIGINAL operation (for the caller to apply
    /// literally), not its inverse. Callers (apply_undo/apply_redo in the UI
    /// layer) apply undo()'s result inverted and redo()'s result literally,
    /// so redo() returning the inverse silently replays the undo instead of
    /// restoring what was undone.
    #[test]
    fn test_history_redo_returns_original_insert_op_not_its_inverse() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "hello".to_string(),
        });
        history.undo();

        let redone = history.redo().expect("should have a redo entry");
        match redone {
            Operation::Insert { position, text } => {
                assert_eq!(position, 0);
                assert_eq!(text, "hello");
            }
            Operation::Delete { .. } => {
                panic!("redo() returned the inverse (Delete) instead of the original Insert");
            }
        }
    }

    #[test]
    fn test_history_redo_returns_original_delete_op_not_its_inverse() {
        let mut history = OperationHistory::new();
        history.push(Operation::Delete {
            position: 3,
            text: "xyz".to_string(),
        });
        history.undo();

        let redone = history.redo().expect("should have a redo entry");
        match redone {
            Operation::Delete { position, text } => {
                assert_eq!(position, 3);
                assert_eq!(text, "xyz");
            }
            Operation::Insert { .. } => {
                panic!("redo() returned the inverse (Insert) instead of the original Delete");
            }
        }
    }

    /// Full round trip: undo() must still be invertible after a redo() cycle.
    #[test]
    fn test_history_undo_after_redo_after_undo_returns_original_insert() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "hello".to_string(),
        });
        history.undo();
        history.redo();

        let op = history.undo().expect("should be undoable again");
        match op {
            Operation::Insert { position, text } => {
                assert_eq!(position, 0);
                assert_eq!(text, "hello");
            }
            Operation::Delete { .. } => panic!("undo() after redo() returned the wrong op"),
        }
    }

    #[test]
    fn test_history_push_clears_redo() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "test".to_string(),
        });
        history.undo();
        assert!(history.can_redo());

        history.push(Operation::Insert {
            position: 5,
            text: "more".to_string(),
        });
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_clear() {
        let mut history = OperationHistory::new();
        history.push(Operation::Insert {
            position: 0,
            text: "test".to_string(),
        });
        assert!(history.can_undo());
        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_default() {
        let history = OperationHistory::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
