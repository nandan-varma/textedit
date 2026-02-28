use super::init::State;

impl State {
    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: crate::renderer::layout::Colors::BACKGROUND[0] as f64,
                            g: crate::renderer::layout::Colors::BACKGROUND[1] as f64,
                            b: crate::renderer::layout::Colors::BACKGROUND[2] as f64,
                            a: crate::renderer::layout::Colors::BACKGROUND[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. Render UI backgrounds (gutter, status bar)
            if let Some(pipeline) = &self.color_pipeline {
                render_pass.set_pipeline(pipeline);
                if let (Some(vb), Some(ib)) = (&self.ui_bg_vertex_buffer, &self.ui_bg_index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.ui_bg_index_count, 0, 0..1);
                }
            }

            // 2. Render text
            if let Some(pipeline) = &self.text_pipeline {
                render_pass.set_pipeline(pipeline);
                if let Some(bind_group) = &self.atlas_bind_group {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }
                if let (Some(vb), Some(ib)) = (&self.text_vertex_buffer, &self.text_index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.text_index_count, 0, 0..1);
                }
            }

            // 3. Render line numbers
            if let Some(pipeline) = &self.text_pipeline {
                render_pass.set_pipeline(pipeline);
                if let Some(bind_group) = &self.atlas_bind_group {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }
                if let (Some(vb), Some(ib)) = (
                    &self.line_numbers_vertex_buffer,
                    &self.line_numbers_index_buffer,
                ) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.line_numbers_index_count, 0, 0..1);
                }
            }

            // 4. Render status bar text
            if let Some(pipeline) = &self.text_pipeline {
                render_pass.set_pipeline(pipeline);
                if let Some(bind_group) = &self.atlas_bind_group {
                    render_pass.set_bind_group(0, bind_group, &[]);
                }
                if let (Some(vb), Some(ib)) = (
                    &self.status_bar_vertex_buffer,
                    &self.status_bar_index_buffer,
                ) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.status_bar_index_count, 0, 0..1);
                }
            }

            // 5. Render cursor (on top)
            if let Some(pipeline) = &self.color_pipeline {
                render_pass.set_pipeline(pipeline);
                // Scrollbar
                if let (Some(vb), Some(ib)) =
                    (&self.scrollbar_vertex_buffer, &self.scrollbar_index_buffer)
                {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.scrollbar_index_count, 0, 0..1);
                }

                // Cursor and selection
                if let (Some(vb), Some(ib)) =
                    (&self.cursor_vertex_buffer, &self.cursor_index_buffer)
                {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.cursor_index_count, 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
