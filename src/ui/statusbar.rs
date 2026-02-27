use egui::{Response, TopBottomPanel, Ui, Widget};

pub struct StatusBar {
    pub visible: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self { visible: true }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StatusBarWidget {
    line_info: String,
    col_info: String,
    encoding_info: String,
    zoom_level: String,
}

impl StatusBarWidget {
    pub fn new(
        line_info: String,
        col_info: String,
        encoding_info: String,
        zoom_level: String,
    ) -> Self {
        Self {
            line_info,
            col_info,
            encoding_info,
            zoom_level,
        }
    }
}

impl Widget for StatusBarWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show_separator_line(true)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;

                    ui.label(&self.line_info);
                    ui.label(&self.col_info);

                    ui.separator();

                    ui.label(&self.encoding_info);

                    ui.separator();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(&self.zoom_level);
                    });
                });
            })
            .response
    }
}
