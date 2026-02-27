use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditAction {
    Insert {
        position: usize,
        text: String,
    },
    Delete {
        position: usize,
        text: String,
    },
    Replace {
        position: usize,
        old_text: String,
        new_text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoState {
    pub actions: Vec<EditAction>,
    pub cursor_position: usize,
}

pub struct UndoManager {
    undo_stack: VecDeque<UndoState>,
    redo_stack: VecDeque<UndoState>,
    max_history: usize,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new(100)
    }
}

impl UndoManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history,
        }
    }

    pub fn push(&mut self, state: UndoState) {
        self.undo_stack.push_back(state);

        if self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }

        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Option<UndoState> {
        if let Some(state) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(state.clone());
            Some(state)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<UndoState> {
        if let Some(state) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(state.clone());
            Some(state)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn get_inverse_action(action: &EditAction) -> EditAction {
        match action {
            EditAction::Insert { position, text } => EditAction::Delete {
                position: *position,
                text: text.clone(),
            },
            EditAction::Delete { position, text } => EditAction::Insert {
                position: *position,
                text: text.clone(),
            },
            EditAction::Replace {
                position,
                old_text,
                new_text,
            } => EditAction::Replace {
                position: *position,
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_manager_new() {
        let manager = UndoManager::new(10);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_push_and_undo() {
        let mut manager = UndoManager::new(10);

        let state = UndoState {
            actions: vec![EditAction::Insert {
                position: 0,
                text: "hello".to_string(),
            }],
            cursor_position: 5,
        };

        manager.push(state.clone());
        assert!(manager.can_undo());

        let undone = manager.undo();
        assert!(undone.is_some());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_redo() {
        let mut manager = UndoManager::new(10);

        let state = UndoState {
            actions: vec![EditAction::Insert {
                position: 0,
                text: "hello".to_string(),
            }],
            cursor_position: 5,
        };

        manager.push(state);
        manager.undo();

        let redone = manager.redo();
        assert!(redone.is_some());
    }

    #[test]
    fn test_max_history() {
        let mut manager = UndoManager::new(3);

        for i in 0..5 {
            let state = UndoState {
                actions: vec![EditAction::Insert {
                    position: 0,
                    text: format!("text{}", i),
                }],
                cursor_position: 0,
            };
            manager.push(state);
        }

        assert_eq!(manager.undo_stack.len(), 3);
    }

    #[test]
    fn test_inverse_insert_delete() {
        let insert = EditAction::Insert {
            position: 5,
            text: "hello".to_string(),
        };
        let inverse = UndoManager::get_inverse_action(&insert);

        match inverse {
            EditAction::Delete { position, text } => {
                assert_eq!(position, 5);
                assert_eq!(text, "hello");
            }
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_inverse_replace() {
        let replace = EditAction::Replace {
            position: 0,
            old_text: "old".to_string(),
            new_text: "new".to_string(),
        };

        let inverse = UndoManager::get_inverse_action(&replace);

        match inverse {
            EditAction::Replace {
                old_text, new_text, ..
            } => {
                assert_eq!(old_text, "new");
                assert_eq!(new_text, "old");
            }
            _ => panic!("Expected Replace"),
        }
    }

    #[test]
    fn test_clear_redo_on_new_edit() {
        let mut manager = UndoManager::new(10);

        let state1 = UndoState {
            actions: vec![EditAction::Insert {
                position: 0,
                text: "a".to_string(),
            }],
            cursor_position: 1,
        };

        manager.push(state1);
        manager.undo();
        assert!(manager.can_redo());

        let state2 = UndoState {
            actions: vec![EditAction::Insert {
                position: 0,
                text: "b".to_string(),
            }],
            cursor_position: 1,
        };

        manager.push(state2);
        assert!(!manager.can_redo());
    }
}
