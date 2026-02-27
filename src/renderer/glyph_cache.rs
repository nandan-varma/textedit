use fontdue::Font;
use std::collections::HashMap;

pub struct GlyphCache {
    font: Font,
    cache: HashMap<(char, u32), (u32, u32)>, // (char, size) -> (width, height)
}

impl GlyphCache {
    pub fn new(font_data: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| format!("Failed to load font: {}", e))?;

        Ok(Self {
            font,
            cache: HashMap::new(),
        })
    }

    pub fn rasterize(&mut self, ch: char, size: u32) -> (u32, u32) {
        let key = (ch, size);
        if let Some(&dims) = self.cache.get(&key) {
            return dims;
        }

        let (metrics, _bitmap) = self.font.rasterize(ch, size as f32);
        let dims = (metrics.width as u32, metrics.height as u32);

        self.cache.insert(key, dims);
        dims
    }

    pub fn glyph_metrics(&self, ch: char, size: u32) -> Option<(f32, f32)> {
        let (metrics, _) = self.font.rasterize(ch, size as f32);
        Some((metrics.advance_width, metrics.advance_height))
    }
}
