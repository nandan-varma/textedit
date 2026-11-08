mod application;
mod config;
mod domain;
mod error;
mod infrastructure;
mod interface;
mod menu;
mod ports;
mod renderer;
mod state;
mod syntax;
pub mod themes;
mod ui;

use interface::App;
use menu::MenuHandler;
use winit::event_loop::EventLoop;

#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;

fn main() {
    env_logger::init();

    // Build event loop with macOS-specific settings
    #[cfg(target_os = "macos")]
    let event_loop = {
        match EventLoop::<menu::MenuAction>::with_user_event()
            .with_default_menu(false) // We use our own menu
            .build()
        {
            Ok(el) => el,
            Err(e) => {
                eprintln!("Failed to create event loop: {}", e);
                std::process::exit(1);
            }
        }
    };

    #[cfg(not(target_os = "macos"))]
    let event_loop = match EventLoop::<menu::MenuAction>::with_user_event().build() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {}", e);
            std::process::exit(1);
        }
    };

    let proxy = event_loop.create_proxy();

    let mut menu_handler = MenuHandler::new();
    menu_handler.build();

    menu_handler.init(proxy);

    let mut app = App::new().with_menu(menu_handler);

    let _ = event_loop.run_app(&mut app);
}
