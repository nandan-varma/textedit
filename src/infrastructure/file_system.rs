use crate::error::{EditorError, Result};
use crate::ports::FileRepository;
use std::fs;
use std::path::Path;

pub struct NativeFileSystem;

impl NativeFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileRepository for NativeFileSystem {
    fn read(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| EditorError::IoError(e))
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        fs::write(path, content).map_err(|e| EditorError::IoError(e))
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::NativeFileSystem;
    use crate::ports::FileRepository;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_native_file_system_new() {
        let _fs = NativeFileSystem::new();
        assert!(true);
    }

    #[test]
    fn test_native_file_system_default() {
        let _fs = NativeFileSystem::default();
        assert!(true);
    }

    #[test]
    fn test_read_nonexistent_file_returns_error() {
        let fs = NativeFileSystem::new();
        let result = fs.read(std::path::Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let fs = NativeFileSystem::new();
        let result = fs.read(&file_path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_write_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");

        let fs = NativeFileSystem::new();
        let result = fs.write(&file_path, "test content");

        assert!(result.is_ok());
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_overwrites_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "original").unwrap();

        let fs = NativeFileSystem::new();
        let result = fs.write(&file_path, "replaced");

        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "replaced");
    }

    #[test]
    fn test_exists_true_for_existing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let fs = NativeFileSystem::new();
        assert!(fs.exists(&file_path));
    }

    #[test]
    fn test_exists_false_for_nonexistent() {
        let fs = NativeFileSystem::new();
        assert!(!fs.exists(std::path::Path::new("/nonexistent/file.txt")));
    }
}
