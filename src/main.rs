use winit::event_loop::EventLoop;

mod app;
mod config;
mod editor;
mod file;
mod renderer;
mod state;
mod ui;

use app::App;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();

    let _ = event_loop.run_app(&mut app);
}
