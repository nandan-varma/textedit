mod app;
mod config;
mod editor;
mod file;
mod menu;
mod renderer;
mod state;
mod ui;

use app::App;
use menu::{MenuAction, MenuHandler};
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::<MenuAction>::with_user_event()
        .build()
        .unwrap();

    // Create proxy for menu events
    let proxy = event_loop.create_proxy();
    
    // Create and initialize menu handler
    let mut menu_handler = MenuHandler::new();
    menu_handler.build();
    
    // Initialize menu system
    menu_handler.init(proxy);
    
    let mut app = App::new().with_menu(menu_handler);

    let _ = event_loop.run_app(&mut app);
}
