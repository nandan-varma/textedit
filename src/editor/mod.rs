pub mod buffer;
pub mod cursor;
pub mod keyboard;
pub mod operations;

pub use buffer::Buffer;
pub use cursor::Cursor;
pub use keyboard::KeyboardController;
pub use operations::OperationHistory;

pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    history: OperationHistory,
    file_path: Option<String>,
    is_modified: bool,
    show_line_numbers: bool,
    show_status_bar: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            history: OperationHistory::new(),
            file_path: None,
            is_modified: false,
            show_line_numbers: true,
            show_status_bar: true,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.is_modified = true;
        &mut self.buffer
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn history_mut(&mut self) -> &mut OperationHistory {
        &mut self.history
    }

    pub fn set_file_path(&mut self, path: String) {
        self.file_path = Some(path);
        self.is_modified = false;
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.is_modified = modified;
    }

    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    pub fn show_status_bar(&self) -> bool {
        self.show_status_bar
    }

    pub fn set_show_status_bar(&mut self, show: bool) {
        self.show_status_bar = show;
    }

    pub fn toggle_status_bar(&mut self) {
        self.show_status_bar = !self.show_status_bar;
    }
}
