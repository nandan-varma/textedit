use super::super::init::State;
use crate::state::GLYPH_ATLAS_SIZE;

impl State {
    pub fn set_scale_factor(&mut self, scale_factor: f64) -> anyhow::Result<()> {
        let new_scale = scale_factor as f32;
        if (new_scale - self.scale_factor).abs() < 0.001 {
            return Ok(());
        }
        self.scale_factor = new_scale;
        self.scaled_font_size = crate::state::BASE_FONT_SIZE * new_scale;
        self.scroll_visual_offset = 0;
        // Recreate glyph atlas with new font size
        let font_data = super::super::font::load_system_font()?;
        let glyph_atlas = crate::renderer::glyph_cache::GlyphAtlas::new(
            &font_data,
            self.scaled_font_size,
            GLYPH_ATLAS_SIZE,
            GLYPH_ATLAS_SIZE,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;
        // Upload new atlas data
        if let Some(atlas_texture) = &self.atlas_texture {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                glyph_atlas.atlas_data(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(GLYPH_ATLAS_SIZE),
                    rows_per_image: Some(GLYPH_ATLAS_SIZE),
                },
                wgpu::Extent3d {
                    width: GLYPH_ATLAS_SIZE,
                    height: GLYPH_ATLAS_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.glyph_atlas = Some(glyph_atlas);
        Ok(())
    }
}
