use crate::application::EditorService;
use crate::domain::operations::Operation;
use crate::infrastructure::clipboard::ArboardClipboard;
use crate::ports::clipboard_port::Clipboard;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

/// Direction for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    WordLeft,
    WordRight,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
}

/// Action returned by keyboard handler - decouples input handling from editor operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// No action to take
    None,
    /// Insert a single character
    InsertChar(char),
    /// Insert a string (e.g., tab as spaces)
    InsertText(String),
    /// Delete character before cursor
    Backspace,
    /// Delete character after cursor
    Delete,
    /// Move cursor in a direction, optionally extending selection
    MoveCursor(CursorDirection, bool),
    /// Select all text
    SelectAll,
    /// Copy selection to clipboard
    Copy,
    /// Cut selection to clipboard
    Cut,
    /// Paste from clipboard
    Paste,
    /// Undo last operation
    Undo,
    /// Redo last undone operation
    Redo,
    /// Insert newline
    NewLine,
    /// Insert tab (or spaces)
    Tab,
    /// Remove indentation
    Unindent,
    /// Clear selection
    ClearSelection,
    /// Open find modal (handled at app level)
    OpenFind,
    /// Open replace modal (handled at app level)
    OpenReplace,
    /// Find next match
    FindNext,
    /// Find previous match
    FindPrev,
}

pub struct KeyboardController {
    modifiers: ModifiersState,
}

impl KeyboardController {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    /// Handle a key event and return an action (decoupled from EditorService)
    pub fn handle_key(&mut self, event: &KeyEvent) -> KeyAction {
        if event.state != ElementState::Pressed {
            return KeyAction::None;
        }

        let PhysicalKey::Code(code) = event.physical_key else {
            return KeyAction::None;
        };

        let ctrl_or_cmd = self.modifiers.control_key() || self.modifiers.super_key();
        let shift = self.modifiers.shift_key();

        // Handle Ctrl/Cmd shortcuts
        if ctrl_or_cmd {
            return self.map_shortcut(code, shift);
        }

        // Handle special keys (arrows, backspace, etc.)
        if Self::is_special_key(code) {
            return self.map_special_key(code, shift);
        }

        // Handle text input
        if let Some(ref text) = event.text {
            let is_printable = !text.chars().any(|c| c.is_control());
            if is_printable && !text.is_empty() {
                if text.chars().count() == 1 {
                    return KeyAction::InsertChar(text.chars().next().unwrap());
                } else {
                    return KeyAction::InsertText(text.to_string());
                }
            }
        }

        KeyAction::None
    }

    /// Map a Ctrl/Cmd shortcut to a KeyAction
    fn map_shortcut(&self, code: KeyCode, shift: bool) -> KeyAction {
        match code {
            KeyCode::KeyZ => KeyAction::Undo,
            KeyCode::KeyY => KeyAction::Redo,
            KeyCode::KeyC => KeyAction::Copy,
            KeyCode::KeyX => KeyAction::Cut,
            KeyCode::KeyV => KeyAction::Paste,
            KeyCode::KeyA => KeyAction::SelectAll,
            KeyCode::KeyF => KeyAction::OpenFind,
            KeyCode::KeyH => KeyAction::OpenReplace,
            KeyCode::KeyG => {
                if shift {
                    KeyAction::FindPrev
                } else {
                    KeyAction::FindNext
                }
            }
            KeyCode::ArrowLeft => {
                if shift {
                    KeyAction::MoveCursor(CursorDirection::WordLeft, true)
                } else {
                    KeyAction::MoveCursor(CursorDirection::WordLeft, false)
                }
            }
            KeyCode::ArrowRight => {
                if shift {
                    KeyAction::MoveCursor(CursorDirection::WordRight, true)
                } else {
                    KeyAction::MoveCursor(CursorDirection::WordRight, false)
                }
            }
            KeyCode::ArrowUp if shift => {
                KeyAction::MoveCursor(CursorDirection::DocumentStart, true)
            }
            KeyCode::ArrowDown if shift => {
                KeyAction::MoveCursor(CursorDirection::DocumentEnd, true)
            }
            KeyCode::Home => KeyAction::MoveCursor(CursorDirection::DocumentStart, shift),
            KeyCode::End => KeyAction::MoveCursor(CursorDirection::DocumentEnd, shift),
            _ => KeyAction::None,
        }
    }

