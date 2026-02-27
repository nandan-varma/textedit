use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("Failed to read file: {:?}", path.as_ref()))
}

pub fn save_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    fs::write(&path, content).with_context(|| format!("Failed to write file: {:?}", path.as_ref()))
}
