use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowAttributes, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActiveEventLoopExtMacOS, WindowExtMacOS};

use crate::application::EditorService;
use crate::interface::KeyboardController;
use crate::menu::{MenuAction, MenuHandler};
use crate::state::State;
use crate::ui::modal::{ModalAction, ModalState};

/// Tabbing identifier for macOS native tabs - all editor windows share this
const TABBING_IDENTIFIER: &str = "dev.textedit.editor";

#[derive(Clone, Copy, PartialEq)]
pub enum MouseButtonState {
    Released,
    Pressed,
}

/// Per-window state containing rendering state and editor
pub struct WindowState {
    pub state: State,
    pub editor: EditorService,
    pub keyboard: KeyboardController,
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

impl WindowState {
    pub fn new(state: State, editor: EditorService) -> Self {
        Self {
            state,
            editor,
            keyboard: KeyboardController::new(),
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

    /// Reset cursor blink (make cursor visible immediately)
    pub fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_timer = Instant::now();
    }

    /// Update window title based on current document
    pub fn update_window_title(&self) {
        let title = self.editor.title();
        let modified = self.editor.is_modified();
        let prefix = if modified { "● " } else { "" };
        let window_title = format!("{}textedit - {}", prefix, title);
        self.state.window().set_title(&window_title);
    }

    /// Ensure cursor is visible and update all geometry
    pub fn ensure_cursor_visible_and_update(&mut self) {
        self.state.ensure_cursor_visible(
            self.editor.cursor(),
            self.editor.buffer(),
            self.editor.show_line_numbers(),
            self.editor.show_status_bar(),
        );

        if let Err(e) = self.state.update_geometry(
            self.editor.buffer(),
            self.editor.cursor(),
            self.editor.show_line_numbers(),
            self.editor.show_status_bar(),
            None,
            self.editor.file_path(),
        ) {
            eprintln!("Failed to update geometry: {}", e);
        }

        self.update_modal_geometry();
    }

    /// Update modal geometry in state
    pub fn update_modal_geometry(&mut self) {
        let matches = if self.modal_state.is_open() {
            self.editor.find_all_matches()
        } else {
            Vec::new()
        };
        let current_match = if self.modal_state.is_open() {
            self.editor
                .current_match_index()
                .map(|i| i.saturating_sub(1))
        } else {
            None
        };

        if let Err(e) = self.state.update_modal_geometry(
            &self.modal_state,
            &matches,
            current_match,
            self.editor.buffer(),
            self.editor.show_line_numbers(),
            self.editor.show_status_bar(),
            self.cursor_visible,
        ) {
            eprintln!("Failed to update modal geometry: {}", e);
        }
    }
}

pub struct App {
    /// Map of window ID to window state - each window is a "tab" in native macOS terms
    pub windows: HashMap<WindowId, WindowState>,
    pub menu_handler: Option<MenuHandler>,
    /// Track if we've created the initial window
    pub initialized: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            menu_handler: None,
            initialized: false,
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

    /// Create a new editor window (appears as a native tab on macOS)
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop, title: &str) -> Option<WindowId> {
        self.create_window_with_content(event_loop, title, None)
    }

    /// Create a new editor window with optional file content
    pub fn create_window_with_content(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: &str,
        file_path: Option<&str>,
    ) -> Option<WindowId> {
        // Enable automatic window tabbing on macOS
        #[cfg(target_os = "macos")]
        {
            event_loop.set_allows_automatic_window_tabbing(true);
        }

        let window_attributes = WindowAttributes::default().with_title(title);

        let window = match event_loop.create_window(window_attributes) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                return None;
            }
        };
        let window = Arc::new(window);
        let window_id = window.id();

        // Set macOS tabbing identifier so windows can be grouped
        #[cfg(target_os = "macos")]
        {
            window.set_tabbing_identifier(TABBING_IDENTIFIER);
        }

        // Attach menu to window
        if let Some(ref mut menu_handler) = self.menu_handler {
            menu_handler.attach_to_window(&window);
        }

