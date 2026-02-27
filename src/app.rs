use crate::editor::{Document, FileEncoding, LineEnding, TextBuffer};
use crate::features::{FindReplace, Settings, Theme, UndoManager};
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
    pub find_replace: FindReplace,
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
            find_replace: FindReplace::default(),
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
        // After copy, delete the selection
        if let Some(doc) = self.get_current_doc() {
            let mut doc = doc.write();
            let text = doc.buffer.get_text();
            // For now, just mark as modified
            doc.buffer.set_modified(true);
        }
    }

    pub fn copy(&mut self) {
        if let Some(doc) = self.get_current_doc() {
            let doc = doc.read();
            let text = doc.buffer.get_text();

            #[cfg(not(target_arch = "wasm32"))]
            {
                use std::process::Command;
                let _ = Command::new("pbcopy").arg(&text).output();
            }
        }
    }

    pub fn paste(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("pbpaste").output() {
                let text = String::from_utf8_lossy(&output.stdout).to_string();
                if let Some(doc) = self.get_current_doc() {
                    let mut doc = doc.write();
                    let current_text = doc.buffer.get_text();
                    let new_text = format!("{}{}", current_text, text);
                    doc.buffer = TextBuffer::from_string(&new_text);
                    doc.buffer.set_modified(true);
                }
            }
        }
    }

    pub fn select_all(&mut self) {
        // Select all text - the text is already selected in the TextEdit widget
        // This is handled by egui's TextEdit internally
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
                    .desired_width(ui.available_width());

                let response = ui.add_sized(ui.available_size(), text_edit);

                // Update document if text changed
                if response.changed() {
                    if let Some(d) = self.state.get_current_doc() {
                        let mut d = d.write();
                        if d.buffer.get_text() != text {
                            d.buffer = TextBuffer::from_string(&text);
                        }
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

        // Show Find/Replace dialog
        if self.state.menu_state.show_find {
            self.show_find_replace_dialog(ctx);
        }

        // Show Go to Line dialog
        if self.state.menu_state.show_goto {
            self.show_goto_line_dialog(ctx);
        }

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

    fn show_find_replace_dialog(&mut self, ctx: &egui::Context) {
        let mut is_open = self.state.menu_state.show_find;
        let mut is_replace_mode = self.state.menu_state.show_replace;

        egui::Window::new("Find / Replace")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, -100.0])
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut self.state.find_replace.search_text);
                });

                if is_replace_mode {
                    ui.horizontal(|ui| {
                        ui.label("Replace:");
                        ui.text_edit_singleline(&mut self.state.find_replace.replace_text);
                    });
                }

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.state.find_replace.case_sensitive,
                        "Case sensitive",
                    );
                    ui.checkbox(&mut self.state.find_replace.whole_word, "Whole word");
                    ui.checkbox(&mut self.state.find_replace.regex, "Regex");
                });

                ui.horizontal(|ui| {
                    if ui.button("Find Next").clicked() {
                        self.find_next();
                    }

                    if is_replace_mode {
                        if ui.button("Replace").clicked() {
                            self.replace_next();
                        }
                        if ui.button("Replace All").clicked() {
                            self.replace_all();
                        }
                    }

                    ui.toggle_value(&mut is_replace_mode, "Replace");
                });

                ui.separator();

                // Show match count
                if let Some(doc) = self.state.get_current_doc() {
                    let doc = doc.read();
                    let text = doc.buffer.get_text();
                    let count = self.state.find_replace.count_matches(&text);
                    ui.label(format!("Matches: {}", count));
                }

                ui.separator();

                if ui.button("Close").clicked() {
                    self.state.menu_state.show_find = false;
                    self.state.menu_state.show_replace = false;
                }
            });

        self.state.menu_state.show_find = is_open;
        self.state.menu_state.show_replace = is_replace_mode;
    }

    fn show_goto_line_dialog(&mut self, ctx: &egui::Context) {
        let mut is_open = self.state.menu_state.show_goto;
        let mut line_number = String::new();

        egui::Window::new("Go to Line")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, -50.0])
            .collapsible(false)
            .resizable(false)
            .open(&mut is_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Line number:");
                    ui.text_edit_singleline(&mut line_number);
                });

                ui.horizontal(|ui| {
                    if ui.button("Go to").clicked() {
                        if let Ok(line) = line_number.parse::<usize>() {
                            self.goto_line(line.saturating_sub(1)); // Convert to 0-indexed
                            self.state.menu_state.show_goto = false;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.state.menu_state.show_goto = false;
                    }
                });
            });

        self.state.menu_state.show_goto = is_open;
    }

    fn find_next(&mut self) {
        if let Some(doc) = self.state.get_current_doc() {
            let doc = doc.read();
            let text = doc.buffer.get_text();
            let cursor_offset = 0; // Start from beginning for now

            if let Some((start, _end)) = self.state.find_replace.find_next(&text, cursor_offset) {
                log::info!("Found at position: {}", start);
                // TODO: Scroll to and highlight the match
            }
        }
    }

    fn replace_next(&mut self) {
        if let Some(doc) = self.state.get_current_doc() {
            let mut doc = doc.write();
            let text = doc.buffer.get_text();

            if let Some((start, end)) = self.state.find_replace.find_next(&text, 0) {
                let new_text = self.state.find_replace.replace_match(&text, (start, end));
                doc.buffer = TextBuffer::from_string(&new_text);
            }
        }
    }

    fn replace_all(&mut self) {
        if let Some(doc) = self.state.get_current_doc() {
            let mut doc = doc.write();
            let text = doc.buffer.get_text();
            let new_text = self.state.find_replace.replace(&text);
            doc.buffer = TextBuffer::from_string(&new_text);
        }
    }

    fn goto_line(&self, line: usize) {
        // Update cursor position to go to the specified line
        // This would require cursor state management
        log::info!("Going to line: {}", line + 1);
    }
}
