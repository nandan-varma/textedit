// Platform-specific menu attachment
#[cfg(target_os = "macos")]
pub fn attach_to_window(menu: &muda::Menu, _window: &winit::window::Window) {
    menu.init_for_nsapp();
}

#[cfg(target_os = "windows")]
pub fn attach_to_window(menu: &muda::Menu, window: &winit::window::Window) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(h) = handle.as_raw() {
            unsafe {
                menu.init_for_hwnd(h.hwnd.get());
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn attach_to_window(_menu: &muda::Menu, _window: &winit::window::Window) {
    // ponytail: muda 0.17 dropped init_for_xlib; GTK window not accessible via winit
}