        // Create GPU state
        let editor_config = crate::config::EditorConfig::default();
        let state = match pollster::block_on(State::new(window.clone(), editor_config)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to initialize graphics state: {}", e);
                return None;
            }
        };

        // Create editor service
        let mut editor = EditorService::new();

        // Load file if provided
        if let Some(path) = file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                editor.buffer_mut().clear();
                editor.buffer_mut().insert(0, &content);
                editor.set_file_path(path.to_string());
                editor.set_modified(false);
            }
        }

        let mut window_state = WindowState::new(state, editor);
        window_state.update_window_title();

        self.windows.insert(window_id, window_state);

        // Request initial redraw
        window.request_redraw();

        Some(window_id)
    }

    /// Get the currently focused window state
    fn get_focused_window(&mut self) -> Option<&mut WindowState> {
        // For now, just get the first window if any exist
        // In a real implementation, we'd track which window has focus
        self.windows.values_mut().next()
    }

    /// Handle opening a new tab (creates a new window on macOS)
    pub fn open_new_tab(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop, "textedit - Untitled");
    }

    /// Handle opening a file in a new tab
    pub fn open_file_in_new_tab(&mut self, event_loop: &ActiveEventLoop, path: &str) {
        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");
        let title = format!("textedit - {}", file_name);
        self.create_window_with_content(event_loop, &title, Some(path));
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationHandler<MenuAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.initialized {
            // Create initial window
            self.create_window(event_loop, "textedit - Untitled");
            self.initialized = true;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Get the window state for this specific window
        let window_state = match self.windows.get_mut(&window_id) {
            Some(ws) => ws,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                // Remove this window
                self.windows.remove(&window_id);

                // If no windows left, exit the app
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = window_state.state.render() {
                    eprintln!("Render error: {}", e);
                }
                window_state.state.window().request_redraw();
            }
            WindowEvent::Resized(physical_size) => {
                window_state
                    .state
                    .resize(physical_size.width, physical_size.height);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Err(e) = window_state.state.set_scale_factor(scale_factor) {
                    eprintln!("Failed to update scale factor: {}", e);
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                window_state.modifiers = mods.state();
                window_state.keyboard.set_modifiers(window_state.modifiers);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::MouseButton;

                if button == MouseButton::Left {
                    window_state.mouse_button_state = if state
                        == winit::event::ElementState::Pressed
                    {
                        // Check if the click is on the modal
                        if let Some((x, y)) = window_state.mouse_position {
                            if handle_modal_click(window_state, x, y) {
                                return;
                            }
                        }

                        if let Some((x, y)) = window_state.mouse_position {
                            let now = Instant::now();

                            let click_count = if let (Some(last_time), Some((last_x, last_y))) = (
                                window_state.last_click_time,
                                window_state.last_click_position,
                            ) {
                                let time_diff = now.duration_since(last_time);
                                let pos_diff = (x - last_x).powi(2) + (y - last_y).powi(2);
                                let pos_diff = pos_diff.sqrt();

                                if time_diff.as_millis() < 500 && pos_diff < 10.0 {
                                    if window_state.click_count >= 2 {
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

                            window_state.click_count = click_count;
                            window_state.last_click_time = Some(now);
                            window_state.last_click_position = Some((x, y));

                            let (line, col) = window_state
                                .state
                                .get_char_at_position(
                                    x,
                                    y,
                                    window_state.editor.buffer(),
                                    window_state.editor.show_line_numbers(),
                                    window_state.editor.show_status_bar(),
                                )
                                .unwrap_or((0, 0));
                            let char_idx = window_state
                                .editor
                                .buffer()
                                .line_col_to_char(line, col)
                                .unwrap_or(0);

                            match click_count {
                                2 => {
                                    window_state.editor.cursor_select_word();
                                }
                                3 => {
                                    window_state.editor.cursor_select_line();
                                }
                                _ => {
                                    window_state.editor.cursor_mut().set_position(char_idx);
                                }
                            }

                            if let Err(e) = window_state.state.update_geometry(
                                window_state.editor.buffer(),
                                window_state.editor.cursor(),
                                window_state.editor.show_line_numbers(),
                                window_state.editor.show_status_bar(),
                                None,
                                window_state.editor.file_path(),
                            ) {
                                eprintln!("Failed to update geometry: {}", e);
                            }
                            window_state.state.window().request_redraw();
                        }
                        MouseButtonState::Pressed
                    } else {
                        window_state.is_dragging = false;
                        MouseButtonState::Released
                    };
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;

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
                    window_state.state.scroll_by_lines(
                        lines_delta,
                        window_state.editor.buffer(),
                        window_state.editor.show_line_numbers(),
                        window_state.editor.show_status_bar(),
                    );
                    if let Err(e) = window_state.state.update_geometry(
                        window_state.editor.buffer(),
                        window_state.editor.cursor(),
                        window_state.editor.show_line_numbers(),
                        window_state.editor.show_status_bar(),
                        None,
                        window_state.editor.file_path(),
                    ) {
                        eprintln!("Failed to update geometry after scroll: {}", e);
                    }
                    window_state.state.window().request_redraw();
                }

                if window_state.modal_state.is_open() {
                    window_state.update_modal_geometry();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                window_state.mouse_position = Some((position.x, position.y));

                if window_state.mouse_button_state == MouseButtonState::Pressed
                    || window_state.is_dragging
                {
                    let (line, col) = window_state
                        .state
                        .get_char_at_position(
                            position.x,
                            position.y,
                            window_state.editor.buffer(),
                            window_state.editor.show_line_numbers(),
                            window_state.editor.show_status_bar(),
                        )
                        .unwrap_or((0, 0));
                    let char_idx = window_state
                        .editor
                        .buffer()
                        .line_col_to_char(line, col)
                        .unwrap_or(0);
                    window_state.editor.cursor_mut().extend_selection(char_idx);
                    window_state.is_dragging = true;

                    if let Err(e) = window_state.state.update_geometry(
                        window_state.editor.buffer(),
                        window_state.editor.cursor(),
                        window_state.editor.show_line_numbers(),
                        window_state.editor.show_status_bar(),
                        None,
                        window_state.editor.file_path(),
                    ) {
                        eprintln!("Failed to update geometry: {}", e);
                    }
                    window_state.state.window().request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::event::ElementState;
                use winit::keyboard::{KeyCode, PhysicalKey};

                if event.state != ElementState::Pressed {
                    return;
                }

                let ctrl_or_cmd =
                    window_state.modifiers.control_key() || window_state.modifiers.super_key();
                let shift = window_state.modifiers.shift_key();

                // Handle native tab switching on macOS
                #[cfg(target_os = "macos")]
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::Tab if ctrl_or_cmd => {
                            if shift {
                                window_state.state.window().select_previous_tab();
                            } else {
                                window_state.state.window().select_next_tab();
                            }
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle modal input first
                if window_state.modal_state.is_open() {
                    let action = handle_modal_keyboard(window_state, &event);
                    handle_modal_action(window_state, action);
                    return;
                }

                // Handle keyboard shortcuts
                if let PhysicalKey::Code(code) = event.physical_key {
                    match code {
                        KeyCode::KeyF if ctrl_or_cmd => {
                            open_find_modal(window_state);
                            return;
                        }
                        KeyCode::KeyH if ctrl_or_cmd => {
                            open_replace_modal(window_state);
                            return;
                        }
                        KeyCode::KeyG if ctrl_or_cmd => {
                            if shift {
                                window_state.editor.find_prev();
                            } else {
                                window_state.editor.find_next();
                            }
                            window_state.ensure_cursor_visible_and_update();
                            return;
                        }
                        _ => {}
                    }
                }

                // Handle regular keyboard input
                let action = window_state.keyboard.handle_key(&event);
                handle_keyboard_action(window_state, action);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, action: MenuAction) {
        // Find the focused window to apply the action to
        // For simplicity, we'll use the first window in the map
        let window_id = match self.windows.keys().next().cloned() {
            Some(id) => id,
            None => return,
        };

        // Handle actions that need access to event_loop separately
        match &action {
            MenuAction::New => {
                self.create_window(event_loop, "textedit - Untitled");
                return;
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
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    self.open_file_in_new_tab(event_loop, &path_str);
                }
                return;
            }
            MenuAction::Close => {
                self.windows.remove(&window_id);
                if self.windows.is_empty() {
                    event_loop.exit();
                }
                return;
            }
            MenuAction::Quit => {
                event_loop.exit();
                return;
            }
            _ => {}
        }

        // Get the window state for other actions
        let window_state = match self.windows.get_mut(&window_id) {
            Some(ws) => ws,
            None => return,
        };

        handle_menu_action(window_state, action);
    }
}

// Helper functions moved outside impl to avoid borrow issues

fn handle_modal_click(window_state: &mut WindowState, x: f64, y: f64) -> bool {
    use crate::ui::modal::find_modal::FindField;

    if !window_state.modal_state.is_open() {
        return false;
    }

    let x = x as f32;
    let y = y as f32;

    if let Some(modal_rect) = &window_state.state.modal_rect {
        if !modal_rect.contains(x, y) {
            return false;
        }
    } else {
        return false;
    }

    // Check button regions
    for (button, rect) in &window_state.state.modal_button_regions {
        if rect.contains(x, y) {
            let button = *button;
            let action = if let Some(find_modal) = window_state.modal_state.as_find_mut() {
                find_modal.handle_button(button)
            } else {
                return true;
            };
            handle_modal_action(window_state, action);
            return true;
        }
    }

    // Check input field regions
    for (field, rect) in &window_state.state.modal_input_regions {
        if rect.contains(x, y) {
            let field = *field;
            if let Some(find_modal) = window_state.modal_state.as_find_mut() {
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
                window_state.reset_cursor_blink();
                window_state.update_modal_geometry();
            }
            return true;
        }
    }

    true
}

fn handle_modal_keyboard(
    window_state: &mut WindowState,
    event: &winit::event::KeyEvent,
) -> ModalAction {
    use winit::keyboard::PhysicalKey;

    if let Some(find_modal) = window_state.modal_state.as_find_mut() {
        // First check for keyboard shortcuts via handle_key
        if let PhysicalKey::Code(code) = event.physical_key {
            let modifiers = window_state.modifiers;
            let action = find_modal.handle_key(code, modifiers);
            if action != ModalAction::None {
                return action;
            }
        }

        // Handle text input for character entry
        if let Some(ref text) = event.text {
            for ch in text.chars() {
                let action = find_modal.handle_char(ch);
                if action != ModalAction::None {
                    return action;
                }
            }
        }
    }

    ModalAction::None
}

fn handle_modal_action(window_state: &mut WindowState, action: ModalAction) {
    match action {
        ModalAction::None => {}
        ModalAction::Close => {
            window_state.modal_state.close();
            window_state.update_modal_geometry();
        }
        ModalAction::FindNext => {
            update_editor_from_modal(window_state);
            window_state.editor.find_next();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        ModalAction::FindPrev => {
            update_editor_from_modal(window_state);
            window_state.editor.find_prev();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        ModalAction::Replace => {
            update_editor_from_modal(window_state);
            window_state.editor.replace_and_find_next();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        ModalAction::ReplaceAll => {
            update_editor_from_modal(window_state);
            let _ = window_state.editor.replace_all();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        ModalAction::UpdateQuery => {
            update_editor_from_modal(window_state);
            update_modal_matches(window_state);
            window_state.update_modal_geometry();
        }
        ModalAction::Redraw => {
            window_state.reset_cursor_blink();
            window_state.update_modal_geometry();
        }
    }
}

fn update_editor_from_modal(window_state: &mut WindowState) {
    if let Some(find_modal) = window_state.modal_state.as_find() {
        let query = find_modal.query();
        if !query.is_empty() {
            window_state.editor.set_find_query(Some(query.to_string()));
        } else {
            window_state.editor.set_find_query(None);
        }
        let replacement = find_modal.replacement();
        window_state
            .editor
            .set_replace_text(Some(replacement.to_string()));
    }
}

fn update_modal_matches(window_state: &mut WindowState) {
    if let Some(find_modal) = window_state.modal_state.as_find_mut() {
        let matches = window_state.editor.find_all_matches();
        let current_index = window_state.editor.current_match_index();
        find_modal.update_matches(matches, current_index);
    }
}

fn open_find_modal(window_state: &mut WindowState) {
    window_state.modal_state.open_find();
    window_state.reset_cursor_blink();
    sync_modal_with_editor(window_state);
    window_state.update_modal_geometry();
}

fn open_replace_modal(window_state: &mut WindowState) {
    window_state.modal_state.open_replace();
    window_state.reset_cursor_blink();
    sync_modal_with_editor(window_state);
    window_state.update_modal_geometry();
}

fn sync_modal_with_editor(window_state: &mut WindowState) {
    if let Some(find_modal) = window_state.modal_state.as_find_mut() {
        if let Some(query) = window_state.editor.find_query() {
            if find_modal.find_input.is_empty() && !query.is_empty() {
                find_modal.find_input.set_text(query);
            }
        }
        if let Some(replace) = window_state.editor.replace_text() {
            if find_modal.replace_input.is_empty() && !replace.is_empty() {
                find_modal.replace_input.set_text(replace);
            }
        }
    }
}

fn handle_keyboard_action(window_state: &mut WindowState, action: crate::interface::KeyAction) {
    use crate::interface::KeyAction;

    match action {
        KeyAction::None => {}
        KeyAction::InsertChar(ch) => {
            window_state.editor.insert_char(ch);
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::InsertText(text) => {
            for ch in text.chars() {
                window_state.editor.insert_char(ch);
            }
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Backspace => {
            window_state.editor.delete_backward();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Delete => {
            window_state.editor.delete_forward();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::MoveCursor(direction, extend_selection) => {
            use crate::interface::CursorDirection;
            match direction {
                CursorDirection::Left => {
                    if extend_selection {
                        window_state.editor.cursor_move_left_select();
                    } else {
                        window_state.editor.cursor_move_left();
                    }
                }
                CursorDirection::Right => {
                    if extend_selection {
                        window_state.editor.cursor_move_right_select();
                    } else {
                        window_state.editor.cursor_move_right();
                    }
                }
                CursorDirection::Up => {
                    if extend_selection {
                        window_state.editor.cursor_move_up_select();
                    } else {
                        window_state.editor.cursor_move_up();
                    }
                }
                CursorDirection::Down => {
                    if extend_selection {
                        window_state.editor.cursor_move_down_select();
                    } else {
                        window_state.editor.cursor_move_down();
                    }
                }
                CursorDirection::Home => {
                    if extend_selection {
                        window_state.editor.cursor_move_home_select();
                    } else {
                        window_state.editor.cursor_move_home();
                    }
                }
                CursorDirection::End => {
                    if extend_selection {
                        window_state.editor.cursor_move_end_select();
                    } else {
                        window_state.editor.cursor_move_end();
                    }
                }
                CursorDirection::WordLeft => {
                    if extend_selection {
                        window_state.editor.cursor_move_word_left_select();
                    } else {
                        window_state.editor.cursor_move_word_left();
                    }
                }
                CursorDirection::WordRight => {
                    if extend_selection {
                        window_state.editor.cursor_move_word_right_select();
                    } else {
                        window_state.editor.cursor_move_word_right();
                    }
                }
                CursorDirection::PageUp => {
                    window_state.editor.cursor_page_up();
                    if !extend_selection {
                        window_state.editor.clear_selection();
                    }
                }
                CursorDirection::PageDown => {
                    window_state.editor.cursor_page_down();
                    if !extend_selection {
                        window_state.editor.clear_selection();
                    }
                }
                CursorDirection::DocumentStart => {
                    window_state
                        .editor
                        .cursor_move_to_document_start(extend_selection);
                }
                CursorDirection::DocumentEnd => {
                    window_state
                        .editor
                        .cursor_move_to_document_end(extend_selection);
                }
            }
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::SelectAll => {
            window_state.editor.select_all();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Copy => {
            window_state.editor.copy_selection();
        }
        KeyAction::Cut => {
            window_state.editor.cut_selection();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Paste => {
            window_state.editor.paste();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Undo => {
            window_state.editor.undo();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Redo => {
            window_state.editor.redo();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::NewLine => {
            window_state.editor.insert_char('\n');
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Tab => {
            // Insert 4 spaces for tab
            for _ in 0..4 {
                window_state.editor.insert_char(' ');
            }
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::Unindent => {
            window_state.editor.unindent();
            window_state.ensure_cursor_visible_and_update();
        }
        KeyAction::ClearSelection => {
            window_state.editor.clear_selection();
            window_state.ensure_cursor_visible_and_update();
        }
        // These actions are handled at the app level before handle_keyboard_action
        KeyAction::OpenFind
        | KeyAction::OpenReplace
        | KeyAction::FindNext
        | KeyAction::FindPrev => {}
    }

    window_state.update_window_title();
}

fn handle_menu_action(window_state: &mut WindowState, action: MenuAction) {
    match action {
        MenuAction::Save => {
            let do_save_as = if let Some(path) = window_state.editor.file_path() {
                let content = window_state.editor.buffer().as_str();
                if std::fs::write(path, content).is_ok() {
                    window_state.editor.set_modified(false);
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
                            "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css", "js",
                            "ts",
                        ],
                    )
                    .add_filter("All Files", &["*"])
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .save_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    let content = window_state.editor.buffer().as_str();
                    if std::fs::write(&path_str, content).is_ok() {
                        window_state.editor.set_file_path(path_str);
                        window_state.editor.set_modified(false);
                    }
                }
            }
            window_state.update_window_title();
        }
        MenuAction::SaveAs => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "Text Files",
                    &[
                        "txt", "md", "rs", "json", "toml", "yaml", "yml", "html", "css", "js", "ts",
                    ],
                )
                .add_filter("All Files", &["*"])
                .set_directory(std::env::current_dir().unwrap_or_default())
                .save_file()
            {
                let path_str = path.to_string_lossy().to_string();
                let content = window_state.editor.buffer().as_str();
                if std::fs::write(&path_str, content).is_ok() {
                    window_state.editor.set_file_path(path_str);
                    window_state.editor.set_modified(false);
                }
            }
            window_state.update_window_title();
        }
        MenuAction::Find => {
            open_find_modal(window_state);
        }
        MenuAction::FindNext => {
            window_state.editor.find_next();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::FindPrev => {
            window_state.editor.find_prev();
            update_modal_matches(window_state);
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::Replace => {
            open_replace_modal(window_state);
        }
        MenuAction::ToggleLineNumbers => {
            window_state.editor.toggle_line_numbers();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::ToggleStatusBar => {
            window_state.editor.toggle_status_bar();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::About => {
            let _ = rfd::MessageDialog::new()
                .set_title("About textedit")
                .set_description("textedit v0.1.0\n\nA fast, cross-platform text editor built with Rust and wgpu.")
                .set_level(rfd::MessageLevel::Info)
                .show();
        }
        MenuAction::SetEditorTheme(ref theme_name) => {
            use crate::themes::EditorTheme;
            let new_theme = match theme_name.as_str() {
                "Dracula" => EditorTheme::Dracula,
                "SolarizedDark" => EditorTheme::SolarizedDark,
                "OneDark" => EditorTheme::OneDark,
                "GruvboxDark" => EditorTheme::GruvboxDark,
                "Light" => EditorTheme::Light,
                _ => EditorTheme::Dracula,
            };
            window_state.state.config.theme = new_theme;
            window_state.state.window().request_redraw();
        }
        MenuAction::Undo => {
            window_state.editor.undo();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::Redo => {
            window_state.editor.redo();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::Cut => {
            window_state.editor.cut_selection();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::Copy => {
            window_state.editor.copy_selection();
        }
        MenuAction::Paste => {
            window_state.editor.paste();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::SelectAll => {
            window_state.editor.select_all();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::Delete => {
            window_state.editor.delete_forward();
            window_state.ensure_cursor_visible_and_update();
        }
        MenuAction::SetTheme(ref theme_name) => {
            use crate::themes::EditorTheme;
            let new_theme = match theme_name.as_str() {
                "Dracula" => EditorTheme::Dracula,
                "SolarizedDark" => EditorTheme::SolarizedDark,
                "OneDark" => EditorTheme::OneDark,
                "GruvboxDark" => EditorTheme::GruvboxDark,
                "Light" => EditorTheme::Light,
                _ => EditorTheme::Dracula,
            };
            window_state.state.config.theme = new_theme;
            window_state.state.window().request_redraw();
        }
        // These are handled in user_event before this function
        MenuAction::New | MenuAction::Open | MenuAction::Close | MenuAction::Quit => {}
    }

    // Update geometry after menu action
    if let Err(e) = window_state.state.update_geometry(
        window_state.editor.buffer(),
        window_state.editor.cursor(),
        window_state.editor.show_line_numbers(),
        window_state.editor.show_status_bar(),
        None,
        window_state.editor.file_path(),
    ) {
        eprintln!("Failed to update geometry: {}", e);
    }
}
