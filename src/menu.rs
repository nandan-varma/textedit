use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use winit::event_loop::EventLoopProxy;

pub mod actions {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MenuAction {
        New,
        Open,
        Save,
        SaveAs,
        Close,
        Undo,
        Redo,
        Cut,
        Copy,
        Paste,
        SelectAll,
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
        let proxy_clone = proxy.clone();
        self.proxy = Some(proxy);
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let id = event.id().as_ref();
            let action = match id {
                "New" => Some(MenuAction::New),
                "Open" => Some(MenuAction::Open),
                "Save" => Some(MenuAction::Save),
                "Save As" => Some(MenuAction::SaveAs),
                "Close" => Some(MenuAction::Close),
                "Undo" => Some(MenuAction::Undo),
                "Redo" => Some(MenuAction::Redo),
                "Cut" => Some(MenuAction::Cut),
                "Copy" => Some(MenuAction::Copy),
                "Paste" => Some(MenuAction::Paste),
                "Select All" => Some(MenuAction::SelectAll),
                "Line Numbers" => Some(MenuAction::ToggleLineNumbers),
                "Status Bar" => Some(MenuAction::ToggleStatusBar),
                "About textedit" => Some(MenuAction::About),
                "Quit textedit" => Some(MenuAction::Close), // Close acts as quit
                _ => None,
            };
            if let Some(a) = action {
                let _ = proxy_clone.send_event(a);
            }
        }));
    }

    pub fn build(&mut self) -> &Menu {
        // App menu (macOS: becomes the app name menu)
        let app_menu = muda::Submenu::new("textedit", true);
        app_menu.append(&item("about", "About textedit")).unwrap();
        app_menu
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        app_menu.append(&item("quit", "Quit textedit")).unwrap();

        // File menu
        let file_menu = muda::Submenu::new("File", true);
        file_menu.append(&item("new", "New")).unwrap();
        file_menu.append(&item("open", "Open")).unwrap();
        file_menu
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        file_menu.append(&item("save", "Save")).unwrap();
        file_menu.append(&item("save_as", "Save As")).unwrap();
        file_menu
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        file_menu.append(&item("close", "Close")).unwrap();

        // Edit menu
        let edit_menu = muda::Submenu::new("Edit", true);
        edit_menu.append(&item("undo", "Undo")).unwrap();
        edit_menu.append(&item("redo", "Redo")).unwrap();
        edit_menu
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        edit_menu.append(&item("cut", "Cut")).unwrap();
        edit_menu.append(&item("copy", "Copy")).unwrap();
        edit_menu.append(&item("paste", "Paste")).unwrap();
        edit_menu
            .append(&muda::PredefinedMenuItem::separator())
            .unwrap();
        edit_menu.append(&item("select_all", "Select All")).unwrap();

        // View menu
        let view_menu = muda::Submenu::new("View", true);
        view_menu
            .append(&check("toggle_line_numbers", "Line Numbers", true))
            .unwrap();
        view_menu
            .append(&check("toggle_status_bar", "Status Bar", true))
            .unwrap();

        // Help menu
        let help_menu = muda::Submenu::new("Help", true);
        help_menu.append(&item("help", "textedit Help")).unwrap();

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

fn item(_id: &str, label: &str) -> MenuItem {
    MenuItem::new(label, true, None)
}

fn check(_id: &str, label: &str, checked: bool) -> CheckMenuItem {
    CheckMenuItem::new(label, checked, true, None)
}
