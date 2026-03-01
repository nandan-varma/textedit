use crate::error::Result;
use std::path::Path;

#[allow(dead_code)]
pub trait FileRepository: Send + Sync {
    fn read(&self, path: &Path) -> Result<String>;
    fn write(&self, path: &Path, content: &str) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
}
