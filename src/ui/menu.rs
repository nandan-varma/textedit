use crate::app::TextEditAppState;
use crate::editor::{FileEncoding, LineEnding};
use egui::{MenuBar, Response, Ui};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuBarState {
    pub show_find: bool,
    pub show_goto: bool,
    pub show_about: bool,
    pub show_replace: bool,
}

pub fn file_menu_button(ui: &mut Ui, app: &mut TextEditAppState) -> Response {
    ui.menu_button("File", |ui| {
        if ui.button("New").clicked() {
            app.new_file();
            ui.close();
        }
        if ui.button("Open...").clicked() {
            app.open_file_dialog();
            ui.close();
        }
        ui.separator();

        // Recent Files submenu
        let recent_files: Vec<String> = app.settings.recent_files.iter().cloned().collect();
        if !recent_files.is_empty() {
            ui.menu_button("Recent Files", |ui| {
                for (i, path) in recent_files.iter().take(10).enumerate() {
                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());

                    let path_clone = path.clone();
                    if ui.button(format!("{}. {}", i + 1, file_name)).clicked() {
                        app.open_file(std::path::PathBuf::from(path_clone));
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Clear Recent Files").clicked() {
                    app.settings.recent_files.clear();
                    ui.close();
                }
            });
        }

        ui.separator();
        if ui.button("Save").clicked() {
            app.save_current_file();
            ui.close();
        }
        if ui.button("Save As...").clicked() {
            app.save_as_file_dialog();
            ui.close();
        }
        ui.separator();

        ui.menu_button("Encoding", |ui| {
            ui.label("Convert to:");
            if ui.button("UTF-8").clicked() {
                app.set_encoding(FileEncoding::Utf8);
                ui.close();
            }
            if ui.button("UTF-16 LE").clicked() {
                app.set_encoding(FileEncoding::Utf16Le);
                ui.close();
            }
            if ui.button("UTF-16 BE").clicked() {
                app.set_encoding(FileEncoding::Utf16Be);
                ui.close();
            }
        });

        ui.menu_button("Line Endings", |ui| {
            ui.label("Convert to:");
            if ui.button("LF (Unix)").clicked() {
                app.set_line_ending(LineEnding::Lf);
                ui.close();
            }
            if ui.button("CRLF (Windows)").clicked() {
                app.set_line_ending(LineEnding::Crlf);
                ui.close();
            }
            if ui.button("CR (Old Mac)").clicked() {
                app.set_line_ending(LineEnding::Cr);
                ui.close();
            }
        });

        ui.separator();
        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    })
    .response
}

pub fn edit_menu_button(ui: &mut Ui, app: &mut TextEditAppState) -> Response {
    ui.menu_button("Edit", |ui| {
        if ui.button("Undo").clicked() {
            app.undo();
            ui.close();
        }
        if ui.button("Redo").clicked() {
            app.redo();
            ui.close();
        }
        ui.separator();
        if ui.button("Cut").clicked() {
            app.cut();
            ui.close();
        }
        if ui.button("Copy").clicked() {
            app.copy();
            ui.close();
        }
        if ui.button("Paste").clicked() {
            app.paste();
            ui.close();
        }
        if ui.button("Select All").clicked() {
            app.select_all();
            ui.close();
        }
        ui.separator();
        if ui.button("Find...").clicked() {
            app.menu_state.show_find = true;
            ui.close();
        }
        if ui.button("Replace...").clicked() {
            app.menu_state.show_find = true;
            app.menu_state.show_replace = true;
            ui.close();
        }
        if ui.button("Go to Line...").clicked() {
            app.menu_state.show_goto = true;
            ui.close();
        }
    })
    .response
}

pub fn view_menu_button(ui: &mut Ui, app: &mut TextEditAppState) -> Response {
    ui.menu_button("View", |ui| {
        ui.toggle_value(&mut app.show_line_numbers, "Line Numbers");
        ui.toggle_value(&mut app.highlight_current_line, "Highlight Current Line");
        ui.toggle_value(&mut app.word_wrap, "Word Wrap");
        ui.toggle_value(&mut app.show_status_bar, "Status Bar");

        ui.separator();
        if ui.button("Zoom In").clicked() {
            app.zoom_in();
            ui.close();
        }
        if ui.button("Zoom Out").clicked() {
            app.zoom_out();
            ui.close();
        }
        if ui.button("Reset Zoom").clicked() {
            app.reset_zoom();
            ui.close();
        }
    })
    .response
}

pub fn help_menu_button(ui: &mut Ui) -> Response {
    ui.menu_button("Help", |ui| {
        if ui.button("About TextEdit").clicked() {
            egui::Window::new("About TextEdit")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.heading("TextEdit");
                    ui.label("Version 0.1.0");
                    ui.label("A cross-platform text editor");
                    ui.separator();
                    ui.label("Built with egui/eframe");
                });
            ui.close();
        }
    })
    .response
}

pub fn show_menu_bar(ui: &mut Ui, app: &mut TextEditAppState) {
    MenuBar::new().ui(ui, |ui| {
        file_menu_button(ui, app);
        edit_menu_button(ui, app);
        view_menu_button(ui, app);
        help_menu_button(ui);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::global_theme_preference_buttons(ui);
        });
    });
}
