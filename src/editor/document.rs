use crate::editor::TextBuffer;
use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FileEncoding {
    #[default]
    Utf8,
    Utf16Le,
    Utf16Be,
    Ascii,
    Other,
}

impl FileEncoding {
    pub fn from_encoding(encoding: &'static Encoding) -> Self {
        if encoding == encoding_rs::UTF_8 {
            FileEncoding::Utf8
        } else if encoding == encoding_rs::UTF_16LE {
            FileEncoding::Utf16Le
        } else if encoding == encoding_rs::UTF_16BE {
            FileEncoding::Utf16Be
        } else {
            FileEncoding::Other
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FileEncoding::Utf8 => "UTF-8",
            FileEncoding::Utf16Le => "UTF-16 LE",
            FileEncoding::Utf16Be => "UTF-16 BE",
            FileEncoding::Ascii => "ASCII",
            FileEncoding::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LineEnding {
    #[default]
    Crlf,
    Lf,
    Cr,
}

impl LineEnding {
    pub fn from_str(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::Crlf
        } else if text.contains('\r') {
            LineEnding::Cr
        } else {
            LineEnding::Lf
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Crlf => "\r\n",
            LineEnding::Lf => "\n",
            LineEnding::Cr => "\r",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LineEnding::Crlf => "CRLF",
            LineEnding::Lf => "LF",
            LineEnding::Cr => "CR",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub buffer: TextBuffer,
    pub file_path: Option<PathBuf>,
    pub encoding: FileEncoding,
    pub line_ending: LineEnding,
    pub is_readonly: bool,
    #[serde(skip)]
    pub original_content: String,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            file_path: None,
            encoding: FileEncoding::Utf8,
            line_ending: LineEnding::Lf,
            is_readonly: false,
            original_content: String::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let line_ending = LineEnding::from_str(text);
        Self {
            buffer: TextBuffer::from_string(text),
            file_path: None,
            encoding: FileEncoding::Utf8,
            line_ending,
            is_readonly: false,
            original_content: text.to_string(),
        }
    }

    pub fn file_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.buffer.set_modified(modified);
    }

    pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;

        let (content, encoding_used) = decode_lossless(&bytes);
        let encoding = FileEncoding::from_encoding(encoding_used);

        let line_ending = LineEnding::from_str(&content);

        self.buffer = TextBuffer::from_string(&content);
        self.file_path = Some(path.clone());
        self.encoding = encoding;
        self.line_ending = line_ending;
        self.original_content = content;
        self.buffer.set_modified(false);

        Ok(())
    }

    pub fn save_to_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let content = self.buffer.get_text();

        let (encoded, encoding_used, _) = match self.encoding {
            FileEncoding::Utf8 => {
                let (enc, _, _) = encoding_rs::UTF_8.encode(&content);
                (enc.into_owned(), encoding_rs::UTF_8, "UTF-8")
            }
            FileEncoding::Utf16Le => {
                let (enc, _, _) = encoding_rs::UTF_16LE.encode(&content);
                (enc.into_owned(), encoding_rs::UTF_16LE, "UTF-16 LE")
            }
            FileEncoding::Utf16Be => {
                let (enc, _, _) = encoding_rs::UTF_16BE.encode(&content);
                (enc.into_owned(), encoding_rs::UTF_16BE, "UTF-16 BE")
            }
            _ => {
                let (enc, _, _) = encoding_rs::UTF_8.encode(&content);
                (enc.into_owned(), encoding_rs::UTF_8, "UTF-8")
            }
        };

        fs::write(path, encoded).map_err(|e| e.to_string())?;

        self.file_path = Some(path.clone());
        self.buffer.set_modified(false);

        log::info!("Saved file as {}", encoding_used.name());

        Ok(())
    }

    pub fn convert_line_endings(&mut self, ending: LineEnding) {
        if self.line_ending == ending {
            return;
        }

        let content = self.buffer.get_text();
        let converted = match (self.line_ending, ending) {
            (LineEnding::Crlf, LineEnding::Lf) => content.replace("\r\n", "\n"),
            (LineEnding::Lf, LineEnding::Crlf) => content.replace("\n", "\r\n"),
            (LineEnding::Cr, LineEnding::Lf) => content.replace('\r', "\n"),
            (LineEnding::Cr, LineEnding::Crlf) => content.replace('\r', "\r\n"),
            (LineEnding::Crlf, LineEnding::Cr) => content.replace("\r\n", "\r"),
            (LineEnding::Lf, LineEnding::Cr) => content.replace('\n', "\r"),
            _ => content,
        };

        self.buffer = TextBuffer::from_string(&converted);
        self.line_ending = ending;
    }

    pub fn is_untitled(&self) -> bool {
        self.file_path.is_none()
    }

    pub fn get_path_string(&self) -> String {
        self.file_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

fn decode_lossless(bytes: &[u8]) -> (String, &'static Encoding) {
    use std::str::from_utf8;

    // Try UTF-8 first
    if let Ok(s) = from_utf8(bytes) {
        return (s.to_string(), encoding_rs::UTF_8);
    }

    // Try UTF-16 LE
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(bytes);
        return (decoded.into_owned(), encoding_rs::UTF_16LE);
    }

    // Try UTF-16 BE
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let (decoded, _, _) = encoding_rs::UTF_16BE.decode(bytes);
        return (decoded.into_owned(), encoding_rs::UTF_16BE);
    }

    // Fallback to UTF-8 with lossy conversion
    (
        String::from_utf8_lossy(bytes).into_owned(),
        encoding_rs::UTF_8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_document() {
        let doc = Document::new();
        assert!(doc.is_untitled());
        assert!(!doc.is_modified());
        assert_eq!(doc.encoding, FileEncoding::Utf8);
    }

    #[test]
    fn test_from_text() {
        let doc = Document::from_text("Line 1\r\nLine 2");
        assert_eq!(doc.line_ending, LineEnding::Crlf);
    }

    #[test]
    fn test_line_ending_detection() {
        assert_eq!(LineEnding::from_str("hello\r\nworld"), LineEnding::Crlf);
        assert_eq!(LineEnding::from_str("hello\nworld"), LineEnding::Lf);
        assert_eq!(LineEnding::from_str("hello\rworld"), LineEnding::Cr);
    }

    #[test]
    fn test_save_and_load() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        use std::io::Write;
        tmpfile.write_all(b"Hello, World!").unwrap();
        let path = tmpfile.path().to_path_buf();

        let mut doc = Document::new();
        doc.buffer = TextBuffer::from_string("Hello, World!");
        doc.save_to_file(&path).unwrap();

        let mut doc2 = Document::new();
        doc2.load_from_file(&path).unwrap();

        assert_eq!(doc2.buffer.get_text(), "Hello, World!");
    }

    #[test]
    fn test_convert_line_endings() {
        let mut doc = Document::from_text("Line 1\nLine 2");

        doc.convert_line_endings(LineEnding::Crlf);
        assert!(doc.buffer.get_text().contains("\r\n"));
    }

    #[test]
    fn test_file_name() {
        let mut doc = Document::new();
        assert_eq!(doc.file_name(), "Untitled");

        doc.file_path = Some(PathBuf::from("/path/to/test.txt"));
        assert_eq!(doc.file_name(), "test.txt");
    }

    #[test]
    fn test_modified_flag() {
        let mut doc = Document::new();
        assert!(!doc.is_modified());

        doc.buffer.insert(0, "test");
        assert!(doc.is_modified());

        doc.set_modified(false);
        assert!(!doc.is_modified());
    }
}
