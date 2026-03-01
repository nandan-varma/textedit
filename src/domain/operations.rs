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

    pub fn undo(&mut self) -> Option<Operation> {
        if let Some(op) = self.undo_stack.pop_back() {
            let reverse = match &op {
                Operation::Insert { position, text } => Operation::Delete {
                    position: *position,
                    text: text.clone(),
                },
                Operation::Delete { position, text } => Operation::Insert {
                    position: *position,
                    text: text.clone(),
                },
            };
            self.redo_stack.push_back(reverse);
            Some(op)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Operation> {
        if let Some(op) = self.redo_stack.pop_back() {
            let reverse = match &op {
                Operation::Insert { position, text } => Operation::Delete {
                    position: *position,
                    text: text.clone(),
                },
                Operation::Delete { position, text } => Operation::Insert {
                    position: *position,
                    text: text.clone(),
                },
            };
            self.undo_stack.push_back(reverse);
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
