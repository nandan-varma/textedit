use muda::{Menu, MenuEvent, accelerator::{Accelerator, Code, Modifiers}, PredefinedMenuItem};
use winit::event_loop::EventLoopProxy;
use crate::menu::actions::MenuAction;
// use crate::menu::helpers::{item_with_accel, check};

pub struct MenuHandler {
    menu: Menu,
    proxy: Option<EventLoopProxy<MenuAction>>,
}

impl MenuHandler {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(),
            proxy: None,
        }
    }

    pub fn init(&mut self, proxy: EventLoopProxy<MenuAction>) {
        self.proxy = Some(proxy);
    }

    pub fn poll_menu_events(&self) {
        if let Some(proxy) = &self.proxy {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                let id = event.id().as_ref();
                let action = match id {
                    "new" => Some(MenuAction::New),
                    "open" => Some(MenuAction::Open),
                    "save" => Some(MenuAction::Save),
                    "save_as" => Some(MenuAction::SaveAs),
                    "close" => Some(MenuAction::Close),
                    "quit" => Some(MenuAction::Quit),
                    "undo" => Some(MenuAction::Undo),
                    "redo" => Some(MenuAction::Redo),
                    "cut" => Some(MenuAction::Cut),
                    "copy" => Some(MenuAction::Copy),
                    "paste" => Some(MenuAction::Paste),
                    "delete" => Some(MenuAction::Delete),
                    "select_all" => Some(MenuAction::SelectAll),
                    "find" => Some(MenuAction::Find),
                    "find_next" => Some(MenuAction::FindNext),
                    "find_prev" => Some(MenuAction::FindPrev),
                    "replace" => Some(MenuAction::Replace),
                    "toggle_line_numbers" => Some(MenuAction::ToggleLineNumbers),
                    "toggle_status_bar" => Some(MenuAction::ToggleStatusBar),
                    "about" => Some(MenuAction::About),
                    id if id.starts_with("theme_ui_") => {
                        let theme = id.trim_start_matches("theme_ui_").to_string();
                        Some(MenuAction::SetEditorTheme(theme))
                    }
                    id if id.starts_with("theme_") => {
                        let theme = id.trim_start_matches("theme_").to_string();
                        Some(MenuAction::SetTheme(theme))
                    }
                    _ => None,
                };
                if let Some(a) = action {
                        let _ = proxy.send_event(a);
                }
            }
        }
    }

    pub fn build(&mut self) -> &Menu {
        use crate::menu::helpers::item_with_accel;
        use crate::menu::helpers::check;
        // App menu (macOS: becomes the app name menu)
        let app_menu = muda::Submenu::new("textedit", true);
        app_menu
            .append(&item_with_accel("about", "About textedit", None))
            .unwrap();
        app_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        app_menu
            .append(&item_with_accel(
                "quit",
                "Quit textedit",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyQ)),
            ))
            .unwrap();

        // File menu
        let file_menu = muda::Submenu::new("File", true);
        file_menu
            .append(&item_with_accel(
                "new",
                "New",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN)),
            ))
            .unwrap();
        file_menu
            .append(&item_with_accel(
                "open",
                "Open",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyO)),
            ))
            .unwrap();
        file_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        file_menu
            .append(&item_with_accel(
                "save",
                "Save",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyS)),
            ))
            .unwrap();
        file_menu
            .append(&item_with_accel(
                "save_as",
                "Save As",
                Some(Accelerator::new(
                    Some(Modifiers::SUPER | Modifiers::SHIFT),
                    Code::KeyS,
                )),
            ))
            .unwrap();
        file_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        file_menu
            .append(&item_with_accel(
                "close",
                "Close",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyW)),
            ))
            .unwrap();

        // Edit menu
        let edit_menu = muda::Submenu::new("Edit", true);
        edit_menu
            .append(&item_with_accel(
                "undo",
                "Undo",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyZ)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "redo",
                "Redo",
                Some(Accelerator::new(
                    Some(Modifiers::SUPER | Modifiers::SHIFT),
                    Code::KeyZ,
                )),
            ))
            .unwrap();
        edit_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "cut",
                "Cut",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyX)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "copy",
                "Copy",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyC)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "paste",
                "Paste",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyV)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "delete",
                "Delete",
                Some(Accelerator::new(None, Code::Delete)),
            ))
            .unwrap();
        edit_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "select_all",
                "Select All",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyA)),
            ))
            .unwrap();
        edit_menu
            .append(&PredefinedMenuItem::separator())
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "find",
                "Find…",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyF)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "find_next",
                "Find Next",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyG)),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "find_prev",
                "Find Previous",
                Some(Accelerator::new(
                    Some(Modifiers::SUPER | Modifiers::SHIFT),
                    Code::KeyG,
                )),
            ))
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "replace",
                "Replace…",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyH)),
            ))
            .unwrap();

        // View menu
        let view_menu = muda::Submenu::new("View", true);
        view_menu
            .append(&check("toggle_line_numbers", "Show Line Numbers", true))
            .unwrap();
        view_menu
            .append(&check("toggle_status_bar", "Show Status Bar", true))
            .unwrap();

        // Theme menu (UI themes)
        let theme_menu = muda::Submenu::new("Theme", true);
        let ui_themes = [
            ("Dracula", "theme_ui_Dracula"),
            ("Solarized Dark", "theme_ui_SolarizedDark"),
            ("One Dark", "theme_ui_OneDark"),
            ("Gruvbox Dark", "theme_ui_GruvboxDark"),
            ("Light", "theme_ui_Light"),
        ];
        for (label, id) in &ui_themes {
            theme_menu
                .append(&item_with_accel(id, label, None))
                .unwrap();
        }

        // Help menu
        let help_menu = muda::Submenu::new("Help", true);
        help_menu
            .append(&item_with_accel(
                "help",
                "textedit Help",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::F1)),
            ))
            .unwrap();

        self.menu.append(&app_menu).unwrap();
        self.menu.append(&file_menu).unwrap();
        self.menu.append(&edit_menu).unwrap();
        self.menu.append(&view_menu).unwrap();
        self.menu.append(&theme_menu).unwrap();
        self.menu.append(&help_menu).unwrap();

        &self.menu
    }

    #[allow(dead_code)]
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    pub fn attach_to_window(&self, window: &winit::window::Window) {
        crate::menu::platform::attach_to_window(&self.menu, window);
    }
}

impl Default for MenuHandler {
    fn default() -> Self {
        Self::new()
    }
}
