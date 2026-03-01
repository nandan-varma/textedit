use crate::error::{EditorError, Result};
use crate::ports::Clipboard;

pub struct ArboardClipboard;

impl ArboardClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArboardClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard for ArboardClipboard {
    fn get_text(&self) -> Result<String> {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| EditorError::ClipboardError(e.to_string()))?;

        clipboard
            .get_text()
            .map_err(|e| EditorError::ClipboardError(e.to_string()))
    }

    fn set_text(&self, text: &str) -> Result<()> {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| EditorError::ClipboardError(e.to_string()))?;

        clipboard
            .set_text(text)
            .map_err(|e| EditorError::ClipboardError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::ArboardClipboard;

    #[test]
    fn test_arboard_clipboard_new() {
        let clipboard = ArboardClipboard::new();
        assert!(true);
    }

    #[test]
    fn test_arboard_clipboard_default() {
        let clipboard = ArboardClipboard::default();
        assert!(true);
    }
}
