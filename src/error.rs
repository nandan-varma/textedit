use thiserror::Error;

#[derive(Error, Debug)]
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
}

pub type Result<T> = std::result::Result<T, EditorError>;
