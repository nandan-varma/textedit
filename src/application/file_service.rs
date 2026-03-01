use crate::error::Result;
use crate::ports::FileRepository;
use std::path::Path;

pub struct FileService<F: FileRepository> {
    repository: F,
}

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
