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

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

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
