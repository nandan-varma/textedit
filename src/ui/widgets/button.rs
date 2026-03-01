//! Button widget - clickable button with text

use crate::ui::primitives::{z_index, Color, Point, Primitive, Rect, RenderList};
use crate::ui::widget::{
    EventContext, MouseButton, RenderContext, Widget, WidgetAction, WidgetEvent,
};

/// A clickable button widget
pub struct Button {
    label: String,
    action: WidgetAction,
    pressed: bool,
    hovered: bool,
    bg_color: Option<Color>,
    text_color: Option<Color>,
    corner_radius: f32,
}

impl Button {
    pub fn new(label: impl Into<String>, action: WidgetAction) -> Self {
        Self {
            label: label.into(),
            action,
            pressed: false,
            hovered: false,
            bg_color: None,
            text_color: None,
            corner_radius: 4.0,
        }
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
}

impl Widget for Button {
    fn render(&self, bounds: Rect, ctx: &mut RenderContext) -> RenderList {
        let mut list = RenderList::new();

        // Determine colors based on state
        let bg_color = if self.pressed {
            // Darken when pressed
            let base = self.bg_color.unwrap_or(ctx.colors.button_background);
            [base[0] * 0.8, base[1] * 0.8, base[2] * 0.8, base[3]]
        } else if self.hovered {
            // Use hover color or lighten base
            self.bg_color
                .map(|base| {
                    [
                        (base[0] * 1.2).min(1.0),
                        (base[1] * 1.2).min(1.0),
                        (base[2] * 1.2).min(1.0),
                        base[3],
                    ]
                })
                .unwrap_or(ctx.colors.button_hover)
        } else {
            self.bg_color.unwrap_or(ctx.colors.button_background)
        };

        let text_color = self.text_color.unwrap_or(ctx.colors.text_color);

        // Background
        if self.corner_radius > 0.0 {
            list.push(Primitive::rounded_rect(
                bounds,
                bg_color,
                self.corner_radius,
                z_index::MODAL_BUTTON,
            ));
        } else {
            list.push(Primitive::rect(bounds, bg_color, z_index::MODAL_BUTTON));
        }

        // Text (centered)
        let text_width = ctx.text_width(&self.label);
        let text_x = bounds.x + (bounds.width - text_width) / 2.0;
        let text_y = bounds.y + (bounds.height - ctx.line_height) / 2.0;

        list.push(Primitive::text(
            &self.label,
            Point::new(text_x, text_y),
            text_color,
            z_index::MODAL_BUTTON_TEXT,
        ));

        list
    }

    fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        _ctx: &mut EventContext,
    ) -> (bool, WidgetAction) {
        match event {
            WidgetEvent::MousePress {
                position,
                button: MouseButton::Left,
            } => {
                if bounds.contains(*position) {
                    self.pressed = true;
                    return (true, WidgetAction::Redraw);
                }
            }
            WidgetEvent::MouseRelease {
                position,
                button: MouseButton::Left,
            } => {
                if self.pressed {
                    self.pressed = false;
                    if bounds.contains(*position) {
                        return (true, self.action.clone());
                    }
                    return (true, WidgetAction::Redraw);
                }
            }
            WidgetEvent::MouseMove { position } => {
                let was_hovered = self.hovered;
                self.hovered = bounds.contains(*position);
                if was_hovered != self.hovered {
                    return (false, WidgetAction::Redraw);
                }
            }
            _ => {}
        }

        (false, WidgetAction::None)
    }

    fn preferred_size(&self, ctx: &mut RenderContext) -> (f32, f32) {
        let text_width = ctx.text_width(&self.label);
        let padding = 16.0 * ctx.scale_factor;
        (text_width + padding * 2.0, ctx.line_height + padding)
    }

    fn is_focusable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_new() {
        let button = Button::new("Click Me", WidgetAction::Close);
        assert_eq!(button.label(), "Click Me");
        assert!(!button.is_pressed());
        assert!(!button.is_hovered());
    }

    #[test]
    fn test_button_focusable() {
        let button = Button::new("Test", WidgetAction::None);
        assert!(button.is_focusable());
    }
}
