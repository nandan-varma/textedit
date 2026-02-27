use std::path::PathBuf;

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("textedit"))
}

pub fn get_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("textedit"))
}

pub fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("textedit"))
}

pub fn create_config_dirs() -> Result<(), std::io::Error> {
    if let Some(config) = get_config_dir() {
        std::fs::create_dir_all(config)?;
    }
    if let Some(data) = get_data_dir() {
        std::fs::create_dir_all(data)?;
    }
    if let Some(cache) = get_cache_dir() {
        std::fs::create_dir_all(cache)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_open_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(
            "Text Files",
            &[
                "txt", "md", "rs", "py", "js", "ts", "html", "css", "json", "toml", "yaml", "yml",
            ],
        )
        .add_filter("All Files", &["*"])
        .pick_file()
}

#[cfg(target_arch = "wasm32")]
pub fn show_open_dialog() -> Option<PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn show_save_dialog(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(
            "Text Files",
            &[
                "txt", "md", "rs", "py", "js", "ts", "html", "css", "json", "toml", "yaml", "yml",
            ],
        )
        .add_filter("All Files", &["*"])
        .set_file_name(default_name)
        .save_file()
}

#[cfg(target_arch = "wasm32")]
pub fn show_save_dialog(_default_name: &str) -> Option<PathBuf> {
    None
}
