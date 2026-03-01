pub mod cursor;
pub mod glyph_cache;
pub mod layout;
pub mod line_numbers;
pub mod modal;
pub mod primitive_builders;
pub mod primitive_renderer;
pub mod scrollbar;
pub mod status_bar;
pub mod text;
pub mod text_geometry;
pub mod ui_background;

pub use primitive_builders::{
    build_background, build_cursor, build_match_highlights, build_modal_background,
    build_modal_overlay, build_scrollbar, build_selection, build_status_bar,
};
pub use primitive_renderer::{
    ColorGeometry, ColorVertex, PrimitiveRenderer, TextGeometry, TextVertex,
};
