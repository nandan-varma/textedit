use crate::app::{App, MouseButtonState};
use crate::menu::MenuAction;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowAttributes;
// (all helper/clipboard imports are now in mod.rs)

impl ApplicationHandler<MenuAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = WindowAttributes::default().with_title("textedit - Untitled");

            let window = event_loop.create_window(window_attributes).unwrap();
            let window = Arc::new(window);

            // TODO: Load config from file if available
            let editor_config = crate::config::EditorConfig::default();
            let state = pollster::block_on(crate::state::State::new(window.clone(), editor_config))
                .expect("Failed to initialize graphics state");

            // Initialize menu handler if we have one
            if let Some(ref mut menu_handler) = self.menu_handler {
                menu_handler.attach_to_window(&window);
            }

            let editor = crate::editor::Editor::new();

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
                    let state: &mut crate::state::State = state;
                    if let Err(e) = state.render() {
                        eprintln!("Render error: {}", e);
                    }
                    state.window().request_redraw();
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    let state: &mut crate::state::State = state;
                    state.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(state) = &mut self.state {
                    let state: &mut crate::state::State = state;
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
                            let editor: &mut crate::editor::Editor = editor;
                            let state: &mut crate::state::State = state;
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
                                    editor.command_bar_status_text(),
                                    editor.file_path(),
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
                    let editor: &mut crate::editor::Editor = editor;
                    let state: &mut crate::state::State = state;
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
                            editor.command_bar_status_text(),
                            editor.file_path(),
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
                        let editor: &mut crate::editor::Editor = editor;
                        let state: &mut crate::state::State = state;
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
                            editor.command_bar_status_text(),
                            editor.file_path(),
                        ) {
                            eprintln!("Failed to update geometry: {}", e);
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let (Some(editor), Some(state)) = (&mut self.editor, &mut self.state) {
                    let editor: &mut crate::editor::Editor = editor;
                    let state: &mut crate::state::State = state;
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
                        editor.command_bar_status_text(),
                        editor.file_path(),
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
