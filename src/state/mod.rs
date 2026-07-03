pub const GLYPH_ATLAS_SIZE: u32 = 1024;
pub const BASE_FONT_SIZE: f32 = 14.0;

// Other module contents...
mod font;
mod geometry;
mod init;
mod render;
mod scroll;

pub use init::*;
