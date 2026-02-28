use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::ModifiersState;
use winit::window::WindowAttributes;

use crate::editor::Editor;
use crate::editor::KeyboardController;
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
            modifiers: ModifiersState::empty(),
            mouse_button_state: MouseButtonState::Released,
            mouse_position: None,
            last_click_time: None,
            last_click_position: None,
            is_dragging: false,
            click_count: 0,
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
                        // Mouse pressed - position cursor
                        if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                            if let Some((x, y)) = self.mouse_position {
                                let now = Instant::now();

                                // Determine click count
                                let click_count = if let (Some(last_time), Some((last_x, last_y))) =
                                    (self.last_click_time, self.last_click_position)
                                {
                                    let time_diff = now.duration_since(last_time);
                                    let pos_diff =
                                        ((x - last_x).powi(2) + (y - last_y).powi(2)).sqrt();

                                    if time_diff.as_millis() < 500 && pos_diff < 10.0 {
                                        if self.click_count >= 2 {
                                            3 // triple click
                                        } else {
                                            2 // double click
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

                                // Handle click based on count
                                let (line, col) = state.get_char_at_position(x, y, editor.buffer());
                                let char_idx =
                                    editor.buffer().line_col_to_char(line, col).unwrap_or(0);

                                match click_count {
                                    2 => {
                                        // Double click - select word
                                        let buffer = editor.buffer().clone();
                                        editor.cursor_mut().select_word_at_cursor(&buffer);
                                    }
                                    3 => {
                                        // Triple click - select line
                                        let buffer = editor.buffer().clone();
                                        editor.cursor_mut().select_line(&buffer);
                                    }
                                    _ => {
                                        // Single click - position cursor
                                        editor.cursor_mut().set_position(char_idx);
                                    }
                                }

                                if let Err(e) =
                                    state.update_geometry(editor.buffer(), editor.cursor())
                                {
                                    eprintln!("Failed to update geometry: {}", e);
                                }
                            }
                        }
                        MouseButtonState::Pressed
                    } else {
                        // Mouse released
                        self.is_dragging = false;
                        MouseButtonState::Released
                    };
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some((position.x, position.y));

                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    if self.mouse_button_state == MouseButtonState::Pressed || self.is_dragging {
                        let (line, col) =
                            state.get_char_at_position(position.x, position.y, editor.buffer());
                        let char_idx = editor.buffer().line_col_to_char(line, col).unwrap_or(0);
                        editor.cursor_mut().extend_selection(char_idx);
                        self.is_dragging = true;

                        if let Err(e) = state.update_geometry(editor.buffer(), editor.cursor()) {
                            eprintln!("Failed to update geometry: {}", e);
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    self.keyboard.handle_key_event(editor, event);
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
