use textedit::interface::App;
use textedit::menu::{MenuAction, MenuHandler};
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = match EventLoop::<MenuAction>::with_user_event().build() {
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
