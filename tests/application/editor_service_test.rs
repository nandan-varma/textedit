use textedit::application::EditorService;

fn create_editor_with_content(content: &str) -> EditorService {
    let mut editor = EditorService::new();
    editor.load_content(content.to_string());
    editor
}

#[test]
fn test_editor_service_new_default_state() {
    let editor = EditorService::new();

    assert_eq!(editor.buffer().len_chars(), 0);
    assert_eq!(editor.cursor().position(), 0);
    assert!(editor.file_path().is_none());
    assert!(!editor.is_modified());
    assert!(editor.show_line_numbers());
    assert!(editor.show_status_bar());
}

#[test]
fn test_editor_service_buffer_access() {
    let editor = EditorService::new();
    let _buffer = editor.buffer();
    // Just verify we can access the buffer
}

#[test]
fn test_editor_service_cursor_access() {
    let editor = EditorService::new();
    let _cursor = editor.cursor();
    // Just verify we can access the cursor
}

#[test]
fn test_editor_service_set_file_path() {
    let mut editor = EditorService::new();
    editor.set_file_path("/path/to/file.txt".to_string());

    assert_eq!(editor.file_path(), Some("/path/to/file.txt"));
    assert!(!editor.is_modified());
}

#[test]
fn test_editor_service_file_path() {
    let mut editor = EditorService::new();
    assert!(editor.file_path().is_none());

    editor.set_file_path("test.rs".to_string());
    assert_eq!(editor.file_path(), Some("test.rs"));
}

#[test]
fn test_editor_service_is_modified_default_false() {
    let editor = EditorService::new();
    assert!(!editor.is_modified());
}

#[test]
fn test_editor_service_set_modified() {
    let mut editor = EditorService::new();
    assert!(!editor.is_modified());

    editor.set_modified(true);
    assert!(editor.is_modified());

    editor.set_modified(false);
    assert!(!editor.is_modified());
}

#[test]
fn test_editor_service_buffer_mut_sets_modified() {
    let mut editor = EditorService::new();
    assert!(!editor.is_modified());

    editor.buffer_mut().insert(0, "test");

    assert!(editor.is_modified());
}

#[test]
fn test_editor_service_toggle_line_numbers() {
    let mut editor = EditorService::new();
    assert!(editor.show_line_numbers());

    editor.toggle_line_numbers();
    assert!(!editor.show_line_numbers());

    editor.toggle_line_numbers();
    assert!(editor.show_line_numbers());
}

#[test]
fn test_editor_service_show_line_numbers_default_true() {
    let editor = EditorService::new();
    assert!(editor.show_line_numbers());
}

#[test]
fn test_editor_service_toggle_status_bar() {
    let mut editor = EditorService::new();
    assert!(editor.show_status_bar());

    editor.toggle_status_bar();
    assert!(!editor.show_status_bar());

    editor.toggle_status_bar();
    assert!(editor.show_status_bar());
}

#[test]
fn test_editor_service_show_status_bar_default_true() {
    let editor = EditorService::new();
    assert!(editor.show_status_bar());
}

#[test]
fn test_editor_service_find_next_no_query_returns_false() {
    let mut editor = create_editor_with_content("hello world");
    let result = editor.find_next();
    assert!(!result);
}

#[test]
fn test_editor_service_find_next_finds_match() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_find_query(Some("world".to_string()));

    let result = editor.find_next();
    assert!(result);

    let sel = editor.cursor().selection().unwrap();
    let (start, end) = sel.range();
    assert_eq!(start, 6);
    assert_eq!(end, 11);
}

#[test]
fn test_editor_service_find_next_wraps_around() {
    let mut editor = create_editor_with_content("hello world hello");
    editor.set_find_query(Some("hello".to_string()));

    // First find should find first "hello"
    let result1 = editor.find_next();
    assert!(result1);
    let sel1 = editor.cursor().selection().unwrap();
    assert_eq!(sel1.range().0, 0);

    // Second find should find second "hello"
    let result2 = editor.find_next();
    assert!(result2);
    let sel2 = editor.cursor().selection().unwrap();
    assert_eq!(sel2.range().0, 12);
}

#[test]
fn test_editor_service_find_prev_no_query_returns_false() {
    let mut editor = create_editor_with_content("hello world");
    let result = editor.find_prev();
    assert!(!result);
}

#[test]
fn test_editor_service_find_prev_finds_match() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_find_query(Some("hello".to_string()));

    let result = editor.find_prev();
    assert!(result);

    let sel = editor.cursor().selection().unwrap();
    let (start, end) = sel.range();
    assert_eq!(start, 0);
    assert_eq!(end, 5);
}

#[test]
fn test_editor_service_find_prev_wraps_around() {
    let mut editor = create_editor_with_content("hello world hello");
    editor.set_find_query(Some("hello".to_string()));

    // First find_prev should find last "hello" (starting from position 0)
    let result1 = editor.find_prev();
    assert!(result1);
    let sel1 = editor.cursor().selection().unwrap();
    assert_eq!(sel1.range().0, 12);
}

#[test]
fn test_editor_service_set_find_query() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_find_query(Some("test".to_string()));

    assert_eq!(editor.find_query(), Some("test"));
}

#[test]
fn test_editor_service_new_file() {
    let mut editor = create_editor_with_content("some content");
    editor.set_file_path("/path/to/file.txt".to_string());
    editor.set_modified(true);

    editor.new_file();

    assert_eq!(editor.buffer().len_chars(), 0);
    assert_eq!(editor.cursor().position(), 0);
    assert!(editor.file_path().is_none());
    assert!(!editor.is_modified());
}

#[test]
fn test_editor_service_load_content() {
    let mut editor = EditorService::new();
    editor.load_content("hello world".to_string());

    assert_eq!(editor.buffer().as_str(), "hello world");
    assert_eq!(editor.cursor().position(), 0);
}

#[test]
fn test_editor_service_load_content_clears_history() {
    let mut editor = EditorService::new();
    editor.buffer_mut().insert(0, "test");
    editor.history_mut().undo();

    editor.load_content("new content".to_string());

    // History should be cleared after load
    assert!(!editor.history_mut().can_undo());
}

#[test]
fn test_editor_service_find_next_empty_query() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_find_query(Some("".to_string()));

    let result = editor.find_next();
    assert!(!result);
}

#[test]
fn test_editor_service_find_not_found() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_find_query(Some("xyz".to_string()));

    let result = editor.find_next();
    assert!(!result);
}
