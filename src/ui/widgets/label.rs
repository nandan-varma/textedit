//! Label widget - displays static text

use crate::ui::primitives::{z_index, Color, Point, Primitive, Rect, RenderList};
use crate::ui::widget::{EventContext, RenderContext, Widget, WidgetAction, WidgetEvent};

/// A simple text label widget
pub struct Label {
    text: String,
    color: Option<Color>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Widget for Label {
    fn render(&self, bounds: Rect, ctx: &mut RenderContext) -> RenderList {
        let mut list = RenderList::new();

        let color = self.color.unwrap_or(ctx.colors.text_color);

        list.push(Primitive::text(
            &self.text,
            Point::new(bounds.x, bounds.y),
            color,
            z_index::MODAL_BUTTON_TEXT,
        ));

        list
    }

    fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _bounds: Rect,
        _ctx: &mut EventContext,
    ) -> (bool, WidgetAction) {
        // Labels don't handle events
        (false, WidgetAction::None)
    }

    fn preferred_size(&self, ctx: &mut RenderContext) -> (f32, f32) {
        let width = ctx.text_width(&self.text);
        let height = ctx.line_height;
        (width, height)
    }

    fn is_focusable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_new() {
        let label = Label::new("Hello");
        assert_eq!(label.text(), "Hello");
    }

    #[test]
    fn test_label_set_text() {
        let mut label = Label::new("Hello");
        label.set_text("World");
        assert_eq!(label.text(), "World");
    }

    #[test]
    fn test_label_not_focusable() {
        let label = Label::new("Test");
        assert!(!label.is_focusable());
    }
}
