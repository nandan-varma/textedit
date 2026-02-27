use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::WindowAttributes;

use crate::editor::operations::Operation;
use crate::editor::Editor;
use crate::file;
use crate::state::State;

pub struct App {
    state: Option<State>,
    editor: Option<Editor>,
    modifiers: ModifiersState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            editor: None,
            modifiers: ModifiersState::empty(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = WindowAttributes::default().with_title("textedit - Untitled");

            let window = event_loop.create_window(window_attributes).unwrap();
            let window = Arc::new(window);

            let state = pollster::block_on(State::new(window.clone()))
                .expect("Failed to initialize graphics state");

            let editor = Editor::new();

            self.state = Some(state);
            self.editor = Some(editor);

            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    if let Err(e) = state.render() {
                        eprintln!("Render error: {}", e);
                    }
                    state.window().request_redraw();
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    handle_keyboard_input(editor, event, self.modifiers);
                    // Update geometry (text and cursor) after buffer modification
                    if let Err(e) = state.update_geometry(editor.buffer(), editor.cursor()) {
                        eprintln!("Failed to update geometry: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }
}

fn handle_keyboard_input(editor: &mut Editor, event: KeyEvent, modifiers: ModifiersState) {
    if event.state != ElementState::Pressed {
        return;
    }

    let PhysicalKey::Code(code) = event.physical_key else {
        return;
    };

    use winit::keyboard::KeyCode;

    // Handle Ctrl+X/C/V/Z/Y/S shortcuts
    if modifiers.control_key() {
        match code {
            KeyCode::KeyZ => {
                if let Some(op) = editor.history_mut().undo() {
                    apply_operation_reverse(editor, op);
                }
                return;
            }
            KeyCode::KeyY => {
                if let Some(op) = editor.history_mut().redo() {
                    apply_operation(editor, op);
                }
                return;
            }
            KeyCode::KeyS => {
                let path_opt = editor.file_path().map(|p| p.to_string());
                if let Some(path) = path_opt {
                    let content = editor.buffer().as_str();
                    if let Err(e) = file::save_file(&path, &content) {
                        eprintln!("Failed to save file: {}", e);
                    } else {
                        editor.set_modified(false);
                    }
                }
                return;
            }
            KeyCode::KeyC => {
                if let Some(sel) = editor.cursor().selection() {
                    copy_to_clipboard(editor, sel);
                }
                return;
            }
            KeyCode::KeyX => {
                if let Some(sel) = editor.cursor().selection() {
                    let start = sel.start;
                    let len = sel.len();
                    if len > 0 {
                        let text = editor.buffer().as_str();
                        let selected = text.chars().skip(start).take(len).collect::<String>();
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(selected.clone());
                        }
                        editor.buffer_mut().remove(start, len);
                        editor.cursor_mut().set_position(start);
                        editor.history_mut().push(Operation::Delete {
                            position: start,
                            text: selected,
                        });
                    }
                }
                return;
            }
            KeyCode::KeyV => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(text) = clipboard.get_text() {
                        let pos = editor.cursor().position();
                        editor.buffer_mut().insert(pos, &text);
                        let buffer_len = editor.buffer().len_chars();
                        editor.cursor_mut().move_forward(buffer_len);
                        editor.history_mut().push(Operation::Insert {
                            position: pos,
                            text,
                        });
                    }
                }
                return;
            }
            _ => {}
        }
    }

    // Handle text input
    if let Some(text) = event.text {
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }
            let pos = editor.cursor().position();
            editor.buffer_mut().insert(pos, &ch.to_string());
            let buffer_len = editor.buffer().len_chars();
            editor.cursor_mut().move_forward(buffer_len);
            editor.history_mut().push(Operation::Insert {
                position: pos,
                text: ch.to_string(),
            });
        }
        return;
    }

    // Handle special keys
    match code {
        KeyCode::ArrowLeft => editor.cursor_mut().move_backward(),
        KeyCode::ArrowRight => {
            let buffer_len = editor.buffer().len_chars();
            editor.cursor_mut().move_forward(buffer_len);
        }
        KeyCode::ArrowUp => {
            let buffer = editor.buffer().clone();
            editor.cursor_mut().move_up(&buffer);
        }
        KeyCode::ArrowDown => {
            let buffer = editor.buffer().clone();
            editor.cursor_mut().move_down(&buffer);
        }
        KeyCode::Home => {
            let buffer = editor.buffer().clone();
            editor.cursor_mut().move_to_line_start(&buffer);
        }
        KeyCode::End => {
            let buffer = editor.buffer().clone();
            editor.cursor_mut().move_to_line_end(&buffer);
        }
        KeyCode::Backspace => {
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
        KeyCode::Delete => {
            let pos = editor.cursor().position();
            if pos < editor.buffer().len_chars() {
                if let Some(ch) = editor.buffer().get_char(pos) {
                    editor.buffer_mut().remove(pos, 1);
                    editor.history_mut().push(Operation::Delete {
                        position: pos,
                        text: ch.to_string(),
                    });
                }
            }
        }
        KeyCode::Tab => {
            let tab_str = "    ";
            let pos = editor.cursor().position();
            editor.buffer_mut().insert(pos, tab_str);
            let buffer_len = editor.buffer().len_chars();
            editor.cursor_mut().move_forward(buffer_len);
            editor.history_mut().push(Operation::Insert {
                position: pos,
                text: tab_str.to_string(),
            });
        }
        KeyCode::Enter => {
            let pos = editor.cursor().position();
            editor.buffer_mut().insert(pos, "\n");
            let buffer_len = editor.buffer().len_chars();
            editor.cursor_mut().move_forward(buffer_len);
            editor.history_mut().push(Operation::Insert {
                position: pos,
                text: "\n".to_string(),
            });
        }
        _ => {}
    }
}

fn copy_to_clipboard(editor: &Editor, sel: crate::editor::cursor::Selection) {
    let start = sel.start;
    let len = sel.len();
    if len > 0 {
        let text = editor.buffer().as_str();
        let selected = text.chars().skip(start).take(len).collect::<String>();
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(selected);
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
