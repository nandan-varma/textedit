use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    CheckMenuItem, Menu, MenuEvent, MenuItem,
};
use winit::event_loop::EventLoopProxy;

pub mod actions {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MenuAction {
        New,
        Open,
        Save,
        SaveAs,
        Close,
        Quit,
        Undo,
        Redo,
        Cut,
        Copy,
        Paste,
        Delete,
        SelectAll,
        Find,
        FindNext,
        FindPrev,
        Replace,
        ToggleLineNumbers,
        ToggleStatusBar,
        About,
    }
}

pub use actions::MenuAction;

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
                    // ids are the lowercase strings we assign when creating items
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
                    _ => None,
                };
                if let Some(a) = action {
                    let _ = proxy.send_event(a);
                }
            }
        }
    }

    pub fn build(&mut self) -> &Menu {
        // App menu (macOS: becomes the app name menu)
        let app_menu = muda::Submenu::new("textedit", true);
        app_menu
            .append(&item_with_accel("about", "About textedit", None))
            .unwrap();
        app_menu
            .append(&muda::PredefinedMenuItem::separator())
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
            .append(&muda::PredefinedMenuItem::separator())
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
            .append(&muda::PredefinedMenuItem::separator())
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
            .append(&muda::PredefinedMenuItem::separator())
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
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        edit_menu
            .append(&item_with_accel(
                "select_all",
                "Select All",
                Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyA)),
            ))
            .unwrap();
        edit_menu
            .append(&muda::PredefinedMenuItem::separator())
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
        self.menu.append(&help_menu).unwrap();

        &self.menu
    }

    #[allow(dead_code)]
    pub fn menu(&self) -> &Menu {
        &self.menu
    }

    #[cfg(target_os = "macos")]
    pub fn attach_to_window(&self, _window: &winit::window::Window) {
        self.menu.init_for_nsapp();
    }

    #[cfg(target_os = "windows")]
    pub fn attach_to_window(&self, window: &winit::window::Window) {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let Ok(handle) = window.window_handle() {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                unsafe {
                    self.menu.init_for_hwnd(h.hwnd.get());
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn attach_to_window(&self, window: &winit::window::Window) {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if let Ok(handle) = window.window_handle() {
            if let RawWindowHandle::Xlib(h) = handle.as_raw() {
                unsafe {
                    let _ = self.menu.init_for_xlib(h.window as *mut _, None);
                }
            }
        }
    }
}

impl Default for MenuHandler {
    fn default() -> Self {
        Self::new()
    }
}

fn item(id: &str, label: &str) -> MenuItem {
    MenuItem::with_id(id, label, true, None)
}

fn item_with_accel(id: &str, label: &str, accelerator: Option<Accelerator>) -> MenuItem {
    MenuItem::with_id(id, label, true, accelerator)
}

fn check(id: &str, label: &str, checked: bool) -> CheckMenuItem {
    CheckMenuItem::with_id(id, label, true, checked, None)
}
