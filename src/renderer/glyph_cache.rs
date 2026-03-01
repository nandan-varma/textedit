use fontdue::Font;
use std::collections::HashMap;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GlyphMetrics {
    pub advance_width: f32,
    pub advance_height: f32,
    pub bounds_width: u32,
    pub bounds_height: u32,
    pub xmin: i32, // Horizontal offset from origin
    pub ymin: i32, // Vertical offset from baseline (positive = above baseline)
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub metrics: GlyphMetrics,
    pub uv_min_x: f32,
    pub uv_min_y: f32,
    pub uv_max_x: f32,
    pub uv_max_y: f32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GlyphAtlas {
    font: Font,
    font_size: f32,
    atlas_width: u32,
    atlas_height: u32,
    atlas_data: Vec<u8>,
    cache: HashMap<char, AtlasEntry>,
    current_x: u32,
    current_y: u32,
    row_height: u32,
    padding: u32,
    ascent: f32,  // Distance from baseline to top of tallest glyph
    descent: f32, // Distance from baseline to bottom of lowest glyph (negative)
}

impl GlyphAtlas {
    pub fn new(
        font_data: &[u8],
        font_size: f32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> Result<Self, String> {
        let font = Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|e| format!("Failed to load font: {}", e))?;

        // Get font metrics for baseline calculations
        let line_metrics = font.horizontal_line_metrics(font_size);
        let (ascent, descent) = if let Some(lm) = line_metrics {
            (lm.ascent, lm.descent)
        } else {
            // Fallback: estimate from font size
            (font_size * 0.8, font_size * -0.2)
        };

        Ok(Self {
            font,
            font_size,
            atlas_width,
            atlas_height,
            atlas_data: vec![0u8; (atlas_width * atlas_height) as usize],
            cache: HashMap::new(),
            current_x: 0,
            current_y: 0,
            row_height: 0,
            padding: 2,
            ascent,
            descent,
        })
    }

    /// Get the ascent (distance from baseline to top)
    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    /// Get the descent (distance from baseline to bottom, typically negative)
    #[allow(dead_code)]
    pub fn descent(&self) -> f32 {
        self.descent
    }

    /// Get the total line height based on font metrics
    #[allow(dead_code)]
    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent
    }

    pub fn get_or_rasterize(&mut self, ch: char) -> Result<&AtlasEntry, String> {
        if self.cache.contains_key(&ch) {
            return Ok(&self.cache[&ch]);
        }

        let (metrics, bitmap) = self.font.rasterize(ch, self.font_size);

        let width = metrics.width as u32;
        let height = metrics.height as u32;

        if width == 0 || height == 0 {
            // Space or invisible character
            let entry = AtlasEntry {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                metrics: GlyphMetrics {
                    advance_width: metrics.advance_width,
                    advance_height: metrics.advance_height,
                    bounds_width: 0,
                    bounds_height: 0,
                    xmin: metrics.xmin,
                    ymin: metrics.ymin,
                },
                uv_min_x: 0.0,
                uv_min_y: 0.0,
                uv_max_x: 0.0,
                uv_max_y: 0.0,
            };
            self.cache.insert(ch, entry);
            return Ok(&self.cache[&ch]);
        }

        // Check if we need to wrap to next row
        if self.current_x + width + self.padding * 2 > self.atlas_width {
            self.current_x = 0;
            self.current_y += self.row_height + self.padding;
            self.row_height = 0;
        }

        if self.current_y + height + self.padding * 2 > self.atlas_height {
            return Err("Glyph atlas full".to_string());
        }

        // Copy bitmap to atlas
        let padded_x = self.current_x + self.padding;
        let padded_y = self.current_y + self.padding;

        for y in 0..height {
            for x in 0..width {
                let src_idx = (y * width + x) as usize;
                let dst_idx = ((padded_y + y) * self.atlas_width + padded_x + x) as usize;
                if dst_idx < self.atlas_data.len() && src_idx < bitmap.len() {
                    self.atlas_data[dst_idx] = bitmap[src_idx];
                }
            }
        }

        let atlas_width_f = self.atlas_width as f32;
        let atlas_height_f = self.atlas_height as f32;

        let entry = AtlasEntry {
            x: padded_x,
            y: padded_y,
            width,
            height,
            metrics: GlyphMetrics {
                advance_width: metrics.advance_width,
                advance_height: metrics.advance_height,
                bounds_width: width,
                bounds_height: height,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
            },
            uv_min_x: padded_x as f32 / atlas_width_f,
            uv_min_y: padded_y as f32 / atlas_height_f,
            uv_max_x: (padded_x + width) as f32 / atlas_width_f,
            uv_max_y: (padded_y + height) as f32 / atlas_height_f,
        };

        self.row_height = self.row_height.max(height);
        self.current_x += width + self.padding * 2;

        self.cache.insert(ch, entry);
        Ok(&self.cache[&ch])
    }

    pub fn atlas_data(&self) -> &[u8] {
        &self.atlas_data
    }

    #[allow(dead_code)]
    pub fn atlas_width(&self) -> u32 {
        self.atlas_width
    }

    #[allow(dead_code)]
    pub fn atlas_height(&self) -> u32 {
        self.atlas_height
    }

    /// Get the advance width of a character (how much to move pen after drawing)
    pub fn char_advance_width(&mut self, ch: char) -> f32 {
        if let Ok(entry) = self.get_or_rasterize(ch) {
            entry.metrics.advance_width
        } else {
            self.font_size * 0.6 // fallback
        }
    }
}
