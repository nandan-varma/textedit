mod state;
mod event;
mod clipboard;
mod helpers;

pub use state::*;

use crate::menu::MenuAction;
use crate::app::helpers::{apply_undo, apply_redo, delete_selection_or_char};
use crate::app::clipboard::{copy_selection, cut_selection, paste_at_cursor};
// use crate::themes::EditorTheme;

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            editor: None,
            keyboard: crate::editor::KeyboardController::new(),
            menu_handler: None,
            modifiers: winit::keyboard::ModifiersState::empty(),
            mouse_button_state: MouseButtonState::Released,
            mouse_position: None,
            last_click_time: None,
            last_click_position: None,
            is_dragging: false,
            click_count: 0,
        }
    }

    pub fn with_menu(mut self, menu_handler: crate::menu::MenuHandler) -> Self {
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
                        MenuAction::Save => {
                            if let Some(path) = editor.file_path() {
                                let content = editor.buffer().as_str();
                                if std::fs::write(path, content).is_ok() {
                                    editor.set_modified(false);
                                }
                            } else {
                                // No file path, so do Save As
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
                                .set_directory(&std::env::current_dir().unwrap())
                                .set_file_name(editor.file_path().map(|p| std::path::Path::new(p).file_name().and_then(|n| n.to_str()).unwrap_or("")).unwrap_or(""))
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
                            let _ = editor.find_next();
                        }
                        MenuAction::FindPrev => {
                            let _ = editor.find_prev();
                        }
                        MenuAction::Replace => {
                            editor.begin_replace();
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
            MenuAction::SetEditorTheme(theme_name) => {
                use crate::themes::EditorTheme;
                if let Some(state) = &mut self.state {
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
                editor.buffer_mut().clear();
                editor.cursor_mut().set_position(0);
                editor.history_mut().clear();
            }
            MenuAction::Close => {
                std::process::exit(0);
            }
            MenuAction::Quit => {
                std::process::exit(0);
            }
            MenuAction::Undo => {
                if let Some(op) = editor.history_mut().undo() {
                    apply_undo(editor, op);
                }
            }
            MenuAction::Redo => {
                if let Some(op) = editor.history_mut().redo() {
                    apply_redo(editor, op);
                }
            }
            MenuAction::Cut => {
                if let Some(sel) = editor.cursor().selection() {
                    cut_selection(editor, sel);
                }
            }
            MenuAction::Copy => {
                if let Some(sel) = editor.cursor().selection() {
                    copy_selection(editor, sel);
                }
            }
            MenuAction::Paste => {
                paste_at_cursor(editor);
            }
            MenuAction::Delete => {
                delete_selection_or_char(editor);
            }
            MenuAction::SelectAll => {
                let len = editor.buffer().len_chars();
                if len > 0 {
                    editor.cursor_mut().set_selection_start(0);
                    editor.cursor_mut().set_selection_end(len);
                }
            }
            MenuAction::Find => {
                editor.begin_find();
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
            MenuAction::SetTheme(theme_name) => {
                if let Some(state) = &mut self.state {
                    // Update syntax highlighter
                    state.syntax = crate::syntax::SyntaxHighlighter::new(&theme_name);
                    // Update UI colors from syntect theme
                    let ts = syntect::highlighting::ThemeSet::load_defaults();
                    let theme = ts.themes.get(&theme_name).unwrap_or_else(|| ts.themes.values().next().unwrap());
                    // Only update syntax theme, not UI theme
                    state.config.syntax_theme = theme_name.clone();
                    // Redraw
                    state.window().request_redraw();
                }
            }
            // MenuAction::SetEditorTheme(editor_theme) => {
            //     if let Some(state) = &mut self.state {
            //         state.config.theme = editor_theme;
            //         state.window().request_redraw();
            //     }
            // }
        } // end match action

        if let Some(state) = &mut self.state {
            let state: &mut crate::state::State = state;
            let status_override = editor.command_bar_status_text();
            if let Err(e) = state.update_geometry(
                editor.buffer(),
                editor.cursor(),
                editor.show_line_numbers(),
                editor.show_status_bar(),
                status_override,
                editor.file_path(),
            ) {
                eprintln!("Failed to update geometry: {}", e);
            }
        }
    }
}