    /// Map a special key to a KeyAction
    fn map_special_key(&self, code: KeyCode, shift: bool) -> KeyAction {
        match code {
            KeyCode::ArrowLeft => KeyAction::MoveCursor(CursorDirection::Left, shift),
            KeyCode::ArrowRight => KeyAction::MoveCursor(CursorDirection::Right, shift),
            KeyCode::ArrowUp => KeyAction::MoveCursor(CursorDirection::Up, shift),
            KeyCode::ArrowDown => KeyAction::MoveCursor(CursorDirection::Down, shift),
            KeyCode::Home => KeyAction::MoveCursor(CursorDirection::Home, shift),
            KeyCode::End => KeyAction::MoveCursor(CursorDirection::End, shift),
            KeyCode::PageUp => KeyAction::MoveCursor(CursorDirection::PageUp, shift),
            KeyCode::PageDown => KeyAction::MoveCursor(CursorDirection::PageDown, shift),
            KeyCode::Backspace => KeyAction::Backspace,
            KeyCode::Delete => KeyAction::Delete,
            KeyCode::Tab => {
                if shift {
                    KeyAction::Unindent
                } else {
                    KeyAction::Tab
                }
            }
            KeyCode::Enter => KeyAction::NewLine,
            KeyCode::Space => KeyAction::InsertChar(' '),
            KeyCode::Escape => KeyAction::ClearSelection,
            _ => KeyAction::None,
        }
    }

