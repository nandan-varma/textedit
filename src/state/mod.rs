pub const GLYPH_ATLAS_SIZE: u32 = 1024;
pub const BASE_FONT_SIZE: f32 = 14.0;

// Other module contents...
mod init;
mod geometry;
mod render;
mod scroll;
mod font;

pub use init::*;
