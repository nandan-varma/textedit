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
