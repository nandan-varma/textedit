use textedit::domain::operations::{Operation, OperationHistory};

#[test]
fn test_operation_insert_clone() {
    let op1 = Operation::Insert {
        position: 5,
        text: "hello".to_string(),
    };
    let op2 = op1.clone();

    match (op1, op2) {
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) => {
            assert_eq!(p1, p2);
            assert_eq!(t1, t2);
        }
        _ => panic!("Expected Insert operation"),
    }
}

#[test]
fn test_operation_delete_clone() {
    let op1 = Operation::Delete {
        position: 5,
        text: "hello".to_string(),
    };
    let op2 = op1.clone();

    match (op1, op2) {
        (
            Operation::Delete {
                position: p1,
                text: t1,
            },
            Operation::Delete {
                position: p2,
                text: t2,
            },
        ) => {
            assert_eq!(p1, p2);
            assert_eq!(t1, t2);
        }
        _ => panic!("Expected Delete operation"),
    }
}

#[test]
fn test_history_new_is_empty() {
    let history = OperationHistory::new();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_push_adds_to_undo_stack() {
    let mut history = OperationHistory::new();

    history.push(Operation::Insert {
        position: 0,
        text: "test".to_string(),
    });

    assert!(history.can_undo());
}

#[test]
fn test_history_push_clears_redo_stack() {
    let mut history = OperationHistory::new();

    // Push and undo to create redo
    history.push(Operation::Insert {
        position: 0,
        text: "test".to_string(),
    });
    history.undo();
    assert!(history.can_redo());

    // Push new operation should clear redo
    history.push(Operation::Insert {
        position: 5,
        text: "more".to_string(),
    });

    assert!(!history.can_redo());
}

#[test]
fn test_history_undo_insert_returns_delete_op() {
    let mut history = OperationHistory::new();

    history.push(Operation::Insert {
        position: 0,
        text: "hello".to_string(),
    });

    let op = history.undo();

    assert!(op.is_some());
    match op.unwrap() {
        Operation::Delete { position, text } => {
            assert_eq!(position, 0);
            assert_eq!(text, "hello");
        }
        _ => panic!("Expected Delete operation"),
    }
}

#[test]
fn test_history_undo_delete_returns_insert_op() {
    let mut history = OperationHistory::new();

    history.push(Operation::Delete {
        position: 0,
        text: "hello".to_string(),
    });

    let op = history.undo();

    assert!(op.is_some());
    match op.unwrap() {
        Operation::Insert { position, text } => {
            assert_eq!(position, 0);
            assert_eq!(text, "hello");
        }
        _ => panic!("Expected Insert operation"),
    }
}

#[test]
fn test_history_undo_empty_returns_none() {
    let mut history = OperationHistory::new();
    let op = history.undo();
    assert!(op.is_none());
}

#[test]
fn test_history_redo_insert_returns_insert_op() {
    let mut history = OperationHistory::new();

    history.push(Operation::Insert {
        position: 0,
        text: "hello".to_string(),
    });
    history.undo(); // undo the insert
    let op = history.redo();

    assert!(op.is_some());
    match op.unwrap() {
        Operation::Insert { position, text } => {
            assert_eq!(position, 0);
            assert_eq!(text, "hello");
        }
        _ => panic!("Expected Insert operation"),
    }
}

#[test]
fn test_history_redo_delete_returns_delete_op() {
    let mut history = OperationHistory::new();

    history.push(Operation::Delete {
        position: 0,
        text: "hello".to_string(),
    });
    history.undo(); // undo the delete
    let op = history.redo();

    assert!(op.is_some());
    match op.unwrap() {
        Operation::Delete { position, text } => {
            assert_eq!(position, 0);
            assert_eq!(text, "hello");
        }
        _ => panic!("Expected Delete operation"),
    }
}

#[test]
fn test_history_redo_empty_returns_none() {
    let mut history = OperationHistory::new();
    let op = history.redo();
    assert!(op.is_none());
}

#[test]
fn test_history_can_undo_true_after_push() {
    let mut history = OperationHistory::new();
    history.push(Operation::Insert {
        position: 0,
        text: "test".to_string(),
    });
    assert!(history.can_undo());
}

#[test]
fn test_history_can_undo_false_when_empty() {
    let history = OperationHistory::new();
    assert!(!history.can_undo());
}

#[test]
fn test_history_can_redo_true_after_undo() {
    let mut history = OperationHistory::new();
    history.push(Operation::Insert {
        position: 0,
        text: "test".to_string(),
    });
    history.undo();
    assert!(history.can_redo());
}

#[test]
fn test_history_can_redo_false_when_empty() {
    let history = OperationHistory::new();
    assert!(!history.can_redo());
}

#[test]
fn test_history_clear() {
    let mut history = OperationHistory::new();

    history.push(Operation::Insert {
        position: 0,
        text: "test".to_string(),
    });
    history.undo();

    assert!(history.can_undo());
    assert!(history.can_redo());

    history.clear();

    assert!(!history.can_undo());
    assert!(!history.can_redo());
}

#[test]
fn test_history_undo_redo_sequence() {
    let mut history = OperationHistory::new();

    // Insert "hello" at position 0
    history.push(Operation::Insert {
        position: 0,
        text: "hello".to_string(),
    });

    // Undo should return Delete operation
    let op1 = history.undo();
    assert!(op1.is_some());

    // Redo should return Insert operation
    let op2 = history.redo();
    assert!(op2.is_some());

    // Another undo
    let op3 = history.undo();
    assert!(op3.is_some());

    // Another undo - should still work since we have more in stack
    let op4 = history.undo();
    assert!(op4.is_none()); // Stack is now empty
}

#[test]
fn test_history_multiple_undos() {
    let mut history = OperationHistory::new();

    history.push(Operation::Insert {
        position: 0,
        text: "a".to_string(),
    });
    history.push(Operation::Insert {
        position: 1,
        text: "b".to_string(),
    });
    history.push(Operation::Insert {
        position: 2,
        text: "c".to_string(),
    });

    assert!(history.can_undo());

    let op1 = history.undo();
    assert!(op1.is_some());

    let op2 = history.undo();
    assert!(op2.is_some());

    let op3 = history.undo();
    assert!(op3.is_some());

    // Fourth undo should fail
    let op4 = history.undo();
    assert!(op4.is_none());
}

#[test]
fn test_history_default() {
    let history = OperationHistory::default();
    assert!(!history.can_undo());
    assert!(!history.can_redo());
}
