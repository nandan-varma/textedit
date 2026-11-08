//! Document model representing an open file/buffer with its associated state.
//!
//! Each Document encapsulates a Buffer, Cursor, History, and metadata about the file.

use super::buffer::Buffer;
use super::cursor::Cursor;
use super::operations::OperationHistory;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(Uuid);

impl DocumentId {
    /// Create a new unique document ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the underlying UUID
    #[inline]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Document metadata
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// Display title (filename or "Untitled")
    pub title: String,
    /// Detected or set language mode
    pub language: Option<String>,
    /// File encoding (default UTF-8)
    pub encoding: String,
    /// Line ending style
    pub line_ending: LineEnding,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            language: None,
            encoding: "UTF-8".to_string(),
            line_ending: LineEnding::default(),
        }
    }
}

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    #[default]
    LF,
    CRLF,
    CR,
}

impl LineEnding {
    /// Get the string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
            LineEnding::CR => "\r",
        }
    }

    /// Detect line ending from content
    pub fn detect(content: &str) -> Self {
        if content.contains("\r\n") {
            LineEnding::CRLF
        } else if content.contains('\r') {
            LineEnding::CR
        } else {
            LineEnding::LF
        }
    }
}

/// A complete document with buffer, cursor, history, and metadata
#[derive(Debug)]
pub struct Document {
    /// Unique identifier
    id: DocumentId,
    /// Text buffer
    buffer: Buffer,
    /// Cursor state
    cursor: Cursor,
    /// Edit history for undo/redo
    history: OperationHistory,
    /// File path (None for unsaved)
    file_path: Option<PathBuf>,
    /// Whether document has unsaved changes
    modified: bool,
    /// Document metadata
    metadata: DocumentMetadata,
    /// Scroll position (visual line offset)
    scroll_offset: usize,
}

impl Document {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            id: DocumentId::new(),
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            history: OperationHistory::new(),
            file_path: None,
            modified: false,
            metadata: DocumentMetadata::default(),
            scroll_offset: 0,
        }
    }

    /// Create a document from file content
    pub fn from_file(path: PathBuf, content: String) -> Self {
        let line_ending = LineEnding::detect(&content);
        let language = Self::detect_language(&path);
        let title = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let mut buffer = Buffer::new();
        buffer.set_content(&content);

        Self {
            id: DocumentId::new(),
            buffer,
            cursor: Cursor::new(),
            history: OperationHistory::new(),
            file_path: Some(path),
            modified: false,
            metadata: DocumentMetadata {
                title,
                language,
                encoding: "UTF-8".to_string(),
                line_ending,
            },
            scroll_offset: 0,
        }
    }

    /// Detect language from file extension
    fn detect_language(path: &PathBuf) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "js" | "mjs" | "cjs" => "javascript",
                "ts" | "mts" | "cts" => "typescript",
                "tsx" => "tsx",
                "jsx" => "jsx",
                "py" | "pyw" => "python",
                "go" => "go",
                "c" | "h" => "c",
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
                "json" => "json",
                "toml" => "toml",
                "yaml" | "yml" => "yaml",
                "html" | "htm" => "html",
                "css" => "css",
                "scss" | "sass" => "scss",
                "md" | "markdown" => "markdown",
                "sh" | "bash" | "zsh" => "shell",
                "sql" => "sql",
                "xml" => "xml",
                "java" => "java",
                "kt" | "kts" => "kotlin",
                "swift" => "swift",
                "rb" => "ruby",
                "php" => "php",
                "lua" => "lua",
                "vim" => "vim",
                _ => ext,
            })
            .map(String::from)
    }

    // Getters

    #[inline]
    pub fn id(&self) -> DocumentId {
        self.id
    }

    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.modified = true;
        &mut self.buffer
    }

    #[inline]
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    #[inline]
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    #[inline]
    pub fn history(&self) -> &OperationHistory {
        &self.history
    }

    #[inline]
    pub fn history_mut(&mut self) -> &mut OperationHistory {
        &mut self.history
    }

    #[inline]
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn set_file_path(&mut self, path: PathBuf) {
        let title = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        self.metadata.language = Self::detect_language(&path);
        self.metadata.title = title;
        self.file_path = Some(path);
        self.modified = false;
    }

    #[inline]
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    #[inline]
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    #[inline]
    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    #[inline]
    pub fn title(&self) -> &str {
        &self.metadata.title
    }

    #[inline]
    pub fn language(&self) -> Option<&str> {
        self.metadata.language.as_deref()
    }

    #[inline]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    #[inline]
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Get content as string for saving
    pub fn content(&self) -> String {
        self.buffer.as_str()
    }

    /// Load content into document
    pub fn load_content(&mut self, content: &str) {
        self.buffer.set_content(content);
        self.cursor = Cursor::new();
        self.history.clear();
        self.scroll_offset = 0;
        self.modified = false;
        self.metadata.line_ending = LineEnding::detect(content);
    }

    /// Clear the document
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = Cursor::new();
        self.history.clear();
        self.scroll_offset = 0;
        self.modified = false;
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = Document::new();
        assert!(!doc.is_modified());
        assert!(doc.file_path().is_none());
        assert_eq!(doc.title(), "Untitled");
    }

    #[test]
    fn test_document_from_file() {
        let path = PathBuf::from("/test/file.rs");
        let content = "fn main() {}\n".to_string();
        let doc = Document::from_file(path, content);

        assert_eq!(doc.title(), "file.rs");
        assert_eq!(doc.language(), Some("rust"));
        assert!(!doc.is_modified());
    }

    #[test]
    fn test_document_id_unique() {
        let id1 = DocumentId::new();
        let id2 = DocumentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_line_ending_detect() {
        assert_eq!(LineEnding::detect("hello\nworld"), LineEnding::LF);
        assert_eq!(LineEnding::detect("hello\r\nworld"), LineEnding::CRLF);
        assert_eq!(LineEnding::detect("hello\rworld"), LineEnding::CR);
    }

    #[test]
    fn test_language_detection() {
        let path = PathBuf::from("test.js");
        let doc = Document::from_file(path, String::new());
        assert_eq!(doc.language(), Some("javascript"));
    }
}
