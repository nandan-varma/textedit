use crate::editor::{Document, FileEncoding, LineEnding, TextBuffer};
use crate::features::{Settings, Theme, UndoManager};
use crate::platform;
use crate::ui::{MenuBarState, TabBar};
use egui::{Event, Key, TextEdit};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TextEditAppState {
    pub tab_bar: TabBar,
    pub settings: Settings,
    pub theme: Theme,
    pub menu_state: MenuBarState,
    pub show_status_bar: bool,
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub word_wrap: bool,
    #[serde(skip)]
    undo_manager: UndoManager,
    #[serde(skip)]
    zoom_level: f32,
}

impl Default for TextEditAppState {
    fn default() -> Self {
        let settings = Settings::default();
        let tab_bar = TabBar::new();

        Self {
            tab_bar,
            settings,
            theme: Theme::default(),
            menu_state: MenuBarState::default(),
            show_status_bar: true,
            show_line_numbers: true,
            highlight_current_line: true,
            word_wrap: false,
            undo_manager: UndoManager::new(100),
            zoom_level: 1.0,
        }
    }
}

impl TextEditAppState {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = Self::default();

        let doc = Document::new();
        state.tab_bar.add_tab(doc);

        let _ = platform::create_config_dirs();

        state
    }

    pub fn new_file(&mut self) {
        let doc = Document::new();
        self.tab_bar.add_tab(doc);
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(path) = platform::show_open_dialog() {
            self.open_file(path);
        }
    }

    pub fn open_file(&mut self, path: std::path::PathBuf) {
        let mut doc = Document::new();

        match doc.load_from_file(&path) {
            Ok(()) => {
                self.tab_bar.add_tab(doc);
                self.settings
                    .add_recent_file(path.to_string_lossy().to_string());
            }
            Err(e) => {
                log::error!("Failed to open file: {}", e);
            }
        }
    }

    pub fn save_current_file(&mut self) {
        if let Some(idx) = self.tab_bar.get_active_index() {
            if let Some(tab) = self.tab_bar.get_tab_mut(idx) {
                let mut doc = tab.document.write();

                let should_save_as = doc.file_path.is_none();
                if should_save_as {
                    drop(doc);
                    self.save_as_file_dialog();
                } else if let Some(ref path) = doc.file_path.clone() {
                    if let Err(e) = doc.save_to_file(path) {
                        log::error!("Failed to save file: {}", e);
                    }
                }
            }
        }
    }

    pub fn save_as_file_dialog(&mut self) {
        if let Some(idx) = self.tab_bar.get_active_index() {
            if let Some(tab) = self.tab_bar.get_tab(idx) {
                let doc = tab.document.read();
                let default_name = doc.file_name();
                drop(doc);

                if let Some(path) = platform::show_save_dialog(&default_name) {
                    if let Some(tab) = self.tab_bar.get_tab_mut(idx) {
                        let mut doc = tab.document.write();
                        if let Err(e) = doc.save_to_file(&path) {
                            log::error!("Failed to save file: {}", e);
                        } else {
                            self.settings
                                .add_recent_file(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn close_current_tab(&mut self) {
        if let Some(idx) = self.tab_bar.get_active_index() {
            self.tab_bar.close_tab(idx);

            if self.tab_bar.is_empty() {
                self.new_file();
            }
        }
    }

    pub fn set_encoding(&mut self, encoding: FileEncoding) {
        if let Some(idx) = self.tab_bar.get_active_index() {
            if let Some(tab) = self.tab_bar.get_tab_mut(idx) {
                let mut doc = tab.document.write();
                doc.encoding = encoding;
                doc.buffer.set_modified(true);
            }
        }
    }

    pub fn set_line_ending(&mut self, ending: LineEnding) {
        if let Some(idx) = self.tab_bar.get_active_index() {
            if let Some(tab) = self.tab_bar.get_tab_mut(idx) {
                let mut doc = tab.document.write();
                doc.convert_line_endings(ending);
            }
        }
    }

    pub fn undo(&mut self) {
        if let Some(_state) = self.undo_manager.undo() {
            // Apply undo to current document
        }
    }

    pub fn redo(&mut self) {
        if let Some(_state) = self.undo_manager.redo() {
            // Apply redo to current document
        }
    }

    pub fn cut(&mut self) {
        self.copy();
    }

    pub fn copy(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::process::Command;
            let _ = Command::new("pbcopy").output();
        }
    }

    pub fn paste(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::process::Command;
            let _ = Command::new("pbpaste").output();
        }
    }

    pub fn select_all(&mut self) {
        // Select all text in current document
    }

    pub fn get_current_doc(&self) -> Option<Arc<RwLock<Document>>> {
        self.tab_bar.get_active_document()
    }

    pub fn get_status_info(&self) -> (String, String, String) {
        if let Some(doc) = self.get_current_doc() {
            let doc = doc.read();
            (
                "Ln 1".to_string(),
                "Col 1".to_string(),
                format!("{} | Saved", doc.encoding.name()),
            )
        } else {
            (
                "Ln 1".to_string(),
                "Col 1".to_string(),
                "UTF-8 | Saved".to_string(),
            )
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + 0.1).min(3.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - 0.1).max(0.25);
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }
}

pub struct TextEditApp {
    state: TextEditAppState,
}

impl TextEditApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = TextEditAppState::new(cc);

        state.theme.apply_to_egui(&cc.egui_ctx);

        Self { state }
    }
}

impl eframe::App for TextEditApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            crate::ui::show_menu_bar(ui, &mut self.state);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(doc) = self.state.get_current_doc() {
                let mut text = doc.read().buffer.get_text();

                let mut text_edit = TextEdit::multiline(&mut text)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(20);

                ui.add(text_edit);

                if let Some(d) = self.state.get_current_doc() {
                    let mut d = d.write();
                    if d.buffer.get_text() != text {
                        d.buffer = TextBuffer::from_string(&text);
                    }
                }
            } else {
                ui.label("No document open");
            }
        });

        if self.state.show_status_bar {
            let (line, col, info) = self.state.get_status_info();
            let zoom_str = format!("{}%", (self.state.zoom_level * 100.0) as i32);

            egui::TopBottomPanel::bottom("status_bar")
                .exact_height(24.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 16.0;
                        ui.label(line);
                        ui.label(col);
                        ui.separator();
                        ui.label(info);
                        ui.separator();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(zoom_str);
                        });
                    });
                });
        }

        // Find dialog disabled for now
        // if self.state.menu_state.show_find {
        //     self.show_find_dialog(ctx);
        // }

        self.handle_shortcuts(ctx);
    }
}

impl TextEditApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            let ctrl = input.modifiers.ctrl || input.modifiers.command;

            for event in &input.events {
                if let Event::Key { key, modifiers, .. } = event {
                    if ctrl {
                        match key {
                            Key::N => {
                                self.state.new_file();
                            }
                            Key::O => {
                                self.state.open_file_dialog();
                            }
                            Key::S => {
                                if modifiers.shift {
                                    self.state.save_as_file_dialog();
                                } else {
                                    self.state.save_current_file();
                                }
                            }
                            Key::W => {
                                self.state.close_current_tab();
                            }
                            Key::F => {
                                self.state.menu_state.show_find = true;
                            }
                            Key::Z => {
                                if modifiers.shift {
                                    self.state.redo();
                                } else {
                                    self.state.undo();
                                }
                            }
                            Key::Y => {
                                self.state.redo();
                            }
                            Key::A => {
                                self.state.select_all();
                            }
                            Key::Equals => {
                                self.state.zoom_in();
                            }
                            Key::Minus => {
                                self.state.zoom_out();
                            }
                            Key::Num0 => {
                                self.state.reset_zoom();
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}
