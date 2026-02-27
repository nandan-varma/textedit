use crate::editor::{FileEncoding, LineEnding};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub word_wrap: bool,
    pub show_status_bar: bool,
    pub auto_save: bool,
    pub auto_save_interval_secs: u32,
    pub tab_size: usize,
    pub use_spaces_for_tabs: bool,
    pub default_encoding: FileEncoding,
    pub default_line_ending: LineEnding,
    pub theme: String,
    pub font_size: f32,
    pub recent_files: Vec<String>,
    pub max_recent_files: usize,
    pub zoom_level: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_line_numbers: true,
            highlight_current_line: true,
            word_wrap: false,
            show_status_bar: true,
            auto_save: false,
            auto_save_interval_secs: 60,
            tab_size: 4,
            use_spaces_for_tabs: true,
            default_encoding: FileEncoding::Utf8,
            default_line_ending: LineEnding::Lf,
            theme: "Dark".to_string(),
            font_size: 14.0,
            recent_files: Vec::new(),
            max_recent_files: 10,
            zoom_level: 1.0,
        }
    }
}

impl Settings {
    pub fn add_recent_file(&mut self, path: String) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);

        if self.recent_files.len() > self.max_recent_files {
            self.recent_files.truncate(self.max_recent_files);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert!(settings.show_line_numbers);
        assert!(settings.highlight_current_line);
        assert!(!settings.word_wrap);
        assert_eq!(settings.tab_size, 4);
        assert_eq!(settings.zoom_level, 1.0);
    }

    #[test]
    fn test_add_recent_file() {
        let mut settings = Settings::default();

        settings.add_recent_file("/path/to/file1.txt".to_string());
        settings.add_recent_file("/path/to/file2.txt".to_string());
        settings.add_recent_file("/path/to/file1.txt".to_string());

        assert_eq!(settings.recent_files.len(), 2);
        assert_eq!(settings.recent_files[0], "/path/to/file1.txt");
    }

    #[test]
    fn test_max_recent_files() {
        let mut settings = Settings::default();
        settings.max_recent_files = 3;

        for i in 0..5 {
            settings.add_recent_file(format!("/path/to/file{}.txt", i));
        }

        assert_eq!(settings.recent_files.len(), 3);
    }

    #[test]
    fn test_zoom() {
        let mut settings = Settings::default();

        settings.zoom_in();
        assert!((settings.zoom_level - 1.1).abs() < 0.01);

        for _ in 0..20 {
            settings.zoom_in();
        }
        assert!(settings.zoom_level <= 3.0);

        for _ in 0..20 {
            settings.zoom_out();
        }
        assert!(settings.zoom_level >= 0.25);

        settings.reset_zoom();
        assert_eq!(settings.zoom_level, 1.0);
    }
}
