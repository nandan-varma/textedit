use textedit::domain::Buffer;

#[test]
fn test_buffer_new_is_empty() {
    let buffer = Buffer::new();
    assert_eq!(buffer.len_chars(), 0);
    assert_eq!(buffer.len_lines(), 1);
}

#[test]
fn test_buffer_from_str_creates_content() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.len_chars(), 5);
    assert_eq!(buffer.as_str(), "hello");
}

#[test]
fn test_buffer_from_str_multibyte_characters() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.len_chars(), 5);
}

#[test]
fn test_buffer_insert_at_beginning() {
    let mut buffer = Buffer::from_str("hello");
    buffer.insert(0, "start");
    assert_eq!(buffer.as_str(), "starthello");
}

#[test]
fn test_buffer_insert_at_middle() {
    let mut buffer = Buffer::from_str("hello");
    buffer.insert(2, "X");
    assert_eq!(buffer.as_str(), "heXllo");
}

#[test]
fn test_buffer_insert_at_end() {
    let mut buffer = Buffer::from_str("hello");
    buffer.insert(5, " world");
    assert_eq!(buffer.as_str(), "hello world");
}

#[test]
fn test_buffer_insert_out_of_bounds_no_op() {
    let mut buffer = Buffer::from_str("hello");
    buffer.insert(100, "test");
    assert_eq!(buffer.as_str(), "hello");
}

#[test]
fn test_buffer_insert_at_exact_length() {
    let mut buffer = Buffer::from_str("hello");
    buffer.insert(5, "!");
    assert_eq!(buffer.as_str(), "hello!");
}

#[test]
fn test_buffer_remove_middle() {
    let mut buffer = Buffer::from_str("hello world");
    buffer.remove(5, 6);
    assert_eq!(buffer.as_str(), "hellord");
}

#[test]
fn test_buffer_remove_from_beginning() {
    let mut buffer = Buffer::from_str("hello");
    buffer.remove(0, 2);
    assert_eq!(buffer.as_str(), "llo");
}

#[test]
fn test_buffer_remove_from_end() {
    let mut buffer = Buffer::from_str("hello");
    buffer.remove(3, 2);
    assert_eq!(buffer.as_str(), "hel");
}

#[test]
fn test_buffer_remove_beyond_length_no_op() {
    let mut buffer = Buffer::from_str("hello");
    buffer.remove(2, 100);
    assert_eq!(buffer.as_str(), "hello");
}

#[test]
fn test_buffer_remove_exact_length() {
    let mut buffer = Buffer::from_str("hello");
    buffer.remove(0, 5);
    assert_eq!(buffer.as_str(), "");
}

#[test]
fn test_buffer_get_char_valid_index() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.get_char(0), Some('h'));
    assert_eq!(buffer.get_char(4), Some('o'));
    assert_eq!(buffer.get_char(2), Some('l'));
}

#[test]
fn test_buffer_get_char_out_of_bounds_returns_none() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.get_char(5), None);
    assert_eq!(buffer.get_char(100), None);
}

#[test]
fn test_buffer_len_chars_empty() {
    let buffer = Buffer::new();
    assert_eq!(buffer.len_chars(), 0);
}

#[test]
fn test_buffer_len_chars_with_content() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.len_chars(), 5);
}

#[test]
fn test_buffer_len_lines_single_line() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.len_lines(), 1);
}

#[test]
fn test_buffer_len_lines_multiple_lines() {
    let buffer = Buffer::from_str("line1\nline2\nline3");
    assert_eq!(buffer.len_lines(), 3);
}

#[test]
fn test_buffer_len_lines_empty_with_newlines() {
    let buffer = Buffer::from_str("\n\n");
    assert_eq!(buffer.len_lines(), 3);
}

#[test]
fn test_buffer_line_valid_index() {
    let buffer = Buffer::from_str("line1\nline2\nline3");
    assert_eq!(buffer.line(0), Some("line1".to_string()));
    assert_eq!(buffer.line(1), Some("line2".to_string()));
    assert_eq!(buffer.line(2), Some("line3".to_string()));
}

#[test]
fn test_buffer_line_out_of_bounds_returns_none() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.line(5), None);
    assert_eq!(buffer.line(100), None);
}

#[test]
fn test_buffer_char_to_line_col_single_line() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.char_to_line_col(0), (0, 0));
    assert_eq!(buffer.char_to_line_col(2), (0, 2));
    assert_eq!(buffer.char_to_line_col(4), (0, 4));
}

