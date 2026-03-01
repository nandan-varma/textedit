use textedit::domain::buffer::Buffer;
use textedit::domain::cursor::{Cursor, Selection};

#[test]
fn test_selection_new() {
    let sel = Selection::new(5, 10);
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
}

#[test]
fn test_selection_range_ordered() {
    let sel = Selection::new(5, 10);
    let (start, end) = sel.range();
    assert_eq!(start, 5);
    assert_eq!(end, 10);
}

#[test]
fn test_selection_range_reverse_order() {
    let sel = Selection::new(10, 5);
    let (start, end) = sel.range();
    assert_eq!(start, 5);
    assert_eq!(end, 10);
}

#[test]
fn test_selection_is_empty_true() {
    let sel = Selection::new(5, 5);
    assert!(sel.is_empty());
}

#[test]
fn test_selection_is_empty_false() {
    let sel = Selection::new(5, 10);
    assert!(!sel.is_empty());
}

#[test]
fn test_selection_len() {
    let sel = Selection::new(5, 10);
    assert_eq!(sel.len(), 5);
}

#[test]
fn test_selection_len_reverse() {
    let sel = Selection::new(10, 5);
    assert_eq!(sel.len(), 5);
}

#[test]
fn test_selection_len_zero() {
    let sel = Selection::new(5, 5);
    assert_eq!(sel.len(), 0);
}

#[test]
fn test_cursor_new_default_position_zero() {
    let cursor = Cursor::new();
    assert_eq!(cursor.position(), 0);
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_set_position() {
    let mut cursor = Cursor::new();
    cursor.set_position(10);
    assert_eq!(cursor.position(), 10);
}

#[test]
fn test_cursor_set_position_clears_selection() {
    let mut cursor = Cursor::new();
    cursor.set_selection_start(5);
    cursor.set_selection_end(10);
    assert!(cursor.selection().is_some());

    cursor.set_position(15);
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_selection_none_when_no_selection() {
    let cursor = Cursor::new();
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_extend_selection_new() {
    let mut cursor = Cursor::new();
    cursor.set_position(5);
    cursor.extend_selection(10);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
    assert_eq!(cursor.position(), 10);
}

#[test]
fn test_cursor_extend_selection_existing() {
    let mut cursor = Cursor::new();
    cursor.set_selection_start(5);
    cursor.set_position(8);
    cursor.extend_selection(10);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
}

#[test]
fn test_cursor_set_selection_start() {
    let mut cursor = Cursor::new();
    cursor.set_position(10);
    cursor.set_selection_start(5);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
}

#[test]
fn test_cursor_set_selection_end() {
    let mut cursor = Cursor::new();
    cursor.set_position(5);
    cursor.set_selection_end(10);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
}

#[test]
fn test_cursor_clear_selection() {
    let mut cursor = Cursor::new();
    cursor.set_selection_start(5);
    cursor.set_selection_end(10);
    assert!(cursor.selection().is_some());

    cursor.clear_selection();
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_extend_selection_backward() {
    let mut cursor = Cursor::new();
    cursor.set_position(10);
    cursor.extend_selection_backward();

    assert_eq!(cursor.position(), 9);
    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 10);
    assert_eq!(sel.end, 9);
}

#[test]
fn test_cursor_extend_selection_forward() {
    let mut cursor = Cursor::new();
    cursor.set_position(5);
    cursor.extend_selection_forward(20);

    assert_eq!(cursor.position(), 6);
    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 6);
}

#[test]
fn test_cursor_extend_selection_backward_at_start() {
    let mut cursor = Cursor::new();
    cursor.set_position(0);
    cursor.extend_selection_backward();

    // Should not go below 0
    assert_eq!(cursor.position(), 0);
}

#[test]
fn test_cursor_extend_selection_forward_at_end() {
    let mut cursor = Cursor::new();
    cursor.set_position(10);
    cursor.extend_selection_forward(10);

    // Should not go past buffer length
    assert_eq!(cursor.position(), 10);
}

#[test]
fn test_cursor_move_forward() {
    let mut cursor = Cursor::new();
    cursor.set_position(5);
    cursor.move_forward(20);

    assert_eq!(cursor.position(), 6);
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_move_forward_at_end() {
    let mut cursor = Cursor::new();
    cursor.set_position(10);
    cursor.move_forward(10);

    assert_eq!(cursor.position(), 10);
}

#[test]
fn test_cursor_move_backward() {
    let mut cursor = Cursor::new();
    cursor.set_position(5);
    cursor.move_backward();

    assert_eq!(cursor.position(), 4);
    assert!(cursor.selection().is_none());
}

#[test]
fn test_cursor_move_backward_at_start() {
    let mut cursor = Cursor::new();
    cursor.set_position(0);
    cursor.move_backward();

    assert_eq!(cursor.position(), 0);
}

#[test]
fn test_cursor_move_to_line_start() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(8); // somewhere in "world"
    cursor.move_to_line_start(&buffer);

    assert_eq!(cursor.position(), 6); // start of line 2
}

#[test]
fn test_cursor_move_to_line_end() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(6); // start of "world"
    cursor.move_to_line_end(&buffer);

    assert_eq!(cursor.position(), 11); // end of "world"
}

#[test]
fn test_cursor_move_up() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(8); // in "world"
    cursor.move_up(&buffer);

    assert_eq!(cursor.position(), 2); // corresponding position in "hello"
}

#[test]
fn test_cursor_move_down() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(2); // in "hello"
    cursor.move_down(&buffer);

    assert_eq!(cursor.position(), 8); // corresponding position in "world"
}

#[test]
fn test_cursor_move_up_at_first_line() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(2); // in "hello"
    cursor.move_up(&buffer);

    // Should stay in place when at first line
    assert_eq!(cursor.position(), 2);
}

#[test]
fn test_cursor_move_down_at_last_line() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(8); // in "world"
    cursor.move_down(&buffer);

    // Should stay in place when at last line
    assert_eq!(cursor.position(), 8);
}

#[test]
fn test_cursor_select_line() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();
    cursor.set_position(7); // in "world"
    cursor.select_line(&buffer);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 6); // start of "world"
    assert_eq!(sel.end, 11); // end of "world"
    assert_eq!(cursor.position(), 11);
}

#[test]
fn test_cursor_select_word_at_cursor() {
    let buffer = Buffer::from_str("hello world foo");
    let mut cursor = Cursor::new();
    cursor.set_position(7); // in "world"
    cursor.select_word_at_cursor(&buffer);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 6); // start of "world"
    assert_eq!(sel.end, 11); // end of "world"
    assert_eq!(cursor.position(), 11);
}

#[test]
fn test_cursor_select_word_single_word() {
    let buffer = Buffer::from_str("hello");
    let mut cursor = Cursor::new();
    cursor.set_position(2); // in "hello"
    cursor.select_word_at_cursor(&buffer);

    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 0);
    assert_eq!(sel.end, 5);
}

#[test]
fn test_cursor_select_range() {
    let mut cursor = Cursor::new();
    cursor.select_range(5, 10);

    assert_eq!(cursor.position(), 5);
    let sel = cursor.selection().unwrap();
    assert_eq!(sel.start, 5);
    assert_eq!(sel.end, 10);
}

#[test]
fn test_cursor_preferred_col_preserved() {
    let buffer = Buffer::from_str("hello\nworld");
    let mut cursor = Cursor::new();

    // Move to position 3 in first line
    cursor.set_position(3);
    cursor.move_down(&buffer);

    // Preferred column should be preserved
    assert_eq!(cursor.position(), 3);
}
