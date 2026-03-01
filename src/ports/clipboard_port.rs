use crate::error::Result;

pub trait Clipboard: Send + Sync {
    fn get_text(&self) -> Result<String>;
    fn set_text(&self, text: &str) -> Result<()>;
}
