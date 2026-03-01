//! Core primitive types for the rendering system.
//!
//! All UI elements are ultimately rendered as primitives. This provides
//! a clean abstraction between UI logic and GPU rendering.

/// A 2D point in pixel coordinates
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A rectangle in pixel coordinates
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(top_left: Point, bottom_right: Point) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: bottom_right.x - top_left.x,
            height: bottom_right.y - top_left.y,
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn inset(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - amount * 2.0).max(0.0),
            height: (self.height - amount * 2.0).max(0.0),
        }
    }
}

/// RGBA color as [r, g, b, a] with values 0.0-1.0
pub type Color = [f32; 4];

/// Z-index constants for rendering order
pub mod z_index {
    pub const BACKGROUND: i32 = 0;
    pub const GUTTER: i32 = 10;
    pub const SELECTION: i32 = 15;
    pub const MATCH_HIGHLIGHT: i32 = 18;
    pub const TEXT: i32 = 20;
    pub const LINE_NUMBERS: i32 = 25;
    pub const CURSOR: i32 = 30;
    pub const STATUS_BAR_BG: i32 = 40;
    pub const STATUS_BAR_TEXT: i32 = 45;
    pub const SCROLLBAR: i32 = 50;
    pub const MODAL_OVERLAY: i32 = 100;
    pub const MODAL_BG: i32 = 110;
    pub const MODAL_INPUT_BG: i32 = 120;
    pub const MODAL_INPUT_SELECTION: i32 = 125;
    pub const MODAL_INPUT_TEXT: i32 = 130;
    pub const MODAL_INPUT_CURSOR: i32 = 135;
    pub const MODAL_BUTTON: i32 = 140;
    pub const MODAL_BUTTON_TEXT: i32 = 145;
    pub const TOOLTIP: i32 = 200;
}

/// Core primitive types that can be rendered
#[derive(Debug, Clone)]
pub enum Primitive {
    /// A solid color rectangle
    Rect {
        bounds: Rect,
        color: Color,
        z_index: i32,
    },

    /// A rectangle with rounded corners
    RoundedRect {
        bounds: Rect,
        color: Color,
        radius: f32,
        z_index: i32,
    },

    /// A border (outline) around a rectangle
    Border {
        bounds: Rect,
        color: Color,
        width: f32,
        radius: f32,
        z_index: i32,
    },

    /// A line between two points
    Line {
        start: Point,
        end: Point,
        color: Color,
        width: f32,
        z_index: i32,
    },

    /// A text string at a position (rendered using glyph atlas)
    Text {
        text: String,
        position: Point,
        color: Color,
        z_index: i32,
    },

    /// A single glyph/character (more efficient for editor text)
    Glyph {
        char: char,
        position: Point,
        color: Color,
        z_index: i32,
    },
}

impl Primitive {
    pub fn z_index(&self) -> i32 {
        match self {
            Primitive::Rect { z_index, .. } => *z_index,
            Primitive::RoundedRect { z_index, .. } => *z_index,
            Primitive::Border { z_index, .. } => *z_index,
            Primitive::Line { z_index, .. } => *z_index,
            Primitive::Text { z_index, .. } => *z_index,
            Primitive::Glyph { z_index, .. } => *z_index,
        }
    }

    /// Create a solid rectangle primitive
    pub fn rect(bounds: Rect, color: Color, z_index: i32) -> Self {
        Primitive::Rect {
            bounds,
            color,
            z_index,
        }
    }

    /// Create a rounded rectangle primitive
    pub fn rounded_rect(bounds: Rect, color: Color, radius: f32, z_index: i32) -> Self {
        Primitive::RoundedRect {
            bounds,
            color,
            radius,
            z_index,
        }
    }

    /// Create a border primitive
    pub fn border(bounds: Rect, color: Color, width: f32, radius: f32, z_index: i32) -> Self {
        Primitive::Border {
            bounds,
            color,
            width,
            radius,
            z_index,
        }
    }

    /// Create a line primitive
    pub fn line(start: Point, end: Point, color: Color, width: f32, z_index: i32) -> Self {
        Primitive::Line {
            start,
            end,
            color,
            width,
            z_index,
        }
    }

    /// Create a text primitive
    pub fn text(text: impl Into<String>, position: Point, color: Color, z_index: i32) -> Self {
        Primitive::Text {
            text: text.into(),
            position,
            color,
            z_index,
        }
    }

    /// Create a glyph primitive
    pub fn glyph(char: char, position: Point, color: Color, z_index: i32) -> Self {
        Primitive::Glyph {
            char,
            position,
            color,
            z_index,
        }
    }
}

/// A collection of primitives to render
#[derive(Debug, Clone, Default)]
pub struct RenderList {
    primitives: Vec<Primitive>,
}

impl RenderList {
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            primitives: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    pub fn extend(&mut self, primitives: impl IntoIterator<Item = Primitive>) {
        self.primitives.extend(primitives);
    }

    pub fn append(&mut self, other: &mut RenderList) {
        self.primitives.append(&mut other.primitives);
    }

    pub fn merge(&mut self, other: RenderList) {
        self.primitives.extend(other.primitives);
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    /// Sort primitives by z_index for proper rendering order
    pub fn sort_by_z_index(&mut self) {
        self.primitives.sort_by_key(|p| p.z_index());
    }

    /// Get primitives sorted by z_index
    pub fn sorted(mut self) -> Self {
        self.sort_by_z_index();
        self
    }

    /// Iterate over primitives
    pub fn iter(&self) -> impl Iterator<Item = &Primitive> {
        self.primitives.iter()
    }

    /// Consume and iterate over primitives
    pub fn into_iter(self) -> impl Iterator<Item = Primitive> {
        self.primitives.into_iter()
    }

    /// Filter primitives into color (shapes) and text categories
    pub fn partition(&self) -> (Vec<&Primitive>, Vec<&Primitive>) {
        let mut shapes = Vec::new();
        let mut text = Vec::new();

        for p in &self.primitives {
            match p {
                Primitive::Text { .. } | Primitive::Glyph { .. } => text.push(p),
                _ => shapes.push(p),
            }
        }

        shapes.sort_by_key(|p| p.z_index());
        text.sort_by_key(|p| p.z_index());

        (shapes, text)
    }
}

impl FromIterator<Primitive> for RenderList {
    fn from_iter<T: IntoIterator<Item = Primitive>>(iter: T) -> Self {
        Self {
            primitives: iter.into_iter().collect(),
        }
    }
}

impl Extend<Primitive> for RenderList {
    fn extend<T: IntoIterator<Item = Primitive>>(&mut self, iter: T) {
        self.primitives.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Point::new(50.0, 50.0)));
        assert!(!r.contains(Point::new(150.0, 50.0)));
    }

    #[test]
    fn test_rect_inset() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inset = r.inset(10.0);
        assert_eq!(inset.x, 10.0);
        assert_eq!(inset.y, 10.0);
        assert_eq!(inset.width, 80.0);
        assert_eq!(inset.height, 80.0);
    }

    #[test]
    fn test_render_list_partition() {
        let mut list = RenderList::new();
        list.push(Primitive::rect(Rect::default(), [1.0; 4], 0));
        list.push(Primitive::text("hello", Point::default(), [1.0; 4], 1));
        list.push(Primitive::glyph('x', Point::default(), [1.0; 4], 2));

        let (shapes, text) = list.partition();
        assert_eq!(shapes.len(), 1);
        assert_eq!(text.len(), 2);
    }
}
