use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::WindowAttributes;

use crate::editor::Editor;
use crate::editor::KeyboardController;
use crate::menu::{MenuAction, MenuHandler};
use crate::state::State;

#[derive(Clone, Copy, PartialEq)]
enum MouseButtonState {
    Released,
    Pressed,
}

pub struct App {
    state: Option<State>,
    editor: Option<Editor>,
    keyboard: KeyboardController,
    menu_handler: Option<MenuHandler>,
    modifiers: ModifiersState,
    mouse_button_state: MouseButtonState,
    mouse_position: Option<(f64, f64)>,
    last_click_time: Option<Instant>,
    last_click_position: Option<(f64, f64)>,
    is_dragging: bool,
    click_count: u8,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            editor: None,
            keyboard: KeyboardController::new(),
            menu_handler: None,
            modifiers: ModifiersState::empty(),
            mouse_button_state: MouseButtonState::Released,
            mouse_position: None,
            last_click_time: None,
            last_click_position: None,
            is_dragging: false,
            click_count: 0,
        }
    }

    pub fn with_menu(mut self, menu_handler: MenuHandler) -> Self {
        self.menu_handler = Some(menu_handler);
        self
    }

    pub fn poll_menu_events(&mut self) {
        if let Some(ref mut menu_handler) = self.menu_handler {
            menu_handler.poll_menu_events();
        }
    }

    fn handle_menu_action(&mut self, action: MenuAction) {
        let editor = match &mut self.editor {
            Some(e) => e,
            None => return,
        };

        match action {
            MenuAction::New => {
                editor.buffer_mut().clear();
                editor.cursor_mut().set_position(0);
                editor.history_mut().clear();
            }
            MenuAction::Open => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(
                        "Text Files",
                        &[
                            "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css", "js",
                            "ts",
                        ],
                    )
                    .add_filter("All Files", &["*"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    if let Ok(content) = std::fs::read_to_string(&path_str) {
                        editor.buffer_mut().set_content(&content);
                        editor.cursor_mut().set_position(0);
                        editor.history_mut().clear();
                        editor.set_file_path(path_str);
                    }
                }
            }
            MenuAction::Save => {
                let needs_save_as = editor.file_path().is_none();
                if !needs_save_as {
                    let content = editor.buffer().as_str();
                    if let Some(path) = editor.file_path() {
                        if std::fs::write(path, content).is_ok() {
                            editor.set_modified(false);
                        }
                    }
                } else {
                    let _ = editor;
                    self.handle_menu_action(MenuAction::SaveAs);
                    return;
                }
            }
            MenuAction::SaveAs => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(
                        "Text Files",
                        &[
                            "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css", "js",
                            "ts",
                        ],
                    )
                    .add_filter("All Files", &["*"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    let content = editor.buffer().as_str();
                    if std::fs::write(&path_str, content).is_ok() {
                        editor.set_file_path(path_str);
                        editor.set_modified(false);
                    }
                }
            }
            MenuAction::Close => {
                std::process::exit(0);
            }
            MenuAction::Quit => {
                std::process::exit(0);
            }
            MenuAction::Undo => {
                if let Some(op) = editor.history_mut().undo() {
                    Self::apply_undo(editor, op);
                }
            }
            MenuAction::Redo => {
                if let Some(op) = editor.history_mut().redo() {
                    Self::apply_redo(editor, op);
                }
            }
            MenuAction::Cut => {
                if let Some(sel) = editor.cursor().selection() {
                    Self::cut_selection(editor, sel);
                }
            }
            MenuAction::Copy => {
                if let Some(sel) = editor.cursor().selection() {
                    Self::copy_selection(editor, sel);
                }
            }
            MenuAction::Paste => {
                Self::paste_at_cursor(editor);
            }
            MenuAction::Delete => {
                Self::delete_selection_or_char(editor);
            }
            MenuAction::SelectAll => {
                let len = editor.buffer().len_chars();
                if len > 0 {
                    editor.cursor_mut().set_selection_start(0);
                    editor.cursor_mut().set_selection_end(len);
                }
            }
            MenuAction::ToggleLineNumbers => {
                editor.toggle_line_numbers();
            }
            MenuAction::ToggleStatusBar => {
                editor.toggle_status_bar();
            }
            MenuAction::About => {
                let _ = rfd::MessageDialog::new()
                    .set_title("About textedit")
                    .set_description("textedit v0.1.0\n\nA fast, cross-platform text editor built with Rust and wgpu.")
                    .set_level(rfd::MessageLevel::Info)
                    .show();
            }
        }

        if let Some(state) = &mut self.state {
            if let Err(e) = state.update_geometry(
                editor.buffer(),
                editor.cursor(),
                editor.show_line_numbers(),
                editor.show_status_bar(),
            ) {
                eprintln!("Failed to update geometry: {}", e);
            }
        }
    }

    fn apply_undo(editor: &mut Editor, op: crate::editor::operations::Operation) {
        use crate::editor::operations::Operation;
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

    fn apply_redo(editor: &mut Editor, op: crate::editor::operations::Operation) {
        use crate::editor::operations::Operation;
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

    fn copy_selection(editor: &Editor, sel: crate::editor::cursor::Selection) {
        if sel.len() > 0 {
            let (s, e) = sel.range();
            let text = editor
                .buffer()
                .as_str()
                .chars()
                .skip(s)
                .take(e - s)
                .collect::<String>();
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text);
            }
        }
    }

    fn cut_selection(editor: &mut Editor, sel: crate::editor::cursor::Selection) {
        if sel.len() > 0 {
            let (s, e) = sel.range();
            let text = editor
                .buffer()
                .as_str()
                .chars()
                .skip(s)
                .take(e - s)
                .collect::<String>();
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(text.clone());
            }
            editor.buffer_mut().remove(s, e - s);
            editor.cursor_mut().set_position(s);
            editor
                .history_mut()
                .push(crate::editor::operations::Operation::Delete { position: s, text });
        }
    }

    fn paste_at_cursor(editor: &mut Editor) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                if let Some(sel) = editor.cursor().selection() {
                    if sel.len() > 0 {
                        let (s, e) = sel.range();
                        let txt = editor
                            .buffer()
                            .as_str()
                            .chars()
                            .skip(s)
                            .take(e - s)
                            .collect::<String>();
                        editor.buffer_mut().remove(s, e - s);
                        editor
                            .history_mut()
                            .push(crate::editor::operations::Operation::Delete {
                                position: s,
                                text: txt,
                            });
                        editor.cursor_mut().set_position(s);
                    }
                }

                let pos = editor.cursor().position();
                editor.buffer_mut().insert(pos, &text);
                editor.cursor_mut().set_position(pos + text.len());
                editor
                    .history_mut()
                    .push(crate::editor::operations::Operation::Insert {
                        position: pos,
                        text,
                    });
            }
        }
    }

    fn delete_selection_or_char(editor: &mut Editor) {
        use crate::editor::operations::Operation;

        if let Some(sel) = editor.cursor().selection() {
            if sel.len() > 0 {
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
}

impl ApplicationHandler<MenuAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = WindowAttributes::default().with_title("textedit - Untitled");

            let window = event_loop.create_window(window_attributes).unwrap();
            let window = Arc::new(window);

            let state = pollster::block_on(State::new(window.clone()))
                .expect("Failed to initialize graphics state");

            // Initialize menu handler if we have one
            if let Some(ref mut menu_handler) = self.menu_handler {
                menu_handler.attach_to_window(&window);
            }

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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(state) = &mut self.state {
                    if let Err(e) = state.set_scale_factor(scale_factor) {
                        eprintln!("Failed to update scale factor: {}", e);
                    }
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                self.keyboard.set_modifiers(self.modifiers);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::MouseButton;

                if button == MouseButton::Left {
                    self.mouse_button_state = if state == winit::event::ElementState::Pressed {
                        if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                            if let Some((x, y)) = self.mouse_position {
                                let now = Instant::now();

                                let click_count = if let (Some(last_time), Some((last_x, last_y))) =
                                    (self.last_click_time, self.last_click_position)
                                {
                                    let time_diff = now.duration_since(last_time);
                                    let pos_diff =
                                        ((x - last_x).powi(2) + (y - last_y).powi(2)).sqrt();

                                    if time_diff.as_millis() < 500 && pos_diff < 10.0 {
                                        if self.click_count >= 2 {
                                            3
                                        } else {
                                            2
                                        }
                                    } else {
                                        1
                                    }
                                } else {
                                    1
                                };

                                self.click_count = click_count;
                                self.last_click_time = Some(now);
                                self.last_click_position = Some((x, y));

                                let (line, col) = state
                                    .get_char_at_position(
                                        x,
                                        y,
                                        editor.buffer(),
                                        editor.show_line_numbers(),
                                        editor.show_status_bar(),
                                    )
                                    .unwrap_or((0, 0));
                                let char_idx =
                                    editor.buffer().line_col_to_char(line, col).unwrap_or(0);

                                match click_count {
                                    2 => {
                                        let buffer = editor.buffer().clone();
                                        editor.cursor_mut().select_word_at_cursor(&buffer);
                                    }
                                    3 => {
                                        let buffer = editor.buffer().clone();
                                        editor.cursor_mut().select_line(&buffer);
                                    }
                                    _ => {
                                        editor.cursor_mut().set_position(char_idx);
                                    }
                                }

                                if let Err(e) = state.update_geometry(
                                    editor.buffer(),
                                    editor.cursor(),
                                    editor.show_line_numbers(),
                                    editor.show_status_bar(),
                                ) {
                                    eprintln!("Failed to update geometry: {}", e);
                                }
                            }
                        }
                        MouseButtonState::Pressed
                    } else {
                        self.is_dragging = false;
                        MouseButtonState::Released
                    };
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;

                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    let lines_delta: i32 = match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            // Positive y is typically scroll up.
                            -(y.round() as i32)
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 {
                                -3
                            } else if pos.y < 0.0 {
                                3
                            } else {
                                0
                            }
                        }
                    };

                    if lines_delta != 0 {
                        state.scroll_by_lines(
                            lines_delta,
                            editor.buffer(),
                            editor.show_line_numbers(),
                            editor.show_status_bar(),
                        );
                        if let Err(e) = state.update_geometry(
                            editor.buffer(),
                            editor.cursor(),
                            editor.show_line_numbers(),
                            editor.show_status_bar(),
                        ) {
                            eprintln!("Failed to update geometry after scroll: {}", e);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some((position.x, position.y));

                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    if self.mouse_button_state == MouseButtonState::Pressed || self.is_dragging {
                        let (line, col) = state
                            .get_char_at_position(
                                position.x,
                                position.y,
                                editor.buffer(),
                                editor.show_line_numbers(),
                                editor.show_status_bar(),
                            )
                            .unwrap_or((0, 0));
                        let char_idx = editor.buffer().line_col_to_char(line, col).unwrap_or(0);
                        editor.cursor_mut().extend_selection(char_idx);
                        self.is_dragging = true;

                        if let Err(e) = state.update_geometry(
                            editor.buffer(),
                            editor.cursor(),
                            editor.show_line_numbers(),
                            editor.show_status_bar(),
                        ) {
                            eprintln!("Failed to update geometry: {}", e);
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    self.keyboard.handle_key_event(editor, event);

                    // Keep the cursor in view after keyboard navigation/editing.
                    state.ensure_cursor_visible(
                        editor.cursor(),
                        editor.buffer(),
                        editor.show_line_numbers(),
                        editor.show_status_bar(),
                    );

                    if let Err(e) = state.update_geometry(
                        editor.buffer(),
                        editor.cursor(),
                        editor.show_line_numbers(),
                        editor.show_status_bar(),
                    ) {
                        eprintln!("Failed to update geometry: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: MenuAction) {
        self.handle_menu_action(event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Poll for menu events
        self.poll_menu_events();

        if self.state.is_some() {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }
}