#[test]
fn test_buffer_char_to_line_col_multiple_lines() {
    let buffer = Buffer::from_str("line1\nline2");
    // "line1\n" = 6 chars, so 'l' in line2 is at index 6
    assert_eq!(buffer.char_to_line_col(0), (0, 0));
    assert_eq!(buffer.char_to_line_col(4), (0, 4));
    assert_eq!(buffer.char_to_line_col(6), (1, 0));
    assert_eq!(buffer.char_to_line_col(7), (1, 1));
    assert_eq!(buffer.char_to_line_col(11), (1, 5));
}

#[test]
fn test_buffer_char_to_line_col_beyond_length() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.char_to_line_col(100), (0, 5));
}

#[test]
fn test_buffer_line_col_to_char_valid() {
    let buffer = Buffer::from_str("line1\nline2");
    assert_eq!(buffer.line_col_to_char(0, 0), Some(0));
    assert_eq!(buffer.line_col_to_char(0, 4), Some(4));
    assert_eq!(buffer.line_col_to_char(1, 0), Some(6));
    assert_eq!(buffer.line_col_to_char(1, 4), Some(10));
}

#[test]
fn test_buffer_line_col_to_char_out_of_bounds_returns_none() {
    let buffer = Buffer::from_str("line1\nline2");
    assert_eq!(buffer.line_col_to_char(0, 10), None);
    assert_eq!(buffer.line_col_to_char(1, 10), None);
    assert_eq!(buffer.line_col_to_char(2, 0), None);
}

#[test]
fn test_buffer_line_to_char() {
    let buffer = Buffer::from_str("line1\nline2\nline3");
    assert_eq!(buffer.line_to_char(0), 0);
    assert_eq!(buffer.line_to_char(1), 6);
    assert_eq!(buffer.line_to_char(2), 12);
}

#[test]
fn test_buffer_line_to_char_out_of_bounds() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.line_to_char(1), 5);
    assert_eq!(buffer.line_to_char(100), 5);
}

#[test]
fn test_buffer_as_str() {
    let buffer = Buffer::from_str("hello world");
    assert_eq!(buffer.as_str(), "hello world");
}

#[test]
fn test_buffer_as_str_multiline() {
    let buffer = Buffer::from_str("line1\nline2\nline3");
    assert_eq!(buffer.as_str(), "line1\nline2\nline3");
}

#[test]
fn test_buffer_clear() {
    let mut buffer = Buffer::from_str("hello");
    buffer.clear();
    assert_eq!(buffer.len_chars(), 0);
    assert_eq!(buffer.as_str(), "");
}

#[test]
fn test_buffer_set_content() {
    let mut buffer = Buffer::new();
    buffer.set_content("initial");
    assert_eq!(buffer.as_str(), "initial");

    buffer.set_content("replaced");
    assert_eq!(buffer.as_str(), "replaced");
}

#[test]
fn test_buffer_set_content_overwrites() {
    let mut buffer = Buffer::from_str("hello");
    buffer.set_content("world");
    assert_eq!(buffer.len_chars(), 5);
    assert_eq!(buffer.as_str(), "world");
}

#[test]
fn test_buffer_clone() {
    let buffer1 = Buffer::from_str("hello");
    let buffer2 = buffer1.clone();

    assert_eq!(buffer1.as_str(), buffer2.as_str());
    assert_eq!(buffer1.len_chars(), buffer2.len_chars());
}

#[test]
fn test_buffer_clone_is_independent() {
    let buffer1 = Buffer::from_str("hello");
    let mut buffer2 = buffer1.clone();

    buffer2.insert(0, "prefix-");
    assert_eq!(buffer1.as_str(), "hello");
    assert_eq!(buffer2.as_str(), "prefix-hello");
}

#[test]
fn test_buffer_default() {
    let buffer = Buffer::default();
    assert_eq!(buffer.len_chars(), 0);
}

#[test]
fn test_buffer_line_len_chars() {
    let buffer = Buffer::from_str("hello\nworld");
    assert_eq!(buffer.line_len_chars(0), 5);
    assert_eq!(buffer.line_len_chars(1), 5);
}

#[test]
fn test_buffer_line_len_chars_out_of_bounds() {
    let buffer = Buffer::from_str("hello");
    assert_eq!(buffer.line_len_chars(5), 0);
}

#[test]
fn test_buffer_line_slice() {
    let buffer = Buffer::from_str("hello\nworld");
    let slice = buffer.line_slice(0);
    assert!(slice.is_some());
    assert_eq!(slice.unwrap().to_string(), "hello");
}

#[test]
fn test_buffer_line_slice_out_of_bounds() {
    let buffer = Buffer::from_str("hello");
    assert!(buffer.line_slice(5).is_none());
}
