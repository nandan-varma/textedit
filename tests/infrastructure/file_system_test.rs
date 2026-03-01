use std::fs;
use tempfile::TempDir;
use textedit::infrastructure::NativeFileSystem;

#[test]
fn test_native_file_system_read_nonexistent_returns_error() {
    let fs = NativeFileSystem::new();
    let result = fs.read(std::path::Path::new("/nonexistent/file.txt"));

    assert!(result.is_err());
}

#[test]
fn test_native_file_system_read_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "hello world").unwrap();

    let fs = NativeFileSystem::new();
    let result = fs.read(&file_path);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello world");
}

#[test]
fn test_native_file_system_read_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    let content = "line1\nline2\nline3";
    fs::write(&file_path, content).unwrap();

    let fs = NativeFileSystem::new();
    let result = fs.read(&file_path).unwrap();

    assert_eq!(result, content);
}

#[test]
fn test_native_file_system_write_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("new_file.txt");

    let fs = NativeFileSystem::new();
    let result = fs.write(&file_path, "test content");

    assert!(result.is_ok());
    assert!(file_path.exists());

    let read_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_content, "test content");
}

#[test]
fn test_native_file_system_write_overwrites() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "original").unwrap();

    let fs = NativeFileSystem::new();
    let result = fs.write(&file_path, "replaced");

    assert!(result.is_ok());

    let read_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_content, "replaced");
}

#[test]
fn test_native_file_system_exists_true() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "content").unwrap();

    let fs = NativeFileSystem::new();
    assert!(fs.exists(&file_path));
}

#[test]
fn test_native_file_system_exists_false() {
    let fs = NativeFileSystem::new();
    assert!(!fs.exists(std::path::Path::new("/nonexistent/file.txt")));
}

#[test]
fn test_native_file_system_new() {
    let fs = NativeFileSystem::new();
    // Just verify it can be created
    assert!(true);
}

#[test]
fn test_native_file_system_default() {
    let fs = NativeFileSystem::default();
    // Just verify it can be created
    assert!(true);
}
