use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use crate::editor::operations::Operation;
use crate::editor::Editor;

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

    pub fn handle_key_event(&mut self, editor: &mut Editor, event: KeyEvent) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }

        let PhysicalKey::Code(code) = event.physical_key else {
            return false;
        };

        // Handle Ctrl/Cmd shortcuts first
        if self.modifiers.control_key() || self.modifiers.super_key() {
            return self.handle_shortcut(editor, code);
        }

        // Handle special keys that might have conflicting text
        if Self::is_special_key(code) {
            return self.handle_special_key(editor, code);
        }

        // Handle text input for printable characters
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

    fn handle_shortcut(&mut self, editor: &mut Editor, code: KeyCode) -> bool {
        match code {
            KeyCode::KeyZ => {
                if let Some(op) = editor.history_mut().undo() {
                    apply_operation_reverse(editor, op);
                }
                true
            }
            KeyCode::KeyY => {
                if let Some(op) = editor.history_mut().redo() {
                    apply_operation(editor, op);
                }
                true
            }
            KeyCode::KeyS => {
                if let Some(path) = editor.file_path() {
                    let content = editor.buffer().as_str();
                    if let Err(e) = crate::file::save_file(path, &content) {
                        eprintln!("Failed to save: {}", e);
                    } else {
                        editor.set_modified(false);
                    }
                }
                true
            }
            KeyCode::KeyC => {
                if let Some(sel) = editor.cursor().selection() {
                    copy_selection(editor, sel);
                }
                true
            }
            KeyCode::KeyX => {
                if let Some(sel) = editor.cursor().selection() {
                    cut_selection(editor, sel);
                }
                true
            }
            KeyCode::KeyV => {
                paste_at_cursor(editor);
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
            _ => false,
        }
    }

    fn handle_text_input(&mut self, editor: &mut Editor, text: &str) -> bool {
        if let Some(sel) = editor.cursor().selection() {
            if sel.len() > 0 {
                let (s,e) = sel.range();
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

    fn handle_special_key(&mut self, editor: &mut Editor, code: KeyCode) -> bool {
        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        // Ctrl+Shift+Arrow for word selection
        if ctrl && shift {
            match code {
                KeyCode::ArrowLeft => {
                    let buf = editor.buffer().clone();
                    editor.cursor_mut().extend_selection_to_word_start(&buf);
                    return true;
                }
                KeyCode::ArrowRight => {
                    let buf = editor.buffer().clone();
                    editor.cursor_mut().extend_selection_to_word_end(&buf);
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

        // Ctrl+Arrow for word navigation
        if ctrl && !shift {
            match code {
                KeyCode::ArrowLeft => {
                    let buf = editor.buffer().clone();
                    editor.cursor_mut().move_to_word_start(&buf);
                    return true;
                }
                KeyCode::ArrowRight => {
                    let buf = editor.buffer().clone();
                    editor.cursor_mut().move_to_word_end(&buf);
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
                let buf = editor.buffer().clone();
                if shift {
                    editor.cursor_mut().extend_selection_up(&buf);
                } else {
                    editor.cursor_mut().move_up(&buf);
                }
                true
            }
            KeyCode::ArrowDown => {
                let buf = editor.buffer().clone();
                if shift {
                    editor.cursor_mut().extend_selection_down(&buf);
                } else {
                    editor.cursor_mut().move_down(&buf);
                }
                true
            }
            KeyCode::Home => {
                let buf = editor.buffer().clone();
                if ctrl {
                    editor.cursor_mut().set_position(0);
                } else {
                    editor.cursor_mut().move_to_line_start(&buf);
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            KeyCode::End => {
                let buf = editor.buffer().clone();
                if ctrl {
                    editor.cursor_mut().set_position(buf.len_chars());
                } else {
                    editor.cursor_mut().move_to_line_end(&buf);
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
                let buf = editor.buffer().clone();
                for _ in 0..10 {
                    editor.cursor_mut().move_up(&buf);
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            KeyCode::PageDown => {
                let buf = editor.buffer().clone();
                for _ in 0..10 {
                    editor.cursor_mut().move_down(&buf);
                }
                if !shift {
                    editor.cursor_mut().clear_selection();
                }
                true
            }
            _ => false,
        }
    }

    fn handle_backspace(&self, editor: &mut Editor) {
        if let Some(sel) = editor.cursor().selection() {
            if sel.len() > 0 {
                let (s,e) = sel.range();
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

    fn handle_delete(&self, editor: &mut Editor) {
        if let Some(sel) = editor.cursor().selection() {
            if sel.len() > 0 {
                let (s,e) = sel.range();
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

    fn handle_unindent(&self, editor: &mut Editor) {
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

fn copy_selection(editor: &Editor, sel: crate::editor::cursor::Selection) {
    if sel.len() > 0 {
        let (s,e) = sel.range();
        let text = editor
            .buffer()
            .as_str()
            .chars()
            .skip(s)
            .take(e - s)
            .collect::<String>();
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);
        }
    }
}

fn cut_selection(editor: &mut Editor, sel: crate::editor::cursor::Selection) {
    if sel.len() > 0 {
        let (s,e) = sel.range();
        let text = editor
            .buffer()
            .as_str()
            .chars()
            .skip(s)
            .take(e - s)
            .collect::<String>();
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text.clone());
        }
        editor.buffer_mut().remove(s, e - s);
        editor.cursor_mut().set_position(s);
        editor.history_mut().push(Operation::Delete {
            position: s,
            text,
        });
    }
}

fn paste_at_cursor(editor: &mut Editor) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        if let Ok(text) = cb.get_text() {
            if let Some(sel) = editor.cursor().selection() {
                if sel.len() > 0 {
                    let (s,e) = sel.range();
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
    }
}

fn apply_operation(editor: &mut Editor, op: Operation) {
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

fn apply_operation_reverse(editor: &mut Editor, op: Operation) {
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