    /// Legacy method - handle key event and apply directly to editor
    /// Deprecated: Use handle_key() for decoupled action handling
    pub fn handle_key_event(&mut self, editor: &mut EditorService, event: KeyEvent) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }

        let PhysicalKey::Code(code) = event.physical_key else {
            return false;
        };

        if self.modifiers.control_key() || self.modifiers.super_key() {
            return self.handle_shortcut(editor, code);
        }

        if Self::is_special_key(code) {
            return self.handle_special_key(editor, code);
        }

        if let Some(text) = event.text {
            let is_printable = !text.chars().any(|c| c.is_control());
            if is_printable && !text.is_empty() {
                return self.handle_text_input(editor, &text);
            }
        }

        false
    }

    fn is_special_key(code: KeyCode) -> bool {
        matches!(
            code,
            KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Enter
                | KeyCode::Tab
                | KeyCode::Escape
                | KeyCode::ArrowLeft
                | KeyCode::ArrowRight
                | KeyCode::ArrowUp
                | KeyCode::ArrowDown
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Insert
                | KeyCode::Space
        )
    }

    fn handle_shortcut(&mut self, editor: &mut EditorService, code: KeyCode) -> bool {
        let clipboard = ArboardClipboard::new();

        match code {
            KeyCode::KeyZ => {
                if let Some(op) = editor.history_mut().undo() {
                    apply_undo(editor, op);
                }
                true
            }
            KeyCode::KeyY => {
                if let Some(op) = editor.history_mut().redo() {
                    apply_redo(editor, op);
                }
                true
            }
            KeyCode::KeyS => true,
            KeyCode::KeyC => {
                if let Some(sel) = editor.cursor().selection() {
                    if !sel.is_empty() {
                        let (s, e) = sel.range();
                        let text = editor
                            .buffer()
                            .as_str()
                            .chars()
                            .skip(s)
                            .take(e - s)
                            .collect::<String>();
                        let _ = clipboard.set_text(&text);
                    }
                }
                true
            }
            KeyCode::KeyX => {
                if let Some(sel) = editor.cursor().selection() {
                    if !sel.is_empty() {
                        let (s, e) = sel.range();
                        let text = editor
                            .buffer()
                            .as_str()
                            .chars()
                            .skip(s)
                            .take(e - s)
                            .collect::<String>();
                        let _ = clipboard.set_text(&text);
                        editor.buffer_mut().remove(s, e - s);
                        editor
                            .history_mut()
                            .push(Operation::Delete { position: s, text });
                        editor.cursor_mut().set_position(s);
                    }
                }
                true
            }
            KeyCode::KeyV => {
                if let Ok(text) = clipboard.get_text() {
                    if let Some(sel) = editor.cursor().selection() {
                        if !sel.is_empty() {
                            let (s, e) = sel.range();
                            let txt = editor
                                .buffer()
                                .as_str()
                                .chars()
                                .skip(s)
                                .take(e - s)
                                .collect::<String>();
                            editor.buffer_mut().remove(s, e - s);
                            editor.history_mut().push(Operation::Delete {
                                position: s,
                                text: txt,
                            });
                            editor.cursor_mut().set_position(s);
                        }
                    }
                    let pos = editor.cursor().position();
                    editor.buffer_mut().insert(pos, &text);
                    editor.cursor_mut().set_position(pos + text.len());
                    editor.history_mut().push(Operation::Insert {
                        position: pos,
                        text,
                    });
                }
                true
            }
            KeyCode::KeyA => {
                let len = editor.buffer().len_chars();
                if len > 0 {
                    editor.cursor_mut().set_selection_start(0);
                    editor.cursor_mut().set_selection_end(len);
                }
                true
            }
            KeyCode::KeyF => {
                editor.set_find_query(Some(String::new()));
                true
            }
            KeyCode::KeyH => {
                editor.set_find_query(Some(String::new()));
                true
            }
            KeyCode::KeyG => {
                if self.modifiers.shift_key() {
                    editor.find_prev();
                } else {
                    editor.find_next();
                }
                true
            }
            _ => false,
        }
    }

    fn handle_text_input(&mut self, editor: &mut EditorService, text: &str) -> bool {
        if let Some(sel) = editor.cursor().selection() {
            if !sel.is_empty() {
                let (s, e) = sel.range();
                let txt = editor
                    .buffer()
                    .as_str()
                    .chars()
                    .skip(s)
                    .take(e - s)
                    .collect::<String>();
                editor.buffer_mut().remove(s, e - s);
                editor.history_mut().push(Operation::Delete {
                    position: s,
                    text: txt,
                });
                editor.cursor_mut().set_position(s);
            }
        }

        for ch in text.chars() {
            let pos = editor.cursor().position();
            editor.buffer_mut().insert(pos, &ch.to_string());
            editor.cursor_mut().set_position(pos + 1);
            editor.history_mut().push(Operation::Insert {
                position: pos,
                text: ch.to_string(),
            });
        }
        true
    }

    fn handle_special_key(&mut self, editor: &mut EditorService, code: KeyCode) -> bool {
        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        if ctrl && shift {
            match code {
                KeyCode::ArrowLeft => {
                    editor.cursor_extend_selection_to_word_start();
                    return true;
                }
                KeyCode::ArrowRight => {
                    editor.cursor_extend_selection_to_word_end();
                    return true;
                }
                KeyCode::ArrowUp => {
                    editor.cursor_mut().set_selection_start(0);
                    return true;
                }
                KeyCode::ArrowDown => {
                    let len = editor.buffer().len_chars();
                    editor.cursor_mut().set_selection_end(len);
                    return true;
                }
                _ => {}
            }
        }

        if ctrl && !shift {
            match code {
                KeyCode::ArrowLeft => {
                    editor.cursor_move_to_word_start();
                    return true;
                }
                KeyCode::ArrowRight => {
                    editor.cursor_move_to_word_end();
                    return true;
                }
                _ => {}
            }
        }

        match code {
            KeyCode::ArrowLeft => {
                if shift {
                    editor.cursor_mut().extend_selection_backward();
                } else {
                    editor.cursor_mut().move_backward();
                }
                true
            }
            KeyCode::ArrowRight => {
                let len = editor.buffer().len_chars();
                if shift {
                    editor.cursor_mut().extend_selection_forward(len);
                } else {
                    editor.cursor_mut().move_forward(len);
                }
                true
            }
            KeyCode::ArrowUp => {
                if shift {
                    editor.cursor_extend_selection_up();
                } else {
                    editor.cursor_move_up();
                }
                true
            }
            KeyCode::ArrowDown => {
                if shift {
                    editor.cursor_extend_selection_down();
                } else {
                    editor.cursor_move_down();
                }
                true
            }
            KeyCode::Home => {
                if ctrl {
                    editor.cursor_mut().set_position(0);
                } else {
                    editor.cursor_move_to_line_start();
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            KeyCode::End => {
                if ctrl {
                    let len = editor.buffer().len_chars();
                    editor.cursor_mut().set_position(len);
                } else {
                    editor.cursor_move_to_line_end();
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            KeyCode::Backspace => {
                self.handle_backspace(editor);
                true
            }
            KeyCode::Delete => {
                self.handle_delete(editor);
                true
            }
            KeyCode::Tab => {
                if shift {
                    self.handle_unindent(editor);
                } else {
                    self.handle_text_input(editor, "    ");
                }
                true
            }
            KeyCode::Enter => {
                self.handle_text_input(editor, "\n");
                true
            }
            KeyCode::Space => {
                self.handle_text_input(editor, " ");
                true
            }
            KeyCode::Escape => {
                editor.cursor_mut().clear_selection();
                true
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    editor.cursor_move_up();
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    editor.cursor_move_down();
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            _ => false,
        }
    }

    fn handle_backspace(&self, editor: &mut EditorService) {
        if let Some(sel) = editor.cursor().selection() {
            if !sel.is_empty() {
                let (s, e) = sel.range();
                let txt = editor
                    .buffer()
                    .as_str()
                    .chars()
                    .skip(s)
                    .take(e - s)
                    .collect::<String>();
                editor.buffer_mut().remove(s, e - s);
                editor.cursor_mut().set_position(s);
                editor.history_mut().push(Operation::Delete {
                    position: s,
                    text: txt,
                });
                return;
            }
        }

        let pos = editor.cursor().position();
        if pos > 0 {
            if let Some(ch) = editor.buffer().get_char(pos - 1) {
                editor.buffer_mut().remove(pos - 1, 1);
                editor.cursor_mut().set_position(pos - 1);
                editor.history_mut().push(Operation::Delete {
                    position: pos - 1,
                    text: ch.to_string(),
                });
            }
        }
    }

    fn handle_delete(&self, editor: &mut EditorService) {
        if let Some(sel) = editor.cursor().selection() {
            if !sel.is_empty() {
                let (s, e) = sel.range();
                let txt = editor
                    .buffer()
                    .as_str()
                    .chars()
                    .skip(s)
                    .take(e - s)
                    .collect::<String>();
                editor.buffer_mut().remove(s, e - s);
                editor.cursor_mut().set_position(s);
                editor.history_mut().push(Operation::Delete {
                    position: s,
                    text: txt,
                });
                return;
            }
        }

        let pos = editor.cursor().position();
        let buf_len = editor.buffer().len_chars();
        if pos < buf_len {
            if let Some(ch) = editor.buffer().get_char(pos) {
                editor.buffer_mut().remove(pos, 1);
                editor.history_mut().push(Operation::Delete {
                    position: pos,
                    text: ch.to_string(),
                });
            }
        }
    }

    fn handle_unindent(&self, editor: &mut EditorService) {
        let buf = editor.buffer();
        let (line, _) = buf.char_to_line_col(editor.cursor().position());

        if let Some(line_str) = buf.line(line) {
            let indent = line_str
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .count();
            if indent > 0 {
                let start = buf.line_to_char(line);
                let remove = if indent >= 4 { 4 } else { indent };
                editor.buffer_mut().remove(start, remove);
                editor.history_mut().push(Operation::Delete {
                    position: start,
                    text: line_str.chars().take(remove).collect(),
                });
                let cursor_pos = editor.cursor().position();
                if cursor_pos > start {
                    editor.cursor_mut().set_position(cursor_pos - remove);
                }
            }
        }
    }
}

impl Default for KeyboardController {
    fn default() -> Self {
        Self::new()
    }
}

fn apply_undo(editor: &mut EditorService, op: Operation) {
    match op {
        Operation::Insert { position, text } => {
            editor.buffer_mut().remove(position, text.len());
            editor.cursor_mut().set_position(position);
        }
        Operation::Delete { position, text } => {
            editor.buffer_mut().insert(position, &text);
            editor.cursor_mut().set_position(position + text.len());
        }
    }
}

fn apply_redo(editor: &mut EditorService, op: Operation) {
    match op {
        Operation::Insert { position, text } => {
            editor.buffer_mut().insert(position, &text);
            editor.cursor_mut().set_position(position + text.len());
        }
        Operation::Delete { position, text } => {
            editor.buffer_mut().remove(position, text.len());
            editor.cursor_mut().set_position(position);
        }
    }
}
