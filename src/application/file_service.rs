use crate::error::Result;
use crate::ports::FileRepository;
use std::path::Path;

#[allow(dead_code)]
pub struct FileService<F: FileRepository> {
    repository: F,
}

#[allow(dead_code)]
impl<F: FileRepository> FileService<F> {
    pub fn new(repository: F) -> Self {
        Self { repository }
    }

    pub fn open(&self, path: &Path) -> Result<String> {
        self.repository.read(path)
    }

    pub fn save(&self, path: &Path, content: &str) -> Result<()> {
        self.repository.write(path, content)
    }

    pub fn exists(&self, path: &Path) -> bool {
        self.repository.exists(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{FileRepository, FileService};
    use crate::error::{EditorError, Result};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::RwLock;

    struct MockFileRepository {
        files: RwLock<HashMap<PathBuf, String>>,
    }

    impl MockFileRepository {
        fn new() -> Self {
            Self {
                files: RwLock::new(HashMap::new()),
            }
        }
    }

    impl FileRepository for MockFileRepository {
        fn read(&self, path: &std::path::Path) -> Result<String> {
            let files = self.files.read().unwrap();
            files.get(path).cloned().ok_or_else(|| {
                EditorError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file not found",
                ))
            })
        }

        fn write(&self, path: &std::path::Path, content: &str) -> Result<()> {
            let mut files = self.files.write().unwrap();
            files.insert(path.to_path_buf(), content.to_string());
            Ok(())
        }

        fn exists(&self, path: &std::path::Path) -> bool {
            let files = self.files.read().unwrap();
            files.contains_key(path)
        }
    }

    #[test]
    fn test_file_service_new() {
        let mock = MockFileRepository::new();
        let _service = FileService::new(mock);
        assert!(true);
    }

    #[test]
    fn test_file_service_save_and_open() {
        let mock = MockFileRepository::new();
        let service = FileService::new(mock);

        let path = PathBuf::from("/test/file.txt");
        service.save(&path, "hello world").unwrap();

        let content = service.open(&path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_file_service_exists() {
        let mock = MockFileRepository::new();
        let service = FileService::new(mock);

        let path = PathBuf::from("/test/file.txt");
        assert!(!service.exists(&path));

        service.save(&path, "content").unwrap();
        assert!(service.exists(&path));
    }

    #[test]
    fn test_file_service_open_nonexistent() {
        let mock = MockFileRepository::new();
        let service = FileService::new(mock);

        let path = PathBuf::from("/nonexistent/file.txt");
        let result = service.open(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_service_overwrite() {
        let mock = MockFileRepository::new();
        let service = FileService::new(mock);

        let path = PathBuf::from("/test/file.txt");
        service.save(&path, "original").unwrap();
        service.save(&path, "updated").unwrap();

        let content = service.open(&path).unwrap();
        assert_eq!(content, "updated");
    }
}
