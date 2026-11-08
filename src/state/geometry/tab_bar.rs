//! Tab bar geometry update for GPU rendering.

use super::super::init::State;
use crate::application::TabInfo;
use wgpu::util::DeviceExt;

impl State {
    /// Update tab bar geometry from tab information
    pub fn update_tab_bar_geometry(
        &mut self,
        tabs: &[TabInfo],
        show_line_numbers: bool,
        show_status_bar: bool,
        hovered_tab: Option<usize>,
    ) -> anyhow::Result<()> {
        let size = self.window.inner_size();
        let layout = crate::renderer::layout::EditorLayout::with_tab_bar(
            size.width as f32,
            size.height as f32,
            self.scaled_font_size,
            self.scale_factor,
            true, // show_tab_bar
            show_line_numbers,
            show_status_bar,
        );

        if let Some(glyph_atlas) = &mut self.glyph_atlas {
            let style = crate::renderer::tab_bar::TabBarRenderStyle::default();

            let tab_bar_geom = crate::renderer::tab_bar::TabBarGeometry::build(
                tabs,
                &layout,
                glyph_atlas,
                &style,
                hovered_tab,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

            // Update background geometry (tab backgrounds, indicators)
            if !tab_bar_geom.bg_vertices.is_empty() {
                self.tab_bar_bg_vertex_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("tab bar bg vertex buffer"),
                        contents: bytemuck::cast_slice(&tab_bar_geom.bg_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));
                self.tab_bar_bg_index_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("tab bar bg index buffer"),
                        contents: bytemuck::cast_slice(&tab_bar_geom.bg_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                ));
                self.tab_bar_bg_index_count = tab_bar_geom.bg_indices.len() as u32;
            } else {
                self.tab_bar_bg_index_count = 0;
            }

            // Update text geometry (tab titles, close buttons)
            if !tab_bar_geom.text_vertices.is_empty() {
                self.tab_bar_text_vertex_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("tab bar text vertex buffer"),
                        contents: bytemuck::cast_slice(&tab_bar_geom.text_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                ));
                self.tab_bar_text_index_buffer = Some(self.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("tab bar text index buffer"),
                        contents: bytemuck::cast_slice(&tab_bar_geom.text_indices),
                        usage: wgpu::BufferUsages::INDEX,
                    },
                ));
                self.tab_bar_text_index_count = tab_bar_geom.text_indices.len() as u32;
            } else {
                self.tab_bar_text_index_count = 0;
            }

            // Update atlas texture if new glyphs were rasterized
            if glyph_atlas.is_dirty() {
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
                            bytes_per_row: Some(crate::state::GLYPH_ATLAS_SIZE),
                            rows_per_image: Some(crate::state::GLYPH_ATLAS_SIZE),
                        },
                        wgpu::Extent3d {
                            width: crate::state::GLYPH_ATLAS_SIZE,
                            height: crate::state::GLYPH_ATLAS_SIZE,
                            depth_or_array_layers: 1,
                        },
                    );
                }
                glyph_atlas.mark_clean();
            }
        }

        Ok(())
    }
}
