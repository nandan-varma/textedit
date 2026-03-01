use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::WindowAttributes;

use crate::application::EditorService;
use crate::interface::KeyboardController;
use crate::menu::{MenuAction, MenuHandler};
use crate::state::State;
use crate::ui::modal::{ModalAction, ModalState};

#[derive(Clone, Copy, PartialEq)]
pub enum MouseButtonState {
    Released,
    Pressed,
}

pub struct App {
    pub state: Option<State>,
    pub editor: Option<EditorService>,
    pub keyboard: KeyboardController,
    pub menu_handler: Option<MenuHandler>,
    pub modifiers: ModifiersState,
    pub mouse_button_state: MouseButtonState,
    pub mouse_position: Option<(f64, f64)>,
    pub last_click_time: Option<Instant>,
    pub last_click_position: Option<(f64, f64)>,
    pub is_dragging: bool,
    pub click_count: u8,
    pub modal_state: ModalState,
    pub cursor_blink_timer: Instant,
    pub cursor_visible: bool,
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
            modal_state: ModalState::None,
            cursor_blink_timer: Instant::now(),
            cursor_visible: true,
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

    /// Update cursor blink state
    #[allow(dead_code)]
    fn update_cursor_blink(&mut self) {
        let elapsed = self.cursor_blink_timer.elapsed();
        if elapsed.as_millis() > 530 {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_timer = Instant::now();
        }
    }

