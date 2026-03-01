//! Event routing system for UI widgets
//!
//! This module provides a bridge between winit window events and the widget system,
//! converting raw events into widget events and routing them appropriately.

use crate::ports::clipboard_port::Clipboard;
use crate::ui::primitives::{Point, Rect};
use crate::ui::widget::{EventContext, MouseButton, WidgetAction, WidgetContainer, WidgetEvent};
use winit::event::{ElementState, MouseButton as WinitMouseButton};
use winit::keyboard::{KeyCode, ModifiersState};

/// Converts winit mouse button to widget mouse button
fn convert_mouse_button(button: WinitMouseButton) -> Option<MouseButton> {
    match button {
        WinitMouseButton::Left => Some(MouseButton::Left),
        WinitMouseButton::Right => Some(MouseButton::Right),
        WinitMouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}

/// Event router that dispatches events to widgets
pub struct EventRouter {
    /// Current mouse position
    mouse_position: Point,
    /// Current modifier keys state
    modifiers: ModifiersState,
    /// Whether the left mouse button is pressed
    left_pressed: bool,
}

impl EventRouter {
    pub fn new() -> Self {
        Self {
            mouse_position: Point::new(0.0, 0.0),
            modifiers: ModifiersState::empty(),
            left_pressed: false,
        }
    }

    /// Update modifier key state
    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    /// Get current modifiers
    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    /// Update mouse position
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Point::new(x, y);
    }

    /// Get current mouse position
    pub fn mouse_position(&self) -> Point {
        self.mouse_position
    }

    /// Check if Ctrl (or Cmd on macOS) is pressed
    pub fn ctrl_or_cmd(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.modifiers.super_key()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.modifiers.control_key()
        }
    }

    /// Check if Shift is pressed
    pub fn shift(&self) -> bool {
        self.modifiers.shift_key()
    }

    /// Route a mouse button event to widgets
    pub fn route_mouse_button(
        &mut self,
        button: WinitMouseButton,
        state: ElementState,
        container: &mut WidgetContainer,
        clipboard: &mut dyn Clipboard,
    ) -> (bool, WidgetAction) {
        let widget_button = match convert_mouse_button(button) {
            Some(b) => b,
            None => return (false, WidgetAction::None),
        };

        if button == WinitMouseButton::Left {
            self.left_pressed = state == ElementState::Pressed;
        }

        let event = match state {
            ElementState::Pressed => WidgetEvent::MousePress {
                position: self.mouse_position,
                button: widget_button,
            },
            ElementState::Released => WidgetEvent::MouseRelease {
                position: self.mouse_position,
                button: widget_button,
            },
        };

        let mut ctx = EventContext::new(clipboard, self.modifiers);
        container.handle_event(&event, &mut ctx)
    }

    /// Route a mouse move event to widgets
    pub fn route_mouse_move(
        &mut self,
        x: f32,
        y: f32,
        container: &mut WidgetContainer,
        clipboard: &mut dyn Clipboard,
    ) -> (bool, WidgetAction) {
        self.mouse_position = Point::new(x, y);

        let event = WidgetEvent::MouseMove {
            position: self.mouse_position,
        };

        let mut ctx = EventContext::new(clipboard, self.modifiers);
        container.handle_event(&event, &mut ctx)
    }

    /// Route a key press event to widgets
    pub fn route_key_press(
        &mut self,
        key: KeyCode,
        container: &mut WidgetContainer,
        clipboard: &mut dyn Clipboard,
    ) -> (bool, WidgetAction) {
        let event = WidgetEvent::KeyPress {
            key,
            modifiers: self.modifiers,
        };

        let mut ctx = EventContext::new(clipboard, self.modifiers);
        container.handle_event(&event, &mut ctx)
    }

    /// Check if a point is within a rectangle
    pub fn point_in_rect(&self, rect: Rect) -> bool {
        rect.contains(self.mouse_position)
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_router_new() {
        let router = EventRouter::new();
        assert_eq!(router.mouse_position.x, 0.0);
        assert_eq!(router.mouse_position.y, 0.0);
        assert!(!router.left_pressed);
    }

    #[test]
    fn test_mouse_position() {
        let mut router = EventRouter::new();
        router.set_mouse_position(100.0, 200.0);
        assert_eq!(router.mouse_position().x, 100.0);
        assert_eq!(router.mouse_position().y, 200.0);
    }

    #[test]
    fn test_modifiers() {
        let mut router = EventRouter::new();
        assert!(!router.ctrl_or_cmd());
        assert!(!router.shift());

        router.set_modifiers(ModifiersState::SHIFT);
        assert!(router.shift());
        assert!(!router.ctrl_or_cmd());
    }

    #[test]
    fn test_point_in_rect() {
        let mut router = EventRouter::new();
        router.set_mouse_position(50.0, 50.0);

        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(router.point_in_rect(rect));

        let outside_rect = Rect::new(200.0, 200.0, 100.0, 100.0);
        assert!(!router.point_in_rect(outside_rect));
    }
}
