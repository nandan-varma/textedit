use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum EditorError {
    #[error("File operation failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Buffer error: {0}")]
    BufferError(String),

    #[error("Invalid position: line {0}, column {1}")]
    InvalidPosition(usize, usize),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Window error: {0}")]
    WindowError(String),

    #[error("GPU error: {0}")]
    GpuError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Syntax highlighting error: {0}")]
    SyntaxError(String),

    #[error("Invalid theme: {0}")]
    InvalidTheme(String),
}

pub type Result<T> = std::result::Result<T, EditorError>;

#[cfg(test)]
mod tests {
    use super::EditorError;
    use std::io;

    #[test]
    fn test_editor_error_display_io() {
        let err = EditorError::IoError(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let msg = err.to_string();
        assert!(msg.contains("File operation failed"));
    }

    #[test]
    fn test_editor_error_display_buffer() {
        let err = EditorError::BufferError("out of bounds".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Buffer error"));
        assert!(msg.contains("out of bounds"));
    }

    #[test]
    fn test_editor_error_display_invalid_position() {
        let err = EditorError::InvalidPosition(5, 10);
        let msg = err.to_string();
        assert!(msg.contains("Invalid position"));
        assert!(msg.contains("5"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn test_editor_error_display_clipboard() {
        let err = EditorError::ClipboardError("access denied".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Clipboard error"));
    }

    #[test]
    fn test_editor_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test error");
        let editor_err: EditorError = io_err.into();

        match editor_err {
            EditorError::IoError(_e) => {
                assert!(true);
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_editor_error_debug() {
        let err = EditorError::BufferError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("BufferError"));
    }
}