    /// Reset cursor blink (make cursor visible immediately)
    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_timer = Instant::now();
    }

    /// Open the find modal
    pub fn open_find_modal(&mut self) {
        self.modal_state.open_find();
        self.reset_cursor_blink();
        self.sync_modal_with_editor();
        self.update_modal_geometry();
    }

    /// Open the replace modal
    pub fn open_replace_modal(&mut self) {
        self.modal_state.open_replace();
        self.reset_cursor_blink();
        self.sync_modal_with_editor();
        self.update_modal_geometry();
    }

    /// Close any open modal
    pub fn close_modal(&mut self) {
        self.modal_state.close();
        self.update_modal_geometry();
    }

    /// Sync modal state with editor (set find query from modal input)
    fn sync_modal_with_editor(&mut self) {
        if let (Some(editor), Some(find_modal)) = (&mut self.editor, self.modal_state.as_find_mut())
        {
            // If editor has a previous query, populate the modal
            if let Some(query) = editor.find_query() {
                if find_modal.find_input.is_empty() && !query.is_empty() {
                    find_modal.find_input.set_text(query);
                }
            }
            // If editor has previous replace text, populate the modal
            if let Some(replace) = editor.replace_text() {
                if find_modal.replace_input.is_empty() && !replace.is_empty() {
                    find_modal.replace_input.set_text(replace);
                }
            }
        }
    }

    /// Update editor search state from modal
    fn update_editor_from_modal(&mut self) {
        if let (Some(editor), Some(find_modal)) = (&mut self.editor, self.modal_state.as_find()) {
            let query = find_modal.query();
            if !query.is_empty() {
                editor.set_find_query(Some(query.to_string()));
            } else {
                editor.set_find_query(None);
            }
            let replacement = find_modal.replacement();
            editor.set_replace_text(Some(replacement.to_string()));
        }
    }

    /// Update match information in modal
    fn update_modal_matches(&mut self) {
        if let (Some(editor), Some(find_modal)) = (&mut self.editor, self.modal_state.as_find_mut())
        {
            let matches = editor.find_all_matches();
            let current_index = editor.current_match_index();
            find_modal.update_matches(matches, current_index);
        }
    }

    /// Update modal geometry in state
    fn update_modal_geometry(&mut self) {
        if let (Some(editor), Some(state)) = (&self.editor, &mut self.state) {
            let matches = if self.modal_state.is_open() {
                editor.find_all_matches()
            } else {
                Vec::new()
            };
            let current_match = if self.modal_state.is_open() {
                editor.current_match_index().map(|i| i.saturating_sub(1))
            } else {
                None
            };

            if let Err(e) = state.update_modal_geometry(
                &self.modal_state,
                &matches,
                current_match,
                editor.buffer(),
                editor.show_line_numbers(),
                editor.show_status_bar(),
                self.cursor_visible,
            ) {
                eprintln!("Failed to update modal geometry: {}", e);
            }
        }
    }

    /// Handle a modal action
    fn handle_modal_action(&mut self, action: ModalAction) {
        match action {
            ModalAction::None => {}
            ModalAction::Close => {
                self.close_modal();
            }
            ModalAction::FindNext => {
                self.update_editor_from_modal();
                if let Some(editor) = &mut self.editor {
                    editor.find_next();
                }
                self.update_modal_matches();
                self.ensure_cursor_visible_and_update();
            }
            ModalAction::FindPrev => {
                self.update_editor_from_modal();
                if let Some(editor) = &mut self.editor {
                    editor.find_prev();
                }
                self.update_modal_matches();
                self.ensure_cursor_visible_and_update();
            }
            ModalAction::Replace => {
                self.update_editor_from_modal();
                if let Some(editor) = &mut self.editor {
                    editor.replace_and_find_next();
                }
                self.update_modal_matches();
                self.ensure_cursor_visible_and_update();
            }
            ModalAction::ReplaceAll => {
                self.update_editor_from_modal();
                if let Some(editor) = &mut self.editor {
                    let count = editor.replace_all();
                    // Could show count in status bar
                    let _ = count;
                }
                self.update_modal_matches();
                self.ensure_cursor_visible_and_update();
            }
            ModalAction::UpdateQuery => {
                self.update_editor_from_modal();
                self.update_modal_matches();
                self.update_modal_geometry();
            }
            ModalAction::Redraw => {
                self.reset_cursor_blink();
                self.update_modal_geometry();
            }
        }
    }

    /// Handle a mouse click on the modal, returns true if click was consumed
    fn handle_modal_click(&mut self, x: f64, y: f64) -> bool {
        use crate::ui::modal::find_modal::FindField;

        // Check if modal is open and we have state
        if !self.modal_state.is_open() {
            return false;
        }

        let state = match &self.state {
            Some(s) => s,
            None => return false,
        };

        let x = x as f32;
        let y = y as f32;

        // First check if the click is within the modal bounds at all
        if let Some(modal_rect) = &state.modal_rect {
            if !modal_rect.contains(x, y) {
                // Click outside modal - don't consume, let it fall through
                return false;
            }
        } else {
            return false;
        }

        // Check button regions for hit
        for (button, rect) in &state.modal_button_regions {
            if rect.contains(x, y) {
                let button = *button;
                // Handle the button click - get action first, then handle it
                let action = if let Some(find_modal) = self.modal_state.as_find_mut() {
                    find_modal.handle_button(button)
                } else {
                    return true;
                };
                self.handle_modal_action(action);
                return true;
            }
        }

        // Check input field regions for hit
        for (field, rect) in &state.modal_input_regions {
            if rect.contains(x, y) {
                let field = *field;
                if let Some(find_modal) = self.modal_state.as_find_mut() {
                    // Focus the clicked field
                    match field {
                        FindField::Find => {
                            find_modal.find_input.focused = true;
                            find_modal.replace_input.focused = false;
                            find_modal.focused_field = FindField::Find;
                        }
                        FindField::Replace => {
                            find_modal.find_input.focused = false;
                            find_modal.replace_input.focused = true;
                            find_modal.focused_field = FindField::Replace;
                        }
                    }
                    self.reset_cursor_blink();
                    self.update_modal_geometry();
                }
                return true;
            }
        }

        // Click was inside modal but not on any button or input
        // Still consume the click to prevent it from affecting the editor
        true
    }

    /// Ensure cursor is visible and update all geometry
    fn ensure_cursor_visible_and_update(&mut self) {
        if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
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
                None,
                editor.file_path(),
            ) {
                eprintln!("Failed to update geometry: {}", e);
            }
        }
        self.update_modal_geometry();
    }

    pub fn handle_menu_action(&mut self, action: MenuAction) {
        // Track whether we need to update modal matches after this action
        let mut needs_modal_update = false;

        {
            let editor = match &mut self.editor {
                Some(e) => e,
                None => return,
            };

            match action {
                MenuAction::Save => {
                    let do_save_as = if let Some(path) = editor.file_path() {
                        let content = editor.buffer().as_str();
                        if std::fs::write(path, content).is_ok() {
                            editor.set_modified(false);
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if do_save_as {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter(
                                "Text Files",
                                &[
                                    "txt", "md", "rs", "json", "toml", "yaml", "yml", "html",
                                    "css", "js", "ts",
                                ],
                            )
                            .add_filter("All Files", &["*"])
                            .set_directory(std::env::current_dir().unwrap())
                            .set_file_name(
                                editor
                                    .file_path()
                                    .map(|p| {
                                        std::path::Path::new(p)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                    })
                                    .unwrap_or(""),
                            )
                            .save_file()
                        {
                            let path_str = path.to_string_lossy().to_string();
                            let content = editor.buffer().as_str();
                            if std::fs::write(&path_str, content).is_ok() {
                                editor.set_file_path(path_str);
                                editor.set_modified(false);
                            }
                        }
                    }
                }
                MenuAction::SaveAs => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            "Text Files",
                            &[
                                "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css",
                                "js", "ts",
                            ],
                        )
                        .add_filter("All Files", &["*"])
                        .set_directory(std::env::current_dir().unwrap())
                        .set_file_name(
                            editor
                                .file_path()
                                .map(|p| {
                                    std::path::Path::new(p)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                })
                                .unwrap_or(""),
                        )
                        .save_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        let content = editor.buffer().as_str();
                        if std::fs::write(&path_str, content).is_ok() {
                            editor.set_file_path(path_str);
                            editor.set_modified(false);
                        }
                    }
                }
                MenuAction::FindNext => {
                    editor.find_next();
                    needs_modal_update = true;
                }
                MenuAction::FindPrev => {
                    editor.find_prev();
                    needs_modal_update = true;
                }
                MenuAction::Replace => {
                    // Handled outside borrow scope
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
                MenuAction::SetEditorTheme(ref theme_name) => {
                    if let Some(state) = &mut self.state {
                        use crate::themes::EditorTheme;
                        let new_theme = match theme_name.as_str() {
                            "Dracula" => EditorTheme::Dracula,
                            "SolarizedDark" => EditorTheme::SolarizedDark,
                            "OneDark" => EditorTheme::OneDark,
                            "GruvboxDark" => EditorTheme::GruvboxDark,
                            "Light" => EditorTheme::Light,
                            _ => EditorTheme::Dracula,
                        };
                        state.config.theme = new_theme;
                        state.window().request_redraw();
                    }
                }
                MenuAction::New => {
                    editor.new_file();
                }
                MenuAction::Close => {}
                MenuAction::Quit => {}
                MenuAction::Undo => {
                    if let Some(op) = editor.history_mut().undo() {
                        match op {
                            crate::domain::Operation::Insert { position, text } => {
                                editor.buffer_mut().remove(position, text.len());
                                editor.cursor_mut().set_position(position);
                            }
                            crate::domain::Operation::Delete { position, text } => {
                                editor.buffer_mut().insert(position, &text);
                                editor.cursor_mut().set_position(position + text.len());
                            }
                        }
                    }
                }
                MenuAction::Redo => {
                    if let Some(op) = editor.history_mut().redo() {
                        match op {
                            crate::domain::Operation::Insert { position, text } => {
                                editor.buffer_mut().insert(position, &text);
                                editor.cursor_mut().set_position(position + text.len());
                            }
                            crate::domain::Operation::Delete { position, text } => {
                                editor.buffer_mut().remove(position, text.len());
                                editor.cursor_mut().set_position(position);
                            }
                        }
                    }
                }
                MenuAction::Cut => {
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
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(text.clone());
                            }
                            editor.buffer_mut().remove(s, e - s);
                            editor
                                .history_mut()
                                .push(crate::domain::Operation::Delete { position: s, text });
                            editor.cursor_mut().set_position(s);
                        }
                    }
                }
                MenuAction::Copy => {
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
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(text);
                            }
                        }
                    }
                }
                MenuAction::Paste => {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        if let Ok(text) = cb.get_text() {
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
                                    editor.history_mut().push(crate::domain::Operation::Delete {
                                        position: s,
                                        text: txt,
                                    });
                                    editor.cursor_mut().set_position(s);
                                }
                            }
                            let pos = editor.cursor().position();
                            editor.buffer_mut().insert(pos, &text);
                            editor.cursor_mut().set_position(pos + text.len());
                            editor.history_mut().push(crate::domain::Operation::Insert {
                                position: pos,
                                text,
                            });
                        }
                    }
                }
                MenuAction::Delete => {
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
                            editor.history_mut().push(crate::domain::Operation::Delete {
                                position: s,
                                text: txt,
                            });
                            editor.cursor_mut().set_position(s);
                            // Skip the rest
                        } else {
                            let pos = editor.cursor().position();
                            let buf_len = editor.buffer().len_chars();
                            if pos < buf_len {
                                if let Some(ch) = editor.buffer().get_char(pos) {
                                    editor.buffer_mut().remove(pos, 1);
                                    editor.history_mut().push(crate::domain::Operation::Delete {
                                        position: pos,
                                        text: ch.to_string(),
                                    });
                                }
                            }
                        }
                    } else {
                        let pos = editor.cursor().position();
                        let buf_len = editor.buffer().len_chars();
                        if pos < buf_len {
                            if let Some(ch) = editor.buffer().get_char(pos) {
                                editor.buffer_mut().remove(pos, 1);
                                editor.history_mut().push(crate::domain::Operation::Delete {
                                    position: pos,
                                    text: ch.to_string(),
                                });
                            }
                        }
                    }
                }
                MenuAction::SelectAll => {
                    let len = editor.buffer().len_chars();
                    if len > 0 {
                        editor.cursor_mut().set_selection_start(0);
                        editor.cursor_mut().set_selection_end(len);
                    }
                }
                MenuAction::Find => {
                    // Handled outside borrow scope
                }
                MenuAction::Open => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(
                            "Text Files",
                            &[
                                "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css",
                                "js", "ts",
                            ],
                        )
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Ok(content) = std::fs::read_to_string(&path_str) {
                            editor.load_content(content);
                            editor.set_file_path(path_str);
                        }
                    }
                }
                MenuAction::SetTheme(ref theme_name) => {
                    if let Some(state) = &mut self.state {
                        state.syntax = crate::syntax::SyntaxHighlighter::new(theme_name);
                        state.config.syntax_theme = theme_name.clone();
                        state.window().request_redraw();
                    }
                }
            }
        } // End of editor borrow scope

        // Handle actions that need to borrow self mutably
        match action {
            MenuAction::Find => {
                self.open_find_modal();
                return;
            }
            MenuAction::Replace => {
                self.open_replace_modal();
                return;
            }
            _ => {}
        }

        // Update modal matches if needed
        if needs_modal_update {
            self.update_modal_matches();
        }

        // Update geometry
        if let (Some(editor), Some(state)) = (&self.editor, &mut self.state) {
            if let Err(e) = state.update_geometry(
                editor.buffer(),
                editor.cursor(),
                editor.show_line_numbers(),
                editor.show_status_bar(),
                None,
                editor.file_path(),
            ) {
                eprintln!("Failed to update geometry: {}", e);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler<MenuAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = WindowAttributes::default().with_title("textedit - Untitled");

            let window = match event_loop.create_window(window_attributes) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create window: {}", e);
                    return;
                }
            };
            let window = Arc::new(window);

            let editor_config = crate::config::EditorConfig::default();
            let state =
                match pollster::block_on(crate::state::State::new(window.clone(), editor_config)) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to initialize graphics state: {}", e);
                        return;
                    }
                };

            if let Some(ref mut menu_handler) = self.menu_handler {
                menu_handler.attach_to_window(&window);
            }

            let editor = EditorService::new();

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
                        // Check if the click is on the modal first
                        if let Some((x, y)) = self.mouse_position {
                            if self.handle_modal_click(x, y) {
                                // Click was consumed by modal
                                return;
                            }
                        }

                        if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                            if let Some((x, y)) = self.mouse_position {
                                let now = Instant::now();

                                let click_count = if let (Some(last_time), Some((last_x, last_y))) =
                                    (self.last_click_time, self.last_click_position)
                                {
                                    let time_diff = now.duration_since(last_time);
                                    let pos_diff = (x - last_x).powi(2) + (y - last_y).powi(2);
                                    let pos_diff = pos_diff.sqrt();

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
                                        editor.cursor_select_word();
                                    }
                                    3 => {
                                        editor.cursor_select_line();
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
                                    None,
                                    editor.file_path(),
                                ) {
                                    eprintln!("Failed to update geometry: {}", e);
                                }
                                // Request redraw after mouse click (required for Wait mode)
                                state.window().request_redraw();
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
                        MouseScrollDelta::LineDelta(_, y) => -(y.round() as i32),
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
                            None,
                            editor.file_path(),
                        ) {
                            eprintln!("Failed to update geometry after scroll: {}", e);
                        }
                        // Request redraw after scroll (required for Wait mode)
                        state.window().request_redraw();
                    }
                }
                // Update modal geometry (including match highlights) after scroll
                // This ensures highlights move with the text when scrolling
                if self.modal_state.is_open() {
                    self.update_modal_geometry();
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
                            None,
                            editor.file_path(),
                        ) {
                            eprintln!("Failed to update geometry: {}", e);
                        }
                        // Request redraw after cursor drag (required for Wait mode)
                        state.window().request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;
                use winit::keyboard::{KeyCode, PhysicalKey};

                // Only process on key press
                if event.state != ElementState::Pressed {
                    return;
                }

                let ctrl_or_cmd = self.modifiers.control_key() || self.modifiers.super_key();

                // Check for modal open shortcuts (Ctrl+F, Ctrl+H) when modal is closed
                if !self.modal_state.is_open() {
                    if let PhysicalKey::Code(code) = event.physical_key {
                        match code {
                            KeyCode::KeyF if ctrl_or_cmd => {
                                self.open_find_modal();
                                return;
                            }
                            KeyCode::KeyH if ctrl_or_cmd => {
                                self.open_replace_modal();
                                return;
                            }
                            _ => {}
                        }
                    }
                }

                // Route to modal if open
                if self.modal_state.is_open() {
                    self.reset_cursor_blink();

                    // Handle clipboard operations (Ctrl+C, Ctrl+V, Ctrl+X)
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if ctrl_or_cmd {
                            match code {
                                KeyCode::KeyC => {
                                    // Copy from modal input
                                    if let Some(modal) = self.modal_state.as_find() {
                                        if let Some(text) = modal.focused_input().selected_text() {
                                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                                let _ = cb.set_text(text);
                                            }
                                        }
                                    }
                                    return;
                                }
                                KeyCode::KeyV => {
                                    // Paste into modal input
                                    if let Ok(mut cb) = arboard::Clipboard::new() {
                                        if let Ok(text) = cb.get_text() {
                                            if let Some(modal) = self.modal_state.as_find_mut() {
                                                modal.focused_input_mut().insert_text(&text);
                                                let action = if modal.focused_field
                                                    == crate::ui::modal::FindField::Find
                                                {
                                                    ModalAction::UpdateQuery
                                                } else {
                                                    ModalAction::Redraw
                                                };
                                                self.handle_modal_action(action);
                                            }
                                        }
                                    }
                                    return;
                                }
                                KeyCode::KeyX => {
                                    // Cut from modal input
                                    if let Some(modal) = self.modal_state.as_find_mut() {
                                        if let Some(text) = modal.focused_input().selected_text() {
                                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                                let _ = cb.set_text(text);
                                            }
                                            modal.focused_input_mut().delete_selection();
                                            let action = if modal.focused_field
                                                == crate::ui::modal::FindField::Find
                                            {
                                                ModalAction::UpdateQuery
                                            } else {
                                                ModalAction::Redraw
                                            };
                                            self.handle_modal_action(action);
                                        }
                                    }
                                    return;
                                }
                                KeyCode::KeyF => {
                                    // Ctrl+F while modal is open - just focus find field
                                    if let Some(modal) = self.modal_state.as_find_mut() {
                                        modal.focus_find();
                                        self.handle_modal_action(ModalAction::Redraw);
                                    }
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Route key event to modal
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if let Some(modal) = self.modal_state.as_find_mut() {
                            let action = modal.handle_key(code, self.modifiers);
                            self.handle_modal_action(action);
                        }
                    }

                    // Handle character input for modal
                    if let Some(ref text) = event.text {
                        for c in text.chars() {
                            // Skip control characters handled above
                            if ctrl_or_cmd
                                && matches!(
                                    c,
                                    'c' | 'C'
                                        | 'v'
                                        | 'V'
                                        | 'x'
                                        | 'X'
                                        | 'f'
                                        | 'F'
                                        | 'h'
                                        | 'H'
                                        | 'a'
                                        | 'A'
                                        | 'g'
                                        | 'G'
                                )
                            {
                                continue;
                            }
                            if let Some(modal) = self.modal_state.as_find_mut() {
                                let action = modal.handle_char(c);
                                self.handle_modal_action(action);
                            }
                        }
                    }

                    return;
                }

                // Normal editor key handling
                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    self.keyboard.handle_key_event(editor, event);

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
                        None,
                        editor.file_path(),
                    ) {
                        eprintln!("Failed to update geometry: {}", e);
                    }

                    // Request redraw after keyboard input (required for Wait mode)
                    state.window().request_redraw();
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: MenuAction) {
        self.handle_menu_action(event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.poll_menu_events();

        // Use Wait instead of Poll to avoid burning CPU when idle.
        // The window will be redrawn when events occur or when
        // window.request_redraw() is called after state changes.
        if self.state.is_some() {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
