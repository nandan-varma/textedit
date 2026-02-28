use crate::editor::Editor;
use crate::editor::KeyboardController;
use crate::menu::MenuHandler;
use crate::state::State;
use std::time::Instant;
use winit::keyboard::ModifiersState;

#[derive(Clone, Copy, PartialEq)]
pub enum MouseButtonState {
    Released,
    Pressed,
}

pub struct App {
    pub state: Option<State>,
    pub editor: Option<Editor>,
    pub keyboard: KeyboardController,
    pub menu_handler: Option<MenuHandler>,
    pub modifiers: ModifiersState,
    pub mouse_button_state: MouseButtonState,
    pub mouse_position: Option<(f64, f64)>,
    pub last_click_time: Option<Instant>,
    pub last_click_position: Option<(f64, f64)>,
    pub is_dragging: bool,
    pub click_count: u8,
}
